use crate::database::upload_shift;
use crate::models::{Planning, Shift, WhatsappMessage};
use aws_config::BehaviorVersion;
use aws_sdk_textract::primitives::Blob;
use aws_sdk_textract::types::{Block, BlockType, Document, FeatureType, RelationshipType};
use chrono::{NaiveDate, NaiveTime};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::LazyLock;

static API_KEY: LazyLock<String> = LazyLock::new(|| {
    env::var("WAHA_API_KEY").expect("WAHA_API_KEY must be set in the environment")
});

static WAHA_HOST: LazyLock<String> =
    LazyLock::new(|| env::var("WAHA_HOST").expect("WAHA_HOST must be set in the environment"));

static BOFF_ID: LazyLock<String> =
    LazyLock::new(|| env::var("BOFF_ID").expect("BOFF_ID must be set in the environment"));

static RE_BODY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^Planning (?P<day>MA|DI|WO|DO|VR|ZA|ZO)(?P<type>PA|PO) (?P<date>(?P<DD>0[1-9]|[12][0-9]|3[01])-(?P<MM>0[1-9]|1[0-2])-(?P<YYYY>\d{4}))$",
    )
        .expect("Invalid Regex pattern")
});

pub fn parse_schedule(data: WhatsappMessage) {
    println!("Parsing...");
    if !RE_BODY.is_match(&data.payload.body)
        || data.payload.media.is_none()
        || data.payload.media.as_ref().unwrap().mimetype != "image/jpeg"
    {
        return;
    }

    println!("Regex match");

    tokio::spawn(async move {
        process_schedule(data).await;
    });
}

async fn process_schedule(data: WhatsappMessage) {
    println!("Processing...");

    let Some(meta) = parse_metadata(data.payload.body) else {
        return;
    };

    println!("Meta parsed");

    let url = data
        .payload
        .media
        .unwrap()
        .url
        .replace("localhost", &WAHA_HOST);

    let image_data = match fetch_image(url).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to fetch image: {:?}", e.source());
            return;
        }
    };

    println!("Image fetched");

    let Ok(contents) = parse_image(image_data).await else {
        return;
    };

    println!("Contents parsed");

    let shifts = parse_shifts(meta, contents);

    println!("Shifts parsed");

    let Some(shift) = shifts.iter().find(|s| s.boff_id == *BOFF_ID) else {
        println!("Shift not found");
        return;
    };

    match upload_shift(shift).await {
        Ok(_) => {}
        Err(e) => {
            println!("Failed to upload shift: {:?}", e.source());
            return;
        }
    }

    println!("Shift uploaded");
}

async fn fetch_image(url: String) -> Result<bytes::Bytes, reqwest::Error> {
    println!("Fetching image...");
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header("X-Api-Key", &*API_KEY)
        .send()
        .await?;

    let response = response.error_for_status()?;

    Ok(response.bytes().await?)
}

async fn parse_image(data: bytes::Bytes) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    println!("Parsing image...");
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_textract::Client::new(&config);

    let blob = Blob::new(data);

    let response = client
        .analyze_document()
        .document(Document::builder().bytes(blob).build())
        .feature_types(FeatureType::Tables)
        .send()
        .await?;

    println!("Textract success");

    let result = parse_blocks(&response.blocks.unwrap_or_default());

    Ok(result)
}

fn parse_metadata(data: String) -> Option<(NaiveDate, Planning)> {
    let caps = RE_BODY.captures(&data)?;

    let planning = match &caps["type"] {
        "PO" => Planning::PO,
        "PA" => Planning::PA,
        _ => return None,
    };

    let date = NaiveDate::parse_from_str(&caps["date"], "%-d-%-m-%Y").ok()?;

    Some((date, planning))
}

pub fn parse_shifts(meta: (NaiveDate, Planning), data: Vec<Vec<String>>) -> Vec<Shift> {
    data.iter()
        .filter_map(|e| {
            let start = NaiveTime::parse_from_str(&e[3], "%H:%M").ok();
            let end = NaiveTime::parse_from_str(&e[4], "%H:%M").unwrap_or(NaiveTime::from_hms_opt(21, 0, 0).unwrap());

            match (start) {
                Some(start_time) => Some(Shift {
                    boff_id: e[0].clone(),
                    name: e[1].clone(),
                    info: Some(e[2].clone()).filter(|s| !s.trim().is_empty()),
                    planning: meta.1,
                    date: meta.0,
                    start: start_time,
                    end,
                }),
                _ => None,
            }
        })
        .collect()
}

pub fn parse_blocks(blocks: &[Block]) -> Vec<Vec<String>> {
    // 1. Create a Lookup Map: ID -> Block
    // This allows us to instantly find the "Word" blocks belonging to a "Cell"
    let block_map: HashMap<&str, &Block> = blocks
        .iter()
        .filter_map(|b| b.id.as_deref().map(|id| (id, b)))
        .collect();

    // 2. Filter for Cell Blocks only
    // We only care about blocks that are actual table cells
    let cell_blocks: Vec<&Block> = blocks
        .iter()
        .filter(|b| b.block_type == Option::from(BlockType::Cell))
        .collect();

    if cell_blocks.is_empty() {
        return Vec::new();
    }

    // 3. Determine Table Dimensions
    // Textract is 1-indexed, so we find the max to size our 2D Vector
    let max_row = cell_blocks
        .iter()
        .filter_map(|b| b.row_index)
        .max()
        .unwrap_or(0) as usize;
    let max_col = cell_blocks
        .iter()
        .filter_map(|b| b.column_index)
        .max()
        .unwrap_or(0) as usize;

    // Initialize an empty grid filled with empty strings
    let mut grid = vec![vec![String::new(); max_col]; max_row];

    // 4. Populate the Grid
    for cell in cell_blocks {
        // Textract uses 1-based indexing, convert to 0-based for Rust vectors
        let row_idx = (cell.row_index.unwrap_or(1) as usize).saturating_sub(1);
        let col_idx = (cell.column_index.unwrap_or(1) as usize).saturating_sub(1);

        // Extract text content for this cell
        let cell_text = extract_text_from_cell(cell, &block_map);

        // Safety check to ensure we don't go out of bounds if Textract acts up
        if row_idx < max_row && col_idx < max_col {
            grid[row_idx][col_idx] = cell_text;
        }
    }

    grid
}

fn extract_text_from_cell(cell: &Block, block_map: &HashMap<&str, &Block>) -> String {
    let mut text_parts = Vec::new();

    if let Some(relationships) = &cell.relationships {
        for rel in relationships {
            if rel.r#type == Option::from(RelationshipType::Child) {
                if let Some(ids) = &rel.ids {
                    for id in ids {
                        if let Some(child_block) = block_map.get(id.as_str()) {
                            // Only append if it's a Word (skips selection elements like checkboxes if any)
                            if child_block.block_type == Option::from(BlockType::Word) {
                                if let Some(text) = &child_block.text {
                                    text_parts.push(text.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Join words with a space
    text_parts.join(" ")
}
