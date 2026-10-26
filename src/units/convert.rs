use super::display::{CountDisplayUnit, DoseDisplayUnit};
use super::quantities::{RawCountsPer10s, RawMicroRoentgenPerHour};

const MICRO_SIEVERT_PER_ROENTGEN: f32 = 10_000.0;
const MICRO_ROENTGEN_PER_ROENTGEN: f32 = 1_000_000.0;
const SECONDS_PER_MINUTE: f32 = 60.0;

/// Converts a wire dose rate (R/h) or dose (R) into the display unit (µSv or µR based).
pub fn dose_display_from_rh(dose_rate_rh: f32, unit: DoseDisplayUnit) -> f32 {
    match unit {
        DoseDisplayUnit::MicroSievertPerHour => dose_rate_rh * MICRO_SIEVERT_PER_ROENTGEN,
        DoseDisplayUnit::MicroRoentgenPerHour => dose_rate_rh * MICRO_ROENTGEN_PER_ROENTGEN,
    }
}

/// Converts a count rate in cps into the display unit.
pub fn count_display_from_cps(count_cps: f32, unit: CountDisplayUnit) -> f32 {
    match unit {
        CountDisplayUnit::Cpm => count_cps * SECONDS_PER_MINUTE,
        CountDisplayUnit::Cps => count_cps,
    }
}

/// Encodes a dose-rate alarm threshold given in display units.
pub fn encode_dose_alarm(display: f32, unit: DoseDisplayUnit) -> RawMicroRoentgenPerHour {
    RawMicroRoentgenPerHour::new(encode_dose_accum(display, unit))
}

/// Decodes a dose-rate alarm threshold into display units.
pub fn decode_dose_alarm(raw: RawMicroRoentgenPerHour, unit: DoseDisplayUnit) -> f32 {
    decode_dose_accum(raw.as_u32(), unit)
}

/// Encodes an accumulated-dose alarm threshold given in display units (µSv or µR).
pub fn encode_dose_accum(display_micro: f32, unit: DoseDisplayUnit) -> u32 {
    non_negative_round(display_micro * dose_unit_factor(unit))
}

/// Decodes an accumulated-dose alarm threshold into display units.
pub fn decode_dose_accum(raw: u32, unit: DoseDisplayUnit) -> f32 {
    raw as f32 / dose_unit_factor(unit)
}

/// Encodes a count-rate alarm threshold given in display units.
pub fn encode_count_alarm(display: f32, unit: CountDisplayUnit) -> RawCountsPer10s {
    let multiplier = match unit {
        CountDisplayUnit::Cpm => 1.0 / 6.0,
        CountDisplayUnit::Cps => 10.0,
    };
    RawCountsPer10s::new(non_negative_round(display * multiplier))
}

/// Decodes a count-rate alarm threshold into display units.
pub fn decode_count_alarm(raw: RawCountsPer10s, unit: CountDisplayUnit) -> f32 {
    count_display_from_cps(raw.as_u32() as f32 / 10.0, unit)
}

fn dose_unit_factor(unit: DoseDisplayUnit) -> f32 {
    match unit {
        DoseDisplayUnit::MicroSievertPerHour => 100.0,
        DoseDisplayUnit::MicroRoentgenPerHour => 1.0,
    }
}

fn non_negative_round(value: f32) -> u32 {
    value.round().max(0.0) as u32
}
