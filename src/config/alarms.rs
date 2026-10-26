use crate::error::{Error, Result};
use crate::protocol::VirtSfr;
use crate::units::{
    decode_count_alarm, decode_dose_accum, decode_dose_alarm, encode_count_alarm, encode_dose_accum, encode_dose_alarm,
    CountDisplayUnit, DisplayUnits, DoseDisplayUnit, RawCountsPer10s, RawMicroRoentgenPerHour,
};

/// Registers holding the alarm thresholds and display units, in decode order.
pub const ALARM_REGISTERS: [VirtSfr; 8] = [
    VirtSfr::CrLev1Cp10s,
    VirtSfr::CrLev2Cp10s,
    VirtSfr::DrLev1UrH,
    VirtSfr::DrLev2UrH,
    VirtSfr::DsLev1Ur,
    VirtSfr::DsLev2Ur,
    VirtSfr::DsUnits,
    VirtSfr::CrUnits,
];

/// Alarm thresholds in display units, together with the display units themselves.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlarmLimits {
    /// Count-rate level 1, in `count_unit`.
    pub l1_count_rate: f32,
    /// Count-rate level 2, in `count_unit`.
    pub l2_count_rate: f32,
    /// Dose-rate level 1, µSv/h or µR/h.
    pub l1_dose_rate: f32,
    /// Dose-rate level 2, µSv/h or µR/h.
    pub l2_dose_rate: f32,
    /// Accumulated-dose level 1, µSv or µR.
    pub l1_dose: f32,
    /// Accumulated-dose level 2, µSv or µR.
    pub l2_dose: f32,
    /// Dose unit family.
    pub dose_unit: DoseDisplayUnit,
    /// Count-rate unit.
    pub count_unit: CountDisplayUnit,
}

impl AlarmLimits {
    /// Display units configured on the device.
    pub fn display_units(&self) -> DisplayUnits {
        DisplayUnits { dose: self.dose_unit, count: self.count_unit }
    }

    /// Returns these limits with every field present in `update` replaced.
    pub fn merged(&self, update: &AlarmLimitsUpdate) -> Self {
        Self {
            l1_count_rate: update.l1_count_rate.unwrap_or(self.l1_count_rate),
            l2_count_rate: update.l2_count_rate.unwrap_or(self.l2_count_rate),
            l1_dose_rate: update.l1_dose_rate.unwrap_or(self.l1_dose_rate),
            l2_dose_rate: update.l2_dose_rate.unwrap_or(self.l2_dose_rate),
            l1_dose: update.l1_dose.unwrap_or(self.l1_dose),
            l2_dose: update.l2_dose.unwrap_or(self.l2_dose),
            dose_unit: update.dose_unit.unwrap_or(self.dose_unit),
            count_unit: update.count_unit.unwrap_or(self.count_unit),
        }
    }
}

/// Partial alarm update; `None` fields are left unchanged on the device.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
pub struct AlarmLimitsUpdate {
    pub l1_count_rate: Option<f32>,
    pub l2_count_rate: Option<f32>,
    pub l1_dose_rate: Option<f32>,
    pub l2_dose_rate: Option<f32>,
    pub l1_dose: Option<f32>,
    pub l2_dose: Option<f32>,
    pub dose_unit: Option<DoseDisplayUnit>,
    pub count_unit: Option<CountDisplayUnit>,
}

impl AlarmLimitsUpdate {
    fn provided_fields(&self) -> [bool; 8] {
        [
            self.l1_count_rate.is_some(),
            self.l2_count_rate.is_some(),
            self.l1_dose_rate.is_some(),
            self.l2_dose_rate.is_some(),
            self.l1_dose.is_some(),
            self.l2_dose.is_some(),
            self.dose_unit.is_some(),
            self.count_unit.is_some(),
        ]
    }
}

/// Decodes [`ALARM_REGISTERS`] values (at least eight words) into alarm limits.
pub fn alarm_limits_from_words(values: &[u32]) -> Result<AlarmLimits> {
    let words: [u32; 8] = values
        .get(..ALARM_REGISTERS.len())
        .and_then(|slice| slice.try_into().ok())
        .ok_or(Error::BufferUnderrun { need: ALARM_REGISTERS.len(), have: values.len() })?;
    let [cr1, cr2, dr1, dr2, ds1, ds2, dose_flag, count_flag] = words;
    let dose_unit = DoseDisplayUnit::from_device_flag(dose_flag);
    let count_unit = CountDisplayUnit::from_device_flag(count_flag);
    Ok(AlarmLimits {
        l1_count_rate: decode_count_alarm(RawCountsPer10s::new(cr1), count_unit),
        l2_count_rate: decode_count_alarm(RawCountsPer10s::new(cr2), count_unit),
        l1_dose_rate: decode_dose_alarm(RawMicroRoentgenPerHour::new(dr1), dose_unit),
        l2_dose_rate: decode_dose_alarm(RawMicroRoentgenPerHour::new(dr2), dose_unit),
        l1_dose: decode_dose_accum(ds1, dose_unit),
        l2_dose: decode_dose_accum(ds2, dose_unit),
        dose_unit,
        count_unit,
    })
}

/// Encodes alarm limits as `(register, value)` pairs in [`ALARM_REGISTERS`] order.
pub fn alarm_register_pairs(alarms: &AlarmLimits) -> Vec<(VirtSfr, u32)> {
    let (dose, count) = (alarms.dose_unit, alarms.count_unit);
    let values = [
        encode_count_alarm(alarms.l1_count_rate, count).as_u32(),
        encode_count_alarm(alarms.l2_count_rate, count).as_u32(),
        encode_dose_alarm(alarms.l1_dose_rate, dose).as_u32(),
        encode_dose_alarm(alarms.l2_dose_rate, dose).as_u32(),
        encode_dose_accum(alarms.l1_dose, dose),
        encode_dose_accum(alarms.l2_dose, dose),
        dose.to_device_flag(),
        count.to_device_flag(),
    ];
    ALARM_REGISTERS.into_iter().zip(values).collect()
}

/// Register writes needed to apply `update` on top of `current`; only provided fields are written.
pub fn updated_alarm_pairs(current: &AlarmLimits, update: &AlarmLimitsUpdate) -> Vec<(VirtSfr, u32)> {
    alarm_register_pairs(&current.merged(update))
        .into_iter()
        .zip(update.provided_fields())
        .filter_map(|(pair, provided)| provided.then_some(pair))
        .collect()
}
