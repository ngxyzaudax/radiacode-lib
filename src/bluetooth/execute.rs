use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use futures::StreamExt;
use tokio::time::{timeout, Instant};
use tracing::{debug, warn};

use super::drain::{drain_before_command, drain_until_quiet, NotificationStream};
use super::error::BleError;
use super::link_watch::{wait_action, WaitAction, WaitEvent, LINK_WATCH};
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
    let outcome = await_matching_response(notifications, expected, Instant::now() + RESPONSE_TIMEOUT, || async {
        peripheral.is_connected().await.unwrap_or(false)
    })
    .await;
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
pub(super) async fn await_matching_response<F, Fut>(
    notifications: &mut NotificationStream,
    expected: EchoHeader,
    deadline: Instant,
    mut link_up: F,
) -> Result<BytesBuffer>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let mut assembler = ResponseAssembler::new();
    let mut discarded = 0usize;
    loop {
        let chunk = next_chunk_before(notifications, deadline, &mut link_up).await?;
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

async fn next_chunk_before<F, Fut>(
    notifications: &mut NotificationStream,
    deadline: Instant,
    link_up: &mut F,
) -> Result<Vec<u8>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Error::Timeout);
        }
        let slice = remaining.min(LINK_WATCH);
        let (event, chunk) = match timeout(slice, notifications.next()).await {
            Ok(Some(notification)) => (WaitEvent::Chunk, Some(notification.value)),
            Ok(None) => (WaitEvent::StreamEnded, None),
            Err(_) => (WaitEvent::SliceElapsed, None),
        };
        let connected = link_up().await;
        match (wait_action(event, connected), chunk) {
            (WaitAction::Deliver, Some(value)) => return Ok(value),
            (WaitAction::PollAgain, _) => continue,
            (WaitAction::Closed, _) | (WaitAction::Deliver, None) => return Err(Error::ConnectionClosed),
        }
    }
}
