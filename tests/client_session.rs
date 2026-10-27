//! Session open, retry, restore and link-level behaviour against a virtual device.

mod support;

use std::time::Duration;

use radiacode_lib::data_buf::DeviceTicks;
use radiacode_lib::protocol::{Command, DeviceVersions, FirmwareVersion, VirtSfr, VirtString};
use radiacode_lib::readings::RssiStatus;
use radiacode_lib::{Error, OpenOptions, SessionRestore};
use support::fixtures::sfr_file_reads;
use support::{open_device, try_open_device, try_open_with, ScriptedReply, VirtualDevice};

fn restore_fixture() -> SessionRestore {
    let version = |minor: u16, date: &str| FirmwareVersion { major: 4, minor, date: date.into() };
    SessionRestore {
        versions: DeviceVersions { boot: version(0, "boot"), target: version(9, "target") },
        spectrum_format_version: 1,
    }
}

fn timeouts(count: usize) -> VirtualDevice {
    (0..count)
        .fold(VirtualDevice::default(), |device, _| device.with_script(Command::SetExchange, ScriptedReply::timeout()))
}

#[tokio::test(start_paused = true)]
async fn open_initializes_session_and_caches_versions() {
    let (device, handle) = open_device(VirtualDevice::default()).await;
    assert_eq!(device.spectrum_format_version(), 1);
    assert_eq!(device.sfr_entries().len(), 2);
    let restore = device.session_restore().unwrap();
    assert_eq!((restore.versions.target.major, restore.versions.target.minor), (4, 12));
    let state = handle.device();
    assert_eq!(state.request_count(Command::SetExchange), 1);
    assert_eq!(state.request_count(Command::SetTime), 1);
    assert_eq!(state.register(VirtSfr::DeviceTime), Some(0));
    assert_eq!(state.request_count(Command::GetVersion), 1);
    assert_eq!(sfr_file_reads(&state), 1);
    drop(state);
    assert!(handle.drain_count() >= 4);
}

#[tokio::test(start_paused = true)]
async fn open_rejects_old_firmware_and_disconnects() {
    let (outcome, handle) = try_open_device(VirtualDevice::default().with_firmware(4, 7)).await;
    assert!(matches!(outcome, Err(Error::IncompatibleFirmware { major: 4, minor: 7 })));
    assert!(handle.is_disconnected());
}

#[tokio::test(start_paused = true)]
async fn open_can_ignore_firmware_check() {
    let options = OpenOptions::new().ignore_firmware_check(true);
    let (outcome, _handle) = try_open_with(VirtualDevice::default().with_firmware(4, 7), options).await;
    assert!(outcome.is_ok());
}

#[tokio::test(start_paused = true)]
async fn open_retries_transient_failures_then_succeeds() {
    let (outcome, handle) = try_open_device(timeouts(1)).await;
    assert!(outcome.is_ok());
    assert_eq!(handle.device().request_count(Command::SetExchange), 2);
}

#[tokio::test(start_paused = true)]
async fn open_gives_up_after_repeated_transient_failures() {
    let (outcome, handle) = try_open_device(timeouts(3)).await;
    assert!(matches!(outcome, Err(Error::Timeout)));
    assert!(handle.is_disconnected());
    assert_eq!(handle.device().request_count(Command::SetExchange), 3);
}

#[tokio::test(start_paused = true)]
async fn open_reads_spectrum_version_after_other_config_lines() {
    let configuration = b"Something=1\r\nSpecFormatVersion=0\n";
    let (device, _handle) =
        open_device(VirtualDevice::default().with_string(VirtString::Configuration, configuration)).await;
    assert_eq!(device.spectrum_format_version(), 0);
}

#[tokio::test(start_paused = true)]
async fn reconnect_restores_cached_session_without_version_query() {
    let options = OpenOptions::new().restore(restore_fixture());
    let (outcome, handle) = try_open_with(VirtualDevice::default().with_firmware(4, 7), options).await;
    let device = outcome.unwrap();
    assert_eq!(device.spectrum_format_version(), 1);
    assert_eq!(device.session_restore(), Some(restore_fixture()));
    assert_eq!(handle.device().request_count(Command::GetVersion), 0);
}

#[tokio::test(start_paused = true)]
async fn reconnect_uses_fewer_attempts() {
    let (outcome, handle) = try_open_with(timeouts(2), OpenOptions::new().restore(restore_fixture())).await;
    assert!(outcome.is_err());
    assert_eq!(handle.device().request_count(Command::SetExchange), 2);
}

#[tokio::test(start_paused = true)]
async fn mismatched_response_header_is_protocol_mismatch() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle.device().push_script(Command::GetStatus, ScriptedReply::RawResponse(vec![0xff, 0xff, 0, 0, 1, 0, 0, 0]));
    assert!(matches!(device.status().await, Err(Error::ProtocolMismatch { .. })));
}

#[tokio::test(start_paused = true)]
async fn connection_loss_surfaces_as_connection_closed() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle.device().push_script(Command::GetStatus, ScriptedReply::connection_closed());
    assert!(device.status().await.unwrap_err().is_connection_lost());
}

#[tokio::test(start_paused = true)]
async fn disconnect_releases_transport() {
    let (device, handle) = open_device(VirtualDevice::default()).await;
    device.disconnect().await.unwrap();
    assert!(handle.is_disconnected());
}

#[tokio::test(start_paused = true)]
async fn rssi_and_wall_clock_delegate() {
    let (mut device, _handle) = open_device(VirtualDevice::default()).await;
    assert_eq!(device.rssi().await, RssiStatus::Measured(-60));
    assert_eq!(device.sample_rssi().await, RssiStatus::Measured(-61));
    let expected = device.base_time() + Duration::from_secs(1);
    assert_eq!(device.wall_clock(DeviceTicks::new(100)), expected);
}

#[tokio::test(start_paused = true)]
async fn requests_use_rotating_sequence_numbers() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    device.status().await.unwrap();
    device.status().await.unwrap();
    let state = handle.device();
    let sequences: Vec<u8> = state.requests.iter().map(|request| request.seq).collect();
    assert!(sequences.iter().all(|seq| (0x80..=0x9f).contains(seq)));
    assert!(sequences.windows(2).all(|pair| pair[0] != pair[1]));
}
