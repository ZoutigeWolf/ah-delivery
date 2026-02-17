use chrono::Datelike;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Europe::Amsterdam;
use chrono_tz::Tz;
use crate::models::Planning;

pub fn combine(date: NaiveDate, time: NaiveTime) -> DateTime<Tz> {
    Amsterdam
        .from_local_datetime(&date.and_time(time))
        .single()
        .unwrap()
}

pub fn to_string(dt: DateTime<impl TimeZone>) -> String {
    let utc: DateTime<Utc> = dt.with_timezone(&Utc);

    format!("{}", utc.format("%Y%m%dT%H%M%SZ"))
}