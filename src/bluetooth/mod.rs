//! Bluetooth Low Energy transport (crate feature `bluetooth`, built on `btleplug`).
//!
//! RadiaCode devices expose one GATT service ([`SERVICE_UUID`]) with a write
//! characteristic ([`WRITE_UUID`]) that takes request chunks of up to 18 bytes
//! and a notify characteristic ([`NOTIFY_UUID`]) that streams response chunks.
//! Stale notifications are drained before every command, and responses whose
//! echo header does not match the request are discarded.
//!
//! On Linux this uses BlueZ over D-Bus; on macOS and Windows the native stacks.
//! macOS hides peripheral MAC addresses, so endpoints there carry the platform
//! peripheral UUID instead.

mod adapter;
mod address;
mod advertisement;
mod drain;
mod error;
mod execute;
mod link;
mod link_watch;
mod retry;
mod scan;
mod transport;
mod uuids;

#[cfg(test)]
mod tests;

pub use retry::{
    fast_reconnect_backoff, is_fast_reconnect_failure, retry_connect, FAST_RECONNECT_ATTEMPTS, FAST_RECONNECT_BACKOFF,
};
pub use scan::scan_bluetooth_devices;
pub use transport::BluetoothTransport;
pub use uuids::{NOTIFY_UUID, SERVICE_UUID, WRITE_UUID};
