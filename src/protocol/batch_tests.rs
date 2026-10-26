use super::{
    all_valid_flags, decode_batch_read, decode_batch_words, decode_batch_write, encode_batch_read_args,
    encode_batch_write_args, validate_batch_size, BatchReadResponse, BytesBuffer, VirtSfr, MAX_BATCH_SIZE,
};
use crate::error::Error;

fn words(values: &[u32]) -> BytesBuffer {
    BytesBuffer::new(values.iter().flat_map(|value| value.to_le_bytes()).collect())
}

#[test]
fn all_valid_flags_never_overflows() {
    assert_eq!(all_valid_flags(0), 0);
    assert_eq!(all_valid_flags(1), 0b1);
    assert_eq!(all_valid_flags(31), 0x7FFF_FFFF);
    assert_eq!(all_valid_flags(MAX_BATCH_SIZE), u32::MAX);
    assert_eq!(all_valid_flags(usize::MAX), u32::MAX);
}

#[test]
fn batch_size_bounds() {
    assert!(validate_batch_size(0).is_err());
    assert!(validate_batch_size(1).is_ok());
    assert!(validate_batch_size(MAX_BATCH_SIZE).is_ok());
    assert!(matches!(validate_batch_size(MAX_BATCH_SIZE + 1), Err(Error::InvalidArgument(_))));
}

#[test]
fn encodes_ids_and_values() {
    let read = encode_batch_read_args(&[VirtSfr::DispBrt, VirtSfr::SoundOn]);
    assert_eq!(read, [2, 0, 0, 0, 0x11, 5, 0, 0, 0x22, 5, 0, 0]);
    let write = encode_batch_write_args(&[(VirtSfr::DispBrt, 7)]);
    assert_eq!(write, [1, 0, 0, 0, 0x11, 5, 0, 0, 7, 0, 0, 0]);
}

#[test]
fn decodes_complete_partial_and_empty_reads() {
    assert_eq!(decode_batch_read(words(&[0b11, 4, 5]), 2).ok(), Some(BatchReadResponse::Complete(vec![4, 5])));
    assert_eq!(
        decode_batch_read(words(&[0b101, 4, 6]), 3).ok(),
        Some(BatchReadResponse::Partial(vec![Some(4), None, Some(6)]))
    );
    assert!(matches!(decode_batch_read(words(&[0]), 2), Err(Error::VsfrBatchEmpty)));
    assert!(decode_batch_read(words(&[0b1, 4, 9]), 1).is_err());
}

#[test]
fn decodes_write_acknowledgement() {
    assert!(decode_batch_write(words(&[0b111]), 3).is_ok());
    assert!(matches!(decode_batch_write(words(&[0b011]), 3), Err(Error::ProtocolMismatch { .. })));
}

#[test]
fn batch_words_are_extracted_by_kind() {
    let values = decode_batch_words(&[VirtSfr::DispBrt, VirtSfr::Cps], &[0x0700_0000, 9], &[]);
    assert_eq!(values, vec![7, 9]);
}
