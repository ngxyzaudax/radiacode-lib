//! Lists RadiaCode devices reachable over USB and Bluetooth LE.
//!
//! ```text
//! cargo run --example discover
//! ```
//!
//! On Linux, USB access needs the bundled udev rule (`radiacode.rules`) and
//! Bluetooth scanning needs a running BlueZ daemon.

use radiacode_lib::{discover_devices, DiscoveredDevice, DiscoveryOptions};

#[tokio::main]
async fn main() -> radiacode_lib::Result<()> {
    let devices = discover_devices(&DiscoveryOptions::default()).await?;
    match devices.is_empty() {
        true => println!("no RadiaCode devices found"),
        false => devices.iter().for_each(print_device),
    }
    Ok(())
}

fn print_device(device: &DiscoveredDevice) {
    let rssi = device.rssi.map(|rssi| format!(", RSSI {rssi} dBm")).unwrap_or_default();
    let serial = device.serial.as_deref().unwrap_or("serial unknown");
    println!("{:<10} {} ({serial}{rssi})", device.display_label(), device.endpoint);
}
