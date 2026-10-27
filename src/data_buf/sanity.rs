use std::ops::RangeInclusive;

use super::records::{DataBufRecord, DoseRateDb, RareData, RealTimeData};

const BATTERY_PERCENT_RANGE: RangeInclusive<f32> = 0.0..=150.0;
const TEMPERATURE_C_RANGE: RangeInclusive<f32> = -40.0..=85.0;
const ERROR_PERCENT_RANGE: RangeInclusive<f32> = 0.0..=100.0;

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
    /// A rate error percentage outside 0–100 %.
    ErrorPercentOutOfRange,
}

/// Checks that a decoded record holds physically plausible values.
pub fn check_record(record: &DataBufRecord) -> Result<(), SanityFailure> {
    match record {
        DataBufRecord::RealTime(record) => check_real_time(record),
        DataBufRecord::Raw(record) => check_rates(record.count_rate_cps.as_f32(), record.dose_rate_rh.as_f32()),
        DataBufRecord::DoseRateDb(record) => check_dose_rate_db(record),
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

fn check_real_time(record: &RealTimeData) -> Result<(), SanityFailure> {
    check_rates(record.count_rate_cps.as_f32(), record.dose_rate_rh.as_f32())?;
    check_error_percent(record.count_rate_err_pct)?;
    check_error_percent(record.dose_rate_err_pct)
}

fn check_dose_rate_db(record: &DoseRateDb) -> Result<(), SanityFailure> {
    check_rates(record.count_rate_cps.as_f32(), record.dose_rate_rh.as_f32())?;
    check_error_percent(record.dose_rate_err_pct)
}

fn check_error_percent(value: f32) -> Result<(), SanityFailure> {
    match ERROR_PERCENT_RANGE.contains(&value) {
        true => Ok(()),
        false => Err(SanityFailure::ErrorPercentOutOfRange),
    }
}

fn check_rates(count_rate_cps: f32, dose_rate_rh: f32) -> Result<(), SanityFailure> {
    match (count_rate_cps, dose_rate_rh) {
        (count, dose) if !count.is_finite() || !dose.is_finite() => Err(SanityFailure::NonFiniteRate),
        (count, dose) if count < 0.0 || dose < 0.0 => Err(SanityFailure::NegativeRate),
        _ => Ok(()),
    }
}
