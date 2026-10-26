use radiacode_lib::protocol::{VirtSfr, VirtString};
use radiacode_lib::{CountDisplayUnit, DisplayUnits, DoseDisplayUnit};

use super::encode::{calibration_payload, rare_record, real_time_record, spectrum_v0_payload};
use super::virtual_device::VirtualDevice;

pub const CPM_SIEVERT: DisplayUnits =
    DisplayUnits { dose: DoseDisplayUnit::MicroSievertPerHour, count: CountDisplayUnit::Cpm };

pub fn configured_alarm_device() -> VirtualDevice {
    [
        (VirtSfr::CrLev1Cp10s, 300),
        (VirtSfr::CrLev2Cp10s, 600),
        (VirtSfr::DrLev1UrH, 100),
        (VirtSfr::DrLev2UrH, 200),
        (VirtSfr::DsLev1Ur, 1000),
        (VirtSfr::DsLev2Ur, 2000),
        (VirtSfr::DsUnits, 1),
        (VirtSfr::CrUnits, 0),
        (VirtSfr::AlarmMode, 1),
        (VirtSfr::DispBrt, 12),
        (VirtSfr::DispOffTime, 3),
        (VirtSfr::DispDir, 2),
        (VirtSfr::SoundOn, 1),
        (VirtSfr::SoundCtrl, 0x1f9d),
        (VirtSfr::VibroOn, 0),
        (VirtSfr::VibroCtrl, 0x1fff),
        (VirtSfr::DeviceCtrl, 0x35),
    ]
    .into_iter()
    .fold(VirtualDevice::default(), |device, (register, value)| device.with_register(register, value))
}

pub fn spectrum_device() -> VirtualDevice {
    let payload = spectrum_v0_payload(120, [1.0, 2.0, 3.0], &[5, 6, 7]);
    VirtualDevice::default()
        .with_string(VirtString::Configuration, b"SpecFormatVersion=0\n")
        .with_string(VirtString::Spectrum, &payload)
        .with_string(VirtString::SpecAccum, &payload)
        .with_string(VirtString::SpecDiff, &payload)
        .with_string(VirtString::EnergyCalib, &calibration_payload([1.5, 2.5, 3.5]))
}

pub fn frame_with_rare() -> Vec<u8> {
    [real_time_record(1, 100, 5.0, 0.0001), rare_record(2, 200, 87.5, 24.5), real_time_record(3, 300, 6.0, 0.0002)]
        .concat()
}

pub fn device_with_frame(frame: &[u8]) -> VirtualDevice {
    VirtualDevice::default().with_string(VirtString::DataBuf, frame).with_register(VirtSfr::TempDegC, 31.0f32.to_bits())
}

pub fn sfr_file_reads(device: &VirtualDevice) -> usize {
    let sfr_file_args = u32::from(VirtString::SfrFile).to_le_bytes();
    device
        .requests_for(radiacode_lib::protocol::Command::RdVirtString)
        .into_iter()
        .filter(|request| request.args == sfr_file_args)
        .count()
}
