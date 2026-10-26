use super::{event, header, raw, real_time, realtime};
use crate::data_buf::{decode_data_buf, latest_snapshot, DataBufRecord, DecodeWarningKind, Seq};

fn has_warning(frame: &crate::data_buf::DataBufFrame, test: fn(&DecodeWarningKind) -> bool) -> bool {
    frame.warnings.iter().any(|warning| test(&warning.kind))
}

#[test]
fn real_time_record_yields_rates_with_errors() {
    let rates = latest_snapshot(&real_time(1, 12.5, 0.000_125)).rates;
    let rates = rates.unwrap_or_else(|| panic!("rates missing"));
    assert!((rates.count_rate_cps - 12.5).abs() < 0.001);
    assert!((rates.dose_rate_rh - 0.000_125).abs() < 1e-9);
    assert!((rates.count_rate_err_pct - 1.0).abs() < 0.001);
    assert!((rates.dose_rate_err_pct - 2.0).abs() < 0.001);
}

#[test]
fn seq_jump_is_reported_with_record_offset() {
    let bytes = [realtime(1), realtime(3)].concat();
    let frame = decode_data_buf(&bytes);
    assert_eq!(frame.records.len(), 2);
    let jump = frame.warnings.first().map(|w| (w.offset, w.kind.clone()));
    assert_eq!(jump, Some((22, DecodeWarningKind::SeqJump { expected: 2, got: 3 })));
    assert_eq!(frame.first_header_seq, Some(Seq::new(1)));
    assert_eq!(frame.last_header_seq, Some(Seq::new(3)));
}

#[test]
fn non_finite_rates_are_rejected() {
    let frame = decode_data_buf(&real_time(1, f32::NAN, 0.001));
    assert!(frame.records.is_empty());
    assert_eq!(frame.rejected_records(), 1);
    let negative = decode_data_buf(&real_time(1, -1.0, 0.001));
    assert!(has_warning(&negative, |k| matches!(k, DecodeWarningKind::SanityRejected(_))));
}

#[test]
fn newer_sequence_wins_across_wrap() {
    assert!(Seq::new(2).is_newer_than(Seq::new(250)));
    let bytes = [realtime(250), raw(2, 30.0, 0.001)].concat();
    let rates = latest_snapshot(&bytes).rates.map(|r| r.count_rate_cps);
    assert_eq!(rates, Some(30.0));
}

#[test]
fn real_time_outranks_raw_on_same_sequence() {
    let bytes = [real_time(4, 7.0, 0.001), raw(4, 30.0, 0.001)].concat();
    let rates = latest_snapshot(&bytes).rates.map(|r| r.count_rate_cps);
    assert_eq!(rates, Some(7.0));
}

#[test]
fn unknown_record_without_history_stops_decode() {
    let bytes = [header(1, 99, 99, 0), vec![0; 8], realtime(2)].concat();
    let frame = decode_data_buf(&bytes);
    assert!(frame.records.is_empty());
    assert!(has_warning(&frame, |k| matches!(k, DecodeWarningKind::UnknownRecord { .. })));
    assert!(!has_warning(&frame, |k| matches!(k, DecodeWarningKind::SeqJump { .. })));
    assert!(!has_warning(&frame, |k| matches!(k, DecodeWarningKind::TruncatedTail)));
}

#[test]
fn resyncs_after_garbage_without_phantom_seq_jumps() {
    let bytes = [realtime(1), header(2, 99, 99, 0), vec![0xAA, 0xBB, 0xCC], realtime(2)].concat();
    let frame = decode_data_buf(&bytes);
    assert_eq!(frame.records.len(), 2);
    assert_eq!(frame.resync_count, 1);
    assert!(!has_warning(&frame, |k| matches!(k, DecodeWarningKind::SeqJump { .. })));
}

#[test]
fn event_followed_by_real_time_decodes_both() {
    let bytes = [event(1, 0x04, 0, 0), realtime(2)].concat();
    let frame = decode_data_buf(&bytes);
    assert!(frame.warnings.is_empty());
    assert!(matches!(frame.records.as_slice(), [DataBufRecord::Event(_), DataBufRecord::RealTime(_)]));
}

#[test]
fn payload_underrun_and_truncated_tail_are_reported() {
    let short = realtime(1).into_iter().take(12).collect::<Vec<u8>>();
    let frame = decode_data_buf(&short);
    assert_eq!(frame.warnings.first().map(|w| w.kind.clone()), Some(DecodeWarningKind::PayloadUnderrun));
    let tail = [realtime(1), vec![1, 2, 3]].concat();
    let frame = decode_data_buf(&tail);
    assert_eq!(frame.warnings.last().map(|w| (w.offset, w.kind.clone())), Some((22, DecodeWarningKind::TruncatedTail)));
}

#[test]
fn empty_and_tiny_payloads_never_panic() {
    assert_eq!(decode_data_buf(&[]), Default::default());
    assert!(decode_data_buf(&[0xFF; 6]).records.is_empty());
    let noise: Vec<u8> = (0..=255u8).cycle().take(4096).collect();
    let _ = decode_data_buf(&noise);
}
