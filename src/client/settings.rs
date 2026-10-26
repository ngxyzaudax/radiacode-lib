use super::{DeviceLanguage, RadiaCode};
use crate::config::MAX_BRIGHTNESS;
use crate::error::{Error, Result};
use crate::protocol::VirtSfr;

impl RadiaCode {
    /// Resets the accumulated dose.
    pub async fn dose_reset(&mut self) -> Result<()> {
        self.write_vsfr(VirtSfr::DoseReset, &[]).await
    }

    /// Enables or disables sound.
    pub async fn set_sound_on(&mut self, on: bool) -> Result<()> {
        self.write_vsfr_u32(VirtSfr::SoundOn, u32::from(on)).await
    }

    /// Enables or disables vibration.
    pub async fn set_vibro_on(&mut self, on: bool) -> Result<()> {
        self.write_vsfr_u32(VirtSfr::VibroOn, u32::from(on)).await
    }

    /// Sets display brightness, `0..=9`; larger values are rejected without contacting the device.
    pub async fn set_display_brightness(&mut self, brightness: u8) -> Result<()> {
        match brightness <= MAX_BRIGHTNESS {
            true => self.write_vsfr_u32(VirtSfr::DispBrt, u32::from(brightness)).await,
            false => Err(Error::InvalidArgument(format!("brightness {brightness} outside 0..={MAX_BRIGHTNESS}"))),
        }
    }

    /// Sets the user-interface language.
    pub async fn set_language(&mut self, language: DeviceLanguage) -> Result<()> {
        self.write_vsfr_u32(VirtSfr::DeviceLang, language.to_device_flag()).await
    }

    /// Switches the device on, or off when `on` is `false`.
    pub async fn set_device_on(&mut self, on: bool) -> Result<()> {
        self.write_vsfr_u32(VirtSfr::DeviceOn, u32::from(on)).await
    }
}
