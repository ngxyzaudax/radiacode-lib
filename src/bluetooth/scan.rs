use std::time::Duration;

use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use futures::future::join_all;
use tracing::{debug, info};

use super::adapter::{default_adapter, peripheral_key, scan_peripherals};
use super::advertisement::{discovered_device, is_radiacode_advertisement, Advertisement};
use super::error::{ble_result, BleError};
use crate::discovery::{sort_and_dedup, DiscoveredDevice};
use crate::error::Result;

/// Scans for RadiaCode advertisements for `duration` on the first Bluetooth adapter.
///
/// A peripheral matches when it advertises the RadiaCode GATT service or a
/// local name containing `radiacode`.
pub async fn scan_bluetooth_devices(duration: Duration) -> Result<Vec<DiscoveredDevice>> {
    ble_result(scan(duration).await)
}

async fn scan(duration: Duration) -> std::result::Result<Vec<DiscoveredDevice>, BleError> {
    info!(?duration, "bluetooth scan started");
    let adapter = default_adapter().await?;
    let peripherals = scan_peripherals(&adapter, duration).await?;
    debug!(peripheral_count = peripherals.len(), "bluetooth scan collected peripherals");
    let matched = join_all(peripherals.iter().map(advertisement_of)).await;
    let found = sort_and_dedup(
        matched.into_iter().flatten().filter(is_radiacode_advertisement).map(discovered_device).collect(),
    );
    info!(count = found.len(), "bluetooth scan completed");
    Ok(found)
}

async fn advertisement_of(peripheral: &Peripheral) -> Option<Advertisement> {
    let properties = peripheral.properties().await.ok().flatten()?;
    Some(Advertisement {
        address: peripheral_key(peripheral),
        local_name: properties.local_name,
        services: properties.services,
        rssi: properties.rssi,
    })
}
