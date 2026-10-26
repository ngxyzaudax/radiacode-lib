use tracing::debug;

use super::RadiaCode;
use crate::config::{
    alarm_limits_from_words, config_register_pairs, device_config_from_words, updated_alarm_pairs, with_light_bit,
    AlarmLimits, AlarmLimitsUpdate, DeviceConfig, LightControl, ALARM_REGISTERS, CONFIG_REGISTERS, DEVICE_CTRL_LIGHT,
};
use crate::error::Result;
use crate::protocol::catalog::sfr_entries_support_leds_on;
use crate::protocol::VirtSfr;

impl RadiaCode {
    /// Reads alarm thresholds and display units.
    pub async fn alarm_limits(&mut self) -> Result<AlarmLimits> {
        let values = self.read_vsfr_batch(&ALARM_REGISTERS).await?;
        alarm_limits_from_words(&values)
    }

    /// Updates the provided alarm fields, leaving the others unchanged.
    pub async fn set_alarm_limits(&mut self, update: &AlarmLimitsUpdate) -> Result<()> {
        let current = self.alarm_limits().await?;
        let pairs = updated_alarm_pairs(&current, update);
        match pairs.is_empty() {
            true => Ok(()),
            false => self.write_vsfr_batch(&pairs).await,
        }
    }

    /// Reads every user setting.
    pub async fn load_device_config(&mut self) -> Result<DeviceConfig> {
        let values = self.read_vsfr_batch(&CONFIG_REGISTERS).await?;
        let (leds_on, light_control) = self.load_light_state().await?;
        let config = device_config_from_words(&values, leds_on, light_control)?;
        debug!(?config, "device config loaded");
        Ok(config)
    }

    /// Writes every user setting.
    pub async fn apply_device_config(&mut self, config: &DeviceConfig) -> Result<()> {
        self.write_vsfr_batch(&config_register_pairs(config)).await?;
        match config.light_control {
            LightControl::DeviceCtrlBit => self.apply_device_ctrl_light(config.leds_on).await,
            LightControl::LedsOnRegister => Ok(()),
        }
    }

    /// Light control method supported by the connected firmware.
    pub fn light_control(&self) -> LightControl {
        match sfr_entries_support_leds_on(&self.sfr_entries) {
            true => LightControl::LedsOnRegister,
            false => LightControl::DeviceCtrlBit,
        }
    }

    async fn load_light_state(&mut self) -> Result<(bool, LightControl)> {
        let control = self.light_control();
        let leds_on = match control {
            LightControl::LedsOnRegister => self.read_vsfr_optional(VirtSfr::LedsOn).await?.is_some_and(|v| v != 0),
            LightControl::DeviceCtrlBit => self.read_vsfr_u32(VirtSfr::DeviceCtrl).await? & DEVICE_CTRL_LIGHT != 0,
        };
        Ok((leds_on, control))
    }

    async fn apply_device_ctrl_light(&mut self, leds_on: bool) -> Result<()> {
        let current = self.read_vsfr_u32(VirtSfr::DeviceCtrl).await?;
        let next = with_light_bit(current, leds_on);
        match next == current {
            true => Ok(()),
            false => self.write_vsfr_u32(VirtSfr::DeviceCtrl, next).await,
        }
    }
}
