const ADVERTISEMENT_PREFIX: &str = "radiacode";
const UNKNOWN_MODEL: &str = "Unknown";

/// Derives the model (`RC-110`) from a serial number (`RC-110-006806`).
///
/// Returns `Unknown` when the serial does not have at least two dash-separated parts.
pub fn model_from_serial(serial: &str) -> String {
    let mut parts = serial.split('-');
    match (parts.next(), parts.next()) {
        (Some(prefix), Some(code)) if !prefix.is_empty() && !code.is_empty() => {
            format!("{prefix}-{code}")
        }
        _ => UNKNOWN_MODEL.into(),
    }
}

/// Extracts the serial from a Bluetooth advertised name such as `RadiaCode#RC-110-006806`.
pub fn serial_from_advertisement(name: &str) -> Option<String> {
    let (prefix, serial) = name.split_once('#')?;
    let serial = serial.trim();
    let prefix_matches = prefix.trim().to_ascii_lowercase().starts_with(ADVERTISEMENT_PREFIX);
    (prefix_matches && !serial.is_empty()).then(|| serial.to_string())
}

/// Extracts the model from a Bluetooth advertised name.
pub fn model_from_advertisement(name: &str) -> Option<String> {
    serial_from_advertisement(name).map(|serial| model_from_serial(&serial))
}

/// Returns `true` when an advertised name looks like a RadiaCode device.
pub fn is_radiacode_name(name: &str) -> bool {
    name.to_ascii_lowercase().contains(ADVERTISEMENT_PREFIX)
}
