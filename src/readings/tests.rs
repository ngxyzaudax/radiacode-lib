use super::{
    live_rates_from, monitor_sample_from, plausible_battery, plausible_temperature, status_from_snapshot, DeviceStatus,
};
use crate::data_buf::{decode_data_buf, snapshot_from_frame, DataBufSnapshot, RareStatus, RealTimeRates};
use crate::readings::RssiStatus;
use crate::units::{CountDisplayUnit, DisplayUnits, DoseDisplayUnit};

const SV_CPM: DisplayUnits = DisplayUnits { dose: DoseDisplayUnit::MicroSievertPerHour, count: CountDisplayUnit::Cpm };

fn real_time(seq: u8, cps: f32, rh: f32) -> Vec<u8> {
    [vec![seq, 0, 0, 100, 0, 0, 0], cps.to_le_bytes().to_vec(), rh.to_le_bytes().to_vec(), vec![15, 0, 25, 0, 0, 0, 0]]
        .concat()
}

fn rare(seq: u8, dose_r: f32) -> Vec<u8> {
    [
        vec![seq, 0, 3, 0, 0, 0, 0],
        3600u32.to_le_bytes().to_vec(),
        dose_r.to_le_bytes().to_vec(),
        4500u16.to_le_bytes().to_vec(),
        9000u16.to_le_bytes().to_vec(),
        vec![0, 0],
    ]
    .concat()
}

#[test]
fn live_rates_convert_to_display_units() {
    let rates =
        RealTimeRates { count_rate_cps: 2.0, dose_rate_rh: 0.000_01, count_rate_err_pct: 3.0, dose_rate_err_pct: 4.0 };
    let live = live_rates_from(&rates, SV_CPM);
    assert!((live.dose_rate - 0.1).abs() < 1e-6);
    assert!((live.count_rate - 120.0).abs() < 1e-4);
    assert_eq!((live.dose_unit, live.count_unit), (SV_CPM.dose, SV_CPM.count));
    assert_eq!((live.count_rate_err_pct, live.dose_rate_err_pct), (3.0, 4.0));
}

#[test]
fn monitor_sample_collects_rates_dose_and_counters() {
    let bytes = [real_time(1, 10.0, 0.000_02), real_time(2, f32::NAN, 0.0), rare(3, 0.0005), vec![0xEE]].concat();
    let frame = decode_data_buf(&bytes);
    let sample = monitor_sample_from(&frame, SV_CPM, Vec::new());
    assert_eq!(sample.rates.len(), 1);
    assert_eq!(sample.rates.first().map(|r| r.device_ts.raw()), Some(100));
    assert!(sample.rates.first().is_some_and(|r| (r.count_rate - 600.0).abs() < 1e-3));
    let dose = sample.accumulated.unwrap_or_else(|| panic!("dose missing"));
    assert!((dose.dose - 5.0).abs() < 1e-4);
    assert_eq!(dose.duration_secs, 3600);
    assert_eq!((sample.rejected_records, sample.decode_warnings, sample.resync_count), (1, 1, 0));
}

#[test]
fn status_comes_from_rare_record_and_rejects_implausible_values() {
    let status = status_from_snapshot(&snapshot_from_frame(&decode_data_buf(&rare(1, 0.0))));
    assert_eq!(status.battery_percent, Some(90.0));
    assert_eq!(status.temperature_c, Some(25.0));
    let implausible = DataBufSnapshot {
        rare: Some(RareStatus { duration_secs: 0, dose_r: 0.0, temperature_c: 120.0, battery_percent: -1.0 }),
        rates: None,
    };
    assert_eq!(status_from_snapshot(&implausible), DeviceStatus::default());
    assert!(plausible_battery(100.0) && !plausible_battery(f32::NAN));
    assert!(plausible_temperature(-40.0) && !plausible_temperature(86.0));
}

#[test]
fn status_merge_keeps_previous_values() {
    let previous =
        DeviceStatus { battery_percent: Some(80.0), temperature_c: Some(21.0), rssi: RssiStatus::Measured(-60) };
    let fresh = DeviceStatus { battery_percent: Some(79.0), temperature_c: None, rssi: RssiStatus::NotReported };
    assert_eq!(
        previous.merged(fresh),
        DeviceStatus { battery_percent: Some(79.0), temperature_c: Some(21.0), rssi: RssiStatus::Measured(-60) }
    );
}
