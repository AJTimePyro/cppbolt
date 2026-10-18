//! Section framing and length-prefixed strings.

use super::leb128::write_u32;

pub fn write_section(out: &mut Vec<u8>, id: u8, content: &[u8]) {
    out.push(id);
    write_u32(out, content.len() as u32);
    out.extend_from_slice(content);
}

pub fn write_string(out: &mut Vec<u8>, text: &str) {
    let bytes = text.as_bytes();
    write_u32(out, bytes.len() as u32);
    out.extend_from_slice(bytes);
}
