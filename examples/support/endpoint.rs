use radiacode_lib::{discover_devices, DeviceEndpoint, DiscoveryOptions, Error, Result};

/// Resolves `usb:<serial>`, `ble:<address>`, or the first discovered device when no argument is given.
pub async fn endpoint_from_args() -> Result<DeviceEndpoint> {
    match std::env::args().nth(1) {
        Some(argument) => parse_endpoint(&argument),
        None => first_discovered().await,
    }
}

fn parse_endpoint(argument: &str) -> Result<DeviceEndpoint> {
    match argument.split_once(':') {
        Some(("usb", serial)) => Ok(DeviceEndpoint::Usb { serial: serial.into() }),
        Some(("ble", address)) => Ok(DeviceEndpoint::Bluetooth { address: address.into() }),
        _ => Err(Error::InvalidArgument(format!("expected usb:<serial> or ble:<address>, got {argument}"))),
    }
}

async fn first_discovered() -> Result<DeviceEndpoint> {
    let devices = discover_devices(&DiscoveryOptions::default()).await?;
    devices.into_iter().next().map(|device| device.endpoint).ok_or(Error::DeviceNotFound)
}
