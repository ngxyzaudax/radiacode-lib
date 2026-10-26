use super::{
    count_display_from_cps, decode_count_alarm, decode_dose_accum, decode_dose_alarm, dose_display_from_rh,
    encode_count_alarm, encode_dose_accum, encode_dose_alarm, CountDisplayUnit, DoseDisplayUnit, RawCountsPer10s,
};

const SV: DoseDisplayUnit = DoseDisplayUnit::MicroSievertPerHour;
const R: DoseDisplayUnit = DoseDisplayUnit::MicroRoentgenPerHour;

fn close(left: f32, right: f32) -> bool {
    (left - right).abs() < 0.01
}

#[test]
fn dose_alarm_round_trips_in_both_families() {
    assert_eq!(encode_dose_alarm(1.25, SV).as_u32(), 125);
    assert!(close(decode_dose_alarm(encode_dose_alarm(1.25, SV), SV), 1.25));
    assert_eq!(encode_dose_alarm(125.0, R).as_u32(), 125);
    assert!(close(decode_dose_alarm(encode_dose_alarm(125.0, R), R), 125.0));
}

#[test]
fn count_alarm_round_trips_in_both_units() {
    assert_eq!(encode_count_alarm(42.0, CountDisplayUnit::Cps).as_u32(), 420);
    assert_eq!(encode_count_alarm(600.0, CountDisplayUnit::Cpm).as_u32(), 100);
    let raw = RawCountsPer10s::new(100);
    assert!(close(decode_count_alarm(raw, CountDisplayUnit::Cpm), 600.0));
    assert!(close(decode_count_alarm(raw, CountDisplayUnit::Cps), 10.0));
}

#[test]
fn negative_and_non_finite_thresholds_encode_as_zero() {
    assert_eq!(encode_dose_accum(-5.0, SV), 0);
    assert_eq!(encode_dose_accum(f32::NAN, SV), 0);
    assert!(close(decode_dose_accum(250, SV), 2.5));
}

#[test]
fn realtime_conversions() {
    assert!((dose_display_from_rh(0.000_125, SV) - 1.25).abs() < 0.001);
    assert!((dose_display_from_rh(0.000_125, R) - 125.0).abs() < 0.1);
    assert!(close(count_display_from_cps(10.0, CountDisplayUnit::Cpm), 600.0));
    assert!(close(count_display_from_cps(10.0, CountDisplayUnit::Cps), 10.0));
}

#[test]
fn device_flags_and_labels() {
    assert_eq!(DoseDisplayUnit::from_device_flag(SV.to_device_flag()), SV);
    assert_eq!(DoseDisplayUnit::from_device_flag(R.to_device_flag()), R);
    assert_eq!(DoseDisplayUnit::from_device_flag(7), SV);
    assert_eq!(CountDisplayUnit::from_device_flag(0), CountDisplayUnit::Cps);
    assert_eq!(CountDisplayUnit::from_device_flag(1), CountDisplayUnit::Cpm);
    assert_eq!((SV.rate_label(), SV.dose_label()), ("µSv/h", "µSv"));
    assert_eq!((R.rate_label(), R.dose_label()), ("µR/h", "µR"));
    assert_eq!(CountDisplayUnit::Cpm.label(), "cpm");
}
