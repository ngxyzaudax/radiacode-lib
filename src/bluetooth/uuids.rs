use std::time::Duration;

use uuid::Uuid;

/// GATT service advertised by RadiaCode devices.
pub const SERVICE_UUID: Uuid = Uuid::from_u128(0xe632_15e5_7003_49d8_96b0_b024_798f_b901);
/// Characteristic that receives request chunks (write without response).
pub const WRITE_UUID: Uuid = Uuid::from_u128(0xe632_15e6_7003_49d8_96b0_b024_798f_b901);
/// Characteristic that delivers response chunks as notifications.
pub const NOTIFY_UUID: Uuid = Uuid::from_u128(0xe632_15e7_7003_49d8_96b0_b024_798f_b901);

pub(super) const WRITE_CHUNK_LEN: usize = 18;
pub(super) const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);
