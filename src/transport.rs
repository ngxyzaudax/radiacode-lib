//! The [`Transport`] abstraction implemented by the USB and Bluetooth back ends.

use async_trait::async_trait;

use crate::error::Result;
use crate::protocol::BytesBuffer;

/// A byte link to one RadiaCode device.
///
/// Implementations send a fully framed request (see
/// [`build_request`](crate::protocol::build_request)) and return the matching
/// response payload without its length prefix, but still starting with the
/// four echoed header bytes. Futures are `Send`, so a client can live inside
/// a multi-threaded runtime task.
#[async_trait]
pub trait Transport: Send {
    /// Sends `request` and waits for the response with the same echo header.
    async fn execute(&mut self, request: &[u8]) -> Result<BytesBuffer>;

    /// Discards any unsolicited or stale frames buffered on the link.
    async fn drain_link(&mut self);

    /// Closes the link.
    async fn disconnect(self: Box<Self>) -> Result<()>;

    /// Last known link RSSI in dBm, if the link reports one.
    async fn link_rssi_dbm(&mut self) -> Option<i16> {
        None
    }

    /// Samples a fresh RSSI value, falling back to [`Transport::link_rssi_dbm`].
    async fn sample_link_rssi_dbm(&mut self) -> Option<i16> {
        self.link_rssi_dbm().await
    }
}

/// Owned, type-erased transport.
pub type BoxedTransport = Box<dyn Transport>;
