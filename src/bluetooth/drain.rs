use std::time::Duration;

use btleplug::api::ValueNotification;
use futures::stream::BoxStream;
use futures::StreamExt;
use tokio::time::{timeout, Instant};
use tracing::debug;

/// Notifications from the RadiaCode notify characteristic.
pub(super) type NotificationStream = BoxStream<'static, ValueNotification>;

pub(super) const QUIET_GAP: Duration = Duration::from_millis(120);
const MAX_DRAIN: Duration = Duration::from_millis(2500);
const COMMAND_DRAIN: Duration = Duration::from_millis(400);
const SETTLE_DRAIN: Duration = Duration::from_millis(500);
const PEEK_WAIT: Duration = Duration::from_millis(1);
const POLL_SLICE: Duration = Duration::from_millis(25);

/// How a drain decides it is done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DrainMode {
    /// Return at once on an idle stream; otherwise drain until a quiet gap.
    PeekThenQuiet,
    /// Always wait for a quiet gap (or the deadline).
    WaitForQuiet,
}

pub(super) async fn drain_until_quiet(notifications: &mut NotificationStream) {
    drain_for(notifications, MAX_DRAIN, DrainMode::WaitForQuiet).await;
}

pub(super) async fn drain_for_settle(notifications: &mut NotificationStream) {
    drain_for(notifications, SETTLE_DRAIN, DrainMode::WaitForQuiet).await;
}

pub(super) async fn drain_before_command(notifications: &mut NotificationStream) {
    drain_for(notifications, COMMAND_DRAIN, DrainMode::PeekThenQuiet).await;
}

pub(super) async fn drain_for(notifications: &mut NotificationStream, max_drain: Duration, mode: DrainMode) {
    let peeked = match mode {
        DrainMode::PeekThenQuiet if !peek_notification(notifications).await => return,
        DrainMode::PeekThenQuiet => 1,
        DrainMode::WaitForQuiet => 0,
    };
    let drained = drain_until_gap(notifications, Instant::now() + max_drain).await.saturating_add(peeked);
    if drained > 0 {
        debug!(drained, ?mode, "stale bluetooth notifications drained");
    }
}

async fn peek_notification(notifications: &mut NotificationStream) -> bool {
    matches!(timeout(PEEK_WAIT, notifications.next()).await, Ok(Some(_)))
}

async fn drain_until_gap(notifications: &mut NotificationStream, deadline: Instant) -> usize {
    let mut drained = 0usize;
    let mut last_received = Instant::now();
    loop {
        let slice = deadline.saturating_duration_since(Instant::now());
        if slice.is_zero() {
            return drained;
        }
        match timeout(slice.min(POLL_SLICE), notifications.next()).await {
            Ok(Some(_)) => {
                drained = drained.saturating_add(1);
                last_received = Instant::now();
            }
            Ok(None) => return drained,
            Err(_) if last_received.elapsed() >= QUIET_GAP => return drained,
            Err(_) => {}
        }
    }
}
