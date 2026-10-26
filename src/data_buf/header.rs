use std::time::{Duration, SystemTime};

use super::kind::RecordKind;

const MILLIS_PER_TICK: i64 = 10;
const SEQ_HALF_RANGE: u8 = 128;

/// Device timestamp in 10 ms ticks relative to the session base time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceTicks(i32);

impl DeviceTicks {
    /// Wraps a raw tick count.
    pub const fn new(ticks: i32) -> Self {
        Self(ticks)
    }

    /// Returns the raw tick count.
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Offset from the base time; negative tick counts clamp to zero.
    pub fn as_duration(self) -> Duration {
        ticks_to_duration(i64::from(self.0))
    }

    /// Elapsed time since `origin`; clamps to zero when `origin` is later.
    pub fn duration_since(self, origin: Self) -> Duration {
        ticks_to_duration(i64::from(self.0) - i64::from(origin.0))
    }

    /// Converts to wall-clock time given the session base time.
    pub fn wall_clock_time(self, base: SystemTime) -> SystemTime {
        base.checked_add(self.as_duration()).unwrap_or(base)
    }
}

fn ticks_to_duration(ticks: i64) -> Duration {
    let millis = ticks.saturating_mul(MILLIS_PER_TICK).max(0);
    Duration::from_millis(u64::try_from(millis).unwrap_or(0))
}

/// Wrapping 8-bit record sequence number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Seq(u8);

impl Seq {
    /// Wraps a raw sequence byte.
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// Returns the raw sequence byte.
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// The following sequence number, wrapping at 255.
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Returns `true` when `self` is ahead of `other` within half the sequence space.
    pub fn is_newer_than(self, other: Seq) -> bool {
        self != other && self.0.wrapping_sub(other.0) < SEQ_HALF_RANGE
    }

    /// Number of records skipped between `expected` and `self`, or `None` when `self` is older.
    pub fn lost_since(self, expected: Seq) -> Option<u32> {
        match self == expected {
            true => Some(0),
            false => self.is_newer_than(expected).then(|| u32::from(self.0.wrapping_sub(expected.0))),
        }
    }
}

/// Seven-byte header in front of every DataBuf record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecordHeader {
    /// Record sequence number.
    pub seq: Seq,
    /// Record kind from the entity and group bytes.
    pub kind: RecordKind,
    /// Record timestamp.
    pub ts: DeviceTicks,
}
