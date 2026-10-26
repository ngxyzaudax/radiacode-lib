//! Single and batch VSFR register access against a virtual device.

mod support;

use radiacode_lib::protocol::{Command, VirtSfr, VirtString};
use radiacode_lib::Error;
use support::encode::{u32_words, virt_string_payload};
use support::{open_device, ScriptedReply, VirtualDevice};

const IDS: [VirtSfr; 3] = [VirtSfr::DispBrt, VirtSfr::SoundOn, VirtSfr::VibroOn];

fn populated() -> VirtualDevice {
    VirtualDevice::default()
        .with_register(VirtSfr::DispBrt, 0x0700_0000)
        .with_register(VirtSfr::SoundOn, 1)
        .with_register(VirtSfr::VibroOn, 0)
}

#[tokio::test(start_paused = true)]
async fn full_batch_read_extracts_sub_word_registers() {
    let (mut device, handle) = open_device(populated()).await;
    assert_eq!(device.read_vsfr_batch(&IDS).await.unwrap(), vec![7, 1, 0]);
    assert_eq!(handle.device().request_count(Command::RdVirtSfrBatch), 1);
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 0);
    assert_eq!(device.read_vsfr_batch_raw(&IDS).await.unwrap(), vec![0x0700_0000, 1, 0]);
}

#[tokio::test(start_paused = true)]
async fn sparse_batch_fills_missing_registers_individually() {
    let (mut device, handle) = open_device(populated()).await;
    handle.device().push_script(Command::RdVirtSfrBatch, ScriptedReply::Payload(u32_words(&[0b101, 0x0700_0000, 0])));
    assert_eq!(device.read_vsfr_batch(&IDS).await.unwrap(), vec![7, 1, 0]);
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 1);
}

#[tokio::test(start_paused = true)]
async fn empty_valid_flags_is_reported_without_fallback() {
    let (mut device, handle) = open_device(populated().without_batch()).await;
    assert!(matches!(device.read_vsfr_batch(&IDS).await, Err(Error::VsfrBatchEmpty)));
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 0);
}

#[tokio::test(start_paused = true)]
async fn transient_batch_failures_retry_then_fall_back_to_sequential_reads() {
    let (mut device, handle) = open_device(populated()).await;
    (0..3).for_each(|_| handle.device().push_script(Command::RdVirtSfrBatch, ScriptedReply::timeout()));
    assert_eq!(device.read_vsfr_batch(&IDS).await.unwrap(), vec![7, 1, 0]);
    assert_eq!(handle.device().request_count(Command::RdVirtSfrBatch), 3);
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 3);
}

#[tokio::test(start_paused = true)]
async fn connection_loss_does_not_fall_back() {
    let (mut device, handle) = open_device(populated()).await;
    handle.device().push_script(Command::RdVirtSfrBatch, ScriptedReply::connection_closed());
    assert!(device.read_vsfr_batch(&IDS).await.unwrap_err().is_connection_lost());
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 0);
}

#[tokio::test(start_paused = true)]
async fn batch_size_is_validated() {
    let (mut device, _handle) = open_device(populated()).await;
    assert!(matches!(device.read_vsfr_batch(&[]).await, Err(Error::InvalidArgument(_))));
    assert!(device.write_vsfr_batch(&[]).await.is_err());
    assert!(device.read_vsfr_batch(&[VirtSfr::DispBrt; 33]).await.is_err());
}

#[tokio::test(start_paused = true)]
async fn write_batch_stores_values_and_validates_mask() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    let pairs = [(VirtSfr::DispBrt, 4), (VirtSfr::SoundOn, 1)];
    device.write_vsfr_batch(&pairs).await.unwrap();
    assert_eq!(handle.device().register(VirtSfr::DispBrt), Some(4));
    assert_eq!(handle.device().register(VirtSfr::SoundOn), Some(1));
    (0..3).for_each(|_| handle.device().push_script(Command::WrVirtSfrBatch, ScriptedReply::Payload(u32_words(&[1]))));
    assert!(matches!(device.write_vsfr_batch(&pairs).await, Err(Error::ProtocolMismatch { .. })));
}

#[tokio::test(start_paused = true)]
async fn write_batch_retries_transient_errors() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle.device().push_script(Command::WrVirtSfrBatch, ScriptedReply::timeout());
    device.write_vsfr_batch(&[(VirtSfr::DispBrt, 2)]).await.unwrap();
    assert_eq!(handle.device().request_count(Command::WrVirtSfrBatch), 2);
    (0..3).for_each(|_| handle.device().push_script(Command::WrVirtSfrBatch, ScriptedReply::timeout()));
    assert!(matches!(device.write_vsfr_batch(&[(VirtSfr::DispBrt, 2)]).await, Err(Error::Timeout)));
}

#[tokio::test(start_paused = true)]
async fn single_register_reads_and_writes() {
    let device = populated().with_register(VirtSfr::TempDegC, 1.5f32.to_bits());
    let (mut device, handle) = open_device(device).await;
    assert_eq!(device.read_vsfr_f32(VirtSfr::TempDegC).await.unwrap(), 1.5);
    assert_eq!(device.read_vsfr_optional(VirtSfr::LedsOn).await.unwrap(), None);
    device.write_vsfr_u32(VirtSfr::DispBrt, 5).await.unwrap();
    assert_eq!(handle.device().register(VirtSfr::DispBrt), Some(5));
    handle.device().push_script(Command::WrVirtSfr, ScriptedReply::Payload(u32_words(&[0])));
    assert!(!device.write_vsfr_optional(VirtSfr::LedsOn, &1u32.to_le_bytes()).await.unwrap());
}

#[tokio::test(start_paused = true)]
async fn virt_string_reads_check_retcode_and_length() {
    let (mut device, handle) =
        open_device(VirtualDevice::default().with_string(VirtString::TextMessage, b"hello")).await;
    assert_eq!(device.read_virt_string(VirtString::TextMessage).await.unwrap().data(), b"hello");
    handle.device().push_script(Command::RdVirtString, ScriptedReply::Payload(virt_string_payload(2, b"x")));
    assert!(matches!(device.read_virt_string(VirtString::TextMessage).await, Err(Error::UnexpectedReturnCode(2))));
    let truncated = [u32_words(&[1, 10]), b"abc".to_vec()].concat();
    handle.device().push_script(Command::RdVirtString, ScriptedReply::Payload(truncated));
    assert!(matches!(device.read_virt_string(VirtString::TextMessage).await, Err(Error::BufferUnderrun { .. })));
}
