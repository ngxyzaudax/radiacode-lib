use super::alarms::{alarm_limits_from_words, alarm_register_pairs, AlarmLimits, ALARM_REGISTERS};
use super::enums::{AlarmSignalMode, BacklightOffTime, DisplayDirection};
use super::signal_flags::SignalFlags;
use crate::error::{Error, Result};
use crate::protocol::VirtSfr;

/// Bit of `DEVICE_CTRL` that switches the LEDs on firmware without `LEDS_ON`.
pub const DEVICE_CTRL_LIGHT: u32 = 1 << 3;

/// Maximum display brightness level.
pub const MAX_BRIGHTNESS: u8 = 9;

/// Registers read by [`device_config_from_words`], in order.
pub const CONFIG_REGISTERS: [VirtSfr; 16] = [
    ALARM_REGISTERS[0],
    ALARM_REGISTERS[1],
    ALARM_REGISTERS[2],
    ALARM_REGISTERS[3],
    ALARM_REGISTERS[4],
    ALARM_REGISTERS[5],
    ALARM_REGISTERS[6],
    ALARM_REGISTERS[7],
    VirtSfr::AlarmMode,
    VirtSfr::DispBrt,
    VirtSfr::DispOffTime,
    VirtSfr::DispDir,
    VirtSfr::SoundOn,
    VirtSfr::SoundCtrl,
    VirtSfr::VibroOn,
    VirtSfr::VibroCtrl,
];

/// How the LED state is controlled on the connected firmware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LightControl {
    /// Dedicated `LEDS_ON` register listed in `SFR_FILE`.
    LedsOnRegister,
    /// Bit 3 of `DEVICE_CTRL` on older firmware.
    DeviceCtrlBit,
}

/// User-facing device settings.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceConfig {
    /// Alarm thresholds and display units.
    pub alarms: AlarmLimits,
    /// Alarm signalling mode.
    pub alarm_mode: AlarmSignalMode,
    /// Display brightness, `0..=9`.
    pub brightness: u8,
    /// Backlight timeout.
    pub backlight_off: BacklightOffTime,
    /// Screen orientation.
    pub display_dir: DisplayDirection,
    /// Sound enabled.
    pub sound_on: bool,
    /// Vibration enabled.
    pub vibro_on: bool,
    /// LEDs enabled.
    pub leds_on: bool,
    /// How `leds_on` is stored on the device.
    pub light_control: LightControl,
    /// Events that trigger a sound.
    pub sound_ctrl: SignalFlags,
    /// Events that trigger vibration.
    pub vibro_ctrl: SignalFlags,
}

/// Decodes [`CONFIG_REGISTERS`] values plus the LED state into a [`DeviceConfig`].
pub fn device_config_from_words(values: &[u32], leds_on: bool, light_control: LightControl) -> Result<DeviceConfig> {
    let words: [u32; 16] =
        values.try_into().map_err(|_| Error::BufferUnderrun { need: CONFIG_REGISTERS.len(), have: values.len() })?;
    let [_, _, _, _, _, _, _, _, mode, brightness, off, dir, sound, sound_ctrl, vibro, vibro_ctrl] = words;
    Ok(DeviceConfig {
        alarms: alarm_limits_from_words(&words)?,
        alarm_mode: AlarmSignalMode::from_raw(mode),
        brightness: u8::try_from(brightness.min(u32::from(MAX_BRIGHTNESS))).unwrap_or(MAX_BRIGHTNESS),
        backlight_off: BacklightOffTime::from_raw(off),
        display_dir: DisplayDirection::from_raw(dir),
        sound_on: sound != 0,
        sound_ctrl: SignalFlags::from_raw(sound_ctrl),
        vibro_on: vibro != 0,
        vibro_ctrl: SignalFlags::from_raw(vibro_ctrl).vibro_events(),
        leds_on,
        light_control,
    })
}

/// Encodes a [`DeviceConfig`] as a batch of register writes.
///
/// When LEDs are controlled through `DEVICE_CTRL`, use [`with_light_bit`] separately.
pub fn config_register_pairs(config: &DeviceConfig) -> Vec<(VirtSfr, u32)> {
    let settings = [
        (VirtSfr::AlarmMode, config.alarm_mode.as_raw()),
        (VirtSfr::DispBrt, u32::from(config.brightness.min(MAX_BRIGHTNESS))),
        (VirtSfr::DispOffTime, config.backlight_off.as_raw()),
        (VirtSfr::DispDir, config.display_dir.as_raw()),
        (VirtSfr::SoundOn, u32::from(config.sound_on)),
        (VirtSfr::SoundCtrl, config.sound_ctrl.as_raw()),
        (VirtSfr::VibroOn, u32::from(config.vibro_on)),
        (VirtSfr::VibroCtrl, config.vibro_ctrl.vibro_events().as_raw()),
    ];
    let leds =
        (config.light_control == LightControl::LedsOnRegister).then_some((VirtSfr::LedsOn, u32::from(config.leds_on)));
    alarm_register_pairs(&config.alarms).into_iter().chain(settings).chain(leds).collect()
}

/// Returns `device_ctrl` with the LED bit set or cleared.
pub fn with_light_bit(device_ctrl: u32, leds_on: bool) -> u32 {
    match leds_on {
        true => device_ctrl | DEVICE_CTRL_LIGHT,
        false => device_ctrl & !DEVICE_CTRL_LIGHT,
    }
}
