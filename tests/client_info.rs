//! Identity, spectrum, calibration and settings commands against a virtual device.

mod support;

use radiacode_lib::protocol::{Command, VirtSfr, VirtString};
use radiacode_lib::{DeviceLanguage, EnergyCalibration, Error};
use support::encode::{calibration_payload, u32_words};
use support::fixtures::spectrum_device;
use support::{open_device, ScriptedReply, VirtualDevice};

#[tokio::test(start_paused = true)]
async fn fw_version_retries_transient_decode_failures() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle.device().push_script(Command::GetVersion, ScriptedReply::Payload(vec![1, 2]));
    let versions = device.fw_version().await.unwrap();
    assert_eq!((versions.target.major, versions.target.minor), (4, 12));
    assert_eq!((versions.boot.major, versions.boot.minor), (4, 0));
    assert_eq!(handle.device().request_count(Command::GetVersion), 3);
}

#[tokio::test(start_paused = true)]
async fn fw_version_gives_up_after_three_transient_failures() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    (0..3).for_each(|_| handle.device().push_script(Command::GetVersion, ScriptedReply::Payload(vec![1])));
    assert!(device.fw_version().await.unwrap_err().is_transient());
}

#[tokio::test(start_paused = true)]
async fn identity_strings_decode() {
    let device = VirtualDevice::default().with_string(VirtString::Configuration, &[0xC0, 0xC1]);
    let (mut device, _handle) = open_device(device).await;
    assert_eq!(device.hw_serial_number().await.unwrap(), "01020304-05060708");
    assert_eq!(device.serial_number().await.unwrap(), "RC-103-001234");
    assert_eq!(device.configuration().await.unwrap(), "АБ");
}

#[tokio::test(start_paused = true)]
async fn spectra_decode_with_session_format_version() {
    let (mut device, _handle) = open_device(spectrum_device()).await;
    let spectrum = device.spectrum().await.unwrap();
    assert_eq!(spectrum.duration.as_secs(), 120);
    assert_eq!(spectrum.calibration.coefficients(), [1.0, 2.0, 3.0]);
    assert_eq!(spectrum.counts, vec![5, 6, 7]);
    assert_eq!(device.spectrum_accum().await.unwrap().counts, vec![5, 6, 7]);
    assert_eq!(device.spectrum_diff().await.unwrap().total_counts(), 18);
}

#[tokio::test(start_paused = true)]
async fn energy_calibration_read_and_write() {
    let (mut device, handle) = open_device(spectrum_device()).await;
    assert_eq!(device.energy_calibration().await.unwrap().coefficients(), [1.5, 2.5, 3.5]);
    device.set_energy_calibration(EnergyCalibration::new(0.5, 1.0, 0.0)).await.unwrap();
    let stored = handle.device().strings[&u32::from(VirtString::EnergyCalib)].clone();
    assert_eq!(stored, calibration_payload([0.5, 1.0, 0.0]));
}

#[tokio::test(start_paused = true)]
async fn set_energy_calibration_rejects_non_success_retcode() {
    let (mut device, handle) = open_device(spectrum_device()).await;
    handle.device().push_script(Command::WrVirtString, ScriptedReply::Payload(u32_words(&[0])));
    let outcome = device.set_energy_calibration(EnergyCalibration::new(0.0, 0.0, 0.0)).await;
    assert!(matches!(outcome, Err(Error::UnexpectedReturnCode(0))));
}

#[tokio::test(start_paused = true)]
async fn metadata_combines_serial_model_versions_and_calibration() {
    let (mut device, _handle) = open_device(spectrum_device()).await;
    let metadata = device.metadata().await.unwrap();
    assert_eq!(metadata.serial, "RC-103-001234");
    assert_eq!(metadata.model, "RC-103");
    assert_eq!(metadata.energy_calibration.coefficients(), [1.5, 2.5, 3.5]);
    assert_eq!(metadata.firmware_label(), "4.12 (Feb 02 2024)");
}

#[tokio::test(start_paused = true)]
async fn diagnostics_commands_decode_payloads() {
    let (mut device, _handle) =
        open_device(VirtualDevice::default().with_string(VirtString::TextMessage, b"msg")).await;
    assert_eq!(device.status().await.unwrap(), 0x1234);
    let signature = device.fw_signature().await.unwrap();
    assert_eq!((signature.signature, signature.filename.as_str()), (0xCAFE, "rc-103.fw"));
    assert_eq!(signature.id_string, "RadiaCode");
    assert_eq!(device.text_message().await.unwrap(), "msg");
    assert!(device.sfr_file_text().await.unwrap().contains("VSFR_DISP_BRT"));
}

#[tokio::test(start_paused = true)]
async fn settings_write_expected_registers() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    device.set_sound_on(true).await.unwrap();
    device.set_vibro_on(false).await.unwrap();
    device.set_display_brightness(9).await.unwrap();
    device.set_language(DeviceLanguage::English).await.unwrap();
    device.set_device_on(false).await.unwrap();
    device.dose_reset().await.unwrap();
    let state = handle.device();
    assert_eq!(state.register(VirtSfr::SoundOn), Some(1));
    assert_eq!(state.register(VirtSfr::VibroOn), Some(0));
    assert_eq!(state.register(VirtSfr::DispBrt), Some(9));
    assert_eq!(state.register(VirtSfr::DeviceLang), Some(1));
    assert_eq!(state.register(VirtSfr::DeviceOn), Some(0));
    assert_eq!(state.register(VirtSfr::DoseReset), Some(0));
}

#[tokio::test(start_paused = true)]
async fn brightness_above_nine_is_rejected_locally() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    let writes = handle.device().request_count(Command::WrVirtSfr);
    assert!(matches!(device.set_display_brightness(10).await, Err(Error::InvalidArgument(_))));
    assert_eq!(handle.device().request_count(Command::WrVirtSfr), writes);
}

#[tokio::test(start_paused = true)]
async fn spectrum_reset_writes_empty_spectrum_string() {
    let (mut device, handle) = open_device(spectrum_device()).await;
    device.spectrum_reset().await.unwrap();
    let first_write = handle.device().requests_for(Command::WrVirtString)[0].args.clone();
    assert_eq!(first_write, u32_words(&[u32::from(VirtString::Spectrum), 0]));
    handle.device().push_script(Command::WrVirtString, ScriptedReply::Payload(u32_words(&[2])));
    assert!(device.spectrum_reset().await.is_err());
}
