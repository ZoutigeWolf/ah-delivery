use crate::database::fetch_shifts;
use crate::date;
use crate::models::{Shift, WhatsappMessage};
use crate::parse::parse_schedule;
use axum::Json;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use chrono::Utc;
use chrono::{Datelike, Duration, Local};
use ics::properties::{Description, DtEnd, DtStart, Location, Summary};
use ics::{Event, ICalendar};
use std::collections::HashSet;
use std::env;
use std::sync::LazyLock;

pub static DAYS: LazyLock<HashSet<u8>> = LazyLock::new(|| {
    let s = env::var("DAYS").expect("DAYS must be set");

    let mut set = HashSet::new();

    for part in s.split(',') {
        let day: u8 = part.trim().parse().expect("DAYS must contain numbers 1-7");

        if !(1..=7).contains(&day) {
            panic!("DAYS values must be between 1 and 7");
        }

        if !set.insert(day) {
            panic!("Duplicate day in DAYS: {}", day);
        }
    }

    set
});

pub async fn get_calendar() -> impl IntoResponse {
    let Ok(mut shifts) = fetch_shifts().await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to generate calendar",
        )
            .into_response();
    };

    let latest_date = shifts
        .iter()
        .max_by_key(|s| s.date)
        .map(|s| s.date)
        .unwrap_or_else(|| Local::now().naive_local().date());

    let future_shifts = (1..=28)
        .map(|i| latest_date + Duration::days(i))
        .filter(|d| DAYS.contains(&(d.weekday().number_from_monday() as u8)))
        .map(|d| Shift::default(d))
        .collect::<Vec<_>>();

    shifts.extend(future_shifts);

    let mut cal = ICalendar::new("2.0", "ah-delivery");

    for shift in &shifts {
        let mut event = Event::new(shift.uid(), date::to_string(Utc::now()));

        event.push(DtStart::new(date::to_string(date::combine(
            shift.date,
            shift.start,
        ))));
        event.push(DtEnd::new(date::to_string(date::combine(
            shift.date, shift.end,
        ))));

        event.push(Summary::new(shift.code()));
        event.push(Location::new("HSC Bleiswijk\\nAquamarijnweg 2\\, 2665 PB\\nBleiswijk\\, Netherlands"));

        if let Some(info) = shift.info.clone() {
            event.push(Description::new(info));
        }

        cal.add_event(event);
    }

    let ics_string = cal.to_string();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/calendar".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"calendar.ics\"".parse().unwrap(),
    );

    println!("Calendar generation success");

    (headers, ics_string).into_response()
}

pub async fn process_schedule(
    headers: HeaderMap,
    Json(data): Json<WhatsappMessage>,
) -> impl IntoResponse {
    println!("Received request");

    if !headers
        .get("marco")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "polo")
        .unwrap_or(false)
    {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    println!("Header valid");

    parse_schedule(data);

    (StatusCode::OK, "OK").into_response()
}
