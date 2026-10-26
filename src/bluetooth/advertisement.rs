use uuid::Uuid;

use super::uuids::SERVICE_UUID;
use crate::discovery::{
    is_radiacode_name, model_from_advertisement, serial_from_advertisement, DeviceEndpoint, DiscoveredDevice,
};

const DEFAULT_LABEL: &str = "RadiaCode";

/// The parts of a BLE advertisement used to recognise a RadiaCode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Advertisement {
    pub address: String,
    pub local_name: Option<String>,
    pub services: Vec<Uuid>,
    pub rssi: Option<i16>,
}

pub(super) fn is_radiacode_advertisement(advertisement: &Advertisement) -> bool {
    advertisement.services.contains(&SERVICE_UUID) || advertisement.local_name.as_deref().is_some_and(is_radiacode_name)
}

pub(super) fn discovered_device(advertisement: Advertisement) -> DiscoveredDevice {
    let local_name = advertisement.local_name;
    let serial = local_name.as_deref().and_then(serial_from_advertisement);
    let model = local_name.as_deref().and_then(model_from_advertisement);
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Bluetooth { address: advertisement.address },
        label: model.as_deref().or(serial.as_deref()).or(local_name.as_deref()).unwrap_or(DEFAULT_LABEL).to_string(),
        serial,
        model,
        rssi: advertisement.rssi,
    }
}
