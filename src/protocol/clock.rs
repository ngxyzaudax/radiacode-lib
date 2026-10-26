use chrono::{Datelike, NaiveDateTime, Timelike};

/// Encodes a `SET_TIME` payload: `[day, month, year - 2000, 0, sec, min, hour, 0]`.
///
/// Years outside `2000..=2255` are clamped into the representable range.
pub fn set_time_payload(local: NaiveDateTime) -> [u8; 8] {
    [
        to_byte(local.day()),
        to_byte(local.month()),
        year_offset(local.year()),
        0,
        to_byte(local.second()),
        to_byte(local.minute()),
        to_byte(local.hour()),
        0,
    ]
}

/// Encodes a `SET_TIME` payload for the current local wall-clock time.
pub fn set_time_payload_now() -> [u8; 8] {
    set_time_payload(chrono::Local::now().naive_local())
}

fn to_byte(value: u32) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn year_offset(year: i32) -> u8 {
    u8::try_from(year.saturating_sub(2000).clamp(0, 255)).unwrap_or(0)
}
