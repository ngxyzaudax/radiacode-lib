#![allow(dead_code, unused_imports)]

pub mod encode;
pub mod fixtures;
pub mod wire;

mod dispatch;
mod fake_transport;
mod virtual_device;

pub use fake_transport::DeviceHandle;
pub use virtual_device::{ScriptedReply, VirtualDevice, LEDS_SFR_FILE};

use radiacode_lib::{OpenOptions, RadiaCode, Result};

pub async fn try_open_with(device: VirtualDevice, options: OpenOptions) -> (Result<RadiaCode>, DeviceHandle) {
    let handle = DeviceHandle::new(device);
    let outcome = RadiaCode::open(handle.transport(Some(-60)), options).await;
    (outcome, handle)
}

pub async fn try_open_device(device: VirtualDevice) -> (Result<RadiaCode>, DeviceHandle) {
    try_open_with(device, OpenOptions::default()).await
}

pub async fn open_device(device: VirtualDevice) -> (RadiaCode, DeviceHandle) {
    let (outcome, handle) = try_open_device(device).await;
    (outcome.unwrap(), handle)
}
