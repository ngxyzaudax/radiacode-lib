use std::time::Duration;

use tracing::warn;

use super::endpoint::{DeviceEndpoint, DiscoveredDevice, TransportKind};
use super::merge::merge_discovered;
use crate::client::{OpenOptions, RadiaCode};
use crate::error::Result;
use crate::transport::BoxedTransport;

const DEFAULT_BLUETOOTH_SCAN: Duration = Duration::from_secs(5);

/// Which transports [`discover_devices`] scans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryOptions {
    /// Enumerate USB devices.
    pub usb: bool,
    /// Scan for Bluetooth LE advertisements.
    pub bluetooth: bool,
    /// Bluetooth scan duration.
    pub bluetooth_scan: Duration,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            usb: cfg!(feature = "usb"),
            bluetooth: cfg!(feature = "bluetooth"),
            bluetooth_scan: DEFAULT_BLUETOOTH_SCAN,
        }
    }
}

/// Scans the enabled transports and merges the results, USB first.
///
/// A transport that fails to scan is logged and skipped; the call fails only
/// when every requested transport failed.
pub async fn discover_devices(options: &DiscoveryOptions) -> Result<Vec<DiscoveredDevice>> {
    let usb = match options.usb {
        true => Some(scan_usb().await),
        false => None,
    };
    let bluetooth = match options.bluetooth {
        true => Some(scan_bluetooth(options.bluetooth_scan).await),
        false => None,
    };
    combine_scans(usb, bluetooth)
}

/// Opens a raw transport to `endpoint`.
pub async fn open_transport(endpoint: &DeviceEndpoint) -> Result<BoxedTransport> {
    match endpoint {
        DeviceEndpoint::Usb { serial } => open_usb(serial).await,
        DeviceEndpoint::Bluetooth { address } => open_bluetooth(address).await,
    }
}

/// Opens a transport to `endpoint` and initialises a [`RadiaCode`] session on it.
pub async fn connect(endpoint: &DeviceEndpoint, options: OpenOptions) -> Result<RadiaCode> {
    RadiaCode::open(open_transport(endpoint).await?, options).await
}

fn combine_scans(
    usb: Option<Result<Vec<DiscoveredDevice>>>,
    bluetooth: Option<Result<Vec<DiscoveredDevice>>>,
) -> Result<Vec<DiscoveredDevice>> {
    match (usb, bluetooth) {
        (Some(Err(usb_error)), Some(Err(ble_error))) => {
            warn!(%usb_error, "usb scan failed");
            Err(ble_error)
        }
        (Some(Err(error)), None) | (None, Some(Err(error))) => Err(error),
        (usb, bluetooth) => Ok(merge_discovered(
            scan_or_empty(usb, TransportKind::Usb),
            scan_or_empty(bluetooth, TransportKind::Bluetooth),
        )),
    }
}

fn scan_or_empty(scan: Option<Result<Vec<DiscoveredDevice>>>, kind: TransportKind) -> Vec<DiscoveredDevice> {
    match scan {
        Some(Ok(devices)) => devices,
        Some(Err(error)) => {
            warn!(transport = %kind, %error, "device scan failed; continuing without this transport");
            Vec::new()
        }
        None => Vec::new(),
    }
}

#[cfg(not(all(feature = "usb", feature = "bluetooth")))]
fn feature_disabled(kind: TransportKind) -> crate::error::Error {
    crate::error::Error::TransportUnavailable(format!("{kind} support is not compiled in; enable the crate feature"))
}

#[cfg(feature = "usb")]
async fn scan_usb() -> Result<Vec<DiscoveredDevice>> {
    crate::usb::scan_usb_devices().await
}

#[cfg(not(feature = "usb"))]
async fn scan_usb() -> Result<Vec<DiscoveredDevice>> {
    Err(feature_disabled(TransportKind::Usb))
}

#[cfg(feature = "usb")]
async fn open_usb(serial: &str) -> Result<BoxedTransport> {
    crate::usb::UsbTransport::open(serial).await.map(|transport| Box::new(transport) as BoxedTransport)
}

#[cfg(not(feature = "usb"))]
async fn open_usb(_serial: &str) -> Result<BoxedTransport> {
    Err(feature_disabled(TransportKind::Usb))
}

#[cfg(feature = "bluetooth")]
async fn scan_bluetooth(duration: Duration) -> Result<Vec<DiscoveredDevice>> {
    crate::bluetooth::scan_bluetooth_devices(duration).await
}

#[cfg(not(feature = "bluetooth"))]
async fn scan_bluetooth(_duration: Duration) -> Result<Vec<DiscoveredDevice>> {
    Err(feature_disabled(TransportKind::Bluetooth))
}

#[cfg(feature = "bluetooth")]
async fn open_bluetooth(address: &str) -> Result<BoxedTransport> {
    crate::bluetooth::BluetoothTransport::connect(address).await.map(|transport| Box::new(transport) as BoxedTransport)
}

#[cfg(not(feature = "bluetooth"))]
async fn open_bluetooth(_address: &str) -> Result<BoxedTransport> {
    Err(feature_disabled(TransportKind::Bluetooth))
}
