//! String-error wrappers over `ethean-network-wire` plus base64url for ENR bytes.

use crate::driver::decode_hex_bytes;
use ethean_network_wire::{ResponseChunk, ResponseCode};
use serde_json::Value;

pub type R<T> = Result<T, String>;

pub fn compress_raw(b: &[u8]) -> R<Vec<u8>> {
    ethean_network_wire::compress_raw(b).map_err(|e| e.to_string())
}
pub fn decompress_raw(b: &[u8]) -> R<Vec<u8>> {
    ethean_network_wire::decompress_raw(b).map_err(|e| e.to_string())
}
pub fn compress_frame(b: &[u8]) -> R<Vec<u8>> {
    ethean_network_wire::compress_frame(b).map_err(|e| e.to_string())
}
pub fn decompress_frame(b: &[u8]) -> R<Vec<u8>> {
    ethean_network_wire::decompress_frame(b).map_err(|e| e.to_string())
}
pub fn decode_varint(b: &[u8], at: usize) -> R<(u64, usize)> {
    ethean_network_wire::decode_varint(b, at).map_err(|e| e.to_string())
}
pub fn encode_request(b: &[u8]) -> R<Vec<u8>> {
    ethean_network_wire::encode_request(b).map_err(|e| e.to_string())
}
pub fn decode_request(b: &[u8]) -> R<Vec<u8>> {
    ethean_network_wire::decode_request(b).map_err(|e| e.to_string())
}
pub fn encode_response(c: ResponseCode, b: &[u8]) -> R<Vec<u8>> {
    ethean_network_wire::encode_response(c, b).map_err(|e| e.to_string())
}
pub fn decode_response(b: &[u8]) -> R<(ResponseCode, Vec<u8>)> {
    ethean_network_wire::decode_response(b).map_err(|e| e.to_string())
}
pub fn decode_response_stream(b: &[u8]) -> R<Vec<ResponseChunk>> {
    ethean_network_wire::decode_response_stream(b).map_err(|e| e.to_string())
}

/// Result of one networking vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetOutcome {
    Checked,
    Rejected,
    Unsupported(String),
}

pub fn hex_field(v: &Value, key: &str) -> Result<Vec<u8>, String> {
    decode_hex_bytes(
        v.get(key)
            .and_then(Value::as_str)
            .ok_or(format!("missing {key}"))?,
    )
}

pub fn same(label: &str, got: &[u8], want: &[u8]) -> Result<(), String> {
    if got == want {
        Ok(())
    } else {
        Err(format!(
            "{label}: {} bytes differ from the vector ({} bytes)",
            got.len(),
            want.len()
        ))
    }
}

pub fn base64url(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(T[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(T[n as usize & 63] as char);
        }
    }
    out
}
