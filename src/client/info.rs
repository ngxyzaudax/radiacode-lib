use std::time::Duration;

use tracing::debug;

use super::{DeviceMetadata, RadiaCode};
use crate::discovery::model_from_serial;
use crate::error::{Error, Result};
use crate::protocol::catalog::decode_cp1251;
use crate::protocol::{
    decode_fw_signature, decode_fw_version, decode_hw_serial, decode_retcode_ok, decode_status, decode_text,
    virt_string_write_args, Command, DeviceVersions, FirmwareSignature, VirtString,
};
use crate::spectrum::{decode_energy_calibration, decode_spectrum, EnergyCalibration, Spectrum};

const FW_VERSION_ATTEMPTS: usize = 3;
const FW_VERSION_RETRY_DELAY: Duration = Duration::from_millis(250);

impl RadiaCode {
    /// Reads boot and target firmware versions, retrying transient parse failures.
    pub async fn fw_version(&mut self) -> Result<DeviceVersions> {
        let mut last_error = Error::Timeout;
        for attempt in 0..FW_VERSION_ATTEMPTS {
            if attempt > 0 {
                debug!(attempt, "retrying fw_version after transient failure");
                tokio::time::sleep(FW_VERSION_RETRY_DELAY).await;
            }
            self.drain_transport().await;
            let response = self.execute_raw(Command::GetVersion, &[]).await?;
            match decode_fw_version(response) {
                Err(error) if error.is_transient() => last_error = error,
                outcome => return outcome,
            }
        }
        Err(last_error)
    }

    /// Hardware (MCU) serial number as dash-separated hex groups.
    pub async fn hw_serial_number(&mut self) -> Result<String> {
        self.execute_raw(Command::GetSerial, &[]).await.and_then(decode_hw_serial)
    }

    /// Device serial number, such as `RC-110-006806`.
    pub async fn serial_number(&mut self) -> Result<String> {
        self.read_text(VirtString::SerialNumber).await
    }

    /// Device configuration INI text, decoded from CP1251.
    pub async fn configuration(&mut self) -> Result<String> {
        let payload = self.read_virt_string(VirtString::Configuration).await?;
        Ok(decode_cp1251(payload.data()))
    }

    /// Raw `SFR_FILE` register directory text.
    pub async fn sfr_file_text(&mut self) -> Result<String> {
        self.read_text(VirtString::SfrFile).await
    }

    /// Text message stored on the device.
    pub async fn text_message(&mut self) -> Result<String> {
        self.read_text(VirtString::TextMessage).await
    }

    /// Raw device status word.
    pub async fn status(&mut self) -> Result<u32> {
        self.execute_raw(Command::GetStatus, &[]).await.and_then(decode_status)
    }

    /// Firmware signature, file name and id string.
    pub async fn fw_signature(&mut self) -> Result<FirmwareSignature> {
        self.execute_raw(Command::FwSignature, &[]).await.and_then(decode_fw_signature)
    }

    /// Serial, model, firmware versions and energy calibration.
    pub async fn metadata(&mut self) -> Result<DeviceMetadata> {
        let serial = self.serial_number().await?;
        let energy_calibration = self.energy_calibration().await?;
        let versions = self
            .versions
            .clone()
            .ok_or_else(|| Error::mismatch("initialized session", "no cached firmware version"))?;
        Ok(DeviceMetadata { model: model_from_serial(&serial), serial, versions, energy_calibration })
    }

    /// Current spectrum since the last spectrum reset.
    pub async fn spectrum(&mut self) -> Result<Spectrum> {
        self.read_spectrum(VirtString::Spectrum).await
    }

    /// Accumulated spectrum.
    pub async fn spectrum_accum(&mut self) -> Result<Spectrum> {
        self.read_spectrum(VirtString::SpecAccum).await
    }

    /// Difference spectrum since the previous `SpecDiff` read.
    pub async fn spectrum_diff(&mut self) -> Result<Spectrum> {
        self.read_spectrum(VirtString::SpecDiff).await
    }

    /// Clears the current spectrum.
    pub async fn spectrum_reset(&mut self) -> Result<()> {
        let args = virt_string_write_args(VirtString::Spectrum, &[])?;
        self.execute_raw(Command::WrVirtString, &args).await.and_then(decode_retcode_ok)
    }

    /// Reads the energy calibration coefficients.
    pub async fn energy_calibration(&mut self) -> Result<EnergyCalibration> {
        self.read_virt_string(VirtString::EnergyCalib).await.and_then(decode_energy_calibration)
    }

    /// Writes new energy calibration coefficients.
    pub async fn set_energy_calibration(&mut self, calibration: EnergyCalibration) -> Result<()> {
        let args = virt_string_write_args(VirtString::EnergyCalib, &calibration.to_bytes())?;
        self.execute_raw(Command::WrVirtString, &args).await.and_then(decode_retcode_ok)
    }

    async fn read_spectrum(&mut self, id: VirtString) -> Result<Spectrum> {
        let format = self.spectrum_format()?;
        self.read_virt_string(id).await.and_then(|payload| decode_spectrum(payload, format))
    }

    async fn read_text(&mut self, id: VirtString) -> Result<String> {
        self.read_virt_string(id).await.map(|payload| decode_text(&payload))
    }
}
