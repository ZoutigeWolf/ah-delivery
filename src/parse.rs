use crate::models::{Planning, WhatsappMessage};
use chrono::{NaiveDate};
use regex::Regex;
use reqwest::header::AUTHORIZATION;
use std::env;
use std::sync::LazyLock;
use aws_config::BehaviorVersion;
use aws_sdk_textract::primitives::Blob;
use aws_sdk_textract::types::{Document, FeatureType};

static API_KEY: LazyLock<String> = LazyLock::new(|| {
    env::var("WAHA_API_KEY").expect("WAHA_API_KEY must be set in the environment")
});

static RE_BODY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^Planning (?P<day>MA|DI|WO|DO|VR|ZA|ZO)(?P<type>PA|PO) (?P<date>(?P<DD>0[1-9]|[12][0-9]|3[01])-(?P<MM>0[1-9]|1[0-2])-(?P<YYYY>\d{4}))$",
    )
        .expect("Invalid Regex pattern")
});

pub fn parse_schedule(data: WhatsappMessage) {
    println!("Parsing...");
    if (!RE_BODY.is_match(&data.body) || data.media.is_none()) {
        return;
    }

    println!("Regex match");

    tokio::spawn(async move {
        process_schedule(data).await;
    });
}

async fn process_schedule(data: WhatsappMessage) {
    println!("Processing...");

    let Some(meta) = parse_metadata(data.body) else {
        return;
    };

    println!("Meta parsed");

    let Ok(image_data) = fetch_image(data.media.unwrap().url).await else {
        return;
    };

    println!("Image fetched");

    let contents = parse_image(image_data);

    println!("{:#?}", meta);
}

async fn fetch_image(url: String) -> Result<bytes::Bytes, reqwest::Error> {
    println!("Fetching image...");
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header(AUTHORIZATION, &*API_KEY)
        .send()
        .await?;

    let response = response.error_for_status()?;

    Ok(response.bytes().await?)
}

async fn parse_image(data: bytes::Bytes) -> Result<String, Box<dyn std::error::Error>> {
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

    let blocks = response.blocks.unwrap_or_default();

    println!("{:#?}", blocks);

    Ok("".parse()?)
}

fn parse_metadata(data: String) -> Option<(NaiveDate, Planning)> {
    let caps = RE_BODY.captures(&data)?;

    let planning = match &caps["type"] {
        "PO" => Planning::PO,
        "PA" => Planning::PA,
        _ => return None,
    };

    let date = NaiveDate::parse_from_str(&caps["type"], "%d-%m-%Y").ok()?;

    Some((date, planning))
}
