use super::{VirtSfr, VsfrValueKind};
use crate::protocol::catalog::{SfrCatalogEntry, SfrEntryKind};

/// Extracts the meaningful integer from a raw register word.
///
/// Sub-word values may be stored in either the low or the high bits of the
/// word depending on firmware revision; the populated half is selected.
pub fn extract_vsfr_u32(raw: u32, kind: VsfrValueKind) -> u32 {
    match kind {
        VsfrValueKind::U8 | VsfrValueKind::Bool => pick_subword(raw, 8),
        VsfrValueKind::U16 => pick_subword(raw, 16),
        VsfrValueKind::I16 => sign_extend_halfword(pick_subword(raw, 16)),
        VsfrValueKind::U32 | VsfrValueKind::I32 | VsfrValueKind::F32 => raw,
    }
}

/// Interprets a raw register word as a float, extracting sub-words first.
pub fn extract_vsfr_f32(raw: u32, kind: VsfrValueKind) -> f32 {
    match kind {
        VsfrValueKind::F32 => f32::from_bits(raw),
        other => f32::from_bits(extract_vsfr_u32(raw, other)),
    }
}

/// Resolves the value kind of `register`, preferring the device `SFR_FILE` description.
pub fn resolve_value_kind(register: VirtSfr, entries: &[SfrCatalogEntry]) -> VsfrValueKind {
    entries
        .iter()
        .find(|entry| entry.address == register.address())
        .map(device_entry_kind)
        .unwrap_or_else(|| register.static_value_kind())
}

fn pick_subword(raw: u32, width: u32) -> u32 {
    let mask = (1u32 << width) - 1;
    let low = raw & mask;
    let high = raw >> (32 - width);
    if high != 0 && low == 0 {
        high
    } else {
        low
    }
}

fn sign_extend_halfword(value: u32) -> u32 {
    i32::from(value as u16 as i16) as u32
}

fn device_entry_kind(entry: &SfrCatalogEntry) -> VsfrValueKind {
    match (entry.size, entry.value_kind) {
        (1, SfrEntryKind::U8) => VsfrValueKind::U8,
        (2, SfrEntryKind::U16) if entry.signed => VsfrValueKind::I16,
        (2, SfrEntryKind::U16) => VsfrValueKind::U16,
        (4, SfrEntryKind::F32) => VsfrValueKind::F32,
        (4, SfrEntryKind::U32) if entry.signed => VsfrValueKind::I32,
        (1, _) => VsfrValueKind::Bool,
        _ => VsfrValueKind::U32,
    }
}
