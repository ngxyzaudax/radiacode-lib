//! Wire-level fixtures from `tests/fixtures`, reassembled from transport-sized chunks and decoded.

mod support;

use radiacode_lib::protocol::catalog::{decode_cp1251, parse_configuration_ini, parse_sfr_file, SfrEntryKind};
use radiacode_lib::protocol::{
    decode_batch_read, decode_fw_version, decode_hw_serial, decode_text, extract_vsfr_f32, extract_vsfr_u32,
    spectrum_format_version, BatchReadResponse, Command, VirtSfr, VsfrValueKind,
};
use radiacode_lib::spectrum::{decode_energy_calibration, decode_spectrum};
use radiacode_lib::{Spectrum, SpectrumFormat};
use support::wire::{response_body, virt_string, CHUNK_SIZES};

const GET_VERSION: &str = include_str!("fixtures/get_version.hex");
const GET_SERIAL: &str = include_str!("fixtures/get_serial.hex");
const CONFIGURATION: &str = include_str!("fixtures/configuration.hex");
const SFR_FILE: &str = include_str!("fixtures/sfr_file.hex");
const VSFR_BATCH: &str = include_str!("fixtures/vsfr_batch.hex");
const ENERGY_CALIBRATION: &str = include_str!("fixtures/energy_calibration.hex");
const SPECTRUM_V0: &str = include_str!("fixtures/spectrum_v0.hex");
const SPECTRUM_V1: &str = include_str!("fixtures/spectrum_v1.hex");

#[test]
fn firmware_versions_decode() {
    for chunk_size in CHUNK_SIZES {
        let versions = decode_fw_version(response_body(GET_VERSION, Command::GetVersion, 0x81, chunk_size)).unwrap();
        assert_eq!((versions.boot.major, versions.boot.minor, versions.boot.date.as_str()), (4, 1, "Jul 12 2022"));
        assert_eq!((versions.target.major, versions.target.minor), (4, 14));
        assert_eq!(versions.target.date, "Mar 18 2025");
        assert!(versions.is_supported());
    }
}

#[test]
fn hardware_serial_decodes() {
    let body = response_body(GET_SERIAL, Command::GetSerial, 0x82, 20);
    assert_eq!(decode_hw_serial(body).unwrap(), "31415926-53589793-23846264");
}

#[test]
fn cp1251_configuration_parses() {
    let text = decode_cp1251(virt_string(CONFIGURATION, 0x83, 20).data());
    assert_eq!(spectrum_format_version(&text), 1);
    let catalog = parse_configuration_ini(&text);
    assert_eq!(catalog.spec_format_version, 1);
    let names: Vec<&str> = catalog.groups.iter().map(|group| group.name.as_str()).collect();
    assert_eq!(names, ["GRP_RealTimeData", "GRP_RareData"]);
    let real_time = &catalog.groups[0];
    assert_eq!((real_time.entity, real_time.group, real_time.channels.len()), (0, 0, 2));
    assert_eq!(real_time.channels[0].unit, "имп/с");
    assert_eq!(real_time.channels[1].unit, "мкЗв/ч");
    assert_eq!(real_time.channels[1].scale, 10000.0);
    let temperature = &catalog.groups[1].channels[0];
    assert_eq!((temperature.unit.as_str(), temperature.scale, temperature.offset), ("°C", 0.01, -20.0));
}

#[test]
fn sfr_file_directory_parses() {
    let entries = parse_sfr_file(&decode_text(&virt_string(SFR_FILE, 0x84, 256)));
    assert_eq!(entries.len(), 12);
    let temperature = entries.iter().find(|entry| entry.name == "VSFR_TEMP_degC").unwrap();
    assert_eq!((temperature.address, temperature.size), (VirtSfr::TempDegC.address(), 4));
    assert_eq!(temperature.value_kind, SfrEntryKind::F32);
    assert!(entries.iter().any(|entry| entry.address == VirtSfr::LedsOn.address()));
}

#[test]
fn vsfr_batch_decodes_and_extracts() {
    let body = response_body(VSFR_BATCH, Command::RdVirtSfrBatch, 0x85, 20);
    let BatchReadResponse::Complete(raw) = decode_batch_read(body, 4).unwrap() else {
        panic!("expected a complete batch");
    };
    assert_eq!(extract_vsfr_u32(raw[0], VirtSfr::DispBrt.static_value_kind()), 7);
    assert_eq!(raw[1], 1);
    assert_eq!(extract_vsfr_f32(raw[2], VsfrValueKind::F32), 24.5);
    assert_eq!(raw[3], 42);
}

#[test]
fn energy_calibration_decodes() {
    let calibration = decode_energy_calibration(virt_string(ENERGY_CALIBRATION, 0x86, 20)).unwrap();
    assert_eq!(calibration.coefficients(), [-6.25, 2.4512, 0.00038]);
}

#[test]
fn spectrum_formats_decode_to_identical_counts() {
    for chunk_size in CHUNK_SIZES {
        let plain = decode_spectrum(virt_string(SPECTRUM_V0, 0x87, chunk_size), SpectrumFormat::Plain).unwrap();
        let delta =
            decode_spectrum(virt_string(SPECTRUM_V1, 0x88, chunk_size), SpectrumFormat::DeltaCompressed).unwrap();
        assert_eq!(plain, delta);
        assert_eq!(plain.channel_count(), 1024);
        assert_eq!(plain.duration.as_secs(), 1800);
    }
}

#[test]
fn cs137_spectrum_has_photopeak_at_662_kev() {
    let spectrum = decode_spectrum(virt_string(SPECTRUM_V1, 0x88, 256), SpectrumFormat::DeltaCompressed).unwrap();
    assert_eq!(&spectrum.counts[..3], &[0, 0, 0]);
    assert_eq!(spectrum.counts[1023], 3_250_000);
    let below_overflow = Spectrum { counts: spectrum.counts[..1000].to_vec(), ..spectrum.clone() };
    let peak = below_overflow.peak_channel(40).unwrap();
    assert!((spectrum.channel_energy(peak) - 661.7).abs() < 15.0, "peak at {} keV", spectrum.channel_energy(peak));
    assert!(spectrum.total_counts() > 3_250_000);
    assert!(spectrum.mean_count_rate_cps().is_some_and(|rate| rate > 1000.0));
}
