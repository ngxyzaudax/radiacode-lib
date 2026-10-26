use crate::protocol::registers::VirtSfr;

use super::sfr_file::SfrCatalogEntry;

const LEDS_ON_ENTRY_NAME: &str = "VSFR_LEDS_ON";
const LEDS_ON_ADDRESS_LINE: &str = "Addr=0x00000545";

/// Decodes Windows-1251 text, as used by the device configuration strings.
pub fn decode_cp1251(bytes: &[u8]) -> String {
    encoding_rs::WINDOWS_1251.decode(bytes).0.into_owned()
}

/// Returns `true` when the raw `SFR_FILE` text lists the `LEDS_ON` register.
pub fn sfr_supports_leds_on(text: &str) -> bool {
    text.lines().any(|line| line.contains(LEDS_ON_ENTRY_NAME) || line.contains(LEDS_ON_ADDRESS_LINE))
}

/// Returns `true` when parsed `SFR_FILE` entries contain the `LEDS_ON` register.
pub fn sfr_entries_support_leds_on(entries: &[SfrCatalogEntry]) -> bool {
    entries.iter().any(|entry| entry.address == VirtSfr::LedsOn.address() || entry.name == LEDS_ON_ENTRY_NAME)
}
