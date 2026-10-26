use std::fmt::Write;

use super::buffer::BytesBuffer;
use super::command::Command;
use crate::error::{Error, Result};

/// Length of the command header that the device echoes back in every response.
pub const ECHO_HEADER_LEN: usize = 4;

/// Length of the little-endian length prefix in front of every frame.
pub const LENGTH_PREFIX_LEN: usize = 4;

/// The four header bytes (`command`, reserved `0`, `sequence`) echoed by the device.
pub type EchoHeader = [u8; ECHO_HEADER_LEN];

/// Builds the echo header for a command and sequence number.
pub fn request_header(command: Command, seq: u8) -> EchoHeader {
    let [low, high] = u16::from(command).to_le_bytes();
    [low, high, 0, seq]
}

/// Frames a request: `u32 length`, echo header, then `args`.
pub fn build_request(command: Command, seq: u8, args: &[u8]) -> Vec<u8> {
    let header = request_header(command, seq);
    let body_len = header.len().saturating_add(args.len());
    let length = u32::try_from(body_len).unwrap_or(u32::MAX);
    length.to_le_bytes().into_iter().chain(header).chain(args.iter().copied()).collect()
}

/// Extracts the echo header from a framed request built by [`build_request`].
pub fn framed_request_header(request: &[u8]) -> Result<EchoHeader> {
    request
        .get(LENGTH_PREFIX_LEN..LENGTH_PREFIX_LEN + ECHO_HEADER_LEN)
        .and_then(|bytes| EchoHeader::try_from(bytes).ok())
        .ok_or_else(|| Error::mismatch("framed request", format!("{} bytes", request.len())))
}

/// Returns `true` when an assembled response starts with the expected echo header.
pub fn response_matches_request(payload: &[u8], expected: EchoHeader) -> bool {
    payload.get(..ECHO_HEADER_LEN) == Some(&expected[..])
}

/// Verifies and removes the echo header, returning the response body.
pub fn strip_echoed_header(mut response: BytesBuffer, expected: EchoHeader) -> Result<BytesBuffer> {
    let header = response.take_bytes(ECHO_HEADER_LEN)?;
    if header != expected {
        return Err(Error::mismatch(hex_bytes(&expected), hex_bytes(header)));
    }
    Ok(BytesBuffer::new(response.into_remaining()))
}

/// Formats bytes as contiguous lowercase hex.
pub fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), append_hex_byte)
}

fn append_hex_byte(mut text: String, byte: &u8) -> String {
    let _ = write!(text, "{byte:02x}");
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_frames_length_and_header() {
        let request = build_request(Command::GetVersion, 0x81, &[7, 7]);
        assert_eq!(request, vec![6, 0, 0, 0, 0x0a, 0x00, 0x00, 0x81, 7, 7]);
        assert_eq!(framed_request_header(&request).unwrap(), request_header(Command::GetVersion, 0x81));
    }

    #[test]
    fn framed_request_header_rejects_short_requests() {
        assert!(matches!(framed_request_header(&[1, 2, 3]), Err(Error::ProtocolMismatch { .. })));
    }

    #[test]
    fn response_match_requires_echoed_header() {
        let header = request_header(Command::GetVersion, 0x81);
        let payload = [&header[..], &[1, 2, 3]].concat();
        assert!(response_matches_request(&payload, header));
        assert!(!response_matches_request(&[0, 0, 0, 0], header));
        assert!(!response_matches_request(&[0x0a], header));
    }

    #[test]
    fn strip_echoed_header_returns_body_or_mismatch() {
        let header = request_header(Command::GetStatus, 0x85);
        let response = BytesBuffer::new([&header[..], &[9, 8]].concat());
        assert_eq!(strip_echoed_header(response, header).unwrap().data(), &[9, 8]);
        let wrong = strip_echoed_header(BytesBuffer::new(vec![0; 4]), header);
        assert!(matches!(wrong, Err(Error::ProtocolMismatch { .. })));
        let short = strip_echoed_header(BytesBuffer::new(vec![0]), header);
        assert!(matches!(short, Err(Error::BufferUnderrun { need: 4, have: 1 })));
    }

    #[test]
    fn hex_bytes_is_lowercase_and_contiguous() {
        assert_eq!(hex_bytes(&[0x0a, 0xff, 0x00]), "0aff00");
    }
}
