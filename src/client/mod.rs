//! Asynchronous high-level client for one connected device.

mod batch;
mod config;
mod exec;
mod info;
mod init;
mod monitor;
mod open;
mod settings;
mod types;

use std::time::SystemTime;

pub use types::{DeviceLanguage, DeviceMetadata, OpenOptions, SessionRestore};

use crate::data_buf::DeviceTicks;
use crate::error::Result;
use crate::protocol::catalog::SfrCatalogEntry;
use crate::protocol::{DeviceVersions, Sequence};
use crate::spectrum::SpectrumFormat;
use crate::transport::BoxedTransport;

/// An initialised session with a RadiaCode device over any [`Transport`](crate::transport::Transport).
///
/// Obtain one with [`RadiaCode::open`] on a transport, or with
/// [`discovery::connect`](crate::discovery::connect) for an endpoint. Every
/// method takes `&mut self`; commands are strictly sequential on the link.
pub struct RadiaCode {
    transport: BoxedTransport,
    sequence: Sequence,
    spectrum_format_version: u32,
    versions: Option<DeviceVersions>,
    base_time: SystemTime,
    sfr_entries: Vec<SfrCatalogEntry>,
}

impl std::fmt::Debug for RadiaCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RadiaCode")
            .field("spectrum_format_version", &self.spectrum_format_version)
            .field("versions", &self.versions)
            .field("base_time", &self.base_time)
            .field("sfr_entries", &self.sfr_entries.len())
            .finish_non_exhaustive()
    }
}

impl RadiaCode {
    /// Closes the underlying transport.
    pub async fn disconnect(self) -> Result<()> {
        self.transport.disconnect().await
    }

    /// Last known link RSSI (Bluetooth only).
    pub async fn rssi(&mut self) -> crate::readings::RssiStatus {
        self.transport.link_rssi().await
    }

    /// Samples a fresh link RSSI (Bluetooth only).
    pub async fn sample_rssi(&mut self) -> crate::readings::RssiStatus {
        self.transport.sample_link_rssi().await
    }

    /// Discards stale frames buffered on the link.
    pub async fn drain_transport(&mut self) {
        self.transport.drain_link().await;
    }

    /// Facts needed to restore this session on reconnect.
    pub fn session_restore(&self) -> Option<SessionRestore> {
        self.versions
            .clone()
            .map(|versions| SessionRestore { versions, spectrum_format_version: self.spectrum_format_version })
    }

    /// Firmware versions read at open time.
    pub fn versions(&self) -> Option<&DeviceVersions> {
        self.versions.as_ref()
    }

    /// `SpecFormatVersion` reported by the device configuration.
    pub fn spectrum_format_version(&self) -> u32 {
        self.spectrum_format_version
    }

    /// Spectrum encoding used by the device; fails for versions this crate cannot decode.
    pub fn spectrum_format(&self) -> Result<SpectrumFormat> {
        SpectrumFormat::try_from(self.spectrum_format_version)
    }

    /// Estimated wall-clock time corresponding to device tick zero.
    pub fn base_time(&self) -> SystemTime {
        self.base_time
    }

    /// Register directory read from the device `SFR_FILE` at open time.
    pub fn sfr_entries(&self) -> &[SfrCatalogEntry] {
        &self.sfr_entries
    }

    /// Converts device ticks into estimated wall-clock time.
    pub fn wall_clock(&self, ticks: DeviceTicks) -> SystemTime {
        ticks.wall_clock_time(self.base_time)
    }
}
