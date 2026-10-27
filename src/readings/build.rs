use super::types::{AccumulatedDose, DeviceStatus, LiveRates, MonitorPollSample, TimedRates};
use crate::data_buf::{
    check_battery_and_temperature, latest_rare_record, real_time_records, DataBufFrame, DataBufSnapshot,
    DecodeWarningKind, RareData, RealTimeData, RealTimeRates, SeqGap,
};
use crate::units::{count_display_from_cps, dose_display_from_rh, DisplayUnits};

/// Converts wire-unit rates into [`LiveRates`].
pub fn live_rates_from(rates: &RealTimeRates, units: DisplayUnits) -> LiveRates {
    LiveRates {
        dose_rate: dose_display_from_rh(rates.dose_rate_rh, units.dose),
        count_rate: count_display_from_cps(rates.count_rate_cps, units.count),
        dose_unit: units.dose,
        count_unit: units.count,
        dose_rate_err_pct: rates.dose_rate_err_pct,
        count_rate_err_pct: rates.count_rate_err_pct,
    }
}

/// Converts one real-time record into a [`TimedRates`] sample.
pub fn timed_rates_from(record: &RealTimeData, units: DisplayUnits) -> TimedRates {
    TimedRates {
        device_ts: record.header.ts,
        dose_rate: dose_display_from_rh(record.dose_rate_rh.as_f32(), units.dose),
        count_rate: count_display_from_cps(record.count_rate_cps.as_f32(), units.count),
        dose_rate_err_pct: record.dose_rate_err_pct,
        count_rate_err_pct: record.count_rate_err_pct,
        dose_unit: units.dose,
        count_unit: units.count,
    }
}

/// Converts a rare record into an [`AccumulatedDose`].
pub fn accumulated_dose_from(rare: &RareData, units: DisplayUnits) -> AccumulatedDose {
    AccumulatedDose {
        dose: dose_display_from_rh(rare.dose_r.as_f32(), units.dose),
        duration_secs: rare.duration_secs,
        dose_unit: units.dose,
    }
}

/// Builds a [`MonitorPollSample`] from a decoded frame and its sequence gaps.
pub fn monitor_sample_from(frame: &DataBufFrame, units: DisplayUnits, seq_gaps: Vec<SeqGap>) -> MonitorPollSample {
    MonitorPollSample {
        rates: real_time_records(frame).map(|r| timed_rates_from(r, units)).collect(),
        accumulated: latest_rare_record(frame).map(|rare| accumulated_dose_from(rare, units)),
        decode_warnings: frame.warnings.iter().filter(|w| counts_as_decode_warning(&w.kind)).count(),
        rejected_records: frame.rejected_records(),
        resync_count: usize::try_from(frame.resync_count).unwrap_or(usize::MAX),
        seq_gaps,
    }
}

/// Battery and temperature from a snapshot, discarding implausible values.
pub fn status_from_snapshot(snapshot: &DataBufSnapshot) -> DeviceStatus {
    DeviceStatus {
        battery_percent: snapshot.rare.map(|rare| rare.battery_percent).filter(|b| plausible_battery(*b)),
        temperature_c: snapshot.rare.map(|rare| rare.temperature_c).filter(|t| plausible_temperature(*t)),
        rssi: crate::readings::RssiStatus::NotReported,
    }
}

/// Returns `true` for a battery charge within 0–150 %.
pub fn plausible_battery(battery_percent: f32) -> bool {
    check_battery_and_temperature(battery_percent, 20.0).is_ok()
}

/// Returns `true` for a temperature within −40–85 °C.
pub fn plausible_temperature(temperature_c: f32) -> bool {
    check_battery_and_temperature(50.0, temperature_c).is_ok()
}

fn counts_as_decode_warning(kind: &DecodeWarningKind) -> bool {
    !matches!(kind, DecodeWarningKind::SeqJump { .. } | DecodeWarningKind::SanityRejected(_))
}
