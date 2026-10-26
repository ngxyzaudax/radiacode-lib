use chrono::NaiveDate;

use super::{
    decode_fw_signature, decode_fw_version, decode_hw_serial, decode_retcode_ok, decode_virt_string, decode_vsfr_read,
    decode_vsfr_read_optional, decode_vsfr_write, set_time_payload, spectrum_format_version, virt_string_write_args,
    BytesBuffer, VirtString,
};
use crate::error::Error;

fn ascii_field(value: &str) -> Vec<u8> {
    std::iter::once(value.len() as u8).chain(value.bytes()).collect()
}

fn fw_payload(target_date: &str) -> Vec<u8> {
    fw_payload_with_boot("boot-date", target_date)
}

fn fw_payload_with_boot(boot_date: &str, target_date: &str) -> Vec<u8> {
    [
        4u16.to_le_bytes().to_vec(),
        5u16.to_le_bytes().to_vec(),
        ascii_field(boot_date),
        8u16.to_le_bytes().to_vec(),
        4u16.to_le_bytes().to_vec(),
        ascii_field(target_date),
    ]
    .concat()
}

fn words(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

#[test]
fn fw_version_parses_boot_and_target() {
    let versions = decode_fw_version(BytesBuffer::new(fw_payload("target-date")));
    let versions = versions.unwrap_or_else(|error| panic!("{error}"));
    assert_eq!((versions.boot.major, versions.boot.minor), (5, 4));
    assert_eq!((versions.target.major, versions.target.minor), (4, 8));
    assert_eq!(versions.target.date, "target-date");
    assert!(versions.is_supported());
}

#[test]
fn fw_version_accepts_retcode_prefix_and_nul_padding() {
    let prefixed = [words(&[1]), fw_payload("t")].concat();
    assert!(decode_fw_version(BytesBuffer::new(prefixed)).is_ok());
    let body = fw_payload_with_boot("Jan 15 2024 12:00:00", "Feb 20 2024 08:30:00 build");
    let padded = [body, vec![0; 10]].concat();
    let versions = decode_fw_version(BytesBuffer::new(padded));
    assert_eq!(versions.map(|v| v.target.date).ok().as_deref(), Some("Feb 20 2024 08:30:00 build"));
}

#[test]
fn fw_version_rejects_trailing_garbage_and_short_payload() {
    let garbage = [fw_payload("t"), vec![7]].concat();
    assert!(matches!(decode_fw_version(BytesBuffer::new(garbage)), Err(Error::ProtocolMismatch { .. })));
    assert!(matches!(decode_fw_version(BytesBuffer::new(vec![1, 2])), Err(Error::BufferUnderrun { .. })));
}

#[test]
fn virt_string_trims_and_validates_declared_length() {
    let exact = [words(&[1, 3]), b"abc".to_vec()].concat();
    let nul = [words(&[1, 3]), b"abc\0".to_vec()].concat();
    let long = [words(&[1, 2]), b"abc".to_vec()].concat();
    let short = [words(&[1, 5]), b"abc".to_vec()].concat();
    let decode = |bytes: Vec<u8>| decode_virt_string(BytesBuffer::new(bytes)).map(|b| b.into_remaining());
    assert_eq!(decode(exact).ok(), Some(b"abc".to_vec()));
    assert_eq!(decode(nul).ok(), Some(b"abc".to_vec()));
    assert_eq!(decode(long).ok(), Some(b"ab".to_vec()));
    assert!(matches!(decode(short), Err(Error::BufferUnderrun { need: 5, have: 3 })));
    assert!(matches!(decode(words(&[2, 0])), Err(Error::UnexpectedReturnCode(2))));
}

#[test]
fn vsfr_read_and_write_return_codes() {
    let read = |values: &[u32]| BytesBuffer::new(words(values));
    assert_eq!(decode_vsfr_read(read(&[1, 42])).ok(), Some(42));
    assert!(matches!(decode_vsfr_read(read(&[0])), Err(Error::UnexpectedReturnCode(0))));
    assert_eq!(decode_vsfr_read_optional(read(&[0])).ok(), Some(None));
    assert!(decode_vsfr_read_optional(read(&[7])).is_err());
    assert_eq!(decode_vsfr_write(read(&[1])).ok(), Some(true));
    assert_eq!(decode_vsfr_write(read(&[0])).ok(), Some(false));
    assert!(decode_vsfr_write(read(&[1, 9])).is_err());
    assert!(decode_retcode_ok(read(&[1])).is_ok());
    assert!(decode_retcode_ok(read(&[3])).is_err());
}

#[test]
fn virt_string_write_args_carry_length() {
    let args = virt_string_write_args(VirtString::EnergyCalib, &[9, 9]).unwrap_or_default();
    assert_eq!(args, [words(&[u32::from(VirtString::EnergyCalib), 2]), vec![9, 9]].concat());
}

#[test]
fn identity_decoders() {
    let serial = BytesBuffer::new(words(&[8, 0x0012_ABCD, 0xFFFF_0001]));
    assert_eq!(decode_hw_serial(serial).ok().as_deref(), Some("0012ABCD-FFFF0001"));
    let signature = [words(&[0x1234]), ascii_field("fmw"), ascii_field("RC")].concat();
    let signature = decode_fw_signature(BytesBuffer::new(signature));
    assert_eq!(
        signature.map(|s| (s.signature, s.filename, s.id_string)).ok(),
        Some((0x1234, "fmw".into(), "RC".into()))
    );
    assert_eq!(spectrum_format_version("A=1\nSpecFormatVersion=1\n"), 1);
    assert_eq!(spectrum_format_version("nothing"), 0);
}

#[test]
fn set_time_payload_uses_device_field_order() {
    let moment = NaiveDate::from_ymd_opt(2023, 11, 14).and_then(|d| d.and_hms_opt(22, 13, 20));
    let payload = moment.map(set_time_payload);
    assert_eq!(payload, Some([14, 11, 23, 0, 20, 13, 22, 0]));
}
