use std::time::Duration;

use super::calibration::take_energy_calibration;
use super::Spectrum;
use crate::error::{Error, Result};
use crate::protocol::BytesBuffer;

const RUN_LENGTH_MASK: u16 = 0x0FFF;
const VALUE_LENGTH_MASK: u16 = 0x000F;

/// Channel-count encoding used by the device, from `SpecFormatVersion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpectrumFormat {
    /// Version 0: one little-endian `u32` per channel.
    #[default]
    Plain,
    /// Version 1: run-length groups of variable-width deltas.
    DeltaCompressed,
}

impl SpectrumFormat {
    /// Numeric `SpecFormatVersion` of the format.
    pub fn version(self) -> u32 {
        match self {
            Self::Plain => 0,
            Self::DeltaCompressed => 1,
        }
    }
}

impl TryFrom<u32> for SpectrumFormat {
    type Error = Error;

    fn try_from(version: u32) -> Result<Self> {
        match version {
            0 => Ok(Self::Plain),
            1 => Ok(Self::DeltaCompressed),
            other => Err(Error::mismatch("spectrum format 0 or 1", format!("format {other}"))),
        }
    }
}

/// Decodes a spectrum virtual-string payload (`Spectrum`, `SpecAccum` or `SpecDiff`).
pub fn decode_spectrum(mut payload: BytesBuffer, format: SpectrumFormat) -> Result<Spectrum> {
    let duration = Duration::from_secs(u64::from(payload.take_u32_le()?));
    let calibration = take_energy_calibration(&mut payload)?;
    let counts = match format {
        SpectrumFormat::Plain => decode_plain_counts(&mut payload)?,
        SpectrumFormat::DeltaCompressed => decode_delta_counts(&mut payload)?,
    };
    Ok(Spectrum { duration, calibration, counts })
}

fn decode_plain_counts(payload: &mut BytesBuffer) -> Result<Vec<u32>> {
    (0..payload.size() / 4).map(|_| payload.take_u32_le()).collect()
}

#[derive(Default)]
struct DeltaState {
    last: i32,
    counts: Vec<u32>,
}

fn decode_delta_counts(payload: &mut BytesBuffer) -> Result<Vec<u32>> {
    let mut state = DeltaState::default();
    while payload.size() >= 2 {
        state = decode_run(payload, state)?;
    }
    Ok(state.counts)
}

fn decode_run(payload: &mut BytesBuffer, state: DeltaState) -> Result<DeltaState> {
    let packed = payload.take_u16_le()?;
    let repeat = usize::from((packed >> 4) & RUN_LENGTH_MASK);
    let value_length = packed & VALUE_LENGTH_MASK;
    (0..repeat).try_fold(state, |state, _| decode_next(payload, value_length, state))
}

fn decode_next(payload: &mut BytesBuffer, value_length: u16, mut state: DeltaState) -> Result<DeltaState> {
    let value = read_value(payload, value_length, state.last)?;
    let count =
        u32::try_from(value).map_err(|_| Error::mismatch("non-negative spectrum count", format!("count {value}")))?;
    state.counts.push(count);
    Ok(DeltaState { last: value, counts: state.counts })
}

fn read_value(payload: &mut BytesBuffer, value_length: u16, last: i32) -> Result<i32> {
    match value_length {
        0 => Ok(0),
        1 => payload.take_u8().map(i32::from),
        2 => payload.take_i8().and_then(|delta| apply_delta(last, i32::from(delta))),
        3 => payload.take_i16_le().and_then(|delta| apply_delta(last, i32::from(delta))),
        4 => take_i24(payload).and_then(|delta| apply_delta(last, delta)),
        5 => payload.take_i32_le().and_then(|delta| apply_delta(last, delta)),
        other => Err(Error::mismatch("vlen 0..=5", format!("vlen {other}"))),
    }
}

fn take_i24(payload: &mut BytesBuffer) -> Result<i32> {
    let [low, mid, high] = payload.take_array::<3>()?;
    Ok(i32::from_le_bytes([low, mid, high, sign_extension(high)]))
}

fn sign_extension(high: u8) -> u8 {
    match high & 0x80 {
        0 => 0x00,
        _ => 0xFF,
    }
}

fn apply_delta(last: i32, delta: i32) -> Result<i32> {
    last.checked_add(delta).ok_or_else(|| Error::mismatch("spectrum count within i32", "delta overflow"))
}
