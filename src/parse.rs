use crate::models::{Planning, WhatsappMessage};
use regex::Regex;
use reqwest::header::AUTHORIZATION;
use std::env;
use std::sync::LazyLock;
use chrono::{Date, NaiveDate, Utc};
use image::{DynamicImage, GenericImageView};
use tesseract::Tesseract;

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
    if (!RE_BODY.is_match(&data.body) || data.media.is_none()) {
        return;
    }

    tokio::spawn(async move {
        process_schedule(data).await;
    });
}

async fn process_schedule(data: WhatsappMessage) {
    let Some(meta) = parse_metadata(data.body) else {
      return;
    };

    let Ok(image_data) = fetch_image(data.media.unwrap().url).await else {
        return;
    };

    let contents = parse_image(image_data);

    println!("{:#?}", meta);
    println!("{:#?}", contents);
}

async fn fetch_image(url: String) -> Result<bytes::Bytes, reqwest::Error> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header(AUTHORIZATION, &*API_KEY)
        .send()
        .await?;

    let response = response.error_for_status()?;

    Ok(response.bytes().await?)
}

fn parse_image(data: bytes::Bytes) -> Result<String, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(&data)?.to_luma8();
    let (width, height) = img.dimensions();
    let pixel_data = img.as_raw();

    let tes = Tesseract::new(None, None)?;

    let text = tes.set_frame(
        pixel_data,
        width as i32,
        height as i32,
        1,
        width as i32
    )?.get_text()?;


    Ok(text)
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
