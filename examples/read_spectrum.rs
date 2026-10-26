//! Connects to a device and prints its current spectrum summary and channel data as CSV.
//!
//! ```text
//! cargo run --example read_spectrum
//! cargo run --example read_spectrum -- usb:RC-110-006806 > spectrum.csv
//! cargo run --example read_spectrum -- ble:52:43:0a:bb:cc:dd
//! ```

#[path = "support/endpoint.rs"]
mod endpoint;

use radiacode_lib::{connect, OpenOptions, Spectrum};

const LOW_ENERGY_CHANNELS: usize = 20;

#[tokio::main]
async fn main() -> radiacode_lib::Result<()> {
    let endpoint = endpoint::endpoint_from_args().await?;
    eprintln!("connecting to {endpoint}");
    let mut device = connect(&endpoint, OpenOptions::default()).await?;
    let metadata = device.metadata().await?;
    eprintln!("{} {} firmware {}", metadata.model, metadata.serial, metadata.firmware_label());
    let spectrum = device.spectrum().await?;
    print_summary(&spectrum);
    print_csv(&spectrum);
    device.disconnect().await
}

fn print_summary(spectrum: &Spectrum) {
    eprintln!("{} channels, {} counts over {:?}", spectrum.channel_count(), spectrum.total_counts(), spectrum.duration);
    if let Some(rate) = spectrum.mean_count_rate_cps() {
        eprintln!("mean count rate {rate:.1} cps");
    }
    if let Some(peak) = spectrum.peak_channel(LOW_ENERGY_CHANNELS) {
        eprintln!("highest channel {peak} at {:.1} keV", spectrum.channel_energy(peak));
    }
}

fn print_csv(spectrum: &Spectrum) {
    println!("channel,energy_kev,counts");
    spectrum
        .energy_counts()
        .enumerate()
        .for_each(|(channel, (energy, count))| println!("{channel},{energy:.2},{count}"));
}
