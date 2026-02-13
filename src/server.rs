use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum::response::IntoResponse;

use crate::models::WhatsappMessage;
use crate::parse::parse_schedule;

pub async fn get_calendar() -> impl IntoResponse {

}

pub async fn process_schedule(headers: HeaderMap, Json(data): Json<WhatsappMessage>) -> impl IntoResponse {
    if !headers.get("marco")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "polo")
        .unwrap_or(false) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    parse_schedule(data);

    (StatusCode::OK, "OK").into_response()
}