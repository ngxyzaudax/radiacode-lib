use std::future::Future;
use std::time::Duration;

use btleplug::api::{Characteristic, Peripheral as _};
use btleplug::platform::Peripheral;
use futures::StreamExt;
use tokio::time::timeout;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::adapter::peripheral_key;
use super::drain::{drain_for_settle, NotificationStream};
use super::error::BleError;
use super::uuids::{NOTIFY_UUID, WRITE_UUID};

const LINK_SETTLE: Duration = Duration::from_millis(250);
const STALE_DISCONNECT_COOLDOWN: Duration = Duration::from_millis(500);
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const LINK_OPEN_TIMEOUT: Duration = Duration::from_secs(8);

/// A connected peripheral with its request and notification characteristics.
pub(super) struct BluetoothLink {
    pub peripheral: Peripheral,
    pub write_char: Characteristic,
    pub notify_char: Characteristic,
    pub notifications: NotificationStream,
}

/// Connects, discovers the RadiaCode service, subscribes to notifications and lets the link settle.
pub(super) async fn open_link(peripheral: Peripheral) -> Result<BluetoothLink, BleError> {
    let address = peripheral_key(&peripheral);
    disconnect_stale(&peripheral).await;
    debug!(%address, "bluetooth connecting");
    bounded(&peripheral, peripheral.connect()).await?;
    bounded(&peripheral, peripheral.discover_services()).await?;
    let write_char = find_characteristic(&peripheral, WRITE_UUID)?;
    let notify_char = find_characteristic(&peripheral, NOTIFY_UUID)?;
    let mut notifications = bounded(&peripheral, subscribe(&peripheral, &notify_char)).await?;
    tokio::time::sleep(LINK_SETTLE).await;
    drain_for_settle(&mut notifications).await;
    info!(%address, "bluetooth transport ready");
    Ok(BluetoothLink { peripheral, write_char, notify_char, notifications })
}

/// Disconnects a peripheral left connected by an earlier session.
pub(super) async fn disconnect_stale(peripheral: &Peripheral) {
    if peripheral.is_connected().await.unwrap_or(false) {
        debug!(address = %peripheral_key(peripheral), "bluetooth disconnecting stale session");
        disconnect_with_timeout(peripheral).await;
        tokio::time::sleep(STALE_DISCONNECT_COOLDOWN).await;
    }
}

async fn bounded<T, E>(peripheral: &Peripheral, operation: impl Future<Output = Result<T, E>>) -> Result<T, BleError>
where
    E: Into<BleError>,
{
    match timeout(LINK_OPEN_TIMEOUT, operation).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(error.into()),
        Err(_) => {
            disconnect_with_timeout(peripheral).await;
            Err(BleError::ConnectTimedOut)
        }
    }
}

pub(super) async fn disconnect_with_timeout(peripheral: &Peripheral) {
    match timeout(DISCONNECT_TIMEOUT, peripheral.disconnect()).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => warn!(%error, "bluetooth disconnect failed"),
        Err(_) => warn!("bluetooth disconnect timed out"),
    }
}

async fn subscribe(peripheral: &Peripheral, notify_char: &Characteristic) -> Result<NotificationStream, BleError> {
    let _ = peripheral.unsubscribe(notify_char).await;
    peripheral.subscribe(notify_char).await?;
    Ok(peripheral.notifications().await?.boxed())
}

fn find_characteristic(peripheral: &Peripheral, uuid: Uuid) -> Result<Characteristic, BleError> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|characteristic| characteristic.uuid == uuid)
        .ok_or(BleError::CharacteristicMissing)
}
