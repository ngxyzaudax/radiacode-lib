# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-10-27

### Added

- `RssiStatus` (`Measured`, `NotReported`, `PermissionDenied`). `PermissionDenied` displays as `RSSI unavailable: permission denied`. The BlueZ management socket needs `CAP_NET_ADMIN`; the `bluetooth` group is enough for `bluetoothctl` and is not enough for that socket. This crate does not open the socket.
- Fast Bluetooth reconnect: `DeviceNotFound` and `le-connection-abort-by-local` are retried up to three times, with a 750 ms pause and a fresh scan after the first miss.
- A connect, service discovery, or notification subscribe that does not finish within 8 s is disconnected and retried on that same path.
- A pending Bluetooth read fails as soon as the link drops, instead of waiting out the 30 s response timeout.
- Real-time `DataBuf` records whose error percent is outside 0–100 are rejected. The newest rare record is chosen by sequence number, not by buffer order.

### Changed

- **Breaking:** `DeviceStatus.rssi_dbm` is now `rssi: RssiStatus`. `Transport::link_rssi_dbm` and `sample_link_rssi_dbm`, and `RadiaCode::rssi_dbm` and `sample_rssi_dbm`, are renamed to `link_rssi`, `sample_link_rssi`, `rssi`, and `sample_rssi`.

## [0.1.0] - 2026-10-26

Initial release, extracted from the device crates of
[radiacode-toolkit](https://github.com/ngxyzaudax/radiacode-toolkit) 0.2.0.

### Added

- Wire protocol:
  - request framing, sequence numbers and response reassembly
  - opcodes, virtual strings and the VSFR register table
  - single and batch register access
  - decoders for firmware version, serial, signature and status
  - `SFR_FILE` and CP1251 configuration catalogs, with catalog drift detection
- `DataBuf` record stream decoding:
  - real-time, raw, dose-rate database, rare, event and accelerometer records
  - skipping of opaque and waveform records
  - resynchronisation after corrupt input
  - sanity checks, snapshots and a sequence-gap cursor
- Spectrum decoding for formats 0 (plain) and 1 (delta-compressed), and energy calibration.
- Readings, alarm limits and device configuration in display units (µSv/h, µR/h, cps, cpm).
- `Transport` trait with `Send` futures.
- Feature-gated transports:
  - USB (`usb`, built on `rusb`)
  - Bluetooth LE (`bluetooth`, built on `btleplug`), with a peripheral-id fallback on macOS
- Device discovery over both transports, merging and deduplicating devices seen on both.
- Asynchronous `RadiaCode` client:
  - session initialisation, firmware check and retries
  - session restore on reconnect
  - spectra, monitoring, settings and raw access
- USB access diagnostics and the bundled udev rule.
- Optional `serde` support and a `vendored` feature for native libraries.
- Examples:
  - `discover`
  - `read_spectrum`
  - `live_monitor`
  - `decode_databuf`
- Unit tests, virtual-device integration tests, and wire-level protocol fixtures.

[Unreleased]: https://github.com/ngxyzaudax/radiacode-lib/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ngxyzaudax/radiacode-lib/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ngxyzaudax/radiacode-lib/releases/tag/v0.1.0
