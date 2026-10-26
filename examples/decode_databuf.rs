//! Decodes a recorded DataBuf response offline, without any device or transport.
//!
//! ```text
//! cargo run --example decode_databuf --no-default-features
//! cargo run --example decode_databuf --no-default-features -- path/to/frame.hex
//! ```
//!
//! The input is one complete wire response (length prefix, echoed header and
//! virtual-string body) as hex text; the default is the bundled test fixture.

use radiacode_lib::data_buf::{decode_data_buf, snapshot_from_frame, DataBufFrame, DataBufRecord};
use radiacode_lib::protocol::{decode_virt_string, BytesBuffer, ResponseAssembler, ECHO_HEADER_LEN};
use radiacode_lib::Result;

const BUNDLED_FRAME: &str = include_str!("../tests/fixtures/data_buf_session.hex");

fn main() -> Result<()> {
    let hex = std::env::args().nth(1).map_or_else(|| Ok(BUNDLED_FRAME.to_string()), std::fs::read_to_string);
    let hex = hex.map_err(|error| radiacode_lib::Error::InvalidArgument(error.to_string()))?;
    let frame = decode_data_buf(record_bytes(&parse_hex(&hex))?.data());
    print_frame(&frame);
    Ok(())
}

fn parse_hex(text: &str) -> Vec<u8> {
    let digits: Vec<u8> = text.bytes().filter(u8::is_ascii_hexdigit).collect();
    digits.chunks_exact(2).filter_map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()).collect()
}

fn record_bytes(wire: &[u8]) -> Result<BytesBuffer> {
    let payload = ResponseAssembler::new()
        .push(wire)?
        .ok_or_else(|| radiacode_lib::Error::mismatch("complete frame", "partial frame"))?;
    decode_virt_string(BytesBuffer::new(payload.get(ECHO_HEADER_LEN..).unwrap_or_default().to_vec()))
}

fn print_frame(frame: &DataBufFrame) {
    frame.records.iter().for_each(print_record);
    frame.warnings.iter().for_each(|warning| println!("warning at byte {}: {:?}", warning.offset, warning.kind));
    let snapshot = snapshot_from_frame(frame);
    if let Some(rates) = snapshot.rates {
        println!("latest: {:.2} cps, {:.3} uSv/h", rates.count_rate_cps, rates.dose_rate_rh * 10_000.0);
    }
    if let Some(rare) = snapshot.rare {
        println!("battery {:.1} %, temperature {:.2} C", rare.battery_percent, rare.temperature_c);
    }
}

fn print_record(record: &DataBufRecord) {
    let header = record.header();
    println!("seq {:3}  t={:>8.2}s  {:?}", header.seq.raw(), header.ts.as_duration().as_secs_f64(), header.kind);
}
