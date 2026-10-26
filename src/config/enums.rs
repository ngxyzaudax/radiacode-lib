/// Whether alarms signal once or until acknowledged (`ALARM_MODE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AlarmSignalMode {
    /// Signal once per alarm.
    Once,
    /// Signal continuously while the alarm condition holds.
    Continuous,
}

impl AlarmSignalMode {
    /// Decodes the register value; `0` means continuous.
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            0 => Self::Continuous,
            _ => Self::Once,
        }
    }

    /// Encodes the register value.
    pub fn as_raw(self) -> u32 {
        match self {
            Self::Continuous => 0,
            Self::Once => 1,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Once => "Once",
            Self::Continuous => "Continuous",
        }
    }
}

/// Screen orientation (`DISP_DIR`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DisplayDirection {
    /// Follow the accelerometer.
    Auto,
    /// Fixed right-handed orientation.
    Right,
    /// Fixed left-handed orientation.
    Left,
}

impl DisplayDirection {
    /// Decodes the register value; unknown values map to [`DisplayDirection::Auto`].
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            1 => Self::Right,
            2 => Self::Left,
            _ => Self::Auto,
        }
    }

    /// Encodes the register value.
    pub fn as_raw(self) -> u32 {
        match self {
            Self::Auto => 0,
            Self::Right => 1,
            Self::Left => 2,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Right => "Right",
            Self::Left => "Left",
        }
    }
}

/// Delay before the display backlight switches off (`DISP_OFF_TIME`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BacklightOffTime {
    /// 5 seconds.
    Sec5,
    /// 10 seconds.
    Sec10,
    /// 15 seconds.
    Sec15,
    /// 30 seconds.
    Sec30,
    /// 2 minutes.
    Min2,
    /// 5 minutes.
    Min5,
}

const BACKLIGHT_OFF_TIMES: [(BacklightOffTime, &str); 6] = [
    (BacklightOffTime::Sec5, "5 s"),
    (BacklightOffTime::Sec10, "10 s"),
    (BacklightOffTime::Sec15, "15 s"),
    (BacklightOffTime::Sec30, "30 s"),
    (BacklightOffTime::Min2, "2 min"),
    (BacklightOffTime::Min5, "5 min"),
];

impl BacklightOffTime {
    /// Decodes the register value; unknown values map to [`BacklightOffTime::Sec5`].
    pub fn from_raw(raw: u32) -> Self {
        Self::all().into_iter().find(|value| value.as_raw() == raw).unwrap_or(Self::Sec5)
    }

    /// Encodes the register value (`0..=5`).
    pub fn as_raw(self) -> u32 {
        BACKLIGHT_OFF_TIMES
            .iter()
            .position(|(value, _)| *value == self)
            .and_then(|index| u32::try_from(index).ok())
            .unwrap_or(0)
    }

    /// Human-readable label such as `30 s`.
    pub fn label(self) -> &'static str {
        BACKLIGHT_OFF_TIMES.iter().find_map(|(value, label)| (*value == self).then_some(*label)).unwrap_or("5 s")
    }

    /// Every option in register order.
    pub fn all() -> [Self; 6] {
        BACKLIGHT_OFF_TIMES.map(|(value, _)| value)
    }
}
