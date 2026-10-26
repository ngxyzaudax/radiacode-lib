use crate::protocol::DeviceVersions;
use crate::spectrum::EnergyCalibration;

/// Cached session facts that let a reconnect skip firmware and configuration discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionRestore {
    /// Firmware versions read during the original session.
    pub versions: DeviceVersions,
    /// Spectrum format version read during the original session.
    pub spectrum_format_version: u32,
}

/// Options for [`RadiaCode::open`](super::RadiaCode::open).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpenOptions {
    /// Skip the minimum firmware version check.
    pub ignore_firmware_check: bool,
    /// Restore a previous session instead of re-reading firmware and configuration.
    pub restore: Option<SessionRestore>,
}

impl OpenOptions {
    /// Default options: firmware check enabled, fresh session.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables the firmware version check.
    pub fn ignore_firmware_check(self, ignore: bool) -> Self {
        Self { ignore_firmware_check: ignore, ..self }
    }

    /// Restores `restore` on open.
    pub fn restore(self, restore: SessionRestore) -> Self {
        Self { restore: Some(restore), ..self }
    }
}

/// Identity and calibration summary of a connected device.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceMetadata {
    /// Serial number such as `RC-110-006806`.
    pub serial: String,
    /// Model such as `RC-110`.
    pub model: String,
    /// Firmware versions.
    pub versions: DeviceVersions,
    /// Spectrum energy calibration.
    pub energy_calibration: EnergyCalibration,
}

impl DeviceMetadata {
    /// Target firmware as `major.minor (date)`.
    pub fn firmware_label(&self) -> String {
        let target = &self.versions.target;
        format!("{}.{} ({})", target.major, target.minor, target.date)
    }
}

/// User-interface language of the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceLanguage {
    /// Russian.
    Russian,
    /// English.
    English,
}

impl DeviceLanguage {
    /// Encodes the `DEVICE_LANG` register value.
    pub fn to_device_flag(self) -> u32 {
        match self {
            Self::Russian => 0,
            Self::English => 1,
        }
    }
}
