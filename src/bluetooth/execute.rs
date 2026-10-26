use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use futures::StreamExt;
use tokio::time::{timeout, Instant};
use tracing::{debug, warn};

use super::drain::{drain_before_command, drain_until_quiet, NotificationStream};
use super::error::BleError;
use super::uuids::{RESPONSE_TIMEOUT, WRITE_CHUNK_LEN};
use crate::error::{Error, Result};
use crate::protocol::{framed_request_header, response_matches_request, BytesBuffer, EchoHeader, ResponseAssembler};

/// Writes `request` in 18-byte chunks and waits for the response carrying its echo header.
pub(super) async fn execute_request(
    peripheral: &Peripheral,
    write_char: &Characteristic,
    notifications: &mut NotificationStream,
    request: &[u8],
) -> Result<BytesBuffer> {
    let expected = framed_request_header(request)?;
    debug!(request_len = request.len(), "bluetooth request");
    drain_before_command(notifications).await;
    write_chunks(peripheral, write_char, request).await?;
    let outcome = await_matching_response(notifications, expected, Instant::now() + RESPONSE_TIMEOUT).await;
    if matches!(outcome, Err(Error::Timeout)) {
        warn!(request_len = request.len(), "bluetooth response timed out");
        drain_until_quiet(notifications).await;
    }
    outcome
}

async fn write_chunks(peripheral: &Peripheral, write_char: &Characteristic, request: &[u8]) -> Result<()> {
    for chunk in request.chunks(WRITE_CHUNK_LEN) {
        peripheral
            .write(write_char, chunk, WriteType::WithoutResponse)
            .await
            .map_err(|error| Error::from(BleError::from(error)))?;
    }
    Ok(())
}

/// Assembles notification chunks into frames until one matches `expected` or `deadline` passes.
pub(super) async fn await_matching_response(
    notifications: &mut NotificationStream,
    expected: EchoHeader,
    deadline: Instant,
) -> Result<BytesBuffer> {
    let mut assembler = ResponseAssembler::new();
    let mut discarded = 0usize;
    loop {
        let chunk = next_chunk_before(notifications, deadline).await?;
        let Some(payload) = assembler.push(&chunk)? else {
            continue;
        };
        if response_matches_request(&payload, expected) {
            debug!(response_len = payload.len(), discarded, "bluetooth response complete");
            return Ok(BytesBuffer::new(payload));
        }
        discarded = discarded.saturating_add(1);
        warn!(response_len = payload.len(), discarded, "bluetooth frame discarded: echo header mismatch");
    }
}

async fn next_chunk_before(notifications: &mut NotificationStream, deadline: Instant) -> Result<Vec<u8>> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    match remaining.is_zero() {
        true => Err(Error::Timeout),
        false => timeout(remaining, notifications.next())
            .await
            .map_err(|_| Error::Timeout)?
            .map(|notification| notification.value)
            .ok_or(Error::ConnectionClosed),
    }
}
