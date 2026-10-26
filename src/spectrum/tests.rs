use std::time::Duration;

use super::{decode_energy_calibration, decode_spectrum, EnergyCalibration, Spectrum, SpectrumFormat};
use crate::error::Error;
use crate::protocol::BytesBuffer;

fn header(duration: u32) -> Vec<u8> {
    [duration.to_le_bytes().to_vec(), EnergyCalibration::new(0.0, 1.0, 0.0).to_bytes()].concat()
}

fn group(repeat: u16, vlen: u16) -> Vec<u8> {
    ((repeat << 4) | vlen).to_le_bytes().to_vec()
}

fn decode(bytes: Vec<u8>, format: SpectrumFormat) -> crate::error::Result<Spectrum> {
    decode_spectrum(BytesBuffer::new(bytes), format)
}

#[test]
fn plain_format_reads_u32_counts() {
    let counts: Vec<u8> = [5u32, 10, 0].iter().flat_map(|c| c.to_le_bytes()).collect();
    let spectrum = decode([header(10), counts, vec![1, 2]].concat(), SpectrumFormat::Plain);
    let spectrum = spectrum.unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(spectrum.counts, vec![5, 10, 0]);
    assert_eq!(spectrum.duration, Duration::from_secs(10));
}

#[test]
fn delta_format_decodes_every_vlen_branch() {
    let bytes = [
        header(7),
        group(1, 1),
        vec![10],
        group(1, 2),
        vec![(-3i8) as u8],
        group(1, 3),
        300i16.to_le_bytes().to_vec(),
        group(1, 4),
        vec![0x01, 0x00, 0x00],
        group(1, 5),
        (-8i32).to_le_bytes().to_vec(),
        group(2, 0),
    ]
    .concat();
    let spectrum = decode(bytes, SpectrumFormat::DeltaCompressed);
    assert_eq!(spectrum.map(|s| s.counts).ok(), Some(vec![10, 7, 307, 308, 300, 0, 0]));
}

#[test]
fn delta_format_sign_extends_24_bit_deltas() {
    let bytes = [header(1), group(1, 1), vec![200], group(1, 4), vec![0xFF, 0xFF, 0xFF]].concat();
    let spectrum = decode(bytes, SpectrumFormat::DeltaCompressed);
    assert_eq!(spectrum.map(|s| s.counts).ok(), Some(vec![200, 199]));
}

#[test]
fn delta_format_rejects_negative_counts_bad_vlen_and_overflow() {
    let negative = [header(1), group(1, 2), vec![(-1i8) as u8]].concat();
    let bad_vlen = [header(1), group(1, 9)].concat();
    let overflow = [header(1), group(2, 5), i32::MAX.to_le_bytes().to_vec(), 1i32.to_le_bytes().to_vec()].concat();
    let truncated = [header(1), group(3, 1), vec![1]].concat();
    for bytes in [negative, bad_vlen, overflow] {
        assert!(matches!(decode(bytes, SpectrumFormat::DeltaCompressed), Err(Error::ProtocolMismatch { .. })));
    }
    assert!(matches!(decode(truncated, SpectrumFormat::DeltaCompressed), Err(Error::BufferUnderrun { .. })));
}

#[test]
fn format_versions_and_short_header() {
    assert_eq!(SpectrumFormat::try_from(1).ok(), Some(SpectrumFormat::DeltaCompressed));
    assert!(SpectrumFormat::try_from(2).is_err());
    assert_eq!(SpectrumFormat::Plain.version(), 0);
    assert!(matches!(decode(vec![1, 2, 3], SpectrumFormat::Plain), Err(Error::BufferUnderrun { .. })));
}

#[test]
fn calibration_maps_channels_to_energy() {
    let calibration = EnergyCalibration::from([1.0, 2.0, 3.0]);
    assert_eq!(calibration.channel_energy(0), 1.0);
    assert_eq!(calibration.channel_energy(2), 17.0);
    let decoded = decode_energy_calibration(BytesBuffer::new(calibration.to_bytes()));
    assert_eq!(decoded.ok(), Some(calibration));
}

#[test]
fn spectrum_statistics() {
    let spectrum = Spectrum {
        duration: Duration::from_secs(4),
        calibration: EnergyCalibration::new(0.0, 3.0, 0.0),
        counts: vec![50, 1, 9, 2],
    };
    assert_eq!(spectrum.total_counts(), 62);
    assert_eq!(spectrum.mean_count_rate_cps(), Some(15.5));
    assert_eq!(spectrum.peak_channel(1), Some(2));
    assert_eq!(spectrum.energy_counts().nth(2), Some((6.0, 9)));
    let earlier = Spectrum { counts: vec![10, 5], duration: Duration::from_secs(1), ..spectrum.clone() };
    let diff = spectrum.saturating_sub(&earlier);
    assert_eq!(diff.counts, vec![40, 0, 9, 2]);
    assert_eq!(diff.duration, Duration::from_secs(3));
    assert_eq!(Spectrum::default().mean_count_rate_cps(), None);
}
