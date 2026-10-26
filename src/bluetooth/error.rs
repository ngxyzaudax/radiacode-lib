use crate::error::Error;

#[derive(Debug)]
pub(super) enum BleError {
    AdapterNotFound,
    DeviceNotFound,
    CharacteristicMissing,
    InvalidAddress(String),
    Bluetooth(btleplug::Error),
}

impl From<btleplug::Error> for BleError {
    fn from(error: btleplug::Error) -> Self {
        Self::Bluetooth(error)
    }
}

impl From<BleError> for Error {
    fn from(error: BleError) -> Self {
        match error {
            BleError::AdapterNotFound => Error::TransportUnavailable("bluetooth adapter not found".into()),
            BleError::DeviceNotFound => Error::DeviceNotFound,
            BleError::CharacteristicMissing => {
                Error::TransportUnavailable("required bluetooth characteristic missing".into())
            }
            BleError::InvalidAddress(value) => Error::InvalidArgument(format!("invalid bluetooth address: {value}")),
            BleError::Bluetooth(inner) => bluetooth_error(&inner),
        }
    }
}

fn bluetooth_error(error: &btleplug::Error) -> Error {
    match error {
        btleplug::Error::NotConnected => Error::ConnectionClosed,
        btleplug::Error::TimedOut(_) => Error::Timeout,
        btleplug::Error::DeviceNotFound => Error::DeviceNotFound,
        other => classify_message(Error::TransportUnavailable(format!("bluetooth: {other}"))),
    }
}

fn classify_message(error: Error) -> Error {
    match error.is_connection_lost() {
        true => Error::ConnectionClosed,
        false => error,
    }
}

pub(super) fn ble_result<T>(result: std::result::Result<T, BleError>) -> crate::Result<T> {
    result.map_err(Error::from)
}
