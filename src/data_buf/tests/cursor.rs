use super::{rare, realtime};
use crate::data_buf::{decode_data_buf, DataBufCursor, SeqGap};

fn observe(cursor: &mut DataBufCursor, bytes: &[u8]) -> Vec<SeqGap> {
    cursor.observe_frame(&decode_data_buf(bytes))
}

fn summary(gaps: &[SeqGap]) -> Vec<(u8, u8, u32, bool)> {
    gaps.iter().map(|g| (g.expected, g.got, g.lost, g.reset)).collect()
}

#[test]
fn ignores_non_rate_records_between_polls() {
    let mut cursor = DataBufCursor::new();
    assert!(observe(&mut cursor, &realtime(10)).is_empty());
    assert!(observe(&mut cursor, &[rare(11, 50.0, 20.0), realtime(12)].concat()).is_empty());
}

#[test]
fn reports_gap_between_polls_and_accumulates_totals() {
    let mut cursor = DataBufCursor::new();
    observe(&mut cursor, &realtime(10));
    let gaps = observe(&mut cursor, &realtime(12));
    assert_eq!(summary(&gaps), [(11, 12, 1, false)]);
    assert_eq!((cursor.total_gaps(), cursor.total_lost()), (1, 1));
}

#[test]
fn reports_gap_inside_single_frame_once() {
    let mut cursor = DataBufCursor::new();
    observe(&mut cursor, &realtime(1));
    let gaps = observe(&mut cursor, &[realtime(2), realtime(5)].concat());
    assert_eq!(summary(&gaps), [(3, 5, 2, false)]);
}

#[test]
fn reset_clears_history() {
    let mut cursor = DataBufCursor::new();
    observe(&mut cursor, &realtime(200));
    cursor.reset();
    assert!(observe(&mut cursor, &realtime(5)).is_empty());
    assert_eq!(cursor.last_seq().map(|s| s.raw()), Some(5));
}

#[test]
fn backwards_and_large_jumps_are_resets() {
    let mut cursor = DataBufCursor::new();
    observe(&mut cursor, &realtime(200));
    assert_eq!(summary(&observe(&mut cursor, &realtime(5))), [(201, 5, 0, true)]);
    observe(&mut cursor, &realtime(6));
    assert_eq!(summary(&observe(&mut cursor, &realtime(106))), [(7, 106, 0, true)]);
    assert_eq!(cursor.total_gaps(), 0);
}

#[test]
fn wrapping_sequence_counts_loss_across_boundary() {
    let mut cursor = DataBufCursor::new();
    observe(&mut cursor, &realtime(254));
    assert_eq!(summary(&observe(&mut cursor, &realtime(1))), [(255, 1, 2, false)]);
}
