use std::time::Duration;

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tracing::debug;

use super::address::{endpoint_key, key_matches, normalize_target};
use super::error::BleError;

const CONNECT_DISCOVERY: Duration = Duration::from_millis(800);

pub(super) async fn default_adapter() -> Result<Adapter, BleError> {
    let manager = Manager::new().await?;
    manager.adapters().await?.into_iter().next().ok_or(BleError::AdapterNotFound)
}

/// Endpoint address for a peripheral: its MAC, or its platform id where the MAC is hidden (macOS).
pub(super) fn peripheral_key(peripheral: &Peripheral) -> String {
    endpoint_key(&peripheral.address().to_string(), &peripheral.id().to_string())
}

/// Finds `target` among known peripherals, scanning briefly when it is not cached.
pub(super) async fn resolve_peripheral(adapter: &Adapter, target: &str) -> Result<Peripheral, BleError> {
    match find_cached(adapter, target).await? {
        Some(peripheral) => Ok(peripheral),
        None => {
            debug!(%target, "bluetooth peripheral not cached; scanning");
            find_peripheral(adapter, target, CONNECT_DISCOVERY).await
        }
    }
}

/// Scans for `duration` and returns the peripheral matching `target`.
pub(super) async fn find_peripheral(
    adapter: &Adapter,
    target: &str,
    duration: Duration,
) -> Result<Peripheral, BleError> {
    let normalized = normalize_target(target)?;
    let peripherals = scan_peripherals(adapter, duration).await?;
    peripherals.into_iter().find(|peripheral| matches_target(peripheral, &normalized)).ok_or(BleError::DeviceNotFound)
}

/// Runs an unfiltered scan for `duration` and returns every peripheral seen.
pub(super) async fn scan_peripherals(adapter: &Adapter, duration: Duration) -> Result<Vec<Peripheral>, BleError> {
    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(duration).await;
    let peripherals = adapter.peripherals().await;
    adapter.stop_scan().await?;
    Ok(peripherals?)
}

/// Returns the adapter's already-known peripheral matching `target`, without scanning.
pub(super) async fn find_cached(adapter: &Adapter, target: &str) -> Result<Option<Peripheral>, BleError> {
    let normalized = normalize_target(target)?;
    Ok(adapter.peripherals().await?.into_iter().find(|peripheral| matches_target(peripheral, &normalized)))
}

fn matches_target(peripheral: &Peripheral, normalized: &str) -> bool {
    key_matches(&peripheral_key(peripheral), normalized)
}
