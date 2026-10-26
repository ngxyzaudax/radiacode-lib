use super::endpoint::{DeviceEndpoint, DiscoveredDevice, TransportKind};

/// Combines USB and Bluetooth scan results, dropping Bluetooth entries whose serial is already on USB.
pub fn merge_discovered(
    usb_devices: Vec<DiscoveredDevice>,
    bluetooth_devices: Vec<DiscoveredDevice>,
) -> Vec<DiscoveredDevice> {
    let bluetooth_only: Vec<DiscoveredDevice> =
        bluetooth_devices.into_iter().filter(|device| !serial_listed(&usb_devices, device.serial.as_deref())).collect();
    usb_devices.into_iter().chain(bluetooth_only).collect()
}

/// Sorts devices by endpoint address and removes duplicate endpoints.
pub fn sort_and_dedup(mut devices: Vec<DiscoveredDevice>) -> Vec<DiscoveredDevice> {
    devices.sort_by(|left, right| left.endpoint.address_label().cmp(right.endpoint.address_label()));
    devices.dedup_by(|left, right| left.endpoint == right.endpoint);
    devices
}

/// Picks the USB endpoint to open given a fresh scan and a remembered endpoint.
///
/// Prefers the only USB device, then an exact endpoint match, then a serial
/// match, then the first USB device. Returns `preferred` when no USB device is present.
pub fn resolve_usb_endpoint(devices: &[DiscoveredDevice], preferred: &DeviceEndpoint) -> DeviceEndpoint {
    let usb: Vec<&DiscoveredDevice> =
        devices.iter().filter(|device| device.endpoint.transport() == TransportKind::Usb).collect();
    let key = preferred.address_label();
    let chosen = match usb.as_slice() {
        [only] => Some(*only),
        _ => usb
            .iter()
            .find(|device| device.endpoint.address_label() == key)
            .or_else(|| usb.iter().find(|device| device.serial.as_deref() == Some(key)))
            .or_else(|| usb.first())
            .copied(),
    };
    chosen.map_or_else(|| preferred.clone(), |device| device.endpoint.clone())
}

fn serial_listed(devices: &[DiscoveredDevice], serial: Option<&str>) -> bool {
    serial.is_some_and(|serial| devices.iter().any(|device| device.serial.as_deref() == Some(serial)))
}
