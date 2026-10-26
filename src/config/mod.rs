//! Device settings: alarm thresholds, display, sound, vibration and LEDs.
//!
//! Everything here is a pure codec between typed settings and VSFR register
//! words; [`RadiaCode`](crate::client::RadiaCode) performs the actual reads and writes.

mod alarms;
mod device_config;
mod enums;
mod signal_flags;

#[cfg(test)]
mod tests;

pub use alarms::{
    alarm_limits_from_words, alarm_register_pairs, updated_alarm_pairs, AlarmLimits, AlarmLimitsUpdate, ALARM_REGISTERS,
};
pub use device_config::{
    config_register_pairs, device_config_from_words, with_light_bit, DeviceConfig, LightControl, CONFIG_REGISTERS,
    DEVICE_CTRL_LIGHT, MAX_BRIGHTNESS,
};
pub use enums::{AlarmSignalMode, BacklightOffTime, DisplayDirection};
pub use signal_flags::SignalFlags;
