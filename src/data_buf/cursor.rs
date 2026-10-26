use super::decode::{DataBufFrame, DecodeWarningKind};
use super::header::Seq;

const MAX_PLAUSIBLE_GAP: u32 = 32;

/// A discontinuity in record sequence numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SeqGap {
    /// Expected sequence number.
    pub expected: u8,
    /// Received sequence number.
    pub got: u8,
    /// Number of records presumed lost (zero for resets).
    pub lost: u32,
    /// `true` when the jump is backwards or implausibly large, e.g. after a device restart.
    pub reset: bool,
}

/// Tracks DataBuf sequence continuity across successive polls.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DataBufCursor {
    last_seq: Option<Seq>,
    total_gaps: u64,
    total_lost: u64,
}

impl DataBufCursor {
    /// Creates a cursor with no history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets all history.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Number of non-reset gaps observed so far.
    pub fn total_gaps(&self) -> u64 {
        self.total_gaps
    }

    /// Number of records presumed lost so far.
    pub fn total_lost(&self) -> u64 {
        self.total_lost
    }

    /// Sequence number of the last header seen.
    pub fn last_seq(&self) -> Option<Seq> {
        self.last_seq
    }

    /// Records the frame and returns gaps within it and since the previous frame.
    pub fn observe_frame(&mut self, frame: &DataBufFrame) -> Vec<SeqGap> {
        let in_frame: Vec<SeqGap> = frame.warnings.iter().filter_map(|w| seq_jump_gap(&w.kind)).collect();
        let cross_poll =
            self.last_seq.zip(frame.first_header_seq).and_then(|(last, first)| cross_poll_gap(&in_frame, last, first));
        let gaps: Vec<SeqGap> = in_frame.into_iter().chain(cross_poll).collect();
        self.last_seq = frame.last_header_seq.or(self.last_seq);
        self.account_losses(&gaps);
        gaps
    }

    fn account_losses(&mut self, gaps: &[SeqGap]) {
        let counted = gaps.iter().filter(|gap| !gap.reset);
        let (count, lost) = counted.fold((0u64, 0u64), |(count, lost), gap| {
            (count.saturating_add(1), lost.saturating_add(u64::from(gap.lost)))
        });
        self.total_gaps = self.total_gaps.saturating_add(count);
        self.total_lost = self.total_lost.saturating_add(lost);
    }
}

fn seq_jump_gap(kind: &DecodeWarningKind) -> Option<SeqGap> {
    match kind {
        DecodeWarningKind::SeqJump { expected, got } => Some(seq_gap(Seq::new(*got), Seq::new(*expected))),
        _ => None,
    }
}

fn seq_gap(got: Seq, expected: Seq) -> SeqGap {
    let lost = got.lost_since(expected).filter(|lost| *lost <= MAX_PLAUSIBLE_GAP);
    SeqGap { expected: expected.raw(), got: got.raw(), lost: lost.unwrap_or(0), reset: lost.is_none() }
}

fn cross_poll_gap(in_frame: &[SeqGap], last: Seq, first: Seq) -> Option<SeqGap> {
    let expected = last.next();
    let candidate = (first != expected).then(|| seq_gap(first, expected))?;
    let reported = in_frame.iter().any(|gap| gap.expected == candidate.expected && gap.got == candidate.got);
    (!reported).then_some(candidate)
}
