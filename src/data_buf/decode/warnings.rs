use crate::data_buf::header::Seq;
use crate::data_buf::records::DataBufRecord;
use crate::data_buf::sanity::SanityFailure;

/// Kind of non-fatal problem found while decoding a DataBuf payload.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DecodeWarningKind {
    /// A record payload ended early; decoding stopped.
    PayloadUnderrun,
    /// A header carried an unknown `(entity, group)` pair.
    UnknownRecord {
        /// Entity byte.
        entity: u8,
        /// Group byte.
        group: u8,
        /// Up to 32 bytes following the unknown header, for diagnostics.
        tail: Vec<u8>,
    },
    /// A record sequence number did not follow its predecessor.
    SeqJump {
        /// Expected sequence number.
        expected: u8,
        /// Received sequence number.
        got: u8,
    },
    /// A record was decoded but rejected as implausible.
    SanityRejected(SanityFailure),
    /// Bytes remained that were too short to hold a record header.
    TruncatedTail,
}

/// A non-fatal decode problem and the byte offset of the record it concerns.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecodeWarning {
    /// Offset of the affected record header (or of the trailing bytes) in the payload.
    pub offset: usize,
    /// What went wrong.
    pub kind: DecodeWarningKind,
}

/// Result of decoding one DataBuf payload.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataBufFrame {
    /// Records that decoded and passed sanity checks, in payload order.
    pub records: Vec<DataBufRecord>,
    /// Problems encountered while decoding.
    pub warnings: Vec<DecodeWarning>,
    /// Sequence number of the first header seen, including rejected records.
    pub first_header_seq: Option<Seq>,
    /// Sequence number of the last header seen, including rejected records.
    pub last_header_seq: Option<Seq>,
    /// Number of times the decoder skipped ahead to find the next header.
    pub resync_count: u32,
}

impl DataBufFrame {
    pub(super) fn warn(&mut self, offset: usize, kind: DecodeWarningKind) {
        self.warnings.push(DecodeWarning { offset, kind });
    }

    pub(super) fn note_header_seq(&mut self, seq: Seq) {
        self.first_header_seq = self.first_header_seq.or(Some(seq));
        self.last_header_seq = Some(seq);
    }

    pub(super) fn ended_on_unknown_record(&self) -> bool {
        self.warnings.last().is_some_and(|warning| matches!(warning.kind, DecodeWarningKind::UnknownRecord { .. }))
    }

    /// Number of records rejected by sanity checks.
    pub fn rejected_records(&self) -> usize {
        self.warnings.iter().filter(|warning| matches!(warning.kind, DecodeWarningKind::SanityRejected(_))).count()
    }
}
