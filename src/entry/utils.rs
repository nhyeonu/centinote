use chrono::{DateTime, NaiveDateTime, FixedOffset, TimeZone};

pub fn naive_to_offset(
    naive: NaiveDateTime,
    timezone_offset: i32) -> DateTime<FixedOffset> 
{
    let offset = match FixedOffset::west_opt(timezone_offset * 60) {
        Some(value) => value,
        None => FixedOffset::west_opt(0).unwrap()
    };

    offset.from_utc_datetime(&naive)
}

