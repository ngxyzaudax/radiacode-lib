use tracing::debug;

use super::RadiaCode;
use crate::error::{Error, Result};
use crate::protocol::{
    build_request, decode_virt_string, decode_vsfr_read, decode_vsfr_read_optional, decode_vsfr_write, request_header,
    strip_echoed_header, virt_string_read_args, vsfr_read_args, vsfr_write_args, BytesBuffer, Command, VirtSfr,
    VirtString,
};

impl RadiaCode {
    /// Sends one command and returns the response body after the echoed header.
    pub async fn execute_raw(&mut self, command: Command, args: &[u8]) -> Result<BytesBuffer> {
        let seq = self.sequence.advance();
        debug!(?command, seq, args_len = args.len(), "execute command");
        let request = build_request(command, seq, args);
        let response = self.transport.execute(&request).await?;
        strip_echoed_header(response, request_header(command, seq))
    }

    /// Reads a virtual string and returns its payload.
    pub async fn read_virt_string(&mut self, id: VirtString) -> Result<BytesBuffer> {
        let response = self.execute_raw(Command::RdVirtString, &virt_string_read_args(id)).await?;
        decode_virt_string(response)
    }

    /// Reads the raw 32-bit word of a register.
    pub async fn read_vsfr_u32(&mut self, id: VirtSfr) -> Result<u32> {
        let response = self.execute_raw(Command::RdVirtSfr, &vsfr_read_args(id)).await?;
        decode_vsfr_read(response)
    }

    /// Reads a register, returning `None` when the firmware does not implement it.
    pub async fn read_vsfr_optional(&mut self, id: VirtSfr) -> Result<Option<u32>> {
        let response = self.execute_raw(Command::RdVirtSfr, &vsfr_read_args(id)).await?;
        decode_vsfr_read_optional(response)
    }

    /// Reads a register as an IEEE-754 float.
    pub async fn read_vsfr_f32(&mut self, id: VirtSfr) -> Result<f32> {
        self.read_vsfr_u32(id).await.map(f32::from_bits)
    }

    /// Writes raw bytes to a register; returns `false` if the device rejected the write.
    pub async fn write_vsfr_optional(&mut self, id: VirtSfr, data: &[u8]) -> Result<bool> {
        let response = self.execute_raw(Command::WrVirtSfr, &vsfr_write_args(id, data)).await?;
        decode_vsfr_write(response)
    }

    /// Writes raw bytes to a register, failing if the device rejected the write.
    pub async fn write_vsfr(&mut self, id: VirtSfr, data: &[u8]) -> Result<()> {
        match self.write_vsfr_optional(id, data).await? {
            true => Ok(()),
            false => Err(Error::UnexpectedReturnCode(0)),
        }
    }

    /// Writes a little-endian `u32` to a register.
    pub async fn write_vsfr_u32(&mut self, id: VirtSfr, value: u32) -> Result<()> {
        self.write_vsfr(id, &value.to_le_bytes()).await
    }
}
