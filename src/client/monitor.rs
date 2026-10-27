use tracing::debug;

use super::RadiaCode;
use crate::data_buf::{
    decode_data_buf, snapshot_from_frame, DataBufCursor, DataBufFrame, DataBufSnapshot, DecodeWarningKind,
};
use crate::error::{Error, Result};
use crate::protocol::{hex_bytes, VirtSfr, VirtString};
use crate::readings::{
    live_rates_from, monitor_sample_from, plausible_temperature, status_from_snapshot, DeviceStatus, LiveRates,
    MonitorPollSample,
};
use crate::units::DisplayUnits;

impl RadiaCode {
    /// Reads and decodes the device `DataBuf` record stream.
    pub async fn read_data_buf(&mut self) -> Result<DataBufFrame> {
        let payload = self.read_virt_string(VirtString::DataBuf).await?;
        let frame = decode_data_buf(payload.data());
        log_unknown_records(&frame);
        Ok(frame)
    }

    /// Latest dose and count rate; fails with [`Error::MonitorDataPending`] until the device has buffered a rate record.
    pub async fn live_rates(&mut self, units: DisplayUnits) -> Result<LiveRates> {
        let frame = self.read_data_buf().await?;
        let rates = snapshot_from_frame(&frame).rates.ok_or(Error::MonitorDataPending)?;
        debug!(warnings = frame.warnings.len(), ?rates, "live rates decoded");
        Ok(live_rates_from(&rates, units))
    }

    /// Battery, temperature and RSSI; `refresh_rssi` samples a fresh RSSI value.
    pub async fn device_status(&mut self, refresh_rssi: bool) -> Result<DeviceStatus> {
        let frame = self.read_data_buf().await?;
        Ok(self.status_with_fallbacks(&snapshot_from_frame(&frame), refresh_rssi).await)
    }

    /// Polls the record stream once, returning all new samples and the device status.
    pub async fn poll_monitor(
        &mut self,
        units: DisplayUnits,
        cursor: &mut DataBufCursor,
        refresh_rssi: bool,
    ) -> Result<(MonitorPollSample, DeviceStatus)> {
        let frame = self.read_data_buf().await?;
        let seq_gaps = cursor.observe_frame(&frame);
        let status = self.status_with_fallbacks(&snapshot_from_frame(&frame), refresh_rssi).await;
        let sample = monitor_sample_from(&frame, units, seq_gaps);
        debug!(
            rate_count = sample.rates.len(),
            decode_warnings = sample.decode_warnings,
            rejected_records = sample.rejected_records,
            seq_gap_count = sample.seq_gaps.len(),
            "monitor poll"
        );
        Ok((sample, status))
    }

    async fn status_with_fallbacks(&mut self, snapshot: &DataBufSnapshot, refresh_rssi: bool) -> DeviceStatus {
        let from_records = status_from_snapshot(snapshot);
        let temperature_c = match from_records.temperature_c {
            Some(value) => Some(value),
            None => self.read_vsfr_f32(VirtSfr::TempDegC).await.ok().filter(|t| plausible_temperature(*t)),
        };
        let rssi = match refresh_rssi {
            true => self.sample_rssi().await,
            false => self.rssi().await,
        };
        DeviceStatus { temperature_c, rssi, ..from_records }
    }
}

fn log_unknown_records(frame: &DataBufFrame) {
    frame.warnings.iter().for_each(|warning| {
        if let DecodeWarningKind::UnknownRecord { entity, group, tail } = &warning.kind {
            debug!(entity, group, offset = warning.offset, tail_hex = %hex_bytes(tail), "unknown DataBuf record type");
        }
    });
}
