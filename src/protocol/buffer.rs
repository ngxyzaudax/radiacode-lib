use crate::error::{Error, Result};

/// Little-endian read cursor over an owned response payload.
///
/// Every read is bounds-checked and returns [`Error::BufferUnderrun`] instead of panicking.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BytesBuffer {
    data: Vec<u8>,
    pos: usize,
}

impl BytesBuffer {
    /// Wraps `data` with the cursor positioned at the first byte.
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    /// Number of unread bytes.
    pub fn size(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Returns `true` when every byte has been consumed.
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Unread bytes, without advancing the cursor.
    pub fn data(&self) -> &[u8] {
        self.data.get(self.pos..).unwrap_or(&[])
    }

    /// Reads `count` bytes and advances the cursor.
    pub fn take_bytes(&mut self, count: usize) -> Result<&[u8]> {
        let have = self.size();
        let underrun = Error::BufferUnderrun { need: count, have };
        let start = self.pos;
        let end = start.checked_add(count).filter(|_| count <= have);
        let end = end.ok_or(underrun)?;
        self.pos = end;
        self.data.get(start..end).ok_or(Error::BufferUnderrun { need: count, have })
    }

    /// Reads a fixed-size array and advances the cursor.
    pub fn take_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let have = self.size();
        self.take_bytes(N)
            .and_then(|bytes| <[u8; N]>::try_from(bytes).map_err(|_| Error::BufferUnderrun { need: N, have }))
    }

    /// Reads one unsigned byte.
    pub fn take_u8(&mut self) -> Result<u8> {
        self.take_array::<1>().map(u8::from_le_bytes)
    }

    /// Reads one signed byte.
    pub fn take_i8(&mut self) -> Result<i8> {
        self.take_array::<1>().map(i8::from_le_bytes)
    }

    /// Reads a little-endian `u16`.
    pub fn take_u16_le(&mut self) -> Result<u16> {
        self.take_array().map(u16::from_le_bytes)
    }

    /// Reads a little-endian `i16`.
    pub fn take_i16_le(&mut self) -> Result<i16> {
        self.take_array().map(i16::from_le_bytes)
    }

    /// Reads a little-endian `u32`.
    pub fn take_u32_le(&mut self) -> Result<u32> {
        self.take_array().map(u32::from_le_bytes)
    }

    /// Reads a little-endian `i32`.
    pub fn take_i32_le(&mut self) -> Result<i32> {
        self.take_array().map(i32::from_le_bytes)
    }

    /// Reads a little-endian IEEE-754 `f32`.
    pub fn take_f32_le(&mut self) -> Result<f32> {
        self.take_array().map(f32::from_le_bytes)
    }

    /// Reads a `u32` length word and returns it as `usize`.
    pub fn take_len_u32(&mut self) -> Result<usize> {
        self.take_u32_le().map(|len| usize::try_from(len).unwrap_or(usize::MAX))
    }

    /// Reads a string prefixed by a one-byte length, decoding it lossily as UTF-8.
    pub fn take_length_prefixed_ascii(&mut self) -> Result<String> {
        let len = usize::from(self.take_u8()?);
        self.take_bytes(len).map(|bytes| String::from_utf8_lossy(bytes).into_owned())
    }

    /// Advances the cursor by `count` bytes.
    pub fn skip(&mut self, count: usize) -> Result<()> {
        self.take_bytes(count).map(|_| ())
    }

    /// Returns a new buffer holding at most the first `len` unread bytes.
    pub fn truncated(&self, len: usize) -> Self {
        Self::new(self.data().iter().take(len).copied().collect())
    }

    /// Consumes the buffer and returns the unread bytes.
    pub fn into_remaining(self) -> Vec<u8> {
        self.data().to_vec()
    }
}

impl From<Vec<u8>> for BytesBuffer {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<&[u8]> for BytesBuffer {
    fn from(data: &[u8]) -> Self {
        Self::new(data.to_vec())
    }
}

#[cfg(test)]
#[path = "buffer_tests.rs"]
mod tests;
