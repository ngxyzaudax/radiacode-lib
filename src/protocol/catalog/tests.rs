use super::{
    decode_cp1251, parse_configuration_ini, parse_sfr_file, sfr_entries_support_leds_on, sfr_supports_leds_on,
    validate_catalog, SfrEntryKind,
};
use crate::protocol::registers::VirtSfr;

const CONFIGURATION: &str = "SpecFormatVersion=1\n[[GRP_RealTimeData]]\nId=0\nGId=0\n\
[CHN_CountRate]\nId=1\nUnit=cps\nP1=1\nP2=0\n[CHN_DoseRate]\nId=2\nUnit=uR/h\nP1=1e6\n\
\n[[GRP_RareData]]\nId=0\nGId=3\n[CHN_Temp]\nId=5\nUnit=C\nP1=0.01\nP2=-20\n";

#[test]
fn decodes_cyrillic_unit_label() {
    assert_eq!(decode_cp1251(&[0xEC, 0xEA, 0xC2, 0x2F, 0xF7]), "мкВ/ч");
}

#[test]
fn parses_groups_channels_and_spec_format() {
    let catalog = parse_configuration_ini(CONFIGURATION);
    assert_eq!(catalog.spec_format_version, 1);
    assert_eq!(catalog.groups.len(), 2);
    let real_time = &catalog.groups[0];
    assert_eq!(real_time.name, "GRP_RealTimeData");
    assert_eq!((real_time.entity, real_time.group), (0, 0));
    assert_eq!(real_time.channels.len(), 2);
    assert_eq!(real_time.channels[1].unit, "uR/h");
    assert_eq!(real_time.channels[1].scale, 1e6);
    let rare = &catalog.groups[1];
    assert_eq!(rare.group, 3);
    assert_eq!(rare.channels[0].id, 5);
    assert_eq!(rare.channels[0].offset, -20.0);
}

#[test]
fn configuration_defaults_apply_to_bad_values() {
    let catalog = parse_configuration_ini("SpecFormatVersion=x\n[[GRP_A]]\n[CHN_B]\nP1=bad\n");
    assert_eq!(catalog.spec_format_version, 0);
    assert_eq!(catalog.groups[0].channels[0].scale, 1.0);
    assert!(parse_configuration_ini("").groups.is_empty());
}

#[test]
fn parses_sfr_entry() {
    let entries = parse_sfr_file("[VSFR_DISP_BRT]\nAddr=0x00000511\nSize=1\nType=1\nSigned=0\n");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "VSFR_DISP_BRT");
    assert_eq!(entries[0].address, 0x511);
    assert_eq!(entries[0].size, 1);
    assert_eq!(entries[0].value_kind, SfrEntryKind::U8);
    assert!(!entries[0].signed);
}

#[test]
fn sfr_file_parser_handles_all_fields_and_bad_values() {
    let entries = parse_sfr_file(
        "noise\n[VSFR_A]\nAddr=0X00000511\nSize=x\nType=2\nSigned=1\n[OTHER]\nAddr=0x1\n[VSFR_B]\nAddr=zz\n",
    );
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].address, 0x511);
    assert_eq!(entries[0].size, 0);
    assert_eq!(entries[0].value_kind, SfrEntryKind::U16);
    assert!(entries[0].signed);
    assert_eq!(entries[1].address, 0);
    assert_eq!(entries[1].value_kind, SfrEntryKind::Unknown);
}

#[test]
fn validate_catalog_reports_missing_and_mismatched_registers() {
    assert!(validate_catalog("no entries here").is_none());
    let sfr = "[VSFR_DEVICE_CTRL]\nAddr=0x00000500\nSize=4\nType=4\n\
               [VSFR_DISP_BRT]\nAddr=0x00000511\nSize=4\nType=5\n";
    let drifts = validate_catalog(sfr).unwrap_or_default();
    assert!(drifts.iter().any(|d| d.register == VirtSfr::DispBrt && d.message.contains("device type")));
    assert!(drifts.iter().any(|d| d.register == VirtSfr::DeviceTime && d.message.contains("missing")));
    assert!(!drifts.iter().any(|d| d.register == VirtSfr::DeviceCtrl));
}

#[test]
fn validate_catalog_accepts_unknown_device_types() {
    let drifts = validate_catalog("[VSFR_DEVICE_CTRL]\nAddr=0x00000500\nSize=4\nType=7\n");
    assert!(!drifts.unwrap_or_default().iter().any(|d| d.register == VirtSfr::DeviceCtrl));
}

#[test]
fn detects_leds_on_entry() {
    let sample = "[VSFR_LEDS_ON]\nAddr=0x00000545\nVirtual=1\nSize=1\n";
    assert!(sfr_supports_leds_on(sample));
    assert!(sfr_entries_support_leds_on(&parse_sfr_file(sample)));
    let absent = "[VSFR_VIBRO_ON]\nAddr=0x00000531\n";
    assert!(!sfr_supports_leds_on(absent));
    assert!(!sfr_entries_support_leds_on(&parse_sfr_file(absent)));
    let unnamed = "[VSFR_UNNAMED]\nAddr=0x00000545\n";
    assert!(sfr_entries_support_leds_on(&parse_sfr_file(unnamed)));
}
