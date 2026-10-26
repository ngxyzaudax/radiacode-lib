use std::collections::VecDeque;

use super::access::status_for_denied_access;
use super::error::UsbError;
use super::identity::{discovered_device, select_device_index, UsbIdentity};
use super::read::{drain_with, read_matching_with, read_response_with};
use super::{rule_grants_access, UsbAccessStatus, UDEV_RULE};
use crate::discovery::DeviceEndpoint;
use crate::error::Error;
use crate::protocol::{request_header, Command, EchoHeader};

type ReadOutcome = rusb::Result<Vec<u8>>;

fn scripted_reader(script: Vec<ReadOutcome>) -> impl FnMut(&mut [u8]) -> rusb::Result<usize> {
    let mut queue: VecDeque<ReadOutcome> = script.into();
    move |buffer: &mut [u8]| match queue.pop_front() {
        Some(Ok(bytes)) => {
            buffer[..bytes.len()].copy_from_slice(&bytes);
            Ok(bytes.len())
        }
        Some(Err(error)) => Err(error),
        None => Err(rusb::Error::Timeout),
    }
}

fn frame(payload: &[u8]) -> Vec<u8> {
    let length = u32::try_from(payload.len()).unwrap().to_le_bytes();
    length.iter().chain(payload).copied().collect()
}

fn reply(header: EchoHeader, body: &[u8]) -> Vec<u8> {
    frame(&header.iter().chain(body).copied().collect::<Vec<u8>>())
}

fn identity(address: u8, serial: Option<&str>, product: Option<&str>) -> UsbIdentity {
    UsbIdentity { address, serial: serial.map(str::to_string), product: product.map(str::to_string) }
}

#[test]
fn response_spanning_two_reads_is_reassembled() {
    let full = frame(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let (head, tail) = full.split_at(6);
    let mut reader = scripted_reader(vec![Ok(head.to_vec()), Ok(tail.to_vec())]);
    assert_eq!(read_response_with(&mut reader).unwrap().data(), &[1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn repeated_empty_reads_fail_with_empty_read() {
    let mut reader = scripted_reader(vec![Ok(Vec::new()), Ok(Vec::new()), Ok(Vec::new())]);
    assert!(matches!(read_response_with(&mut reader), Err(UsbError::EmptyRead)));
}

#[test]
fn timeout_before_any_data_is_empty_read() {
    let mut reader = scripted_reader(vec![Err(rusb::Error::Timeout)]);
    assert!(matches!(read_response_with(&mut reader), Err(UsbError::EmptyRead)));
}

#[test]
fn timeout_after_partial_frame_surfaces_timeout() {
    let mut reader = scripted_reader(vec![Ok(frame(&[1, 2, 3])[..5].to_vec())]);
    let error = read_response_with(&mut reader).unwrap_err();
    assert!(matches!(error, UsbError::Usb(rusb::Error::Timeout)));
    assert!(Error::from(error).is_timeout());
}

#[test]
fn unrelated_frames_are_skipped_until_match() {
    let expected = request_header(Command::GetVersion, 0x81);
    let stale = reply(request_header(Command::GetSerial, 0x82), &[9]);
    let mut reader = scripted_reader(vec![Ok(stale), Ok(reply(expected, &[7, 7]))]);
    assert_eq!(read_matching_with(&mut reader, expected).unwrap().size(), 6);
}

#[test]
fn too_many_unrelated_frames_report_connection_closed() {
    let expected = request_header(Command::GetVersion, 0x81);
    let stale = reply(request_header(Command::GetSerial, 0x82), &[9]);
    let mut reader = scripted_reader(vec![Ok(stale); 4]);
    let error = read_matching_with(&mut reader, expected).unwrap_err();
    assert!(matches!(error, UsbError::ResponseMismatch));
    assert!(Error::from(error).is_connection_lost());
}

#[test]
fn drain_consumes_until_timeout_or_empty() {
    let mut reader = scripted_reader(vec![Ok(vec![1]), Ok(vec![2]), Err(rusb::Error::Timeout)]);
    assert!(drain_with(&mut reader).is_ok());
    assert!(drain_with(&mut scripted_reader(vec![Ok(Vec::new())])).is_ok());
    let mut failing = scripted_reader(vec![Err(rusb::Error::NoDevice)]);
    assert!(matches!(drain_with(&mut failing), Err(UsbError::Usb(rusb::Error::NoDevice))));
}

#[test]
fn usb_errors_map_to_crate_errors() {
    assert!(Error::from(UsbError::PermissionDenied).is_usb_permission_denied());
    assert!(Error::from(UsbError::Usb(rusb::Error::Access)).is_usb_permission_denied());
    assert!(matches!(Error::from(UsbError::DeviceNotFound), Error::DeviceNotFound));
    assert!(Error::from(UsbError::Usb(rusb::Error::NoDevice)).is_connection_lost());
    assert!(matches!(Error::from(UsbError::Usb(rusb::Error::Busy)), Error::TransportUnavailable(_)));
}

#[test]
fn serial_or_address_target_selects_device() {
    let serials = [Some("RC-103-000001".to_string()), Some("RC-102-000002".to_string())];
    assert_eq!(select_device_index("RC-102-000002", &[3, 7], &serials), Some(1));
    assert_eq!(select_device_index("usb-3", &[3, 7], &serials), Some(0));
    assert_eq!(select_device_index("RC-999", &[3, 7], &serials), None);
    assert_eq!(select_device_index("RC-999", &[3], &[None]), Some(0));
}

#[test]
fn discovered_device_prefers_model_label_and_serial_endpoint() {
    let device = discovered_device(identity(4, Some("RC-103-000123"), Some("RadiaCode")));
    assert_eq!(device.label, "RC-103");
    assert_eq!(device.model.as_deref(), Some("RC-103"));
    assert_eq!(device.endpoint, DeviceEndpoint::Usb { serial: "RC-103-000123".into() });
}

#[test]
fn discovered_device_without_serial_uses_address_endpoint_and_product_label() {
    let device = discovered_device(identity(9, Some(""), Some("RadiaCode 10x")));
    assert_eq!(device.label, "RadiaCode 10x");
    assert!(device.serial.is_none());
    assert_eq!(device.endpoint.address_label(), "usb-9");
    assert_eq!(discovered_device(identity(2, None, None)).label, "RadiaCode");
}

#[test]
fn bundled_rule_grants_access() {
    assert!(rule_grants_access(UDEV_RULE));
    assert!(!rule_grants_access("SUBSYSTEM==\"usb\", MODE=\"0600\""));
}

#[test]
fn denied_access_status_depends_on_rule_presence() {
    assert_eq!(status_for_denied_access(true), UsbAccessStatus::RuleInstalledNeedReplug);
    assert_eq!(status_for_denied_access(false), UsbAccessStatus::RuleMissing);
}
