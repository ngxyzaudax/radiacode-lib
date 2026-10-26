use std::time::{Duration, SystemTime};

use super::{event, header, rare};
use crate::data_buf::{decode_data_buf, latest_snapshot, DataBufRecord, DeviceTicks, EventId, RecordKind};

#[test]
fn rare_record_decodes_temperature_and_battery() {
    let status = latest_snapshot(&rare(1, 85.0, 23.08)).rare;
    let status = status.unwrap_or_else(|| panic!("rare missing"));
    assert_eq!(status.duration_secs, 600);
    assert!((status.temperature_c - 23.08).abs() < 0.01);
    assert!((status.battery_percent - 85.0).abs() < 0.01);
}

#[test]
fn implausible_rare_values_are_rejected() {
    assert!(decode_data_buf(&rare(1, 160.0, 20.0)).records.is_empty());
    assert!(decode_data_buf(&rare(1, 50.0, 90.0)).records.is_empty());
}

#[test]
fn event_record_fields() {
    let frame = decode_data_buf(&event(1, 0x14, 0x03, 0x1141));
    let Some(DataBufRecord::Event(record)) = frame.records.first() else {
        panic!("expected event record");
    };
    assert_eq!(record.event, EventId::CountRateAlarm1);
    assert_eq!(record.event_param1, 0x03);
    assert_eq!(record.flags.raw(), 0x1141);
}

#[test]
fn event_ids_round_trip() {
    assert!((0..=22u8).all(|code| EventId::from_raw(code).raw() == code));
    assert_eq!(EventId::from_raw(200), EventId::Unknown(200));
    assert_eq!(EventId::Unknown(200).raw(), 200);
}

#[test]
fn opaque_and_waveform_records_are_skipped() {
    let user = [header(1, 0, 4, 0), vec![0; 16]].concat();
    let raw_rate = [header(2, 0, 8, 0), vec![0; 6]].concat();
    let waveform = [header(3, 1, 2, 0), 2u16.to_le_bytes().to_vec(), vec![0; 4 + 32]].concat();
    let accel = [header(4, 0, 6, 0), vec![1, 0, 2, 0, 3, 0]].concat();
    let frame = decode_data_buf(&[user, raw_rate, waveform, accel].concat());
    assert!(frame.warnings.is_empty());
    let kinds: Vec<RecordKind> = frame.records.iter().map(|r| r.header().kind).collect();
    assert_eq!(kinds, [RecordKind::UserData, RecordKind::RawCountRate, RecordKind::Waveform16, RecordKind::AccelData]);
}

#[test]
fn record_kinds_map_from_entity_group() {
    assert_eq!(RecordKind::from_entity_group(0, 2), RecordKind::DoseRateDb);
    assert_eq!(RecordKind::from_entity_group(1, 3), RecordKind::Waveform14);
    assert_eq!(RecordKind::from_entity_group(2, 0), RecordKind::Unknown { entity: 2, group: 0 });
    assert_eq!(RecordKind::RawData.monitor_source_rank(), 1);
}

#[test]
fn device_ticks_convert_to_durations() {
    let first = DeviceTicks::new(-3387);
    let second = DeviceTicks::new(-3287);
    assert_eq!(first.duration_since(first), Duration::ZERO);
    assert_eq!(second.duration_since(first).as_secs(), 1);
    assert_eq!(first.as_duration(), Duration::ZERO);
    let base = SystemTime::UNIX_EPOCH;
    assert_eq!(DeviceTicks::new(150).wall_clock_time(base), base + Duration::from_millis(1500));
    assert_eq!(DeviceTicks::new(i32::MIN).duration_since(DeviceTicks::new(i32::MAX)), Duration::ZERO);
}
