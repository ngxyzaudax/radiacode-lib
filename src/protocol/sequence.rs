use std::time::{SystemTime, UNIX_EPOCH};

const SEQUENCE_BASE: u8 = 0x80;
const SEQUENCE_WINDOW: u8 = 32;

/// Request sequence counter.
///
/// The device expects sequence bytes in `0x80..=0x9F`, wrapping modulo 32.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Sequence {
    value: u8,
}

impl Sequence {
    /// Creates a counter starting at `offset % 32`.
    pub fn starting_at(offset: u8) -> Self {
        Self { value: offset % SEQUENCE_WINDOW }
    }

    /// Creates a counter with a time-derived start, so a new session does not
    /// reuse the sequence numbers of a previous one.
    pub fn session_start() -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.subsec_nanos()).unwrap_or(0);
        Self::starting_at(nanos.to_le_bytes()[0])
    }

    /// Current window offset (`0..32`).
    pub fn start_value(&self) -> u8 {
        self.value
    }

    /// Returns the next sequence byte and advances the counter.
    pub fn advance(&mut self) -> u8 {
        let seq = SEQUENCE_BASE + self.value;
        self.value = (self.value + 1) % SEQUENCE_WINDOW;
        seq
    }
}

#[cfg(test)]
mod tests {
    use super::Sequence;

    #[test]
    fn sequence_wraps_within_32() {
        let mut sequence = Sequence::default();
        assert_eq!(sequence.advance(), 0x80);
        (0..31).for_each(|_| {
            sequence.advance();
        });
        assert_eq!(sequence.advance(), 0x80);
    }

    #[test]
    fn starting_offset_is_reduced_modulo_window() {
        let mut sequence = Sequence::starting_at(33);
        assert_eq!(sequence.start_value(), 1);
        assert_eq!(sequence.advance(), 0x81);
    }

    #[test]
    fn session_start_is_within_window() {
        assert!(Sequence::session_start().start_value() < 32);
    }
}
