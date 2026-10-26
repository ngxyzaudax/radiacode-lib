//! Device endpoints, scan results and transport-independent discovery and connect.
//!
//! [`discover_devices`] and [`connect()`] are always available; transports that
//! were not compiled in report [`Error::TransportUnavailable`](crate::Error::TransportUnavailable).

mod connect;
mod endpoint;
mod merge;
mod model;

#[cfg(test)]
mod tests;

pub use connect::{connect, discover_devices, open_transport, DiscoveryOptions};
pub use endpoint::{DeviceEndpoint, DiscoveredDevice, TransportKind};
pub use merge::{merge_discovered, resolve_usb_endpoint, sort_and_dedup};
pub use model::{is_radiacode_name, model_from_advertisement, model_from_serial, serial_from_advertisement};
