//! Alarm limits and device configuration against a virtual device.

mod support;

use radiacode_lib::config::{
    AlarmLimitsUpdate, AlarmSignalMode, BacklightOffTime, DeviceConfig, DisplayDirection, LightControl,
};
use radiacode_lib::protocol::{Command, VirtSfr, VirtString};
use radiacode_lib::{CountDisplayUnit, DoseDisplayUnit};
use support::fixtures::{configured_alarm_device, sfr_file_reads};
use support::{open_device, LEDS_SFR_FILE};

const DEVICE_CTRL_LIGHT: u32 = 1 << 3;

#[tokio::test(start_paused = true)]
async fn load_device_config_decodes_all_registers_via_device_ctrl() {
    let (mut device, handle) = open_device(configured_alarm_device()).await;
    let config = device.load_device_config().await.unwrap();
    assert_eq!(config.alarms.dose_unit, DoseDisplayUnit::MicroSievertPerHour);
    assert_eq!(config.alarms.count_unit, CountDisplayUnit::Cps);
    assert_eq!(config.alarms.l1_count_rate, 30.0);
    assert_eq!(config.alarm_mode, AlarmSignalMode::Once);
    assert_eq!(config.brightness, 9);
    assert_eq!(config.backlight_off, BacklightOffTime::Sec30);
    assert_eq!(config.display_dir, DisplayDirection::Left);
    assert!(config.sound_on && !config.vibro_on);
    assert!(!config.vibro_ctrl.clicks);
    assert_eq!(config.light_control, LightControl::DeviceCtrlBit);
    assert!(!config.leds_on);
    assert_eq!(sfr_file_reads(&handle.device()), 1);
}

#[tokio::test(start_paused = true)]
async fn load_device_config_uses_leds_on_register_when_catalog_lists_it() {
    let device = configured_alarm_device()
        .with_string(VirtString::SfrFile, LEDS_SFR_FILE.as_bytes())
        .with_register(VirtSfr::LedsOn, 1);
    let (mut device, _handle) = open_device(device).await;
    let config = device.load_device_config().await.unwrap();
    assert!(config.leds_on);
    assert_eq!(config.light_control, LightControl::LedsOnRegister);
}

#[tokio::test(start_paused = true)]
async fn apply_device_config_writes_registers_and_light_bit() {
    let (mut device, handle) = open_device(configured_alarm_device()).await;
    let config = DeviceConfig {
        brightness: 3,
        leds_on: true,
        display_dir: DisplayDirection::Right,
        ..device.load_device_config().await.unwrap()
    };
    device.apply_device_config(&config).await.unwrap();
    let state = handle.device();
    assert_eq!(state.register(VirtSfr::DispBrt), Some(3));
    assert_eq!(state.register(VirtSfr::DispDir), Some(1));
    assert_eq!(state.register(VirtSfr::DeviceCtrl), Some(0x35 | DEVICE_CTRL_LIGHT));
    assert_eq!(state.register(VirtSfr::LedsOn), None);
}

#[tokio::test(start_paused = true)]
async fn apply_device_config_skips_device_ctrl_write_when_unchanged() {
    let (mut device, handle) = open_device(configured_alarm_device()).await;
    let config = device.load_device_config().await.unwrap();
    let writes_before = handle.device().request_count(Command::WrVirtSfr);
    device.apply_device_config(&config).await.unwrap();
    assert_eq!(handle.device().request_count(Command::WrVirtSfr), writes_before);
}

#[tokio::test(start_paused = true)]
async fn apply_device_config_writes_leds_on_register_when_supported() {
    let device = configured_alarm_device()
        .with_string(VirtString::SfrFile, LEDS_SFR_FILE.as_bytes())
        .with_register(VirtSfr::LedsOn, 0);
    let (mut device, handle) = open_device(device).await;
    let config = DeviceConfig { leds_on: true, ..device.load_device_config().await.unwrap() };
    device.apply_device_config(&config).await.unwrap();
    assert_eq!(handle.device().register(VirtSfr::LedsOn), Some(1));
    assert_eq!(handle.device().register(VirtSfr::DeviceCtrl), Some(0x35));
}

#[tokio::test(start_paused = true)]
async fn alarm_limits_round_trip_through_update() {
    let (mut device, handle) = open_device(configured_alarm_device()).await;
    assert_eq!(device.alarm_limits().await.unwrap().l2_count_rate, 60.0);
    let update = AlarmLimitsUpdate {
        l1_count_rate: Some(45.0),
        count_unit: Some(CountDisplayUnit::Cps),
        ..AlarmLimitsUpdate::default()
    };
    device.set_alarm_limits(&update).await.unwrap();
    assert_eq!(handle.device().register(VirtSfr::CrLev1Cp10s), Some(450));
    assert_eq!(handle.device().register(VirtSfr::CrLev2Cp10s), Some(600));
    assert_eq!(device.alarm_limits().await.unwrap().l1_count_rate, 45.0);
}

#[tokio::test(start_paused = true)]
async fn empty_alarm_update_writes_nothing() {
    let (mut device, handle) = open_device(configured_alarm_device()).await;
    device.set_alarm_limits(&AlarmLimitsUpdate::default()).await.unwrap();
    assert_eq!(handle.device().request_count(Command::WrVirtSfrBatch), 0);
}

#[tokio::test(start_paused = true)]
async fn sync_device_clock_sets_time_and_resets_device_time() {
    let (mut device, handle) = open_device(configured_alarm_device()).await;
    handle.device().registers.insert(u32::from(VirtSfr::DeviceTime), 99);
    device.sync_device_clock().await.unwrap();
    assert_eq!(handle.device().request_count(Command::SetTime), 2);
    assert_eq!(handle.device().register(VirtSfr::DeviceTime), Some(0));
}
