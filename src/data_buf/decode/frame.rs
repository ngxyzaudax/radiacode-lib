use std::ops::ControlFlow;

use super::payload::parse_payload;
use super::resync::{next_header_offset, resync_allowed, unknown_record_tail, HEADER_LEN};
use super::warnings::{DataBufFrame, DecodeWarningKind};
use crate::data_buf::header::{DeviceTicks, RecordHeader, Seq};
use crate::data_buf::kind::RecordKind;
use crate::data_buf::sanity::check_record;
use crate::protocol::BytesBuffer;

/// Decodes a DataBuf virtual-string payload into records and warnings.
///
/// Decoding never fails: malformed input yields warnings, an attempt to
/// resynchronise on the next plausible header, and a partial record list.
pub fn decode_data_buf(data: &[u8]) -> DataBufFrame {
    let mut decoder = FrameDecoder::new(data);
    while decoder.has_header() && decoder.step().is_continue() {}
    decoder.finish()
}

struct FrameDecoder {
    total_len: usize,
    buffer: BytesBuffer,
    frame: DataBufFrame,
    expected_seq: Option<Seq>,
}

impl FrameDecoder {
    fn new(data: &[u8]) -> Self {
        Self {
            total_len: data.len(),
            buffer: BytesBuffer::from(data),
            frame: DataBufFrame::default(),
            expected_seq: None,
        }
    }

    fn has_header(&self) -> bool {
        self.buffer.size() >= HEADER_LEN
    }

    fn offset(&self) -> usize {
        self.total_len.saturating_sub(self.buffer.size())
    }

    fn step(&mut self) -> ControlFlow<()> {
        let offset = self.offset();
        let Some(header) = parse_header(&mut self.buffer) else {
            return ControlFlow::Break(());
        };
        self.frame.note_header_seq(header.seq);
        match header.kind {
            RecordKind::Unknown { entity, group } => self.on_unknown(offset, entity, group),
            _ => self.on_known(offset, header),
        }
    }

    fn on_known(&mut self, offset: usize, header: RecordHeader) -> ControlFlow<()> {
        self.check_sequence(offset, header.seq);
        self.expected_seq = Some(header.seq.next());
        self.decode_record(offset, header)
    }

    fn on_unknown(&mut self, offset: usize, entity: u8, group: u8) -> ControlFlow<()> {
        let tail = unknown_record_tail(self.buffer.data());
        let kind = DecodeWarningKind::UnknownRecord { entity, group, tail };
        self.frame.warn(offset, kind);
        match self.expected_seq.filter(|_| resync_allowed(self.frame.resync_count)) {
            Some(expected) => self.resync(expected),
            None => ControlFlow::Break(()),
        }
    }

    fn resync(&mut self, expected: Seq) -> ControlFlow<()> {
        let skipped = next_header_offset(self.buffer.data(), expected).and_then(|skip| self.buffer.skip(skip).ok());
        match skipped {
            Some(()) => {
                self.frame.resync_count = self.frame.resync_count.saturating_add(1);
                ControlFlow::Continue(())
            }
            None => ControlFlow::Break(()),
        }
    }

    fn check_sequence(&mut self, offset: usize, seq: Seq) {
        if let Some(expected) = self.expected_seq.filter(|expected| *expected != seq) {
            let kind = DecodeWarningKind::SeqJump { expected: expected.raw(), got: seq.raw() };
            self.frame.warn(offset, kind);
        }
    }

    fn decode_record(&mut self, offset: usize, header: RecordHeader) -> ControlFlow<()> {
        match parse_payload(&mut self.buffer, header) {
            Ok(record) => {
                match check_record(&record) {
                    Ok(()) => self.frame.records.push(record),
                    Err(failure) => self.frame.warn(offset, DecodeWarningKind::SanityRejected(failure)),
                }
                ControlFlow::Continue(())
            }
            Err(_) => {
                self.frame.warn(offset, DecodeWarningKind::PayloadUnderrun);
                ControlFlow::Break(())
            }
        }
    }

    fn finish(mut self) -> DataBufFrame {
        if !self.buffer.is_empty() && !self.frame.ended_on_unknown_record() {
            let offset = self.offset();
            self.frame.warn(offset, DecodeWarningKind::TruncatedTail);
        }
        self.frame
    }
}

fn parse_header(buffer: &mut BytesBuffer) -> Option<RecordHeader> {
    let seq = Seq::new(buffer.take_u8().ok()?);
    let entity = buffer.take_u8().ok()?;
    let group = buffer.take_u8().ok()?;
    let ts = DeviceTicks::new(buffer.take_i32_le().ok()?);
    let kind = RecordKind::from_entity_group(entity, group);
    Some(RecordHeader { seq, kind, ts })
}
