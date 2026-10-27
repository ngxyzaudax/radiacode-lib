use std::future::Future;
use std::time::Duration;

use tracing::info;

use crate::error::{Error, Result};

/// How many connect attempts are made, including the first.
pub const FAST_RECONNECT_ATTEMPTS: usize = 3;
/// Pause before a fresh-scan retry after a fast connect failure.
pub const FAST_RECONNECT_BACKOFF: Duration = Duration::from_millis(750);

/// Returns the backoff before the next attempt, or `None` when the failure is final.
pub fn fast_reconnect_backoff(attempt: usize, error: &Error) -> Option<Duration> {
    let attempts_remain = attempt.saturating_add(1) < FAST_RECONNECT_ATTEMPTS;
    (attempts_remain && is_fast_reconnect_failure(error)).then_some(FAST_RECONNECT_BACKOFF)
}

/// Failures seen when a connect follows a disconnect within about a second.
pub fn is_fast_reconnect_failure(error: &Error) -> bool {
    match error {
        Error::DeviceNotFound => true,
        Error::TransportUnavailable(message) => message.to_ascii_lowercase().contains("le-connection-abort-by-local"),
        _ => false,
    }
}

/// Runs `open` up to [`FAST_RECONNECT_ATTEMPTS`] times.
///
/// The first call uses `fresh`. After [`Error::DeviceNotFound`] or a local
/// connection abort, later calls pass `fresh = true` so the caller rescans.
pub async fn retry_connect<T, F, Fut>(fresh: bool, mut open: F) -> Result<T>
where
    F: FnMut(bool) -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut fresh = fresh;
    let mut attempt = 0usize;
    loop {
        match open(fresh).await {
            Ok(value) => return Ok(value),
            Err(error) => match fast_reconnect_backoff(attempt, &error) {
                Some(delay) => {
                    info!(attempt, %error, "bluetooth connect failed; retrying with a fresh scan");
                    tokio::time::sleep(delay).await;
                    fresh = true;
                    attempt = attempt.saturating_add(1);
                }
                None => return Err(error),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{
        fast_reconnect_backoff, is_fast_reconnect_failure, retry_connect, FAST_RECONNECT_ATTEMPTS,
        FAST_RECONNECT_BACKOFF,
    };
    use crate::error::Error;

    fn abort() -> Error {
        Error::TransportUnavailable("bluetooth: le-connection-abort-by-local".into())
    }

    #[test]
    fn only_fast_failures_are_retried_and_the_budget_is_bounded() {
        assert!(is_fast_reconnect_failure(&Error::DeviceNotFound));
        assert!(is_fast_reconnect_failure(&abort()));
        assert!(!is_fast_reconnect_failure(&Error::IncompatibleFirmware { major: 4, minor: 0 }));
        assert_eq!(fast_reconnect_backoff(0, &Error::DeviceNotFound), Some(FAST_RECONNECT_BACKOFF));
        assert_eq!(fast_reconnect_backoff(FAST_RECONNECT_ATTEMPTS - 1, &abort()), None);
        assert_eq!(fast_reconnect_backoff(0, &Error::Timeout), None);
    }

    #[tokio::test(start_paused = true)]
    async fn abort_then_fresh_scan_succeeds_within_the_attempt_budget() {
        let calls = AtomicUsize::new(0);
        let outcome = retry_connect(false, |fresh| {
            let index = calls.fetch_add(1, Ordering::SeqCst);
            async move {
                match (index, fresh) {
                    (0, false) => Err(abort()),
                    (1, true) => Ok("ready"),
                    _ => Err(Error::InvalidArgument("unexpected attempt".into())),
                }
            }
        })
        .await;
        assert_eq!(outcome.unwrap_or("err"), "ready");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn repeated_device_not_found_stops_after_three_attempts() {
        let calls = AtomicUsize::new(0);
        let outcome: Result<(), Error> = retry_connect(true, |_fresh| {
            calls.fetch_add(1, Ordering::SeqCst);
            async { Err(Error::DeviceNotFound) }
        })
        .await;
        assert!(matches!(outcome, Err(Error::DeviceNotFound)));
        assert_eq!(calls.load(Ordering::SeqCst), FAST_RECONNECT_ATTEMPTS);
    }

    #[tokio::test(start_paused = true)]
    async fn other_errors_are_not_retried() {
        let calls = AtomicUsize::new(0);
        let outcome: Result<(), Error> = retry_connect(false, |_fresh| {
            calls.fetch_add(1, Ordering::SeqCst);
            async { Err(Error::Timeout) }
        })
        .await;
        assert!(matches!(outcome, Err(Error::Timeout)));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
