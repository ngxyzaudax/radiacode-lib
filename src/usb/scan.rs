use rusb::Context;
use tracing::info;

use super::blocking::run_blocking;
use super::error::{usb_result, UsbError};
use super::identity::discovered_device;
use super::open::{identity_of, radiacode_devices};
use crate::discovery::{sort_and_dedup, DiscoveredDevice};
use crate::error::Result;

/// Enumerates attached RadiaCode devices (VID `0483`, PID `f123`).
///
/// Serial numbers are read from string descriptors when the device node is
/// accessible; otherwise the endpoint falls back to `usb-<address>`.
pub async fn scan_usb_devices() -> Result<Vec<DiscoveredDevice>> {
    run_blocking(|| usb_result(scan_blocking())).await
}

fn scan_blocking() -> std::result::Result<Vec<DiscoveredDevice>, UsbError> {
    info!("usb scan started");
    let context = Context::new()?;
    let found = sort_and_dedup(
        radiacode_devices(&context)?.iter().map(|device| discovered_device(identity_of(device))).collect(),
    );
    info!(count = found.len(), "usb scan completed");
    Ok(found)
}
