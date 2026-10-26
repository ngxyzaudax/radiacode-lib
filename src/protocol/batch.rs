use crate::error::{Error, Result};
use crate::protocol::buffer::BytesBuffer;
use crate::protocol::catalog::SfrCatalogEntry;
use crate::protocol::registers::{extract_vsfr_u32, resolve_value_kind, VirtSfr};
use crate::protocol::responses::ensure_empty_payload;

/// Maximum number of registers in one batch request.
pub const MAX_BATCH_SIZE: usize = 32;

/// Outcome of decoding a `RD_VIRT_SFR_BATCH` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchReadResponse {
    /// Every requested register was returned, in request order.
    Complete(Vec<u32>),
    /// Only some registers were returned; `None` marks registers to read individually.
    Partial(Vec<Option<u32>>),
}

/// Fails unless `count` is within `1..=MAX_BATCH_SIZE`.
pub fn validate_batch_size(count: usize) -> Result<()> {
    match count {
        0 => Err(Error::InvalidArgument("vsfr batch requires at least one register".into())),
        1..=MAX_BATCH_SIZE => Ok(()),
        _ => Err(Error::InvalidArgument(format!("vsfr batch accepts at most {MAX_BATCH_SIZE} registers, got {count}"))),
    }
}

/// Bit mask with one bit set per batch slot.
pub fn all_valid_flags(count: usize) -> u32 {
    match count {
        0 => 0,
        MAX_BATCH_SIZE.. => u32::MAX,
        bits => (1u32 << bits) - 1,
    }
}

/// Encodes the batch read arguments: register count followed by addresses.
pub fn encode_batch_read_args(ids: &[VirtSfr]) -> Vec<u8> {
    let count = u32::try_from(ids.len()).unwrap_or(u32::MAX);
    std::iter::once(count).chain(ids.iter().map(|id| id.address())).flat_map(u32::to_le_bytes).collect()
}

/// Encodes the batch write arguments: count, addresses, then values.
pub fn encode_batch_write_args(pairs: &[(VirtSfr, u32)]) -> Vec<u8> {
    let ids: Vec<VirtSfr> = pairs.iter().map(|(id, _)| *id).collect();
    encode_batch_read_args(&ids).into_iter().chain(pairs.iter().flat_map(|(_, value)| value.to_le_bytes())).collect()
}

/// Decodes a batch read response for `count` requested registers.
pub fn decode_batch_read(mut response: BytesBuffer, count: usize) -> Result<BatchReadResponse> {
    match response.take_u32_le()? {
        0 => Err(Error::VsfrBatchEmpty),
        flags if flags == all_valid_flags(count) => take_words(&mut response, count).map(BatchReadResponse::Complete),
        flags => take_sparse_words(&mut response, count, flags).map(BatchReadResponse::Partial),
    }
}

/// Decodes a batch write acknowledgement for `count` registers.
pub fn decode_batch_write(mut response: BytesBuffer, count: usize) -> Result<()> {
    let valid_flags = response.take_u32_le()?;
    let expected = all_valid_flags(count);
    match valid_flags == expected {
        true => ensure_empty_payload(&response),
        false => Err(Error::mismatch(format!("valid_flags {expected:#x}"), format!("valid_flags {valid_flags:#x}"))),
    }
}

/// Extracts typed values from raw batch words using the device register descriptions.
pub fn decode_batch_words(ids: &[VirtSfr], raw: &[u32], entries: &[SfrCatalogEntry]) -> Vec<u32> {
    ids.iter().zip(raw).map(|(id, word)| extract_vsfr_u32(*word, resolve_value_kind(*id, entries))).collect()
}

fn take_words(response: &mut BytesBuffer, count: usize) -> Result<Vec<u32>> {
    let words = (0..count).map(|_| response.take_u32_le()).collect::<Result<Vec<u32>>>()?;
    ensure_empty_payload(response).map(|()| words)
}

fn take_sparse_words(response: &mut BytesBuffer, count: usize, flags: u32) -> Result<Vec<Option<u32>>> {
    let words = (0..count)
        .map(|index| slot_is_valid(flags, index))
        .map(|valid| valid.then(|| response.take_u32_le()).transpose())
        .collect::<Result<Vec<Option<u32>>>>()?;
    ensure_empty_payload(response).map(|()| words)
}

fn slot_is_valid(flags: u32, index: usize) -> bool {
    u32::try_from(index).ok().and_then(|shift| flags.checked_shr(shift)).is_some_and(|shifted| shifted & 1 == 1)
}
