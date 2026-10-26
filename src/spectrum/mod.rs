//! Gamma spectrum type, energy calibration and the device spectrum encodings.

mod calibration;
mod decode;

#[cfg(test)]
mod tests;

use std::time::Duration;

pub use calibration::{decode_energy_calibration, take_energy_calibration, EnergyCalibration};
pub use decode::{decode_spectrum, SpectrumFormat};

/// A pulse-height spectrum read from the device.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spectrum {
    /// Acquisition time.
    pub duration: Duration,
    /// Channel-to-energy calibration reported with the spectrum.
    pub calibration: EnergyCalibration,
    /// Counts per channel.
    pub counts: Vec<u32>,
}

impl Spectrum {
    /// Number of channels.
    pub fn channel_count(&self) -> usize {
        self.counts.len()
    }

    /// Sum of all channel counts.
    pub fn total_counts(&self) -> u64 {
        self.counts.iter().map(|count| u64::from(*count)).sum()
    }

    /// Average count rate over the acquisition time, or `None` for a zero duration.
    pub fn mean_count_rate_cps(&self) -> Option<f64> {
        let seconds = self.duration.as_secs_f64();
        (seconds > 0.0).then(|| self.total_counts() as f64 / seconds)
    }

    /// Energy in keV of `channel` under the spectrum calibration.
    pub fn channel_energy(&self, channel: usize) -> f32 {
        self.calibration.channel_energy(channel)
    }

    /// Iterates `(energy_kev, count)` pairs for every channel.
    pub fn energy_counts(&self) -> impl Iterator<Item = (f32, u32)> + '_ {
        self.counts.iter().enumerate().map(|(channel, count)| (self.channel_energy(channel), *count))
    }

    /// Channel with the highest count, ignoring the first `skip_channels` channels.
    pub fn peak_channel(&self, skip_channels: usize) -> Option<usize> {
        self.counts.iter().enumerate().skip(skip_channels).max_by_key(|(_, count)| **count).map(|(channel, _)| channel)
    }

    /// Channel-wise saturating difference `self - earlier`, e.g. for interval spectra.
    pub fn saturating_sub(&self, earlier: &Spectrum) -> Spectrum {
        Spectrum {
            duration: self.duration.saturating_sub(earlier.duration),
            calibration: self.calibration,
            counts: self
                .counts
                .iter()
                .enumerate()
                .map(|(i, count)| count.saturating_sub(earlier.counts.get(i).copied().unwrap_or(0)))
                .collect(),
        }
    }
}
