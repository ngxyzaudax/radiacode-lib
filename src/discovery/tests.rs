use super::connect::combine_scans;
use super::{
    is_radiacode_name, merge_discovered, model_from_advertisement, model_from_serial, resolve_usb_endpoint,
    serial_from_advertisement, sort_and_dedup, DeviceEndpoint, DiscoveredDevice, TransportKind,
};
use crate::error::Error;

fn usb(serial: &str) -> DiscoveredDevice {
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Usb { serial: serial.into() },
        label: String::new(),
        serial: Some(serial.into()),
        model: None,
        rssi: None,
    }
}

fn bluetooth(address: &str, serial: Option<&str>) -> DiscoveredDevice {
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Bluetooth { address: address.into() },
        label: String::new(),
        serial: serial.map(Into::into),
        model: Some("RC-103".into()),
        rssi: Some(-50),
    }
}

fn usb_endpoint(serial: &str) -> DeviceEndpoint {
    DeviceEndpoint::Usb { serial: serial.into() }
}

#[test]
fn endpoint_transport_and_address_label() {
    let usb_endpoint = usb("RC-103-1").endpoint;
    assert_eq!(usb_endpoint.transport(), TransportKind::Usb);
    assert_eq!(usb_endpoint.address_label(), "RC-103-1");
    assert_eq!(usb_endpoint.to_string(), "USB RC-103-1");
    let bluetooth_endpoint = bluetooth("AA:BB", None).endpoint;
    assert_eq!(bluetooth_endpoint.transport(), TransportKind::Bluetooth);
    assert_eq!(bluetooth_endpoint.address_label(), "AA:BB");
    assert_eq!(TransportKind::Bluetooth.to_string(), "Bluetooth");
}

#[test]
fn display_label_prefers_explicit_label_then_model_then_serial() {
    let mut device = bluetooth("AA:BB", Some("RC-103-9"));
    assert_eq!(device.display_label(), "RC-103");
    device.model = None;
    assert_eq!(device.display_label(), "RC-103-9");
    device.serial = None;
    assert_eq!(device.display_label(), "RadiaCode");
    device.label = "Named".into();
    assert_eq!(device.display_label(), "Named");
}

#[test]
fn merge_discovered_drops_bluetooth_duplicates_by_serial() {
    let merged = merge_discovered(
        vec![usb("RC-103-1")],
        vec![bluetooth("AA", Some("RC-103-1")), bluetooth("BB", Some("RC-103-2")), bluetooth("CC", None)],
    );
    let labels: Vec<&str> = merged.iter().map(|device| device.endpoint.address_label()).collect();
    assert_eq!(labels, ["RC-103-1", "BB", "CC"]);
}

#[test]
fn sort_and_dedup_orders_by_address_and_removes_repeats() {
    let sorted = sort_and_dedup(vec![bluetooth("CC", None), bluetooth("AA", None), bluetooth("CC", None)]);
    let labels: Vec<&str> = sorted.iter().map(|device| device.endpoint.address_label()).collect();
    assert_eq!(labels, ["AA", "CC"]);
}

#[test]
fn resolve_usb_endpoint_falls_back_to_preferred_without_usb_devices() {
    let preferred = usb_endpoint("RC-103-2");
    assert_eq!(resolve_usb_endpoint(&[], &preferred), preferred);
    assert_eq!(resolve_usb_endpoint(&[bluetooth("AA", None)], &preferred), preferred);
}

#[test]
fn resolve_usb_endpoint_picks_single_or_matching_device() {
    let preferred = usb_endpoint("RC-103-2");
    assert_eq!(resolve_usb_endpoint(&[usb("RC-103-7")], &preferred), usb_endpoint("RC-103-7"));
    assert_eq!(resolve_usb_endpoint(&[usb("RC-103-1"), usb("RC-103-2")], &preferred), preferred);
    let renamed = DiscoveredDevice { endpoint: usb_endpoint("bus-2"), ..usb("RC-103-2") };
    assert_eq!(resolve_usb_endpoint(&[usb("RC-103-1"), renamed], &preferred).address_label(), "bus-2");
    let unmatched = [usb("RC-103-8"), usb("RC-103-9")];
    assert_eq!(resolve_usb_endpoint(&unmatched, &preferred).address_label(), "RC-103-8");
}

#[test]
fn model_from_serial_keeps_first_two_parts() {
    assert_eq!(model_from_serial("RC-110-006806"), "RC-110");
    assert_eq!(model_from_serial("RC-102"), "RC-102");
    assert_eq!(model_from_serial("garbage"), "Unknown");
    assert_eq!(model_from_serial("-110"), "Unknown");
}

#[test]
fn advertisement_parsing_is_case_insensitive() {
    assert_eq!(serial_from_advertisement("RadiaCode#RC-110-006806").as_deref(), Some("RC-110-006806"));
    assert_eq!(serial_from_advertisement("RADIACODE-103# RC-103-000001 ").as_deref(), Some("RC-103-000001"));
    assert_eq!(model_from_advertisement("radiacode#RC-102-000123").as_deref(), Some("RC-102"));
    assert_eq!(serial_from_advertisement("RadiaCode RC-110"), None);
    assert_eq!(serial_from_advertisement("Other#RC-110-1"), None);
    assert_eq!(serial_from_advertisement("RadiaCode#"), None);
}

#[test]
fn radiacode_name_detection() {
    assert!(is_radiacode_name("RadiaCode-103"));
    assert!(is_radiacode_name("my radiacode"));
    assert!(!is_radiacode_name("Heart Rate"));
}

#[test]
fn combine_scans_keeps_partial_results() {
    let combined = combine_scans(Some(Ok(vec![usb("RC-103-1")])), Some(Err(Error::Timeout))).unwrap();
    assert_eq!(combined, vec![usb("RC-103-1")]);
    assert!(combine_scans(None, None).unwrap().is_empty());
    assert!(matches!(combine_scans(None, Some(Err(Error::Timeout))), Err(Error::Timeout)));
}

#[test]
fn combine_scans_reports_both_failures() {
    let usb_failure = Error::TransportUnavailable("usb: no bus".into());
    let ble_failure = Error::TransportUnavailable("bluetooth: no adapter".into());
    let message = combine_scans(Some(Err(usb_failure)), Some(Err(ble_failure))).unwrap_err().to_string();
    assert!(message.contains("no bus") && message.contains("no adapter"), "{message}");
    let denied = combine_scans(Some(Err(Error::UsbPermissionDenied)), Some(Err(Error::Timeout)));
    assert!(matches!(denied, Err(Error::UsbPermissionDenied)));
}
