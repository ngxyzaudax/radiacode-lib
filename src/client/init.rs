use std::time::{Duration, SystemTime};

use tracing::{debug, info, warn};

use super::{OpenOptions, RadiaCode, SessionRestore};
use crate::error::{Error, Result};
use crate::protocol::catalog::{parse_configuration_ini, parse_sfr_file, validate_entries};
use crate::protocol::{set_time_payload_now, spectrum_format_version, Command, VirtSfr};

const INIT_SETTLE: Duration = Duration::from_millis(250);
const BASE_TIME_OFFSET: Duration = Duration::from_secs(128);
const EXCHANGE_HANDSHAKE: [u8; 4] = [0x01, 0xff, 0x12, 0xff];

impl RadiaCode {
    pub(super) async fn initialize(&mut self, options: &OpenOptions) -> Result<()> {
        debug!(reconnect = options.restore.is_some(), "initializing radiacode session");
        self.execute_raw(Command::SetExchange, &EXCHANGE_HANDSHAKE).await?;
        self.settle_link().await;
        self.sync_device_clock().await?;
        self.base_time = SystemTime::now() + BASE_TIME_OFFSET;
        self.load_sfr_entries().await;
        self.settle_link().await;
        match &options.restore {
            Some(restore) => {
                self.restore_session(restore);
                Ok(())
            }
            None => self.discover_session(options.ignore_firmware_check).await,
        }
    }

    /// Sets the device clock to local time and resets the tick counter.
    pub async fn sync_device_clock(&mut self) -> Result<()> {
        self.execute_raw(Command::SetTime, &set_time_payload_now()).await?;
        self.write_vsfr(VirtSfr::DeviceTime, &0u32.to_le_bytes()).await
    }

    async fn settle_link(&mut self) {
        self.drain_transport().await;
        tokio::time::sleep(INIT_SETTLE).await;
        self.drain_transport().await;
    }

    async fn load_sfr_entries(&mut self) {
        match self.sfr_file_text().await {
            Ok(text) => self.sfr_entries = parse_sfr_file(&text),
            Err(error) => debug!(%error, "SFR_FILE unavailable; using static register layout"),
        }
    }

    fn restore_session(&mut self, restore: &SessionRestore) {
        self.versions = Some(restore.versions.clone());
        self.spectrum_format_version = restore.spectrum_format_version;
        info!(spectrum_format_version = self.spectrum_format_version, "session restored from cache");
    }

    async fn discover_session(&mut self, ignore_firmware_check: bool) -> Result<()> {
        let versions = self.fw_version().await?;
        let (major, minor) = (versions.target.major, versions.target.minor);
        info!(major, minor, "device firmware");
        let supported = versions.is_supported();
        self.versions = Some(versions);
        if !ignore_firmware_check && !supported {
            return Err(Error::IncompatibleFirmware { major, minor });
        }
        let configuration = self.configuration().await?;
        self.spectrum_format_version = spectrum_format_from(&configuration);
        self.log_catalog_drift();
        info!(spectrum_format_version = self.spectrum_format_version, "device initialized");
        Ok(())
    }

    fn log_catalog_drift(&self) {
        let drifts = validate_entries(&self.sfr_entries);
        match self.sfr_entries.is_empty() {
            true => debug!("SFR_FILE validation skipped: no parseable entries"),
            false => drifts.iter().for_each(|drift| {
                warn!(register = ?drift.register, message = %drift.message, "register catalog differs from device SFR_FILE");
            }),
        }
    }
}

fn spectrum_format_from(configuration: &str) -> u32 {
    match parse_configuration_ini(configuration).spec_format_version {
        0 => spectrum_format_version(configuration),
        other => other,
    }
}
