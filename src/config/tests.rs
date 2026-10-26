use super::{
    alarm_limits_from_words, config_register_pairs, device_config_from_words, updated_alarm_pairs, with_light_bit,
    AlarmLimitsUpdate, AlarmSignalMode, BacklightOffTime, DisplayDirection, LightControl, SignalFlags, ALARM_REGISTERS,
    DEVICE_CTRL_LIGHT,
};
use crate::protocol::VirtSfr;
use crate::units::{CountDisplayUnit, DoseDisplayUnit};

const CONFIG_WORDS: [u32; 16] = [300, 600, 120, 300, 50000, 100000, 1, 0, 1, 12, 3, 2, 1, 0x1f9d, 0, 0x0302];

#[test]
fn backlight_direction_and_mode_round_trip() {
    assert!(BacklightOffTime::all().into_iter().all(|v| BacklightOffTime::from_raw(v.as_raw()) == v));
    assert_eq!(BacklightOffTime::from_raw(99), BacklightOffTime::Sec5);
    assert_eq!(BacklightOffTime::Min2.label(), "2 min");
    let directions = [DisplayDirection::Auto, DisplayDirection::Right, DisplayDirection::Left];
    assert!(directions.into_iter().all(|d| DisplayDirection::from_raw(d.as_raw()) == d));
    assert_eq!(DisplayDirection::from_raw(7), DisplayDirection::Auto);
    assert_eq!(AlarmSignalMode::from_raw(0), AlarmSignalMode::Continuous);
    assert_eq!(AlarmSignalMode::from_raw(5), AlarmSignalMode::Once);
}

#[test]
fn signal_flags_round_trip_and_reserved_bits() {
    let flags = SignalFlags::from_raw(0x1f9d);
    assert!(flags.buttons && !flags.clicks && flags.dose_rate_alarm1 && flags.connection);
    assert!(flags.power && flags.count_rate_alarm1 && flags.count_rate_out_of_scale);
    assert_eq!(flags.as_raw(), 0x1f9d);
    let all = SignalFlags::from_raw(u32::MAX);
    assert_eq!(all.reserved, 0xFFFF_E000);
    assert_eq!(all.as_raw(), u32::MAX);
    assert_eq!(all.vibro_events().as_raw(), !((1 << 1) | (1 << 8) | (1 << 9)));
}

#[test]
fn alarm_limits_decode_units_and_thresholds() {
    let limits = alarm_limits_from_words(&CONFIG_WORDS).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(limits.dose_unit, DoseDisplayUnit::MicroSievertPerHour);
    assert_eq!(limits.count_unit, CountDisplayUnit::Cps);
    assert_eq!((limits.l1_count_rate, limits.l2_count_rate), (30.0, 60.0));
    assert_eq!((limits.l1_dose_rate, limits.l2_dose_rate), (1.2, 3.0));
    assert_eq!((limits.l1_dose, limits.l2_dose), (500.0, 1000.0));
    assert!(alarm_limits_from_words(&[1, 2]).is_err());
}

#[test]
fn alarm_update_writes_only_provided_fields() {
    let current = alarm_limits_from_words(&CONFIG_WORDS).unwrap_or_else(|e| panic!("{e}"));
    let update =
        AlarmLimitsUpdate { l2_dose_rate: Some(5.0), count_unit: Some(CountDisplayUnit::Cpm), ..Default::default() };
    let pairs = updated_alarm_pairs(&current, &update);
    assert_eq!(pairs, vec![(VirtSfr::DrLev2UrH, 500), (VirtSfr::CrUnits, 1)]);
    assert!(updated_alarm_pairs(&current, &AlarmLimitsUpdate::default()).is_empty());
}

#[test]
fn device_config_decodes_and_reencodes() {
    let config = device_config_from_words(&CONFIG_WORDS, true, LightControl::LedsOnRegister);
    let config = config.unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(config.alarm_mode, AlarmSignalMode::Once);
    assert_eq!(config.brightness, 9);
    assert_eq!(config.backlight_off, BacklightOffTime::Sec30);
    assert_eq!(config.display_dir, DisplayDirection::Left);
    assert!(config.sound_on && !config.vibro_on && !config.vibro_ctrl.connection);
    let pairs = config_register_pairs(&config);
    assert_eq!(pairs.len(), 17);
    assert_eq!(pairs.first().map(|p| p.0), ALARM_REGISTERS.first().copied());
    assert!(pairs.contains(&(VirtSfr::DispBrt, 9)));
    assert_eq!(pairs.last(), Some(&(VirtSfr::LedsOn, 1)));
    let legacy = super::DeviceConfig { light_control: LightControl::DeviceCtrlBit, ..config };
    assert_eq!(config_register_pairs(&legacy).len(), 16);
    assert!(device_config_from_words(&CONFIG_WORDS[..15], false, LightControl::DeviceCtrlBit).is_err());
}

#[test]
fn light_bit_is_set_and_cleared() {
    assert_eq!(with_light_bit(0x01, true), 0x01 | DEVICE_CTRL_LIGHT);
    assert_eq!(with_light_bit(0xFF, false), 0xFF & !DEVICE_CTRL_LIGHT);
}
