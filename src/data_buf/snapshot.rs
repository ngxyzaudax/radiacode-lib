use super::decode::{decode_data_buf, DataBufFrame};
use super::header::Seq;
use super::kind::RecordKind;
use super::records::{DataBufRecord, RareData, RealTimeData};

/// Dose, temperature and battery values from the latest rare record.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RareStatus {
    /// Dose accumulation time in seconds.
    pub duration_secs: u32,
    /// Accumulated dose in Roentgen.
    pub dose_r: f32,
    /// Detector temperature, °C.
    pub temperature_c: f32,
    /// Battery charge, percent.
    pub battery_percent: f32,
}

impl From<RareData> for RareStatus {
    fn from(value: RareData) -> Self {
        Self {
            duration_secs: value.duration_secs,
            dose_r: value.dose_r.as_f32(),
            temperature_c: value.temperature_c,
            battery_percent: value.battery_percent,
        }
    }
}

/// Count and dose rate in wire units with error estimates.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RealTimeRates {
    /// Count rate, cps.
    pub count_rate_cps: f32,
    /// Dose rate, R/h.
    pub dose_rate_rh: f32,
    /// Count-rate error, percent (zero when the source carries none).
    pub count_rate_err_pct: f32,
    /// Dose-rate error, percent (zero when the source carries none).
    pub dose_rate_err_pct: f32,
}

impl From<RealTimeData> for RealTimeRates {
    fn from(value: RealTimeData) -> Self {
        Self {
            count_rate_cps: value.count_rate_cps.as_f32(),
            dose_rate_rh: value.dose_rate_rh.as_f32(),
            count_rate_err_pct: value.count_rate_err_pct,
            dose_rate_err_pct: value.dose_rate_err_pct,
        }
    }
}

/// Most recent rates and rare status found in a DataBuf frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataBufSnapshot {
    /// Latest rare record, if any.
    pub rare: Option<RareStatus>,
    /// Newest rates, preferring real-time over database over raw records on ties.
    pub rates: Option<RealTimeRates>,
}

type RateCandidate = (Seq, RecordKind, RealTimeRates);

/// Decodes `data` and returns its snapshot.
pub fn latest_snapshot(data: &[u8]) -> DataBufSnapshot {
    snapshot_from_frame(&decode_data_buf(data))
}

/// Builds a snapshot from an already decoded frame.
pub fn snapshot_from_frame(frame: &DataBufFrame) -> DataBufSnapshot {
    DataBufSnapshot {
        rare: latest_rare_record(frame).copied().map(RareStatus::from),
        rates: frame.records.iter().filter_map(rate_candidate).fold(None, prefer_newer).map(|(_, _, rates)| rates),
    }
}

/// Iterates the real-time records of a frame in payload order.
pub fn real_time_records(frame: &DataBufFrame) -> impl Iterator<Item = &RealTimeData> {
    frame.records.iter().filter_map(|record| match record {
        DataBufRecord::RealTime(record) => Some(record),
        _ => None,
    })
}

/// Returns the last rare record of a frame.
pub fn latest_rare_record(frame: &DataBufFrame) -> Option<&RareData> {
    frame.records.iter().rev().find_map(|record| match record {
        DataBufRecord::Rare(record) => Some(record),
        _ => None,
    })
}

fn rate_candidate(record: &DataBufRecord) -> Option<RateCandidate> {
    let header = record.header();
    let rates = match record {
        DataBufRecord::RealTime(real_time) => RealTimeRates::from(*real_time),
        DataBufRecord::Raw(raw) => RealTimeRates {
            count_rate_cps: raw.count_rate_cps.as_f32(),
            dose_rate_rh: raw.dose_rate_rh.as_f32(),
            count_rate_err_pct: 0.0,
            dose_rate_err_pct: 0.0,
        },
        DataBufRecord::DoseRateDb(db) => RealTimeRates {
            count_rate_cps: db.count_rate_cps.as_f32(),
            dose_rate_rh: db.dose_rate_rh.as_f32(),
            count_rate_err_pct: 0.0,
            dose_rate_err_pct: db.dose_rate_err_pct,
        },
        _ => return None,
    };
    Some((header.seq, header.kind, rates))
}

fn prefer_newer(best: Option<RateCandidate>, candidate: RateCandidate) -> Option<RateCandidate> {
    match best {
        Some(current) if !replaces(&current, &candidate) => Some(current),
        _ => Some(candidate),
    }
}

fn replaces((best_seq, best_kind, _): &RateCandidate, (seq, kind, _): &RateCandidate) -> bool {
    seq.is_newer_than(*best_seq) || (seq == best_seq && kind.monitor_source_rank() > best_kind.monitor_source_rank())
}
