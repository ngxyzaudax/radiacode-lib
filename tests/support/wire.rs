use radiacode_lib::protocol::{
    decode_virt_string, request_header, strip_echoed_header, BytesBuffer, Command, ResponseAssembler,
};

pub const CHUNK_SIZES: [usize; 3] = [20, 256, usize::MAX];

pub fn parse_hex(text: &str) -> Vec<u8> {
    let digits: Vec<u8> = text.bytes().filter(u8::is_ascii_hexdigit).collect();
    digits.chunks(2).map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()).collect()
}

pub fn assemble(frame: &[u8], chunk_size: usize) -> Vec<u8> {
    let mut assembler = ResponseAssembler::new();
    let completed: Vec<Vec<u8>> =
        frame.chunks(chunk_size.min(frame.len())).filter_map(|chunk| assembler.push(chunk).unwrap()).collect();
    assert_eq!(completed.len(), 1, "exactly one frame per fixture");
    completed.into_iter().next().unwrap()
}

pub fn response_body(fixture: &str, command: Command, seq: u8, chunk_size: usize) -> BytesBuffer {
    let payload = assemble(&parse_hex(fixture), chunk_size);
    strip_echoed_header(BytesBuffer::new(payload), request_header(command, seq)).unwrap()
}

pub fn virt_string(fixture: &str, seq: u8, chunk_size: usize) -> BytesBuffer {
    decode_virt_string(response_body(fixture, Command::RdVirtString, seq, chunk_size)).unwrap()
}
