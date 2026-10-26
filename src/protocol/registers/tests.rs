use super::{
    catalog_registers, extract_vsfr_f32, extract_vsfr_u32, resolve_value_kind, VirtSfr, VsfrValueKind, ALL_REGISTERS,
};
use crate::protocol::catalog::parse_sfr_file;

#[test]
fn extracts_u8_from_either_byte() {
    assert_eq!(extract_vsfr_u32(0x0500_0000, VsfrValueKind::U8), 5);
    assert_eq!(extract_vsfr_u32(1, VsfrValueKind::U8), 1);
}

#[test]
fn extracts_u16_from_either_halfword() {
    assert_eq!(extract_vsfr_u32(0x0001_0000, VsfrValueKind::U16), 1);
    assert_eq!(extract_vsfr_u32(1, VsfrValueKind::U16), 1);
}

#[test]
fn sign_extends_i16() {
    let raw = u32::from(0xFFFEu16);
    assert_eq!(extract_vsfr_u32(raw, VsfrValueKind::I16) as i32, -2);
}

#[test]
fn passes_through_full_words_and_floats() {
    assert_eq!(extract_vsfr_u32(0x1234_5678, VsfrValueKind::U32), 0x1234_5678);
    let bits = 21.5f32.to_bits();
    assert_eq!(extract_vsfr_f32(bits, VsfrValueKind::F32), 21.5);
}

#[test]
fn every_register_round_trips_through_its_address() {
    assert!(ALL_REGISTERS.iter().all(|register| VirtSfr::from_address(register.address()) == Some(*register)));
    assert_eq!(VirtSfr::try_from(0x1234u32), Err(0x1234));
}

#[test]
fn addresses_are_unique() {
    let mut addresses: Vec<u32> = ALL_REGISTERS.iter().map(|r| r.address()).collect();
    addresses.sort_unstable();
    addresses.dedup();
    assert_eq!(addresses.len(), ALL_REGISTERS.len());
}

#[test]
fn static_kinds_match_known_layouts() {
    assert_eq!(VirtSfr::DispBrt.static_value_kind(), VsfrValueKind::U8);
    assert_eq!(VirtSfr::DeviceOn.static_value_kind(), VsfrValueKind::Bool);
    assert_eq!(VirtSfr::AccX.static_value_kind(), VsfrValueKind::I16);
    assert_eq!(VirtSfr::TempDegC.static_value_kind(), VsfrValueKind::F32);
    assert_eq!(VirtSfr::SysMcuVref.static_value_kind(), VsfrValueKind::I32);
    assert_eq!(VirtSfr::DrLev1UrH.static_value_kind(), VsfrValueKind::U32);
}

#[test]
fn device_description_overrides_static_kind() {
    let entries = parse_sfr_file("[VSFR_DISP_BRT]\nAddr=0x511\nSize=2\nType=2\nSigned=1\n");
    assert_eq!(resolve_value_kind(VirtSfr::DispBrt, &entries), VsfrValueKind::I16);
    assert_eq!(resolve_value_kind(VirtSfr::TempDegC, &entries), VsfrValueKind::F32);
}

#[test]
fn catalog_contains_validated_registers() {
    assert_eq!(catalog_registers().len(), 29);
    assert!(catalog_registers().contains(&VirtSfr::SysMcuTemp));
}
