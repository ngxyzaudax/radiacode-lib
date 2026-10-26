use super::constants::ADDRESS_SERIAL_PREFIX;
use crate::discovery::{model_from_serial, DeviceEndpoint, DiscoveredDevice};

const DEFAULT_LABEL: &str = "RadiaCode";

/// Descriptor facts read from one USB device during a scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UsbIdentity {
    pub address: u8,
    pub serial: Option<String>,
    pub product: Option<String>,
}

/// Picks the device to open for `target`, a serial number or `usb-<address>`.
///
/// Falls back to the only device when exactly one is attached.
pub(super) fn select_device_index(target: &str, addresses: &[u8], serials: &[Option<String>]) -> Option<usize> {
    let matched = match target.strip_prefix(ADDRESS_SERIAL_PREFIX) {
        Some(address) => addresses.iter().position(|value| value.to_string() == address),
        None => serials.iter().position(|value| value.as_deref() == Some(target)),
    };
    matched.or((addresses.len() == 1).then_some(0))
}

pub(super) fn discovered_device(identity: UsbIdentity) -> DiscoveredDevice {
    let serial = non_empty(identity.serial);
    let product = non_empty(identity.product);
    let model = serial.as_deref().map(model_from_serial);
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Usb { serial: endpoint_serial(serial.as_deref(), identity.address) },
        label: device_label(model.as_deref(), serial.as_deref(), product.as_deref()),
        serial,
        model,
        rssi: None,
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

fn endpoint_serial(serial: Option<&str>, address: u8) -> String {
    serial.map_or_else(|| format!("{ADDRESS_SERIAL_PREFIX}{address}"), str::to_string)
}

fn device_label(model: Option<&str>, serial: Option<&str>, product: Option<&str>) -> String {
    model.or(serial).or(product).unwrap_or(DEFAULT_LABEL).to_string()
}
