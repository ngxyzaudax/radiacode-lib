use crate::data_buf::header::Seq;
use crate::data_buf::kind::RecordKind;

/// Size of a DataBuf record header in bytes.
pub(super) const HEADER_LEN: usize = 7;
const MAX_RESYNCS_PER_FRAME: u32 = 4;
const UNKNOWN_TAIL_LEN: usize = 32;

pub(super) fn unknown_record_tail(remaining: &[u8]) -> Vec<u8> {
    remaining.iter().take(UNKNOWN_TAIL_LEN).copied().collect()
}

pub(super) fn resync_allowed(resync_count: u32) -> bool {
    resync_count < MAX_RESYNCS_PER_FRAME
}

/// Finds how many bytes to skip so that the next header carries `expected_seq` and a known kind.
pub(super) fn next_header_offset(remaining: &[u8], expected_seq: Seq) -> Option<usize> {
    let last_start = remaining.len().checked_sub(HEADER_LEN)?;
    (1..=last_start).find(|&skip| header_starts_at(remaining, skip, expected_seq))
}

fn header_starts_at(remaining: &[u8], skip: usize, expected_seq: Seq) -> bool {
    match remaining.get(skip..skip.saturating_add(3)) {
        Some([seq, entity, group]) => {
            *seq == expected_seq.raw() && RecordKind::from_entity_group(*entity, *group).is_known()
        }
        _ => false,
    }
}
