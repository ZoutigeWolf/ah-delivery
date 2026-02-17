use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use postgres_types::{ToSql, FromSql};
use crate::models::Planning::PA;

#[derive(Debug, Clone, Deserialize)]
pub struct WhatsappMessage {
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
pub enum Planning {
    #[postgres(name = "PO")]
    PO,

    #[postgres(name = "PA")]
    PA,
}

impl Shift {
    pub fn default(date: NaiveDate) -> Shift {
        Shift {
            boff_id: "".into(),
            name: "".into(),
            date,
            planning: PA,
            start: NaiveTime::from_hms_opt(15, 00, 00).unwrap(),
            end: NaiveTime::from_hms_opt(22, 00, 00).unwrap(),
            info: Some("".into()),

        }
    }
    pub fn uid(&self) -> String {
        format!("{}-{:?}-{}", self.date, self.planning, self.boff_id)
    }
}