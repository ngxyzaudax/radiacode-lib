use crate::error::{Error, Result};
use crate::protocol::buffer::BytesBuffer;
use crate::protocol::registers::VirtSfr;
use crate::protocol::VirtString;

const RETCODE_OK: u32 = 1;
const RETCODE_UNSUPPORTED: u32 = 0;

/// Encodes the argument block of `RD_VIRT_STRING` for `id`.
pub fn virt_string_read_args(id: VirtString) -> Vec<u8> {
    u32::from(id).to_le_bytes().to_vec()
}

/// Encodes the argument block of `WR_VIRT_STRING`: id, payload length and payload.
pub fn virt_string_write_args(id: VirtString, payload: &[u8]) -> Result<Vec<u8>> {
    let len =
        u32::try_from(payload.len()).map_err(|_| Error::InvalidArgument("virt string payload exceeds u32".into()))?;
    Ok([u32::from(id).to_le_bytes(), len.to_le_bytes()].concat().into_iter().chain(payload.iter().copied()).collect())
}

/// Encodes the argument block of `RD_VIRT_SFR` for `id`.
pub fn vsfr_read_args(id: VirtSfr) -> Vec<u8> {
    id.address().to_le_bytes().to_vec()
}

/// Encodes the argument block of `WR_VIRT_SFR`: register id followed by raw data.
pub fn vsfr_write_args(id: VirtSfr, data: &[u8]) -> Vec<u8> {
    id.address().to_le_bytes().into_iter().chain(data.iter().copied()).collect()
}

/// Decodes a `RD_VIRT_STRING` response into the string payload.
///
/// Trims a single trailing NUL emitted by some firmware and drops any bytes
/// beyond the declared length.
pub fn decode_virt_string(mut response: BytesBuffer) -> Result<BytesBuffer> {
    let retcode = response.take_u32_le()?;
    let declared = response.take_len_u32()?;
    ensure_ok(retcode)?;
    match response.size() {
        size if size < declared => Err(Error::BufferUnderrun { need: declared, have: size }),
        _ => Ok(response.truncated(declared)),
    }
}

/// Decodes a `RD_VIRT_SFR` response into the raw register word.
pub fn decode_vsfr_read(response: BytesBuffer) -> Result<u32> {
    decode_vsfr_read_optional(response)?.ok_or(Error::UnexpectedReturnCode(RETCODE_UNSUPPORTED))
}

/// Decodes a `RD_VIRT_SFR` response, mapping return code `0` to `None`.
pub fn decode_vsfr_read_optional(mut response: BytesBuffer) -> Result<Option<u32>> {
    match response.take_u32_le()? {
        RETCODE_OK => response.take_u32_le().map(Some),
        RETCODE_UNSUPPORTED => Ok(None),
        other => Err(Error::UnexpectedReturnCode(other)),
    }
}

/// Decodes a `WR_VIRT_SFR` acknowledgement; `false` means the register was rejected.
pub fn decode_vsfr_write(mut response: BytesBuffer) -> Result<bool> {
    match response.take_u32_le()? {
        RETCODE_OK => ensure_empty_payload(&response).map(|()| true),
        RETCODE_UNSUPPORTED => Ok(false),
        other => Err(Error::UnexpectedReturnCode(other)),
    }
}

/// Decodes a response that must start with the success return code.
pub fn decode_retcode_ok(mut response: BytesBuffer) -> Result<()> {
    response.take_u32_le().and_then(ensure_ok)
}

/// Fails with [`Error::ProtocolMismatch`] when unread bytes remain.
pub fn ensure_empty_payload(response: &BytesBuffer) -> Result<()> {
    match response.size() {
        0 => Ok(()),
        trailing => Err(Error::mismatch("empty payload", format!("{trailing} trailing bytes"))),
    }
}

fn ensure_ok(retcode: u32) -> Result<()> {
    match retcode {
        RETCODE_OK => Ok(()),
        other => Err(Error::UnexpectedReturnCode(other)),
    }
}
