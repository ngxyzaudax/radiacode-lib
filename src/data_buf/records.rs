use super::flags::{EventId, StatusFlags};
use super::header::RecordHeader;
use crate::units::{CountRateCps, DoseRateRh, DoseRoentgen};

/// Filtered real-time count and dose rate.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RealTimeData {
    /// Record header.
    pub header: RecordHeader,
    /// Count rate.
    pub count_rate_cps: CountRateCps,
    /// Dose rate.
    pub dose_rate_rh: DoseRateRh,
    /// Statistical error of the count rate, percent.
    pub count_rate_err_pct: f32,
    /// Statistical error of the dose rate, percent.
    pub dose_rate_err_pct: f32,
    /// Device status flags.
    pub flags: StatusFlags,
    /// Real-time specific flags.
    pub real_time_flags: u8,
}

/// Unfiltered count and dose rate.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawData {
    /// Record header.
    pub header: RecordHeader,
    /// Count rate.
    pub count_rate_cps: CountRateCps,
    /// Dose rate.
    pub dose_rate_rh: DoseRateRh,
}

/// Dose-rate database sample.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DoseRateDb {
    /// Record header.
    pub header: RecordHeader,
    /// Counts in the sample interval.
    pub count: u32,
    /// Count rate.
    pub count_rate_cps: CountRateCps,
    /// Dose rate.
    pub dose_rate_rh: DoseRateRh,
    /// Statistical error of the dose rate, percent.
    pub dose_rate_err_pct: f32,
    /// Device status flags.
    pub flags: StatusFlags,
}

/// Periodic accumulated-dose, temperature and battery record.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RareData {
    /// Record header.
    pub header: RecordHeader,
    /// Dose accumulation time in seconds.
    pub duration_secs: u32,
    /// Accumulated dose.
    pub dose_r: DoseRoentgen,
    /// Detector temperature, °C.
    pub temperature_c: f32,
    /// Battery charge, percent.
    pub battery_percent: f32,
    /// Device status flags.
    pub flags: StatusFlags,
}

/// Accelerometer sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccelData {
    /// Record header.
    pub header: RecordHeader,
    /// X axis raw value.
    pub x: u16,
    /// Y axis raw value.
    pub y: u16,
    /// Z axis raw value.
    pub z: u16,
}

/// Device event such as an alarm or a dose reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventRecord {
    /// Record header.
    pub header: RecordHeader,
    /// Event identifier.
    pub event: EventId,
    /// First event parameter.
    pub event_param1: u8,
    /// Device status flags.
    pub flags: StatusFlags,
}

/// One decoded DataBuf record.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataBufRecord {
    /// Real-time rates.
    RealTime(RealTimeData),
    /// Unfiltered rates.
    Raw(RawData),
    /// Dose-rate database sample.
    DoseRateDb(DoseRateDb),
    /// Dose, temperature and battery.
    Rare(RareData),
    /// Accelerometer sample.
    Accel(AccelData),
    /// Device event.
    Event(EventRecord),
    /// A record of a known kind whose payload is skipped.
    Skipped(RecordHeader),
}

impl DataBufRecord {
    /// Header of the record.
    pub fn header(&self) -> RecordHeader {
        match self {
            Self::RealTime(record) => record.header,
            Self::Raw(record) => record.header,
            Self::DoseRateDb(record) => record.header,
            Self::Rare(record) => record.header,
            Self::Accel(record) => record.header,
            Self::Event(record) => record.header,
            Self::Skipped(header) => *header,
        }
    }
}
