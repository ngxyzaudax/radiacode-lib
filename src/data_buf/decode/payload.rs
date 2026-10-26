use crate::data_buf::flags::{EventId, StatusFlags};
use crate::data_buf::header::RecordHeader;
use crate::data_buf::kind::RecordKind;
use crate::data_buf::records::{AccelData, DataBufRecord, DoseRateDb, EventRecord, RareData, RawData, RealTimeData};
use crate::error::{Error, Result};
use crate::protocol::BytesBuffer;
use crate::units::{CountRateCps, DoseRateRh, DoseRoentgen};

const ERROR_PERCENT_SCALE: f32 = 10.0;
const TEMPERATURE_RAW_OFFSET: f32 = 2000.0;
const HUNDREDTHS: f32 = 100.0;

/// Parses the payload following `header`.
pub(super) fn parse_payload(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<DataBufRecord> {
    match header.kind {
        RecordKind::RealTimeData => parse_real_time(buffer, header).map(DataBufRecord::RealTime),
        RecordKind::RawData => parse_raw(buffer, header).map(DataBufRecord::Raw),
        RecordKind::DoseRateDb => parse_dose_rate_db(buffer, header).map(DataBufRecord::DoseRateDb),
        RecordKind::RareData => parse_rare(buffer, header).map(DataBufRecord::Rare),
        RecordKind::AccelData => parse_accel(buffer, header).map(DataBufRecord::Accel),
        RecordKind::Event => parse_event(buffer, header).map(DataBufRecord::Event),
        other => skip_opaque(buffer, other).map(|()| DataBufRecord::Skipped(header)),
    }
}

fn skip_opaque(buffer: &mut BytesBuffer, kind: RecordKind) -> Result<()> {
    match (kind.opaque_payload_len(), kind.waveform_sample_len()) {
        (Some(len), _) => buffer.skip(len),
        (None, Some(sample_len)) => skip_waveform(buffer, sample_len),
        (None, None) => Err(Error::mismatch("known DataBuf record", format!("{kind:?}"))),
    }
}

fn skip_waveform(buffer: &mut BytesBuffer, sample_len: usize) -> Result<()> {
    let samples = usize::from(buffer.take_u16_le()?);
    buffer.skip(4)?;
    buffer.skip(samples.saturating_mul(sample_len))
}

fn error_percent(raw: u16) -> f32 {
    f32::from(raw) / ERROR_PERCENT_SCALE
}

fn parse_real_time(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<RealTimeData> {
    Ok(RealTimeData {
        header,
        count_rate_cps: CountRateCps::new(buffer.take_f32_le()?),
        dose_rate_rh: DoseRateRh::new(buffer.take_f32_le()?),
        count_rate_err_pct: error_percent(buffer.take_u16_le()?),
        dose_rate_err_pct: error_percent(buffer.take_u16_le()?),
        flags: StatusFlags::new(buffer.take_u16_le()?),
        real_time_flags: buffer.take_u8()?,
    })
}

fn parse_raw(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<RawData> {
    Ok(RawData {
        header,
        count_rate_cps: CountRateCps::new(buffer.take_f32_le()?),
        dose_rate_rh: DoseRateRh::new(buffer.take_f32_le()?),
    })
}

fn parse_dose_rate_db(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<DoseRateDb> {
    Ok(DoseRateDb {
        header,
        count: buffer.take_u32_le()?,
        count_rate_cps: CountRateCps::new(buffer.take_f32_le()?),
        dose_rate_rh: DoseRateRh::new(buffer.take_f32_le()?),
        dose_rate_err_pct: error_percent(buffer.take_u16_le()?),
        flags: StatusFlags::new(buffer.take_u16_le()?),
    })
}

fn parse_rare(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<RareData> {
    Ok(RareData {
        header,
        duration_secs: buffer.take_u32_le()?,
        dose_r: DoseRoentgen::new(buffer.take_f32_le()?),
        temperature_c: (f32::from(buffer.take_u16_le()?) - TEMPERATURE_RAW_OFFSET) / HUNDREDTHS,
        battery_percent: f32::from(buffer.take_u16_le()?) / HUNDREDTHS,
        flags: StatusFlags::new(buffer.take_u16_le()?),
    })
}

fn parse_accel(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<AccelData> {
    Ok(AccelData { header, x: buffer.take_u16_le()?, y: buffer.take_u16_le()?, z: buffer.take_u16_le()? })
}

fn parse_event(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<EventRecord> {
    Ok(EventRecord {
        header,
        event: EventId::from_raw(buffer.take_u8()?),
        event_param1: buffer.take_u8()?,
        flags: StatusFlags::new(buffer.take_u16_le()?),
    })
}
