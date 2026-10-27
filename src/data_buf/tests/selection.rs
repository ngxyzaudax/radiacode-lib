use super::{header, rare, real_time};
use crate::data_buf::{decode_data_buf, latest_rare_record, DecodeWarningKind, SanityFailure};
use crate::readings::{monitor_sample_from, DeviceStatus};
use crate::units::DisplayUnits;

fn with_errors(seq: u8, count_err: u16, dose_err: u16) -> Vec<u8> {
    [
        header(seq, 0, 0, i32::from(seq) * 100),
        16.0f32.to_le_bytes().to_vec(),
        0.000_01f32.to_le_bytes().to_vec(),
        count_err.to_le_bytes().to_vec(),
        dose_err.to_le_bytes().to_vec(),
        vec![0, 0, 0],
    ]
    .concat()
}

#[test]
fn absurd_error_percent_is_rejected_and_does_not_become_the_rate() {
    let bytes = [with_errors(1, 48, 150), with_errors(2, 10, 0x9000)].concat();
    let frame = decode_data_buf(&bytes);
    assert_eq!(frame.records.len(), 1);
    assert_eq!(frame.rejected_records(), 1);
    assert!(frame.warnings.iter().any(|warning| {
        matches!(warning.kind, DecodeWarningKind::SanityRejected(SanityFailure::ErrorPercentOutOfRange))
    }));
    let sample = monitor_sample_from(&frame, DisplayUnits::default(), Vec::new());
    assert_eq!(sample.rates.len(), 1);
    assert!((sample.rates[0].dose_rate_err_pct - 15.0).abs() < 0.001);
    assert!((sample.rates[0].count_rate_err_pct - 4.8).abs() < 0.001);
}

#[test]
fn newest_rare_record_follows_sequence_not_buffer_order() {
    let later_but_older = [rare(10, 99.0, 20.0), rare(3, 80.0, 21.0)].concat();
    let frame = decode_data_buf(&later_but_older);
    let newest = latest_rare_record(&frame).expect("rare");
    assert!((newest.battery_percent - 99.0).abs() < 0.001);
    let wrapped = decode_data_buf(&[rare(2, 70.0, 20.0), rare(250, 40.0, 21.0)].concat());
    let wrapped_rare = latest_rare_record(&wrapped).expect("wrapped rare");
    assert!((wrapped_rare.battery_percent - 70.0).abs() < 0.001);
    let status = crate::readings::status_from_snapshot(&crate::data_buf::snapshot_from_frame(&frame));
    assert_eq!(status.battery_percent, Some(99.0));
    assert_eq!(status.rssi, DeviceStatus::default().rssi);
}

#[test]
fn ordinary_error_percents_stay_in_the_frame() {
    let frame = decode_data_buf(&real_time(4, 5.0, 0.001));
    assert_eq!(frame.rejected_records(), 0);
    assert_eq!(frame.records.len(), 1);
}
