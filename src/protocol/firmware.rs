use crate::error::{Error, Result};
use crate::protocol::buffer::BytesBuffer;

const NUL_PADDING_TRIM_FLOOR: usize = 51;
const SUCCESS_RETCODE: u32 = 1;

/// Oldest target firmware supported by this crate, as `(major, minor)`.
pub const MIN_SUPPORTED_FIRMWARE: (u16, u16) = (4, 8);

/// Version and build date of one firmware image.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FirmwareVersion {
    /// Major version.
    pub major: u16,
    /// Minor version.
    pub minor: u16,
    /// Build date string as reported by the device.
    pub date: String,
}

impl FirmwareVersion {
    /// Returns `true` when this version is at least `major.minor`.
    pub fn is_at_least(&self, major: u16, minor: u16) -> bool {
        (self.major, self.minor) >= (major, minor)
    }
}

/// Bootloader and application firmware versions (`GET_VERSION` response).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceVersions {
    /// Bootloader firmware.
    pub boot: FirmwareVersion,
    /// Application (target) firmware.
    pub target: FirmwareVersion,
}

impl DeviceVersions {
    /// Returns `true` when the target firmware meets [`MIN_SUPPORTED_FIRMWARE`].
    pub fn is_supported(&self) -> bool {
        let (major, minor) = MIN_SUPPORTED_FIRMWARE;
        self.target.is_at_least(major, minor)
    }
}

/// Decodes a `GET_VERSION` response body.
///
/// Tolerates trailing NUL padding and an optional leading success return code,
/// both of which have been observed on different firmware revisions.
pub fn decode_fw_version(response: BytesBuffer) -> Result<DeviceVersions> {
    decode_candidates(response.clone())
        .into_iter()
        .find_map(|candidate| decode_fw_version_body(candidate).ok())
        .map_or_else(|| decode_fw_version_body(response), Ok)
}

fn decode_candidates(response: BytesBuffer) -> Vec<BytesBuffer> {
    let trimmed: Vec<BytesBuffer> = std::iter::successors(Some(response), trim_one_padding_byte).collect();
    let retcode_skipped = trimmed.last().and_then(skip_success_retcode);
    trimmed.into_iter().chain(retcode_skipped).collect()
}

fn trim_one_padding_byte(buffer: &BytesBuffer) -> Option<BytesBuffer> {
    let padded = buffer.size() > NUL_PADDING_TRIM_FLOOR && buffer.data().last() == Some(&0);
    padded.then(|| buffer.truncated(buffer.size().saturating_sub(1)))
}

fn skip_success_retcode(buffer: &BytesBuffer) -> Option<BytesBuffer> {
    let mut candidate = buffer.clone();
    (candidate.take_u32_le().ok() == Some(SUCCESS_RETCODE)).then_some(candidate)
}

fn decode_fw_version_body(mut response: BytesBuffer) -> Result<DeviceVersions> {
    let boot = take_firmware_version(&mut response)?;
    let target = take_firmware_version(&mut response)?;
    match response.size() {
        0 => Ok(DeviceVersions { boot, target }),
        trailing => Err(Error::mismatch("empty fw_version tail", format!("{trailing} trailing bytes"))),
    }
}

fn take_firmware_version(response: &mut BytesBuffer) -> Result<FirmwareVersion> {
    let minor = response.take_u16_le()?;
    let major = response.take_u16_le()?;
    let date = response.take_length_prefixed_ascii()?;
    Ok(FirmwareVersion { major, minor, date: date.trim_end_matches('\0').to_string() })
}
