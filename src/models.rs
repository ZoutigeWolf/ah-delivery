use std::env;
use std::sync::LazyLock;
use chrono::Datelike;
use crate::models::Planning::PA;
use chrono::{NaiveDate, NaiveTime, Weekday};
use postgres_types::{FromSql, ToSql};
use serde::Deserialize;

static BOFF_ID: LazyLock<String> =
    LazyLock::new(|| env::var("BOFF_ID").expect("BOFF_ID must be set in the environment"));


#[derive(Debug, Clone, Deserialize)]
pub struct WhatsappMessage {
    #[serde(rename = "payload")]
    pub payload: WhatsappPayload,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WhatsappPayload {
    #[serde(rename = "body")]
    pub body: String,

    #[serde(rename = "media")]
    pub media: Option<WhatsappMedia>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WhatsappMedia {
    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "mimetype")]
    pub mimetype: String,
}

#[derive(Debug)]
pub struct Shift {
    pub boff_id: String,
    pub name: String,

    pub date: NaiveDate,
    pub planning: Planning,

    pub start: NaiveTime,
    pub end: NaiveTime,

    pub info: Option<String>,
}

#[derive(Debug, ToSql, FromSql, Copy, Clone)]
#[postgres(name = "planning")]
pub enum Planning {
    #[postgres(name = "PO")]
    PO,

    #[postgres(name = "PA")]
    PA,
}

impl Shift {
    pub fn default(date: NaiveDate) -> Shift {
        Shift {
            boff_id: BOFF_ID.clone(),
            name: "".into(),
            date,
            planning: PA,
            start: NaiveTime::from_hms_opt(15, 00, 00).unwrap(),
            end: NaiveTime::from_hms_opt(22, 00, 00).unwrap(),
            info: None,
        }
    }
    pub fn uid(&self) -> String {
        format!("{}-{:?}-{}", self.date, self.planning, self.boff_id)
    }

    pub fn code(&self) -> String {
        let day_prefix = match self.date.weekday() {
            Weekday::Mon => "MA",
            Weekday::Tue => "DI",
            Weekday::Wed => "WO",
            Weekday::Thu => "DO",
            Weekday::Fri => "VR",
            Weekday::Sat => "ZA",
            Weekday::Sun => "ZO",
        };

        let planning_suffix = match self.planning {
            Planning::PO => "PO",
            Planning::PA => "PA",
        };

        format!("{}{}", day_prefix, planning_suffix)
    }
}
