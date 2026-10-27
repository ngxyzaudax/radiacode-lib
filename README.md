# radiacode-lib

[![CI](https://github.com/ngxyzaudax/radiacode-lib/actions/workflows/ci.yml/badge.svg)](https://github.com/ngxyzaudax/radiacode-lib/actions/workflows/ci.yml)
[![License: AGPL-3.0-only](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)](LICENSE)
![MSRV 1.83](https://img.shields.io/badge/MSRV-1.83-orange.svg)

A Rust library for talking to [RadiaCode](https://www.radiacode.com/) gamma
spectrometers and dosimeters over USB and Bluetooth Low Energy, and for decoding
what they report: 1024-channel gamma spectra, dose rate, count rate,
accumulated dose, battery, temperature, alarms and device settings.

It is the device layer of
[radiacode-toolkit](https://github.com/ngxyzaudax/radiacode-toolkit), extracted
into a standalone crate with no GUI dependencies.

## Supported devices and transports

| Device | Firmware | USB | Bluetooth LE |
| --- | --- | --- | --- |
| RadiaCode RC-101, RC-102, RC-103, RC-110 | 4.8 or newer | yes | yes |

Older firmware is rejected with `Error::IncompatibleFirmware` unless
`OpenOptions::ignore_firmware_check(true)` is set.

| Transport | Crate feature | Backend | Platforms |
| --- | --- | --- | --- |
| USB bulk (VID `0483`, PID `f123`) | `usb` (default) | [`rusb`](https://crates.io/crates/rusb) / libusb | Linux (tested), macOS, Windows |
| Bluetooth LE GATT | `bluetooth` (default) | [`btleplug`](https://crates.io/crates/btleplug) | Linux / BlueZ (tested), macOS, Windows |
| Your own | none | implement `radiacode_lib::Transport` | any |

Upstream development and hardware testing happen on Linux. The macOS and Windows
backends compile through `rusb` and `btleplug`, but nobody has tested them against
a device yet.

## Installation

The crate is not on crates.io yet. Add it as a git dependency:

```toml
[dependencies]
radiacode-lib = { git = "https://github.com/ngxyzaudax/radiacode-lib" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Cargo features:

| Feature | Default | Effect |
| --- | --- | --- |
| `usb` | yes | USB transport and USB discovery (`rusb`) |
| `bluetooth` | yes | Bluetooth LE transport and scanning (`btleplug`) |
| `vendored` | no | Build libusb and libdbus from source instead of linking system libraries |
| `serde` | no | `Serialize` / `Deserialize` for data types |

For protocol decoding only, with no native dependencies, use
`default-features = false`.

### System requirements on Linux

Building the default features needs the libusb and D-Bus development packages
and `pkg-config`, for example on Debian or Ubuntu:

```bash
sudo apt install pkg-config libusb-1.0-0-dev libdbus-1-dev
```

Alternatively, enable the `vendored` feature to compile both libraries from
source. Bluetooth needs a running BlueZ daemon.

Non-root USB access needs the bundled udev rule. The library never installs it,
but it exports the rule text (`usb::UDEV_RULE`) and an `usb::access_status()`
diagnostic.

```bash
sudo cp radiacode.rules /etc/udev/rules.d/99-radiacode.rules
sudo udevadm control --reload && sudo udevadm trigger
```

Replug the device after installing the rule.

## Usage

All I/O is asynchronous and needs a Tokio runtime. Client futures are `Send`,
so a session can live inside a spawned task.

```rust,no_run
use radiacode_lib::data_buf::DataBufCursor;
use radiacode_lib::{connect, discover_devices, DiscoveryOptions, OpenOptions};

#[tokio::main]
async fn main() -> radiacode_lib::Result<()> {
    let devices = discover_devices(&DiscoveryOptions::default()).await?;
    let device = devices.first().ok_or(radiacode_lib::Error::DeviceNotFound)?;
    let mut session = connect(&device.endpoint, OpenOptions::default()).await?;

    let spectrum = session.spectrum().await?;
    println!("{} counts in {:?}", spectrum.total_counts(), spectrum.duration);

    let units = session.alarm_limits().await?.display_units();
    let (sample, status) = session.poll_monitor(units, &mut DataBufCursor::default(), true).await?;
    if let Some(rates) = sample.rates.last() {
        println!("{:.3} {}", rates.dose_rate, rates.dose_unit.rate_label());
    }
    println!("battery {:?} %", status.battery_percent);

    session.disconnect().await
}
```

To open a specific transport directly:

```rust,ignore
use radiacode_lib::{usb::UsbTransport, bluetooth::BluetoothTransport, OpenOptions, RadiaCode};

let usb = RadiaCode::open(Box::new(UsbTransport::open("RC-110-006806").await?), OpenOptions::default()).await?;
let ble = RadiaCode::open(Box::new(BluetoothTransport::connect("52:43:0a:bb:cc:dd").await?), OpenOptions::default()).await?;
```

The decoders are pure functions over bytes and need no device:

```rust
use radiacode_lib::data_buf::{decode_data_buf, snapshot_from_frame};

let frame = decode_data_buf(&[]);
assert!(frame.records.is_empty());
assert!(snapshot_from_frame(&frame).rates.is_none());
```

### What the client covers

| Area | Methods |
| --- | --- |
| Identity | `fw_version`, `serial_number`, `hw_serial_number`, `fw_signature`, `metadata`, `status` |
| Spectra | `spectrum`, `spectrum_accum`, `spectrum_diff`, `spectrum_reset`, `energy_calibration`, `set_energy_calibration` |
| Monitoring | `read_data_buf`, `live_rates`, `poll_monitor`, `device_status`, `rssi` |
| Settings | `alarm_limits`, `set_alarm_limits`, `load_device_config`, `apply_device_config`, `set_sound_on`, `set_vibro_on`, `set_display_brightness`, `set_language`, `set_device_on`, `dose_reset`, `sync_device_clock` |
| Raw access | `execute_raw`, `read_virt_string`, `read_vsfr_*`, `write_vsfr*`, `read_vsfr_batch`, `write_vsfr_batch`, `configuration`, `sfr_file_text` |

### Examples

| Example | Needs a device | Command |
| --- | --- | --- |
| `decode_databuf` | no | `cargo run --example decode_databuf --no-default-features` |
| `discover` | yes | `cargo run --example discover` |
| `read_spectrum` | yes | `cargo run --example read_spectrum -- usb:RC-110-006806 > spectrum.csv` |
| `live_monitor` | yes | `cargo run --example live_monitor -- ble:52:43:0a:bb:cc:dd` |

## Crate layout

| Module | Contents |
| --- | --- |
| `protocol` | Request framing, sequence numbers, response reassembly, opcodes, VSFR registers, response decoders, `SFR_FILE` and configuration catalogs |
| `data_buf` | `DataBuf` record stream decoder with resynchronisation, sanity checks, snapshots and a sequence-gap cursor |
| `spectrum` | Spectrum decoding (format 0 plain, format 1 delta-compressed) and energy calibration |
| `readings` | Dose rate, count rate, accumulated dose and status in display units |
| `config` | Alarm limits, display, sound, vibration and LED settings |
| `units` | Dose and count units and the device's fixed-point encodings |
| `transport` | The `Transport` trait |
| `usb`, `bluetooth` | Transport implementations (feature-gated) |
| `discovery` | Device endpoints, scanning, merging USB and Bluetooth results, `connect` |
| `client` | `RadiaCode`, the asynchronous session |

Library code never panics: `unwrap`, `expect`, indexing and `panic!` are
denied by lint outside tests, and every failure is a typed `radiacode_lib::Error`.

## Relation to radiacode-toolkit

[radiacode-toolkit](https://github.com/ngxyzaudax/radiacode-toolkit) is a
Linux-first desktop application (egui) for RadiaCode devices. This crate merges
its `radiacode-protocol`, `radiacode-core`, `radiacode-usb` and
`radiacode-bluetooth` workspace crates (toolkit version 0.2.0) into one library,
with these changes:

- one crate with feature-gated transports, instead of four;
- a `Send` transport trait, so sessions can run on multi-threaded runtimes;
- an MSRV of 1.83 (edition 2021) instead of 1.92 (edition 2024);
- no GUI-oriented helpers. The library reports USB permission status but never
  runs `pkexec`. Bluetooth RSSI from `btleplug` is [`RssiStatus::Measured`] or
  [`RssiStatus::NotReported`]. A caller that queries the BlueZ management socket
  and is refused reports [`RssiStatus::PermissionDenied`], whose display text is
  exactly `RSSI unavailable: permission denied`. That socket needs
  `CAP_NET_ADMIN`; the `bluetooth` group is enough for `bluetoothctl` and is not
  enough for the management socket. The library does not open that socket and
  keeps no global scan-session cache;
- explicit `Error::InvalidArgument` for out-of-range settings, and checked
  arithmetic everywhere.

The GUI, nuclide database, peak detection and spectrum analysis stay in the
toolkit.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
```

Add `--features vendored` if the system lacks libusb or D-Bus headers. The
protocol fixtures in `tests/fixtures/*.hex` are generated by
`python3 tests/fixtures/generate_fixtures.py`. The script encodes the frames
independently of the Rust decoders.

## Disclaimer

The RadiaCode protocol is undocumented. Everything here is based on reverse
engineering and observation of device behaviour, and may be incomplete or wrong
for some firmware versions. This project is not affiliated with, endorsed by,
or supported by RadiaCode or the manufacturer of RadiaCode devices.
"RadiaCode" is used only to identify compatible hardware. Writing settings or
calibration to a device is at your own risk. Do not rely on this software for
radiation safety decisions.

## License

Copyright (C) Nikola Galiot and radiacode-lib contributors.

Licensed under the GNU Affero General Public License v3.0 only
([AGPL-3.0-only](LICENSE)), the same license as radiacode-toolkit, from which
this code is derived.
