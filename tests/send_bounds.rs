//! Compile-time and runtime checks that client futures and public types are thread-safe.

mod support;

use radiacode_lib::data_buf::DataBufCursor;
use radiacode_lib::{BoxedTransport, DisplayUnits, OpenOptions, RadiaCode};
use support::fixtures::{device_with_frame, frame_with_rare};
use support::DeviceHandle;

fn assert_send<T: Send>(value: T) -> T {
    value
}

#[tokio::test(start_paused = true)]
async fn client_futures_are_send_and_run_on_spawned_tasks() {
    let handle = DeviceHandle::new(device_with_frame(&frame_with_rare()));
    let transport: BoxedTransport = handle.transport(None);
    let session = tokio::spawn(assert_send(RadiaCode::open(transport, OpenOptions::default()))).await.unwrap();
    let mut device = session.unwrap();
    let polled = tokio::spawn(async move {
        let mut cursor = DataBufCursor::default();
        let sample = device.poll_monitor(DisplayUnits::default(), &mut cursor, false).await;
        (device, sample)
    });
    let (device, sample) = polled.await.unwrap();
    assert_eq!(sample.unwrap().0.rates.len(), 2);
    assert_send(device.disconnect()).await.unwrap();
}

#[test]
fn public_types_are_send_and_sync() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<radiacode_lib::Error>();
    send_sync::<radiacode_lib::Spectrum>();
    send_sync::<radiacode_lib::DeviceEndpoint>();
    send_sync::<radiacode_lib::protocol::ResponseAssembler>();
    send_sync::<radiacode_lib::data_buf::DataBufFrame>();
}
