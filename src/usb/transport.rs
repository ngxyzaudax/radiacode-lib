use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use rusb::{Context, DeviceHandle};
use tracing::{debug, info, warn};

use super::blocking::run_blocking;
use super::constants::{ENDPOINT_OUT, EXCHANGE_TIMEOUT, INTERFACE};
use super::error::{usb_result, UsbError};
use super::open::open_handle;
use super::read::{drain_handle, read_matching_response};
use crate::error::{Error, Result};
use crate::protocol::{framed_request_header, BytesBuffer};
use crate::transport::Transport;

type SharedHandle = Arc<Mutex<DeviceHandle<Context>>>;

/// USB bulk transport to one RadiaCode device.
///
/// Requires a Tokio runtime: libusb calls run on the blocking thread pool.
/// On Linux the device node must be accessible; see [`access_status`](super::access_status).
#[derive(Debug)]
pub struct UsbTransport {
    handle: SharedHandle,
}

impl UsbTransport {
    /// Opens the device whose serial is `serial`, or `usb-<address>` for devices without a readable serial.
    ///
    /// When exactly one RadiaCode is attached it is opened regardless of `serial`.
    pub async fn open(serial: &str) -> Result<Self> {
        let target = serial.to_string();
        info!(serial = %target, "usb transport opening");
        run_blocking(move || usb_result(open_shared(&target))).await
    }

    /// Opens the first attached RadiaCode device.
    pub async fn open_first() -> Result<Self> {
        info!("usb transport opening first device");
        run_blocking(|| usb_result(open_first_shared())).await
    }
}

fn open_shared(target: &str) -> std::result::Result<UsbTransport, UsbError> {
    let context = Context::new()?;
    open_handle(&context, Some(target)).map(wrap_handle)
}

fn open_first_shared() -> std::result::Result<UsbTransport, UsbError> {
    let context = Context::new()?;
    open_handle(&context, None).map(wrap_handle)
}

fn wrap_handle(handle: DeviceHandle<Context>) -> UsbTransport {
    UsbTransport { handle: Arc::new(Mutex::new(handle)) }
}

fn lock(handle: &SharedHandle) -> std::result::Result<MutexGuard<'_, DeviceHandle<Context>>, UsbError> {
    handle.lock().map_err(|_| UsbError::Protocol(Error::TransportUnavailable("usb handle lock poisoned".into())))
}

fn execute_blocking(handle: &SharedHandle, request: &[u8]) -> std::result::Result<BytesBuffer, UsbError> {
    let expected = framed_request_header(request)?;
    let mut guard = lock(handle)?;
    debug!(request_len = request.len(), "usb request");
    drain_handle(&mut guard)?;
    write_request(&mut guard, request)?;
    read_matching_response(&mut guard, expected)
}

fn write_request(handle: &mut DeviceHandle<Context>, request: &[u8]) -> std::result::Result<(), UsbError> {
    let written = handle.write_bulk(ENDPOINT_OUT, request, EXCHANGE_TIMEOUT)?;
    if written != request.len() {
        warn!(written, expected = request.len(), "usb bulk write incomplete");
    }
    Ok(())
}

fn drain_blocking(handle: &SharedHandle) -> std::result::Result<(), UsbError> {
    drain_handle(&mut *lock(handle)?)
}

fn release_blocking(handle: &SharedHandle) -> std::result::Result<(), UsbError> {
    Ok(lock(handle)?.release_interface(INTERFACE)?)
}

#[async_trait]
impl Transport for UsbTransport {
    async fn execute(&mut self, request: &[u8]) -> Result<BytesBuffer> {
        let handle = Arc::clone(&self.handle);
        let request = request.to_vec();
        run_blocking(move || usb_result(execute_blocking(&handle, &request))).await
    }

    async fn drain_link(&mut self) {
        let handle = Arc::clone(&self.handle);
        if let Err(error) = run_blocking(move || usb_result(drain_blocking(&handle))).await {
            debug!(%error, "usb drain failed");
        }
    }

    async fn disconnect(self: Box<Self>) -> Result<()> {
        info!("usb transport closing");
        let handle = Arc::clone(&self.handle);
        run_blocking(move || usb_result(release_blocking(&handle))).await
    }
}
