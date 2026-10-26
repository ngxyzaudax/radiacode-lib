//! RadiaCode wire protocol: request framing, opcodes, registers and response decoders.
//!
//! Every request is framed as `u32 length (LE)`, `u16 command (LE)`, a reserved
//! zero byte, a sequence byte in `0x80..=0x9F`, and the command arguments. The
//! device answers with a length prefix, an echo of the four header bytes, and
//! the response body. The functions in this module are pure and transport-agnostic.

mod assembler;
mod batch;
mod buffer;
pub mod catalog;
mod clock;
mod command;
mod firmware;
mod framing;
mod identity;
mod registers;
mod responses;
mod sequence;

#[cfg(test)]
mod batch_tests;
#[cfg(test)]
mod decoder_tests;

pub use assembler::ResponseAssembler;
pub use batch::{
    all_valid_flags, decode_batch_read, decode_batch_words, decode_batch_write, encode_batch_read_args,
    encode_batch_write_args, validate_batch_size, BatchReadResponse, MAX_BATCH_SIZE,
};
pub use buffer::BytesBuffer;
pub use clock::{set_time_payload, set_time_payload_now};
pub use command::{Command, VirtString, ALL_COMMANDS};
pub use firmware::{decode_fw_version, DeviceVersions, FirmwareVersion, MIN_SUPPORTED_FIRMWARE};
pub use framing::{
    build_request, framed_request_header, hex_bytes, request_header, response_matches_request, strip_echoed_header,
    EchoHeader, ECHO_HEADER_LEN, LENGTH_PREFIX_LEN,
};
pub use identity::{
    decode_fw_signature, decode_hw_serial, decode_status, decode_text, spectrum_format_version, FirmwareSignature,
};
pub use registers::{
    catalog_registers, extract_vsfr_f32, extract_vsfr_u32, resolve_value_kind, VirtSfr, VsfrValueKind, ALL_REGISTERS,
};
pub use responses::{
    decode_retcode_ok, decode_virt_string, decode_vsfr_read, decode_vsfr_read_optional, decode_vsfr_write,
    ensure_empty_payload, virt_string_read_args, virt_string_write_args, vsfr_read_args, vsfr_write_args,
};
pub use sequence::Sequence;
