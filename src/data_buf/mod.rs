//! Decoder for the `DataBuf` virtual string: the device's stream of timestamped records.
//!
//! Each record starts with a seven-byte header (`u8 seq`, `u8 entity`, `u8 group`,
//! `i32 ticks`) followed by a kind-specific payload. Decoding is lenient: malformed
//! input produces [`DecodeWarning`]s instead of errors.

mod cursor;
mod decode;
mod flags;
mod header;
mod kind;
mod records;
mod sanity;
mod snapshot;

#[cfg(test)]
mod tests;

pub use cursor::{DataBufCursor, SeqGap};
pub use decode::{decode_data_buf, DataBufFrame, DecodeWarning, DecodeWarningKind};
pub use flags::{EventId, StatusFlags};
pub use header::{DeviceTicks, RecordHeader, Seq};
pub use kind::RecordKind;
pub use records::{AccelData, DataBufRecord, DoseRateDb, EventRecord, RareData, RawData, RealTimeData};
pub use sanity::{check_battery_and_temperature, check_record, SanityFailure};
pub use snapshot::{
    latest_rare_record, latest_snapshot, real_time_records, snapshot_from_frame, DataBufSnapshot, RareStatus,
    RealTimeRates,
};
