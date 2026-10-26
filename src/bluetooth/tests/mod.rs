use std::time::Duration;

use super::address::{endpoint_key, key_matches, normalize_mac, normalize_target};
use super::advertisement::{discovered_device, is_radiacode_advertisement, Advertisement};
use super::error::BleError;
use super::SERVICE_UUID;
use crate::discovery::DeviceEndpoint;
use crate::error::Error;

mod streams;

fn advertisement(address: &str, local_name: Option<&str>, with_service: bool) -> Advertisement {
    Advertisement {
        address: address.to_string(),
        local_name: local_name.map(str::to_string),
        services: with_service.then_some(SERVICE_UUID).into_iter().collect(),
        rssi: Some(-55),
    }
}

#[test]
fn normalize_mac_accepts_dash_and_mixed_case() {
    assert_eq!(normalize_mac(" 52-43-0A-bb-CC-dd ").unwrap(), "52:43:0a:bb:cc:dd");
}

#[test]
fn normalize_mac_rejects_malformed_addresses() {
    for input in ["52:43:0a:bb:cc", "52:43:0a:bb:cc:zz", "52430abbccdd", ""] {
        assert!(matches!(normalize_mac(input), Err(BleError::InvalidAddress(_))), "{input:?} accepted");
    }
    assert!(matches!(Error::from(BleError::InvalidAddress("x".into())), Error::InvalidArgument(_)));
}

#[test]
fn targets_accept_mac_or_peripheral_uuid() {
    assert_eq!(normalize_target("52:43:0A:BB:CC:DD").unwrap(), "52:43:0a:bb:cc:dd");
    let uuid = "6F1C2A3B-4D5E-6F70-8192-A3B4C5D6E7F8";
    assert_eq!(normalize_target(uuid).unwrap(), uuid.to_ascii_lowercase());
    assert!(normalize_target("not-an-address").is_err());
}

#[test]
fn endpoint_key_falls_back_to_peripheral_id_for_hidden_mac() {
    assert_eq!(endpoint_key("52:43:0A:BB:CC:DD", "ignored"), "52:43:0a:bb:cc:dd");
    assert_eq!(endpoint_key("00:00:00:00:00:00", "ABC-UUID"), "abc-uuid");
    assert!(key_matches("52:43:0a:bb:cc:dd", "52:43:0A:BB:CC:DD"));
}

#[test]
fn advertisement_matches_by_service_or_name() {
    assert!(is_radiacode_advertisement(&advertisement("aa", None, true)));
    assert!(is_radiacode_advertisement(&advertisement("aa", Some("RadiaCode#RC-103-001234"), false)));
    assert!(!is_radiacode_advertisement(&advertisement("aa", Some("Other sensor"), false)));
}

#[test]
fn discovered_device_derives_model_serial_and_label() {
    let device = discovered_device(advertisement("aa:bb:cc:dd:ee:ff", Some("RadiaCode#RC-103-001234"), true));
    assert_eq!(device.serial.as_deref(), Some("RC-103-001234"));
    assert_eq!(device.model.as_deref(), Some("RC-103"));
    assert_eq!(device.label, "RC-103");
    assert_eq!(device.rssi, Some(-55));
    assert_eq!(device.endpoint, DeviceEndpoint::Bluetooth { address: "aa:bb:cc:dd:ee:ff".into() });
    assert_eq!(discovered_device(advertisement("b", None, true)).label, "RadiaCode");
}

#[test]
fn btleplug_errors_are_classified() {
    assert!(Error::from(BleError::Bluetooth(btleplug::Error::NotConnected)).is_connection_lost());
    assert!(Error::from(BleError::Bluetooth(btleplug::Error::TimedOut(Duration::from_secs(1)))).is_timeout());
    let busy = Error::from(BleError::Bluetooth(btleplug::Error::RuntimeError("adapter busy".into())));
    assert!(matches!(busy, Error::TransportUnavailable(_)));
}
