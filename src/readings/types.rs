use crate::data_buf::{DeviceTicks, SeqGap};
use crate::units::{CountDisplayUnit, DoseDisplayUnit};

/// Latest dose and count rate converted to display units.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiveRates {
    /// Dose rate in `dose_unit` (µSv/h or µR/h).
    pub dose_rate: f32,
    /// Count rate in `count_unit`.
    pub count_rate: f32,
    /// Dose unit family.
    pub dose_unit: DoseDisplayUnit,
    /// Count-rate unit.
    pub count_unit: CountDisplayUnit,
    /// Dose-rate error, percent.
    pub dose_rate_err_pct: f32,
    /// Count-rate error, percent.
    pub count_rate_err_pct: f32,
}

/// One timestamped real-time rate sample in display units.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimedRates {
    /// Device timestamp of the record.
    pub device_ts: DeviceTicks,
    /// Dose rate in `dose_unit`.
    pub dose_rate: f32,
    /// Count rate in `count_unit`.
    pub count_rate: f32,
    /// Dose-rate error, percent.
    pub dose_rate_err_pct: f32,
    /// Count-rate error, percent.
    pub count_rate_err_pct: f32,
    /// Dose unit family.
    pub dose_unit: DoseDisplayUnit,
    /// Count-rate unit.
    pub count_unit: CountDisplayUnit,
}

/// Accumulated dose since the last dose reset.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccumulatedDose {
    /// Dose in µSv or µR.
    pub dose: f32,
    /// Accumulation time in seconds.
    pub duration_secs: u32,
    /// Dose unit family.
    pub dose_unit: DoseDisplayUnit,
}

/// Everything extracted from one DataBuf poll.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MonitorPollSample {
    /// Real-time samples in payload order.
    pub rates: Vec<TimedRates>,
    /// Accumulated dose from the latest rare record.
    pub accumulated: Option<AccumulatedDose>,
    /// Decoder warnings other than sequence jumps and sanity rejections.
    pub decode_warnings: usize,
    /// Records rejected by sanity checks.
    pub rejected_records: usize,
    /// Number of decoder resynchronisations.
    pub resync_count: usize,
    /// Sequence gaps within and since the previous poll.
    pub seq_gaps: Vec<SeqGap>,
}

/// Battery, temperature and link quality.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceStatus {
    /// Battery charge, percent.
    pub battery_percent: Option<f32>,
    /// Detector temperature, °C.
    pub temperature_c: Option<f32>,
    /// Link RSSI in dBm, for Bluetooth links.
    pub rssi_dbm: Option<i16>,
}

impl DeviceStatus {
    /// Combines `fresh` values with `self`, keeping previous values where `fresh` has none.
    pub fn merged(self, fresh: DeviceStatus) -> DeviceStatus {
        DeviceStatus {
            battery_percent: fresh.battery_percent.or(self.battery_percent),
            temperature_c: fresh.temperature_c.or(self.temperature_c),
            rssi_dbm: fresh.rssi_dbm.or(self.rssi_dbm),
        }
    }
}
