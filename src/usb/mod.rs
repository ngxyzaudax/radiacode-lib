//! USB transport (crate feature `usb`, built on `rusb` / libusb).
//!
//! RadiaCode devices enumerate as a vendor-specific bulk interface (VID
//! `0483`, PID `f123`, interface 0, endpoints `0x01` OUT and `0x81` IN).
//! Requests are written in one bulk transfer; responses arrive in chunks of up
//! to 256 bytes and are reassembled with
//! [`ResponseAssembler`](crate::protocol::ResponseAssembler).
//!
//! On Linux, non-root access needs the udev rule in [`UDEV_RULE`]. This crate
//! never installs it; [`access_status`] only diagnoses the situation.

mod access;
mod blocking;
mod constants;
mod error;
mod identity;
mod open;
mod read;
mod scan;
mod transport;

#[cfg(test)]
mod tests;

pub use access::{access_status, rule_grants_access, rule_installed, UsbAccessStatus, UDEV_RULE, UDEV_RULE_PATH};
pub use constants::{PRODUCT_ID, VENDOR_ID};
pub use scan::scan_usb_devices;
pub use transport::UsbTransport;
