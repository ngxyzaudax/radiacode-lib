use std::ops::RangeInclusive;

use super::records::{DataBufRecord, RareData};

const BATTERY_PERCENT_RANGE: RangeInclusive<f32> = 0.0..=150.0;
const TEMPERATURE_C_RANGE: RangeInclusive<f32> = -40.0..=85.0;

/// Reason a decoded record was rejected as implausible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SanityFailure {
    /// A rate was NaN or infinite.
    NonFiniteRate,
    /// A rate was negative.
    NegativeRate,
    /// Battery charge outside 0–150 %.
    BatteryOutOfRange,
    /// Temperature outside −40–85 °C.
    TemperatureOutOfRange,
}

/// Checks that a decoded record holds physically plausible values.
pub fn check_record(record: &DataBufRecord) -> Result<(), SanityFailure> {
    match record {
        DataBufRecord::RealTime(r) => check_rates(r.count_rate_cps.as_f32(), r.dose_rate_rh.as_f32()),
        DataBufRecord::Raw(r) => check_rates(r.count_rate_cps.as_f32(), r.dose_rate_rh.as_f32()),
        DataBufRecord::DoseRateDb(r) => check_rates(r.count_rate_cps.as_f32(), r.dose_rate_rh.as_f32()),
        DataBufRecord::Rare(r) => check_rare(r),
        DataBufRecord::Accel(_) | DataBufRecord::Event(_) | DataBufRecord::Skipped(_) => Ok(()),
    }
}

/// Checks battery charge and temperature ranges.
pub fn check_battery_and_temperature(battery: f32, temperature: f32) -> Result<(), SanityFailure> {
    match (BATTERY_PERCENT_RANGE.contains(&battery), TEMPERATURE_C_RANGE.contains(&temperature)) {
        (false, _) => Err(SanityFailure::BatteryOutOfRange),
        (_, false) => Err(SanityFailure::TemperatureOutOfRange),
        _ => Ok(()),
    }
}

fn check_rare(record: &RareData) -> Result<(), SanityFailure> {
    check_battery_and_temperature(record.battery_percent, record.temperature_c)
}

fn check_rates(count_rate_cps: f32, dose_rate_rh: f32) -> Result<(), SanityFailure> {
    match (count_rate_cps, dose_rate_rh) {
        (c, d) if !c.is_finite() || !d.is_finite() => Err(SanityFailure::NonFiniteRate),
        (c, d) if c < 0.0 || d < 0.0 => Err(SanityFailure::NegativeRate),
        _ => Ok(()),
    }
}
