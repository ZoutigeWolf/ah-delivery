use serde::Deserialize;

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
pub enum Planning {
    PO,
    PA,
}