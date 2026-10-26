use crate::error::Result;
use crate::protocol::buffer::BytesBuffer;

/// Firmware image signature returned by `FW_SIGNATURE`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FirmwareSignature {
    /// Image signature word.
    pub signature: u32,
    /// Firmware file name.
    pub filename: String,
    /// Firmware identification string.
    pub id_string: String,
}

/// Decodes a `FW_SIGNATURE` response.
pub fn decode_fw_signature(mut response: BytesBuffer) -> Result<FirmwareSignature> {
    Ok(FirmwareSignature {
        signature: response.take_u32_le()?,
        filename: response.take_length_prefixed_ascii()?,
        id_string: response.take_length_prefixed_ascii()?,
    })
}

/// Decodes a `GET_SERIAL` response into dash-separated upper-case hex groups.
pub fn decode_hw_serial(mut response: BytesBuffer) -> Result<String> {
    let group_count = response.take_len_u32()? / 4;
    (0..group_count)
        .map(|_| response.take_u32_le().map(|word| format!("{word:08X}")))
        .collect::<Result<Vec<String>>>()
        .map(|groups| groups.join("-"))
}

/// Decodes a `GET_STATUS` response into the raw status word.
pub fn decode_status(mut response: BytesBuffer) -> Result<u32> {
    response.take_u32_le()
}

/// Decodes a text virtual string (serial number, text message, `SFR_FILE`) lossily as UTF-8.
pub fn decode_text(payload: &BytesBuffer) -> String {
    String::from_utf8_lossy(payload.data()).into_owned()
}

/// Extracts `SpecFormatVersion` from configuration text, defaulting to `0`.
pub fn spectrum_format_version(configuration: &str) -> u32 {
    configuration
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("SpecFormatVersion="))
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0)
}
