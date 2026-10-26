use std::time::Duration;

use async_trait::async_trait;
use btleplug::api::Peripheral as _;
use tracing::info;

use super::adapter::{default_adapter, find_cached, find_peripheral, peripheral_key, resolve_peripheral};
use super::drain::drain_until_quiet;
use super::error::{ble_result, BleError};
use super::execute::execute_request;
use super::link::{disconnect_with_timeout, open_link, BluetoothLink};
use crate::error::Result;
use crate::protocol::BytesBuffer;
use crate::transport::Transport;

const FRESH_SCAN: Duration = Duration::from_secs(2);
const RECONNECT_COOLDOWN: Duration = Duration::from_millis(500);

/// Bluetooth LE transport to one RadiaCode device (built on `btleplug`).
///
/// Requests are written in 18-byte chunks to the write characteristic and
/// responses are reassembled from notifications. Must be used inside a Tokio runtime.
pub struct BluetoothTransport {
    link: BluetoothLink,
}

impl std::fmt::Debug for BluetoothTransport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BluetoothTransport")
            .field("address", &peripheral_key(&self.link.peripheral))
            .finish_non_exhaustive()
    }
}

impl BluetoothTransport {
    /// Connects to `address`, a MAC address such as `52:43:0a:bb:cc:dd` (or a peripheral UUID on macOS).
    ///
    /// Uses the adapter's cached peripheral when available and scans briefly otherwise.
    pub async fn connect(address: &str) -> Result<Self> {
        info!(%address, "bluetooth transport connecting");
        ble_result(connect_cached(address).await)
    }

    /// Drops any cached connection, rescans and connects; use after the link was lost.
    pub async fn connect_fresh(address: &str) -> Result<Self> {
        info!(%address, "bluetooth transport reconnecting with fresh scan");
        ble_result(connect_after_rescan(address).await)
    }

    /// Endpoint address of the connected peripheral.
    pub fn address(&self) -> String {
        peripheral_key(&self.link.peripheral)
    }
}

async fn connect_cached(address: &str) -> std::result::Result<BluetoothTransport, BleError> {
    let adapter = default_adapter().await?;
    let peripheral = resolve_peripheral(&adapter, address).await?;
    Ok(BluetoothTransport { link: open_link(peripheral).await? })
}

async fn connect_after_rescan(address: &str) -> std::result::Result<BluetoothTransport, BleError> {
    let adapter = default_adapter().await?;
    if let Some(cached) = find_cached(&adapter, address).await? {
        disconnect_with_timeout(&cached).await;
    }
    tokio::time::sleep(RECONNECT_COOLDOWN).await;
    let peripheral = find_peripheral(&adapter, address, FRESH_SCAN).await?;
    Ok(BluetoothTransport { link: open_link(peripheral).await? })
}

#[async_trait]
impl Transport for BluetoothTransport {
    async fn execute(&mut self, request: &[u8]) -> Result<BytesBuffer> {
        let link = &mut self.link;
        execute_request(&link.peripheral, &link.write_char, &mut link.notifications, request).await
    }

    async fn drain_link(&mut self) {
        drain_until_quiet(&mut self.link.notifications).await;
    }

    async fn disconnect(self: Box<Self>) -> Result<()> {
        info!(address = %self.address(), "bluetooth transport closing");
        let _ = self.link.peripheral.unsubscribe(&self.link.notify_char).await;
        ble_result(self.link.peripheral.disconnect().await.map_err(BleError::from))
    }

    async fn link_rssi_dbm(&mut self) -> Option<i16> {
        self.link.peripheral.properties().await.ok().flatten().and_then(|properties| properties.rssi)
    }
}
