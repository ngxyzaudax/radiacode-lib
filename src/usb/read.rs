use rusb::{Context, DeviceHandle};
use tracing::warn;

use super::constants::{
    DRAIN_TIMEOUT, EMPTY_READ_RETRIES, ENDPOINT_IN, EXCHANGE_TIMEOUT, MAX_DRAIN_CHUNKS, MAX_UNRELATED_FRAMES,
    READ_CHUNK_LEN,
};
use super::error::UsbError;
use crate::protocol::{response_matches_request, BytesBuffer, EchoHeader, ResponseAssembler, LENGTH_PREFIX_LEN};

/// A bulk-IN reader: fills the buffer and returns the byte count.
pub(super) type BulkReader<'a> = dyn FnMut(&mut [u8]) -> rusb::Result<usize> + 'a;

#[derive(Default)]
struct ReadProgress {
    assembler: ResponseAssembler,
    empty_reads: usize,
    bytes_received: usize,
}

pub(super) fn read_matching_response(
    handle: &mut DeviceHandle<Context>,
    expected: EchoHeader,
) -> Result<BytesBuffer, UsbError> {
    read_matching_with(&mut |buffer| handle.read_bulk(ENDPOINT_IN, buffer, EXCHANGE_TIMEOUT), expected)
}

pub(super) fn drain_handle(handle: &mut DeviceHandle<Context>) -> Result<(), UsbError> {
    drain_with(&mut |buffer| handle.read_bulk(ENDPOINT_IN, buffer, DRAIN_TIMEOUT))
}

/// Reads frames until one carries `expected`, skipping a bounded number of stale frames.
pub(super) fn read_matching_with(reader: &mut BulkReader<'_>, expected: EchoHeader) -> Result<BytesBuffer, UsbError> {
    (1..=MAX_UNRELATED_FRAMES)
        .find_map(|attempt| matching_frame(read_response_with(reader), expected, attempt))
        .unwrap_or(Err(UsbError::ResponseMismatch))
}

fn matching_frame(
    frame: Result<BytesBuffer, UsbError>,
    expected: EchoHeader,
    attempt: usize,
) -> Option<Result<BytesBuffer, UsbError>> {
    match frame {
        Ok(payload) if !response_matches_request(payload.data(), expected) => {
            warn!(response_len = payload.size(), discarded = attempt, "usb frame discarded: echo header mismatch");
            None
        }
        other => Some(other),
    }
}

/// Reads bulk chunks until one complete length-prefixed frame has been assembled.
pub(super) fn read_response_with(reader: &mut BulkReader<'_>) -> Result<BytesBuffer, UsbError> {
    let mut buffer = vec![0u8; READ_CHUNK_LEN];
    let mut progress = ReadProgress::default();
    loop {
        let outcome = reader(&mut buffer);
        if let Some(result) = advance(&mut progress, outcome, &buffer)? {
            return Ok(result);
        }
    }
}

fn advance(
    progress: &mut ReadProgress,
    outcome: rusb::Result<usize>,
    buffer: &[u8],
) -> Result<Option<BytesBuffer>, UsbError> {
    match outcome {
        Ok(0) => register_empty_read(progress).map(|()| None),
        Ok(read) => push_chunk(progress, buffer.get(..read).unwrap_or(buffer)),
        Err(rusb::Error::Timeout) if progress.bytes_received < LENGTH_PREFIX_LEN => Err(UsbError::EmptyRead),
        Err(error) => Err(error.into()),
    }
}

fn register_empty_read(progress: &mut ReadProgress) -> Result<(), UsbError> {
    progress.empty_reads = progress.empty_reads.saturating_add(1);
    match progress.empty_reads >= EMPTY_READ_RETRIES {
        true => Err(UsbError::EmptyRead),
        false => Ok(()),
    }
}

fn push_chunk(progress: &mut ReadProgress, chunk: &[u8]) -> Result<Option<BytesBuffer>, UsbError> {
    progress.empty_reads = 0;
    progress.bytes_received = progress.bytes_received.saturating_add(chunk.len());
    Ok(progress.assembler.push(chunk)?.map(BytesBuffer::new))
}

/// Reads and discards chunks until the device stops sending.
pub(super) fn drain_with(reader: &mut BulkReader<'_>) -> Result<(), UsbError> {
    let mut buffer = vec![0u8; READ_CHUNK_LEN];
    (0..MAX_DRAIN_CHUNKS).find_map(|_| drain_step(reader(&mut buffer))).unwrap_or(Ok(()))
}

fn drain_step(outcome: rusb::Result<usize>) -> Option<Result<(), UsbError>> {
    match outcome {
        Ok(0) | Err(rusb::Error::Timeout) => Some(Ok(())),
        Err(error) => Some(Err(error.into())),
        Ok(_) => None,
    }
}
