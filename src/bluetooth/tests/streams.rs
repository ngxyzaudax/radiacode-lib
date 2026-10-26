use std::time::Duration;

use btleplug::api::ValueNotification;
use futures::{stream, StreamExt};
use tokio::time::Instant;

use super::super::drain::{drain_for, DrainMode, NotificationStream, QUIET_GAP};
use super::super::execute::await_matching_response;
use crate::error::Error;
use crate::protocol::{request_header, Command, EchoHeader};

fn notification(value: Vec<u8>) -> ValueNotification {
    ValueNotification { uuid: uuid::Uuid::nil(), value }
}

fn reply(header: EchoHeader, body: &[u8]) -> Vec<u8> {
    let payload: Vec<u8> = header.iter().chain(body).copied().collect();
    u32::try_from(payload.len()).unwrap().to_le_bytes().into_iter().chain(payload).collect()
}

fn stream_of(chunks: Vec<Vec<u8>>) -> NotificationStream {
    stream::iter(chunks.into_iter().map(notification)).boxed()
}

fn far_deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}

#[tokio::test]
async fn chunked_response_is_reassembled() {
    let expected = request_header(Command::GetVersion, 0x81);
    let frame = reply(expected, &[1, 2, 3, 4, 5, 6]);
    let (head, tail) = frame.split_at(7);
    let mut notifications = stream_of(vec![head.to_vec(), tail.to_vec()]);
    let response = await_matching_response(&mut notifications, expected, far_deadline()).await.unwrap();
    assert_eq!(response.size(), 10);
}

#[tokio::test]
async fn unrelated_frames_are_discarded_before_match() {
    let expected = request_header(Command::GetVersion, 0x81);
    let stale = reply(request_header(Command::GetSerial, 0x82), &[9]);
    let mut notifications = stream_of(vec![stale, reply(expected, &[7])]);
    let response = await_matching_response(&mut notifications, expected, far_deadline()).await.unwrap();
    assert_eq!(response.size(), 5);
}

#[tokio::test]
async fn closed_stream_reports_connection_closed() {
    let expected = request_header(Command::GetVersion, 0x81);
    let outcome = await_matching_response(&mut stream_of(Vec::new()), expected, far_deadline()).await;
    assert!(matches!(outcome, Err(Error::ConnectionClosed)));
}

#[tokio::test(start_paused = true)]
async fn silent_stream_times_out_at_deadline() {
    let mut notifications: NotificationStream = stream::pending().boxed();
    let deadline = Instant::now() + Duration::from_millis(50);
    let expected = request_header(Command::GetVersion, 0x81);
    let outcome = await_matching_response(&mut notifications, expected, deadline).await;
    assert!(matches!(outcome, Err(Error::Timeout)));
}

#[tokio::test]
async fn peek_then_quiet_returns_immediately_on_idle_stream() {
    let mut notifications: NotificationStream = stream::empty().boxed();
    let started = std::time::Instant::now();
    drain_for(&mut notifications, Duration::from_secs(1), DrainMode::PeekThenQuiet).await;
    assert!(started.elapsed() < Duration::from_millis(50));
}

#[tokio::test]
async fn wait_for_quiet_consumes_late_notification() {
    let late = async {
        tokio::time::sleep(Duration::from_millis(5)).await;
        notification(vec![0xAA])
    };
    let mut notifications: NotificationStream = stream::once(late).boxed();
    drain_for(&mut notifications, QUIET_GAP + Duration::from_millis(200), DrainMode::WaitForQuiet).await;
    assert!(notifications.next().await.is_none());
}

#[tokio::test]
async fn peek_then_quiet_drains_immediate_backlog() {
    let backlog = stream::iter(vec![notification(vec![1]), notification(vec![2])]);
    let mut notifications: NotificationStream = backlog.chain(stream::pending()).boxed();
    drain_for(&mut notifications, QUIET_GAP + Duration::from_millis(100), DrainMode::PeekThenQuiet).await;
    assert!(tokio::time::timeout(Duration::from_millis(20), notifications.next()).await.is_err());
}
