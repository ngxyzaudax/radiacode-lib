//! Polls dose rate, count rate, battery and temperature once per second.
//!
//! ```text
//! cargo run --example live_monitor
//! cargo run --example live_monitor -- ble:52:43:0a:bb:cc:dd
//! ```

#[path = "support/endpoint.rs"]
mod endpoint;

use std::time::Duration;

use radiacode_lib::data_buf::DataBufCursor;
use radiacode_lib::{connect, DeviceStatus, MonitorPollSample, OpenOptions, RadiaCode};

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const POLL_COUNT: usize = 30;

#[tokio::main]
async fn main() -> radiacode_lib::Result<()> {
    let endpoint = endpoint::endpoint_from_args().await?;
    let mut device = connect(&endpoint, OpenOptions::default()).await?;
    let units = device.alarm_limits().await?.display_units();
    let mut cursor = DataBufCursor::default();
    for _ in 0..POLL_COUNT {
        let (sample, status) = device.poll_monitor(units, &mut cursor, true).await?;
        print_poll(&sample, &status);
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    finish(device, &cursor).await
}

fn print_poll(sample: &MonitorPollSample, status: &DeviceStatus) {
    if let Some(rates) = sample.rates.last() {
        println!(
            "{:8.3} {} ±{:.0}%  {:8.1} {}  battery {:?} %  temperature {:?} C  rssi {:?} dBm",
            rates.dose_rate,
            rates.dose_unit.rate_label(),
            rates.dose_rate_err_pct,
            rates.count_rate,
            rates.count_unit.label(),
            status.battery_percent,
            status.temperature_c,
            status.rssi_dbm,
        );
    }
}

async fn finish(device: RadiaCode, cursor: &DataBufCursor) -> radiacode_lib::Result<()> {
    println!("sequence gaps: {}, records lost: {}", cursor.total_gaps(), cursor.total_lost());
    device.disconnect().await
}
