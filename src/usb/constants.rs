use std::time::Duration;

/// USB vendor id of RadiaCode devices (STMicroelectronics).
pub const VENDOR_ID: u16 = 0x0483;
/// USB product id of RadiaCode devices.
pub const PRODUCT_ID: u16 = 0xF123;

pub(super) const INTERFACE: u8 = 0;
pub(super) const ENDPOINT_OUT: u8 = 0x01;
pub(super) const ENDPOINT_IN: u8 = 0x81;
pub(super) const READ_CHUNK_LEN: usize = 256;
pub(super) const EXCHANGE_TIMEOUT: Duration = Duration::from_millis(3000);
pub(super) const DRAIN_TIMEOUT: Duration = Duration::from_millis(100);
pub(super) const STRING_READ_TIMEOUT: Duration = Duration::from_millis(100);
pub(super) const EMPTY_READ_RETRIES: usize = 3;
pub(super) const MAX_UNRELATED_FRAMES: usize = 4;
pub(super) const MAX_DRAIN_CHUNKS: usize = 1024;
pub(super) const ADDRESS_SERIAL_PREFIX: &str = "usb-";
