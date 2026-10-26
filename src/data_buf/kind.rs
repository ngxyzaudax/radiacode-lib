/// DataBuf record kind, identified by the `(entity, group)` header bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RecordKind {
    /// `(0, 0)`: filtered count and dose rate with error estimates.
    RealTimeData,
    /// `(0, 1)`: unfiltered count and dose rate.
    RawData,
    /// `(0, 2)`: dose-rate database sample.
    DoseRateDb,
    /// `(0, 3)`: accumulated dose, temperature and battery charge.
    RareData,
    /// `(0, 4)`: user data, skipped.
    UserData,
    /// `(0, 5)`: schedule data, skipped.
    ScheduleData,
    /// `(0, 6)`: accelerometer sample.
    AccelData,
    /// `(0, 7)`: device event.
    Event,
    /// `(0, 8)`: raw count rate, skipped.
    RawCountRate,
    /// `(0, 9)`: raw dose rate, skipped.
    RawDoseRate,
    /// `(1, 1)`: 8-byte waveform samples, skipped.
    Waveform8,
    /// `(1, 2)`: 16-byte waveform samples, skipped.
    Waveform16,
    /// `(1, 3)`: 14-byte waveform samples, skipped.
    Waveform14,
    /// Any other `(entity, group)` pair.
    Unknown {
        /// Entity byte.
        entity: u8,
        /// Group byte.
        group: u8,
    },
}

impl RecordKind {
    /// Maps header bytes to a record kind.
    pub fn from_entity_group(entity: u8, group: u8) -> Self {
        match (entity, group) {
            (0, 0) => Self::RealTimeData,
            (0, 1) => Self::RawData,
            (0, 2) => Self::DoseRateDb,
            (0, 3) => Self::RareData,
            (0, 4) => Self::UserData,
            (0, 5) => Self::ScheduleData,
            (0, 6) => Self::AccelData,
            (0, 7) => Self::Event,
            (0, 8) => Self::RawCountRate,
            (0, 9) => Self::RawDoseRate,
            (1, 1) => Self::Waveform8,
            (1, 2) => Self::Waveform16,
            (1, 3) => Self::Waveform14,
            (entity, group) => Self::Unknown { entity, group },
        }
    }

    /// Returns `true` unless the kind is [`RecordKind::Unknown`].
    pub fn is_known(self) -> bool {
        !matches!(self, Self::Unknown { .. })
    }

    /// Preference when several records report rates for the same sequence number.
    pub fn monitor_source_rank(self) -> u8 {
        match self {
            Self::RealTimeData => 3,
            Self::DoseRateDb => 2,
            Self::RawData => 1,
            _ => 0,
        }
    }

    /// Fixed payload length that is skipped for opaque kinds, if any.
    pub(super) fn opaque_payload_len(self) -> Option<usize> {
        match self {
            Self::UserData | Self::ScheduleData => Some(16),
            Self::RawCountRate | Self::RawDoseRate => Some(6),
            _ => None,
        }
    }

    /// Bytes per waveform sample for waveform kinds.
    pub(super) fn waveform_sample_len(self) -> Option<usize> {
        match self {
            Self::Waveform8 => Some(8),
            Self::Waveform16 => Some(16),
            Self::Waveform14 => Some(14),
            _ => None,
        }
    }
}
