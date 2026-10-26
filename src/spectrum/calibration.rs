use crate::error::Result;
use crate::protocol::BytesBuffer;

/// Quadratic channel-to-energy calibration: `E(ch) = a0 + a1·ch + a2·ch²` (keV).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnergyCalibration {
    /// Constant term, keV.
    pub a0: f32,
    /// Linear term, keV per channel.
    pub a1: f32,
    /// Quadratic term, keV per channel².
    pub a2: f32,
}

impl EnergyCalibration {
    /// Creates a calibration from its three coefficients.
    pub const fn new(a0: f32, a1: f32, a2: f32) -> Self {
        Self { a0, a1, a2 }
    }

    /// Coefficients as `[a0, a1, a2]`.
    pub const fn coefficients(self) -> [f32; 3] {
        [self.a0, self.a1, self.a2]
    }

    /// Energy in keV at a (possibly fractional) channel position.
    pub fn energy_at(self, channel: f32) -> f32 {
        self.a0 + self.a1 * channel + self.a2 * channel * channel
    }

    /// Energy in keV of an integer channel.
    pub fn channel_energy(self, channel: usize) -> f32 {
        self.energy_at(channel as f32)
    }

    /// Encodes the coefficients as 12 little-endian bytes (the `EnergyCalib` payload).
    pub fn to_bytes(self) -> Vec<u8> {
        self.coefficients().into_iter().flat_map(f32::to_le_bytes).collect()
    }
}

impl From<[f32; 3]> for EnergyCalibration {
    fn from([a0, a1, a2]: [f32; 3]) -> Self {
        Self::new(a0, a1, a2)
    }
}

/// Reads three little-endian `f32` coefficients from `buffer`.
pub fn take_energy_calibration(buffer: &mut BytesBuffer) -> Result<EnergyCalibration> {
    Ok(EnergyCalibration::new(buffer.take_f32_le()?, buffer.take_f32_le()?, buffer.take_f32_le()?))
}

/// Decodes an `EnergyCalib` virtual-string payload.
pub fn decode_energy_calibration(mut payload: BytesBuffer) -> Result<EnergyCalibration> {
    take_energy_calibration(&mut payload)
}
