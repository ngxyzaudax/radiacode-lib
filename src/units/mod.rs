//! Physical quantities, display units and alarm-threshold conversions.
//!
//! The device reports dose rate in R/h and dose in R regardless of the display
//! unit; alarm thresholds are stored in unit-dependent integer encodings.

mod convert;
mod display;
mod quantities;

#[cfg(test)]
mod tests;

pub use convert::{
    count_display_from_cps, decode_count_alarm, decode_dose_accum, decode_dose_alarm, dose_display_from_rh,
    encode_count_alarm, encode_dose_accum, encode_dose_alarm,
};
pub use display::{CountDisplayUnit, DisplayUnits, DoseDisplayUnit};
pub use quantities::{CountRateCps, DoseRateRh, DoseRoentgen, RawCountsPer10s, RawMicroRoentgenPerHour};
