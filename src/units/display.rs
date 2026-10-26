/// Dose unit family selected on the device (`DS_UNITS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DoseDisplayUnit {
    /// Sievert based: rates in µSv/h, accumulated dose in µSv.
    MicroSievertPerHour,
    /// Roentgen based: rates in µR/h, accumulated dose in µR.
    MicroRoentgenPerHour,
}

impl DoseDisplayUnit {
    /// Decodes the `DS_UNITS` register value; any non-zero value selects Sievert.
    pub fn from_device_flag(flag: u32) -> Self {
        match flag {
            0 => Self::MicroRoentgenPerHour,
            _ => Self::MicroSievertPerHour,
        }
    }

    /// Encodes the unit as a `DS_UNITS` register value.
    pub fn to_device_flag(self) -> u32 {
        match self {
            Self::MicroSievertPerHour => 1,
            Self::MicroRoentgenPerHour => 0,
        }
    }

    /// Returns `true` for the Sievert family.
    pub fn is_sv(self) -> bool {
        matches!(self, Self::MicroSievertPerHour)
    }

    /// Rate label, `µSv/h` or `µR/h`.
    pub fn rate_label(self) -> &'static str {
        match self {
            Self::MicroSievertPerHour => "µSv/h",
            Self::MicroRoentgenPerHour => "µR/h",
        }
    }

    /// Accumulated dose label, `µSv` or `µR`.
    pub fn dose_label(self) -> &'static str {
        match self {
            Self::MicroSievertPerHour => "µSv",
            Self::MicroRoentgenPerHour => "µR",
        }
    }
}

/// Dose and count-rate units used to present readings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DisplayUnits {
    /// Dose unit family.
    pub dose: DoseDisplayUnit,
    /// Count-rate unit.
    pub count: CountDisplayUnit,
}

impl Default for DisplayUnits {
    fn default() -> Self {
        Self { dose: DoseDisplayUnit::MicroSievertPerHour, count: CountDisplayUnit::Cps }
    }
}

/// Count-rate unit selected on the device (`CR_UNITS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CountDisplayUnit {
    /// Counts per second.
    Cps,
    /// Counts per minute.
    Cpm,
}

impl CountDisplayUnit {
    /// Decodes the `CR_UNITS` register value; any non-zero value selects cpm.
    pub fn from_device_flag(flag: u32) -> Self {
        match flag {
            0 => Self::Cps,
            _ => Self::Cpm,
        }
    }

    /// Encodes the unit as a `CR_UNITS` register value.
    pub fn to_device_flag(self) -> u32 {
        match self {
            Self::Cpm => 1,
            Self::Cps => 0,
        }
    }

    /// Unit label, `cps` or `cpm`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Cpm => "cpm",
            Self::Cps => "cps",
        }
    }
}
