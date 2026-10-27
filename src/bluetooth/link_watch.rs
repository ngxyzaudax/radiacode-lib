use std::time::Duration;

/// How often a pending Bluetooth read checks that the link is still up.
pub(super) const LINK_WATCH: Duration = Duration::from_millis(200);

/// What the notification wait observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WaitEvent {
    /// A notification chunk arrived.
    Chunk,
    /// The notification stream ended.
    StreamEnded,
    /// The watch interval elapsed with no chunk.
    SliceElapsed,
}

/// What the pending request should do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WaitAction {
    /// Deliver the chunk to the assembler.
    Deliver,
    /// The link is gone; fail the request now.
    Closed,
    /// Keep waiting until the response deadline.
    PollAgain,
}

/// Maps one watch step to the next action.
///
/// A slice with the link down fails the request instead of waiting out the
/// full response timeout.
pub(super) fn wait_action(event: WaitEvent, link_up: bool) -> WaitAction {
    match (event, link_up) {
        (WaitEvent::Chunk, _) => WaitAction::Deliver,
        (WaitEvent::StreamEnded, _) => WaitAction::Closed,
        (WaitEvent::SliceElapsed, true) => WaitAction::PollAgain,
        (WaitEvent::SliceElapsed, false) => WaitAction::Closed,
    }
}

#[cfg(test)]
mod tests {
    use super::{wait_action, WaitAction, WaitEvent};

    #[test]
    fn a_dropped_link_closes_the_pending_request() {
        assert_eq!(wait_action(WaitEvent::SliceElapsed, false), WaitAction::Closed);
        assert_eq!(wait_action(WaitEvent::StreamEnded, true), WaitAction::Closed);
    }

    #[test]
    fn a_live_link_keeps_waiting_and_a_chunk_is_delivered() {
        assert_eq!(wait_action(WaitEvent::SliceElapsed, true), WaitAction::PollAgain);
        assert_eq!(wait_action(WaitEvent::Chunk, false), WaitAction::Deliver);
    }
}
