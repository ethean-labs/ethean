//! Spec req/resp block response: one SignedBlock per SUCCESS chunk, read to EOF.

use ethean_network_wire::limits::MAX_BLOCKS_PER_REQUEST;
use ethean_network_wire::{decode_response_stream, encode_response, encode_response_stream, ResponseCode};

use crate::error::{NetworkError, Result};

/// Encode each SignedBlock SSZ blob as its own SUCCESS response chunk.
pub fn encode_blocks_by_root_response(blocks: &[Vec<u8>]) -> Result<Vec<u8>> {
    if blocks.len() as u64 > MAX_BLOCKS_PER_REQUEST {
        return Err(NetworkError::Handshake(format!(
            "blocks {} exceeds {}",
            blocks.len(),
            MAX_BLOCKS_PER_REQUEST
        )));
    }
    let chunks: Vec<(ResponseCode, &[u8])> = blocks
        .iter()
        .map(|b| (ResponseCode::Success, b.as_slice()))
        .collect();
    encode_response_stream(&chunks).map_err(|e| NetworkError::Handshake(e.to_string()))
}

/// Decode a concatenated response stream into SUCCESS payloads (SignedBlock SSZ).
///
/// Error chunks (INVALID_REQUEST / SERVER_ERROR / RESOURCE_UNAVAILABLE) stop
/// the iterator; payloads already collected are returned.
pub fn decode_blocks_by_root_response(input: &[u8]) -> Result<Vec<Vec<u8>>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    let chunks =
        decode_response_stream(input).map_err(|e| NetworkError::Handshake(e.to_string()))?;
    if chunks.len() as u64 > MAX_BLOCKS_PER_REQUEST {
        return Err(NetworkError::Handshake(format!(
            "blocks {} exceeds {MAX_BLOCKS_PER_REQUEST}",
            chunks.len()
        )));
    }
    let mut out = Vec::new();
    for chunk in chunks {
        match chunk.code {
            ResponseCode::Success => out.push(chunk.payload),
            _ => break,
        }
    }
    Ok(out)
}

/// Encode a single error chunk.
#[allow(dead_code)]
pub fn encode_error_chunk(code: ResponseCode, message: &str) -> Result<Vec<u8>> {
    encode_response(code, message.as_bytes()).map_err(|e| NetworkError::Handshake(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_two_blocks() {
        let blocks = vec![vec![1, 2, 3], vec![9; 8]];
        let enc = encode_blocks_by_root_response(&blocks).unwrap();
        let dec = decode_blocks_by_root_response(&enc).unwrap();
        assert_eq!(dec, blocks);
    }

    #[test]
    fn empty_list_ok() {
        let enc = encode_blocks_by_root_response(&[]).unwrap();
        assert_eq!(
            decode_blocks_by_root_response(&enc).unwrap(),
            Vec::<Vec<u8>>::new()
        );
    }

    #[test]
    fn success_then_unavailable_keeps_delivered() {
        let mut stream = encode_blocks_by_root_response(&[vec![0x11; 16]]).unwrap();
        stream.extend_from_slice(
            &encode_error_chunk(ResponseCode::ResourceUnavailable, "not found").unwrap(),
        );
        let dec = decode_blocks_by_root_response(&stream).unwrap();
        assert_eq!(dec, vec![vec![0x11; 16]]);
    }
}
