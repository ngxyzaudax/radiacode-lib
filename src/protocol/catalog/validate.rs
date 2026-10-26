use super::sfr_file::{parse_sfr_file, SfrCatalogEntry, SfrEntryKind};
use crate::protocol::registers::{catalog_registers, VirtSfr, VsfrValueKind};

/// A difference between the built-in register table and the device `SFR_FILE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogDrift {
    /// Register whose description differs.
    pub register: VirtSfr,
    /// Human-readable description of the difference.
    pub message: String,
}

/// Compares the built-in register catalog with the device `SFR_FILE` text.
///
/// Returns `None` when the text contains no register entries at all.
pub fn validate_catalog(sfr_text: &str) -> Option<Vec<CatalogDrift>> {
    let entries = parse_sfr_file(sfr_text);
    (!entries.is_empty()).then(|| validate_entries(&entries))
}

/// Compares the built-in register catalog with already parsed `SFR_FILE` entries.
pub fn validate_entries(entries: &[SfrCatalogEntry]) -> Vec<CatalogDrift> {
    catalog_registers().iter().filter_map(|register| register_drift(*register, entries)).collect()
}

fn register_drift(register: VirtSfr, entries: &[SfrCatalogEntry]) -> Option<CatalogDrift> {
    match entries.iter().find(|entry| entry.address == register.address()) {
        None => Some(CatalogDrift { register, message: "missing from device SFR_FILE".into() }),
        Some(entry) => kind_drift(register, entry.value_kind),
    }
}

fn kind_drift(register: VirtSfr, device_kind: SfrEntryKind) -> Option<CatalogDrift> {
    let expected = register.static_value_kind();
    (!kinds_compatible(expected, device_kind))
        .then(|| CatalogDrift { register, message: format!("device type {device_kind:?} != static {expected:?}") })
}

fn kinds_compatible(static_kind: VsfrValueKind, device_kind: SfrEntryKind) -> bool {
    matches!(
        (static_kind, device_kind),
        (VsfrValueKind::U32, SfrEntryKind::U32)
            | (VsfrValueKind::I32, SfrEntryKind::U32)
            | (VsfrValueKind::F32, SfrEntryKind::F32)
            | (VsfrValueKind::U8, SfrEntryKind::U8)
            | (VsfrValueKind::U16, SfrEntryKind::U16)
            | (VsfrValueKind::I16, SfrEntryKind::U16)
            | (VsfrValueKind::Bool, SfrEntryKind::U8)
            | (_, SfrEntryKind::Unknown)
    )
}
