use rusb::{Context, Device, DeviceHandle, UsbContext};
use tracing::debug;

use super::constants::{INTERFACE, PRODUCT_ID, STRING_READ_TIMEOUT, VENDOR_ID};
use super::error::UsbError;
use super::identity::{select_device_index, UsbIdentity};
use super::read::drain_handle;

pub(super) fn is_radiacode_device<T: UsbContext>(device: &Device<T>) -> bool {
    device
        .device_descriptor()
        .is_ok_and(|descriptor| descriptor.vendor_id() == VENDOR_ID && descriptor.product_id() == PRODUCT_ID)
}

pub(super) fn radiacode_devices(context: &Context) -> Result<Vec<Device<Context>>, UsbError> {
    Ok(context.devices()?.iter().filter(is_radiacode_device).collect())
}

pub(super) fn open_handle(context: &Context, target: Option<&str>) -> Result<DeviceHandle<Context>, UsbError> {
    let candidates = radiacode_devices(context)?;
    let device = pick_device(&candidates, target)?;
    let mut handle = device.open().map_err(map_open_error)?;
    detach_kernel_driver(&mut handle);
    handle.claim_interface(INTERFACE)?;
    drain_handle(&mut handle)?;
    Ok(handle)
}

pub(super) fn identity_of(device: &Device<Context>) -> UsbIdentity {
    let descriptor = device.device_descriptor().ok();
    let handle = device.open().ok();
    let read = |index: Option<u8>| handle.as_ref().and_then(|opened| read_string(opened, index));
    let identity = UsbIdentity {
        address: device.address(),
        serial: descriptor.as_ref().and_then(|descriptor| read(descriptor.serial_number_string_index())),
        product: descriptor.as_ref().and_then(|descriptor| read(descriptor.product_string_index())),
    };
    debug!(address = identity.address, serial = ?identity.serial, product = ?identity.product, "usb device matched");
    identity
}

fn pick_device<'a>(devices: &'a [Device<Context>], target: Option<&str>) -> Result<&'a Device<Context>, UsbError> {
    let index = match target {
        None => Some(0),
        Some(target) => select_device_index(target, &addresses_of(devices), &serials_of(devices)),
    };
    index.and_then(|index| devices.get(index)).ok_or(UsbError::DeviceNotFound)
}

fn addresses_of(devices: &[Device<Context>]) -> Vec<u8> {
    devices.iter().map(Device::address).collect()
}

fn serials_of(devices: &[Device<Context>]) -> Vec<Option<String>> {
    devices.iter().map(|device| identity_of(device).serial).collect()
}

fn map_open_error(error: rusb::Error) -> UsbError {
    match error {
        rusb::Error::Access => UsbError::PermissionDenied,
        other => UsbError::Usb(other),
    }
}

fn detach_kernel_driver(handle: &mut DeviceHandle<Context>) {
    if handle.kernel_driver_active(INTERFACE).unwrap_or(false) {
        let _ = handle.detach_kernel_driver(INTERFACE);
    }
}

fn read_string<T: UsbContext>(handle: &DeviceHandle<T>, index: Option<u8>) -> Option<String> {
    let index = index.filter(|index| *index != 0)?;
    let language = *handle.read_languages(STRING_READ_TIMEOUT).ok()?.first()?;
    handle.read_string_descriptor(language, index, STRING_READ_TIMEOUT).ok()
}
