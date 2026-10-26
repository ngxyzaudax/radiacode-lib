use super::framing::LENGTH_PREFIX_LEN;
use crate::error::{Error, Result};

/// Reassembles a length-prefixed response from transport chunks.
///
/// USB delivers up to 256 bytes per bulk read and Bluetooth LE delivers small
/// notifications; both feed chunks here until the declared length is reached.
/// Bytes beyond the declared length in the final chunk are discarded.
#[derive(Debug, Default, Clone)]
pub struct ResponseAssembler {
    expected: Option<usize>,
    payload: Vec<u8>,
}

impl ResponseAssembler {
    /// Creates an empty assembler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` while a frame is partially assembled.
    pub fn in_progress(&self) -> bool {
        self.expected.is_some()
    }

    /// Feeds one chunk; returns the complete payload (without the length prefix)
    /// once the declared length has been received.
    pub fn push(&mut self, chunk: &[u8]) -> Result<Option<Vec<u8>>> {
        let body = match self.expected {
            Some(_) => chunk,
            None => self.start_frame(chunk)?,
        };
        self.payload.extend_from_slice(body);
        Ok(self.take_if_complete())
    }

    fn start_frame<'a>(&mut self, chunk: &'a [u8]) -> Result<&'a [u8]> {
        let (prefix, body) = chunk
            .split_at_checked(LENGTH_PREFIX_LEN)
            .ok_or(Error::BufferUnderrun { need: LENGTH_PREFIX_LEN, have: chunk.len() })?;
        self.expected = Some(declared_length(prefix)?);
        self.payload.clear();
        Ok(body)
    }

    fn take_if_complete(&mut self) -> Option<Vec<u8>> {
        let expected = self.expected?;
        if self.payload.len() < expected {
            return None;
        }
        self.expected = None;
        self.payload.truncate(expected);
        Some(std::mem::take(&mut self.payload))
    }
}

fn declared_length(prefix: &[u8]) -> Result<usize> {
    let raw = <[u8; LENGTH_PREFIX_LEN]>::try_from(prefix)
        .map(i32::from_le_bytes)
        .map_err(|_| Error::BufferUnderrun { need: LENGTH_PREFIX_LEN, have: prefix.len() })?;
    usize::try_from(raw).map_err(|_| Error::mismatch("non-negative response length", format!("length {raw}")))
}

#[cfg(test)]
mod tests {
    use super::ResponseAssembler;
    use crate::error::Error;

    fn frame(payload: &[u8]) -> Vec<u8> {
        let mut bytes = (payload.len() as i32).to_le_bytes().to_vec();
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn reassembles_chunked_payload() {
        let mut assembler = ResponseAssembler::new();
        let full = frame(&[1, 2, 3, 4, 5]);
        let (head, tail) = full.split_at(6);
        assert_eq!(assembler.push(head).unwrap(), None);
        assert!(assembler.in_progress());
        assert_eq!(assembler.push(tail).unwrap(), Some(vec![1, 2, 3, 4, 5]));
        assert!(!assembler.in_progress());
    }

    #[test]
    fn completes_single_chunk_and_resets() {
        let mut assembler = ResponseAssembler::new();
        let full = frame(&[0xAA, 0xBB]);
        assert_eq!(assembler.push(&full).unwrap(), Some(vec![0xAA, 0xBB]));
        assert_eq!(assembler.push(&full).unwrap(), Some(vec![0xAA, 0xBB]));
    }

    #[test]
    fn zero_length_frame_completes_immediately() {
        let mut assembler = ResponseAssembler::new();
        assert_eq!(assembler.push(&frame(&[])).unwrap(), Some(Vec::new()));
    }

    #[test]
    fn trailing_bytes_beyond_declared_length_are_dropped() {
        let mut assembler = ResponseAssembler::new();
        let mut full = frame(&[1, 2]);
        full.extend_from_slice(&[9, 9, 9]);
        assert_eq!(assembler.push(&full).unwrap(), Some(vec![1, 2]));
    }

    #[test]
    fn rejects_short_and_negative_headers() {
        let mut assembler = ResponseAssembler::new();
        assert!(matches!(assembler.push(&[1, 2]), Err(Error::BufferUnderrun { need: 4, have: 2 })));
        assert!(matches!(assembler.push(&(-1i32).to_le_bytes()), Err(Error::ProtocolMismatch { .. })));
    }
}
