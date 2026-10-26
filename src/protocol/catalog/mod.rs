//! Parsers for the device self-description strings: `Configuration` and `SFR_FILE`.

mod charset;
mod config_ini;
mod sfr_file;
mod validate;

#[cfg(test)]
mod tests;

pub use charset::{decode_cp1251, sfr_entries_support_leds_on, sfr_supports_leds_on};
pub use config_ini::{parse_configuration_ini, ChannelDef, ConfigurationCatalog, MessageGroup};
pub use sfr_file::{parse_sfr_file, SfrCatalogEntry, SfrEntryKind};
pub use validate::{validate_catalog, validate_entries, CatalogDrift};
