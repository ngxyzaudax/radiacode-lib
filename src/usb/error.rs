use crate::error::Error;

#[derive(Debug)]
pub(super) enum UsbError {
    DeviceNotFound,
    PermissionDenied,
    EmptyRead,
    ResponseMismatch,
    Protocol(Error),
    Usb(rusb::Error),
}

impl From<rusb::Error> for UsbError {
    fn from(error: rusb::Error) -> Self {
        Self::Usb(error)
    }
}

impl From<Error> for UsbError {
    fn from(error: Error) -> Self {
        Self::Protocol(error)
    }
}

impl From<UsbError> for Error {
    fn from(error: UsbError) -> Self {
        match error {
            UsbError::DeviceNotFound => Error::DeviceNotFound,
            UsbError::PermissionDenied | UsbError::Usb(rusb::Error::Access) => Error::UsbPermissionDenied,
            UsbError::EmptyRead | UsbError::ResponseMismatch | UsbError::Usb(rusb::Error::NoDevice) => {
                Error::ConnectionClosed
            }
            UsbError::Usb(rusb::Error::Timeout) => Error::Timeout,
            UsbError::Protocol(inner) => inner,
            UsbError::Usb(other) => Error::TransportUnavailable(format!("usb: {other}")),
        }
    }
}

pub(super) fn usb_result<T>(result: std::result::Result<T, UsbError>) -> crate::Result<T> {
    result.map_err(Error::from)
}
