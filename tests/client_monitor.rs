//! DataBuf monitoring, live rates and device status against a virtual device.

mod support;

use radiacode_lib::data_buf::DataBufCursor;
use radiacode_lib::protocol::Command;
use radiacode_lib::{CountDisplayUnit, Error};
use support::encode::{rare_record, real_time_record};
use support::fixtures::{device_with_frame, frame_with_rare, CPM_SIEVERT};
use support::open_device;

#[tokio::test(start_paused = true)]
async fn poll_monitor_converts_records_and_uses_rare_status() {
    let (mut device, handle) = open_device(device_with_frame(&frame_with_rare())).await;
    let mut cursor = DataBufCursor::default();
    let (sample, status) = device.poll_monitor(CPM_SIEVERT, &mut cursor, false).await.unwrap();
    assert_eq!(sample.rates.len(), 2);
    assert_eq!(sample.rates[0].count_rate, 300.0);
    assert!((sample.rates[0].dose_rate - 1.0).abs() < 1e-4);
    assert_eq!(sample.rates[0].count_rate_err_pct, 5.0);
    assert_eq!(sample.rates[0].dose_rate_err_pct, 7.0);
    let accumulated = sample.accumulated.unwrap();
    assert_eq!(accumulated.duration_secs, 3600);
    assert!((accumulated.dose - 25.0).abs() < 1e-3);
    assert_eq!((status.battery_percent, status.temperature_c), (Some(87.5), Some(24.5)));
    assert_eq!(status.rssi_dbm, Some(-60));
    assert_eq!((sample.decode_warnings, sample.rejected_records), (0, 0));
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 0);
}

#[tokio::test(start_paused = true)]
async fn poll_monitor_reads_temperature_register_without_rare_record() {
    let (mut device, handle) = open_device(device_with_frame(&real_time_record(1, 100, 5.0, 0.0001))).await;
    let (sample, status) = device.poll_monitor(CPM_SIEVERT, &mut DataBufCursor::default(), true).await.unwrap();
    assert!(sample.accumulated.is_none());
    assert_eq!(status.battery_percent, None);
    assert_eq!(status.temperature_c, Some(31.0));
    assert_eq!(status.rssi_dbm, Some(-61));
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 1);
}

#[tokio::test(start_paused = true)]
async fn poll_monitor_rejects_out_of_range_rare_values() {
    let frame = [real_time_record(1, 100, 5.0, 0.0001), rare_record(2, 200, 160.0, 24.5)].concat();
    let (mut device, _handle) = open_device(device_with_frame(&frame)).await;
    let (sample, status) = device.poll_monitor(CPM_SIEVERT, &mut DataBufCursor::default(), false).await.unwrap();
    assert_eq!(sample.rejected_records, 1);
    assert_eq!(status.battery_percent, None);
    assert_eq!(status.temperature_c, Some(31.0));
}

#[tokio::test(start_paused = true)]
async fn poll_monitor_reports_sequence_gaps_across_polls() {
    let (mut device, handle) = open_device(device_with_frame(&real_time_record(1, 100, 5.0, 0.0001))).await;
    let mut cursor = DataBufCursor::default();
    device.poll_monitor(CPM_SIEVERT, &mut cursor, false).await.unwrap();
    let next = real_time_record(5, 200, 5.0, 0.0001);
    handle.device().strings.insert(u32::from(radiacode_lib::protocol::VirtString::DataBuf), next);
    let (sample, _) = device.poll_monitor(CPM_SIEVERT, &mut cursor, false).await.unwrap();
    assert_eq!(sample.seq_gaps.len(), 1);
    assert_eq!(cursor.total_lost(), 3);
}

#[tokio::test(start_paused = true)]
async fn live_rates_use_latest_real_time_record() {
    let (mut device, _handle) = open_device(device_with_frame(&frame_with_rare())).await;
    let rates = device.live_rates(CPM_SIEVERT).await.unwrap();
    assert_eq!(rates.count_rate, 360.0);
    assert!((rates.dose_rate - 2.0).abs() < 1e-4);
    assert_eq!(rates.count_unit, CountDisplayUnit::Cpm);
}

#[tokio::test(start_paused = true)]
async fn live_rates_report_pending_when_buffer_has_no_rates() {
    let (mut device, _handle) = open_device(device_with_frame(&[])).await;
    assert!(matches!(device.live_rates(CPM_SIEVERT).await, Err(Error::MonitorDataPending)));
}

#[tokio::test(start_paused = true)]
async fn device_status_reads_rare_record_from_buffer() {
    let (mut device, _handle) = open_device(device_with_frame(&frame_with_rare())).await;
    let status = device.device_status(false).await.unwrap();
    assert_eq!((status.battery_percent, status.temperature_c), (Some(87.5), Some(24.5)));
}
