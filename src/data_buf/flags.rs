/// Raw device status flags attached to several record kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatusFlags(u16);

impl StatusFlags {
    /// Wraps a raw flag word.
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    /// Returns the raw flag word.
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// Device event identifiers carried by [`RecordKind::Event`](super::RecordKind::Event) records.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EventId {
    PowerOff,
    PowerOn,
    LowBatteryShutdown,
    ChangeDeviceParams,
    DoseReset,
    UserEvent,
    BatteryEmptyAlarm,
    ChargeStart,
    ChargeStop,
    DoseRateAlarm1,
    DoseRateAlarm2,
    DoseRateOffscale,
    DoseAlarm1,
    DoseAlarm2,
    DoseOffscale,
    TemperatureTooLow,
    TemperatureTooHigh,
    TextMessage,
    MemorySnapshot,
    SpectrumReset,
    CountRateAlarm1,
    CountRateAlarm2,
    CountRateOffscale,
    /// Event code not known to this crate.
    Unknown(u8),
}

const KNOWN_EVENTS: [EventId; 23] = [
    EventId::PowerOff,
    EventId::PowerOn,
    EventId::LowBatteryShutdown,
    EventId::ChangeDeviceParams,
    EventId::DoseReset,
    EventId::UserEvent,
    EventId::BatteryEmptyAlarm,
    EventId::ChargeStart,
    EventId::ChargeStop,
    EventId::DoseRateAlarm1,
    EventId::DoseRateAlarm2,
    EventId::DoseRateOffscale,
    EventId::DoseAlarm1,
    EventId::DoseAlarm2,
    EventId::DoseOffscale,
    EventId::TemperatureTooLow,
    EventId::TemperatureTooHigh,
    EventId::TextMessage,
    EventId::MemorySnapshot,
    EventId::SpectrumReset,
    EventId::CountRateAlarm1,
    EventId::CountRateAlarm2,
    EventId::CountRateOffscale,
];

impl EventId {
    /// Decodes a raw event code.
    pub fn from_raw(value: u8) -> Self {
        KNOWN_EVENTS.get(usize::from(value)).copied().unwrap_or(Self::Unknown(value))
    }

    /// Returns the raw event code.
    pub fn raw(self) -> u8 {
        match self {
            Self::Unknown(value) => value,
            known => KNOWN_EVENTS
                .iter()
                .position(|event| *event == known)
                .and_then(|index| u8::try_from(index).ok())
                .unwrap_or(u8::MAX),
        }
    }
}
