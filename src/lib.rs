//! Protocol, transports and data parsing for RadiaCode gamma spectrometers and dosimeters.
//!
//! `radiacode-lib` talks to RadiaCode RC-101, RC-102, RC-103 and RC-110 devices
//! (firmware 4.8 or newer) over USB and Bluetooth Low Energy, and decodes what
//! they report: gamma spectra, dose and count rates, accumulated dose, battery,
//! temperature, alarms and device settings.
//!
//! # Layers
//!
//! * [`protocol`] is the pure wire protocol: request framing, opcodes,
//!   VSFR registers, response decoders and the device's own catalog files.
//! * [`data_buf`], [`spectrum`], [`readings`], [`config`] and [`units`] turn
//!   response payloads into typed values. They never perform I/O.
//! * [`transport`] defines the [`Transport`] trait; the `usb` and `bluetooth`
//!   modules implement it behind the `usb` and `bluetooth` crate features.
//! * [`discovery`] scans the enabled transports and opens endpoints.
//! * [`client`] provides [`RadiaCode`], an asynchronous session that combines all of the above.
//!
//! # Example
//!
//! ```no_run
//! use radiacode_lib::{discover_devices, DiscoveryOptions, OpenOptions};
//!
//! # async fn run() -> radiacode_lib::Result<()> {
//! let devices = discover_devices(&DiscoveryOptions::default()).await?;
//! if let Some(device) = devices.first() {
//!     let mut session = radiacode_lib::connect(&device.endpoint, OpenOptions::default()).await?;
//!     let spectrum = session.spectrum().await?;
//!     println!("{} counts over {:?}", spectrum.total_counts(), spectrum.duration);
//!     session.disconnect().await?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Origin and disclaimer
//!
//! This crate is extracted from the device layers of
//! [radiacode-toolkit](https://github.com/ngxyzaudax/radiacode-toolkit) by
//! Nikola Galiot and is distributed under the same AGPL-3.0 license. The
//! protocol is reverse engineered; this project is not affiliated with,
//! endorsed by or supported by the manufacturer of RadiaCode devices.

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod client;
pub mod config;
pub mod data_buf;
pub mod discovery;
pub mod error;
pub mod protocol;
pub mod readings;
pub mod spectrum;
pub mod transport;
pub mod units;

#[cfg(feature = "bluetooth")]
#[cfg_attr(docsrs, doc(cfg(feature = "bluetooth")))]
pub mod bluetooth;
#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
pub mod usb;

pub use client::{DeviceLanguage, DeviceMetadata, OpenOptions, RadiaCode, SessionRestore};
pub use discovery::{
    connect, discover_devices, open_transport, DeviceEndpoint, DiscoveredDevice, DiscoveryOptions, TransportKind,
};
pub use error::{Error, Result};
pub use readings::{AccumulatedDose, DeviceStatus, LiveRates, MonitorPollSample, TimedRates};
pub use spectrum::{EnergyCalibration, Spectrum, SpectrumFormat};
pub use transport::{BoxedTransport, Transport};
pub use units::{CountDisplayUnit, DisplayUnits, DoseDisplayUnit};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
