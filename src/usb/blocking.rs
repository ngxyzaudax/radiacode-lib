use tokio::runtime::Handle;

use crate::error::{Error, Result};

/// Runs blocking libusb work on the Tokio blocking pool.
///
/// Returns [`Error::TransportUnavailable`] instead of panicking when called
/// outside a Tokio runtime or when the blocking task is cancelled.
pub(super) async fn run_blocking<T, F>(work: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    let runtime = Handle::try_current().map_err(|error| Error::TransportUnavailable(format!("usb: {error}")))?;
    runtime
        .spawn_blocking(work)
        .await
        .map_err(|error| Error::TransportUnavailable(format!("usb blocking task failed: {error}")))?
}
