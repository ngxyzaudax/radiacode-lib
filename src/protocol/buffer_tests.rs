use super::BytesBuffer;
use crate::error::Error;

#[test]
fn reads_mixed_little_endian_fields() {
    let mut buffer = BytesBuffer::new(vec![1, 2, 0, 3, 0, 0, 0, 2, b'h', b'i', 0xff]);
    assert_eq!(buffer.take_u8().unwrap(), 1);
    assert_eq!(buffer.take_u16_le().unwrap(), 2);
    assert_eq!(buffer.take_u32_le().unwrap(), 3);
    assert_eq!(buffer.take_length_prefixed_ascii().unwrap(), "hi");
    assert_eq!(buffer.size(), 1);
    assert_eq!(buffer.data(), &[0xff]);
}

#[test]
fn underrun_reports_need_and_have_without_advancing() {
    let mut buffer = BytesBuffer::new(vec![0xff]);
    assert!(matches!(buffer.take_f32_le(), Err(Error::BufferUnderrun { need: 4, have: 1 })));
    assert_eq!(buffer.size(), 1);
    assert!(buffer.skip(1).is_ok());
    assert!(buffer.skip(1).is_err());
    assert!(buffer.is_empty());
    assert!(buffer.into_remaining().is_empty());
}

#[test]
fn signed_reads_preserve_sign() {
    let mut bytes = (-5i32).to_le_bytes().to_vec();
    bytes.extend_from_slice(&(-300i16).to_le_bytes());
    bytes.push(0xfe);
    let mut buffer = BytesBuffer::from(bytes);
    assert_eq!(buffer.take_i32_le().unwrap(), -5);
    assert_eq!(buffer.take_i16_le().unwrap(), -300);
    assert_eq!(buffer.take_i8().unwrap(), -2);
}

#[test]
fn huge_skip_does_not_overflow() {
    let mut buffer = BytesBuffer::from(&[1u8, 2][..]);
    assert!(buffer.skip(usize::MAX).is_err());
    assert_eq!(buffer.size(), 2);
}

#[test]
fn truncated_keeps_prefix_of_unread_bytes() {
    let mut buffer = BytesBuffer::new(vec![9, 1, 2, 3]);
    buffer.skip(1).unwrap();
    assert_eq!(buffer.truncated(2).data(), &[1, 2]);
    assert_eq!(buffer.truncated(10).data(), &[1, 2, 3]);
}
