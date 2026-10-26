//! Crate-wide error type and result alias.

use thiserror::Error as ThisError;

const LINK_LOSS_MARKERS: [&str; 6] =
    ["not connected", "disconnected", "link has been lost", "broken pipe", "connection reset", "device not found"];

/// Errors produced by protocol decoding, transports and the device client.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    /// The device did not answer before the transport deadline.
    #[error("response timed out")]
    Timeout,
    /// The link to the device was closed or lost.
    #[error("connection closed")]
    ConnectionClosed,
    /// A frame or payload did not have the expected shape.
    #[error("protocol mismatch: expected {expected}, got {got}")]
    ProtocolMismatch {
        /// Description of what the decoder expected.
        expected: String,
        /// Description of what was actually received.
        got: String,
    },
    /// The device answered with a return code other than success.
    #[error("unexpected device return code {0}")]
    UnexpectedReturnCode(u32),
    /// A payload ended before a field could be read.
    #[error("buffer underrun: need {need} bytes, have {have}")]
    BufferUnderrun {
        /// Bytes required by the field being read.
        need: usize,
        /// Bytes remaining in the payload.
        have: usize,
    },
    /// A batch register read reported that no register was readable.
    #[error("vsfr batch returned no readable registers")]
    VsfrBatchEmpty,
    /// The transport layer reported a failure that is not a link loss.
    #[error("transport unavailable: {0}")]
    TransportUnavailable(String),
    /// No matching device was found during discovery or connect.
    #[error("device not found")]
    DeviceNotFound,
    /// The device firmware is older than the minimum supported version.
    #[error("incompatible firmware {major}.{minor}, >=4.8 required")]
    IncompatibleFirmware {
        /// Target firmware major version.
        major: u16,
        /// Target firmware minor version.
        minor: u16,
    },
    /// The device data buffer does not contain rate records yet.
    #[error("live rates not yet available in device buffer")]
    MonitorDataPending,
    /// The operating system denied access to the USB device.
    #[error("usb permission denied; install the radiacode udev rule and replug the device")]
    UsbPermissionDenied,
    /// A caller supplied a value the device cannot accept.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Builds a [`Error::ProtocolMismatch`] from expected and received descriptions.
    pub fn mismatch(expected: impl Into<String>, got: impl Into<String>) -> Self {
        Self::ProtocolMismatch { expected: expected.into(), got: got.into() }
    }

    /// Returns `true` for failures that are worth retrying on the same link.
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::Timeout | Self::ProtocolMismatch { .. } | Self::BufferUnderrun { .. })
    }

    /// Returns `true` when the error indicates that the device link is gone.
    pub fn is_connection_lost(&self) -> bool {
        match self {
            Self::ConnectionClosed => true,
            Self::TransportUnavailable(message) => message_reports_link_loss(message),
            _ => false,
        }
    }

    /// Returns `true` for [`Error::Timeout`].
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }

    /// Returns `true` for [`Error::UsbPermissionDenied`].
    pub fn is_usb_permission_denied(&self) -> bool {
        matches!(self, Self::UsbPermissionDenied)
    }
}

fn message_reports_link_loss(message: &str) -> bool {
    let lowered = message.to_ascii_lowercase();
    LINK_LOSS_MARKERS.iter().any(|marker| lowered.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn timeout_is_transient_not_connection_lost() {
        assert!(Error::Timeout.is_transient());
        assert!(Error::Timeout.is_timeout());
        assert!(!Error::Timeout.is_connection_lost());
    }

    #[test]
    fn connection_closed_is_link_loss_not_transient() {
        assert!(Error::ConnectionClosed.is_connection_lost());
        assert!(!Error::ConnectionClosed.is_transient());
    }

    #[test]
    fn transport_messages_are_classified_by_content() {
        let lost = Error::TransportUnavailable("Device Disconnected".into());
        let busy = Error::TransportUnavailable("adapter busy".into());
        assert!(lost.is_connection_lost());
        assert!(!busy.is_connection_lost());
    }

    #[test]
    fn decode_errors_are_transient_and_display_is_stable() {
        assert!(Error::mismatch("a", "b").is_transient());
        assert!(Error::BufferUnderrun { need: 1, have: 0 }.is_transient());
        assert!(!Error::UnexpectedReturnCode(3).is_transient());
        assert!(!Error::InvalidArgument("x".into()).is_transient());
        assert!(Error::UsbPermissionDenied.is_usb_permission_denied());
        assert_eq!(Error::UnexpectedReturnCode(3).to_string(), "unexpected device return code 3");
    }
}
