//! DataBuf record-stream fixtures: a clean monitoring session and a corrupted transfer.

mod support;

use radiacode_lib::data_buf::{
    decode_data_buf, snapshot_from_frame, DataBufCursor, DataBufRecord, DecodeWarningKind, EventId, RecordKind,
};
use radiacode_lib::readings::monitor_sample_from;
use radiacode_lib::DisplayUnits;
use support::wire::{virt_string, CHUNK_SIZES};

const SESSION: &str = include_str!("fixtures/data_buf_session.hex");
const CORRUPT: &str = include_str!("fixtures/data_buf_corrupt.hex");

fn kinds(records: &[DataBufRecord]) -> Vec<RecordKind> {
    records.iter().map(|record| record.header().kind).collect()
}

#[test]
fn session_decodes_every_record_kind_without_warnings() {
    for chunk_size in CHUNK_SIZES {
        let frame = decode_data_buf(virt_string(SESSION, 0x89, chunk_size).data());
        assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
        assert_eq!(
            kinds(&frame.records),
            [
                RecordKind::RealTimeData,
                RecordKind::RawData,
                RecordKind::DoseRateDb,
                RecordKind::RareData,
                RecordKind::Event,
                RecordKind::AccelData,
                RecordKind::UserData,
                RecordKind::Waveform8,
                RecordKind::RealTimeData,
                RecordKind::RealTimeData,
            ]
        );
        assert_eq!(frame.first_header_seq.map(|seq| seq.raw()), Some(40));
        assert_eq!(frame.last_header_seq.map(|seq| seq.raw()), Some(49));
    }
}

#[test]
fn session_records_carry_decoded_values() {
    let frame = decode_data_buf(virt_string(SESSION, 0x89, 20).data());
    let DataBufRecord::Rare(rare) = &frame.records[3] else { panic!("record 3 is rare data") };
    assert_eq!((rare.duration_secs, rare.temperature_c, rare.battery_percent), (86400, 23.75, 82.5));
    let DataBufRecord::Event(event) = &frame.records[4] else { panic!("record 4 is an event") };
    assert_eq!(event.event, EventId::DoseReset);
    let DataBufRecord::Accel(accel) = &frame.records[5] else { panic!("record 5 is accelerometer data") };
    assert_eq!((accel.x, accel.y, accel.z), (512, 498, 16100));
    let DataBufRecord::RealTime(first) = &frame.records[0] else { panic!("record 0 is real-time data") };
    assert_eq!((first.count_rate_err_pct, first.dose_rate_err_pct), (4.5, 6.0));
}

#[test]
fn session_snapshot_and_monitor_sample_use_latest_values() {
    let frame = decode_data_buf(virt_string(SESSION, 0x89, 256).data());
    let snapshot = snapshot_from_frame(&frame);
    let rates = snapshot.rates.unwrap();
    assert!((rates.count_rate_cps - 5.4).abs() < 1e-6);
    let rare = snapshot.rare.unwrap();
    assert_eq!((rare.battery_percent, rare.temperature_c), (82.5, 23.75));
    let mut cursor = DataBufCursor::default();
    let sample = monitor_sample_from(&frame, DisplayUnits::default(), cursor.observe_frame(&frame));
    assert_eq!(sample.rates.len(), 3);
    assert!((sample.rates[2].dose_rate - 0.13).abs() < 1e-4);
    assert!((sample.accumulated.unwrap().dose - 29.0).abs() < 1e-3);
    assert!(sample.seq_gaps.is_empty());
}

#[test]
fn corrupt_transfer_resyncs_and_flags_truncated_tail() {
    for chunk_size in CHUNK_SIZES {
        let frame = decode_data_buf(virt_string(CORRUPT, 0x8A, chunk_size).data());
        assert_eq!(frame.records.len(), 10);
        assert_eq!(frame.resync_count, 1);
        let warning_kinds: Vec<&DecodeWarningKind> = frame.warnings.iter().map(|warning| &warning.kind).collect();
        assert!(matches!(warning_kinds[0], DecodeWarningKind::UnknownRecord { entity: 0x7F, group: 0x7F, .. }));
        assert!(matches!(warning_kinds[1..], [DecodeWarningKind::PayloadUnderrun, DecodeWarningKind::TruncatedTail]));
        assert_eq!(frame.warnings[0].offset, 22 + 15 + 23);
    }
}
