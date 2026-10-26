use uuid::Uuid;

use super::error::BleError;

const MAC_OCTETS: usize = 6;
const UNKNOWN_MAC: &str = "00:00:00:00:00:00";

/// Normalises a MAC address to lowercase, colon-separated form.
pub(super) fn normalize_mac(mac: &str) -> Result<String, BleError> {
    let cleaned = mac.trim().to_ascii_lowercase().replace('-', ":");
    let octets: Vec<&str> = cleaned.split(':').collect();
    match octets.len() == MAC_OCTETS && octets.iter().all(|octet| is_hex_octet(octet)) {
        true => Ok(cleaned),
        false => Err(BleError::InvalidAddress(mac.to_string())),
    }
}

/// Normalises a connect target: a MAC address, or a platform peripheral UUID (macOS).
pub(super) fn normalize_target(target: &str) -> Result<String, BleError> {
    normalize_mac(target)
        .or_else(|error| Uuid::parse_str(target.trim()).map(|uuid| uuid.hyphenated().to_string()).map_err(|_| error))
}

/// Stable key for a peripheral: its MAC, or the platform id when the MAC is hidden.
pub(super) fn endpoint_key(address: &str, peripheral_id: &str) -> String {
    match address.eq_ignore_ascii_case(UNKNOWN_MAC) {
        true => peripheral_id.to_ascii_lowercase(),
        false => address.to_ascii_lowercase(),
    }
}

pub(super) fn key_matches(key: &str, normalized_target: &str) -> bool {
    key.eq_ignore_ascii_case(normalized_target)
}

fn is_hex_octet(octet: &str) -> bool {
    octet.len() == 2 && octet.chars().all(|character| character.is_ascii_hexdigit())
}
