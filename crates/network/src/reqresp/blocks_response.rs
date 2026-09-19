//! Length-prefixed blocks-by-root response codec (scaffold).

use ethean_network_wire::limits::MAX_BLOCKS_PER_REQUEST;

use crate::error::{NetworkError, Result};

/// Encode `count || (len || bytes)*` for a blocks-by-root response body.
pub fn encode_blocks_by_root_response(blocks: &[Vec<u8>]) -> Result<Vec<u8>> {
    if blocks.len() as u64 > MAX_BLOCKS_PER_REQUEST {
        return Err(NetworkError::Handshake(format!(
            "blocks {} exceeds {}",
            blocks.len(),
            MAX_BLOCKS_PER_REQUEST
        )));
    }
    let mut out = Vec::with_capacity(4 + blocks.iter().map(|b| 4 + b.len()).sum::<usize>());
    out.extend_from_slice(&(blocks.len() as u32).to_le_bytes());
    for block in blocks {
        out.extend_from_slice(&(block.len() as u32).to_le_bytes());
        out.extend_from_slice(block);
    }
    Ok(out)
}

/// Decode a length-prefixed blocks-by-root response into raw SSZ blobs.
pub fn decode_blocks_by_root_response(input: &[u8]) -> Result<Vec<Vec<u8>>> {
    if input.len() < 4 {
        return Err(NetworkError::Handshake(
            "blocks-by-root response too short".into(),
        ));
    }
    let n = u32::from_le_bytes(input[0..4].try_into().unwrap_or([0; 4])) as usize;
    if n as u64 > MAX_BLOCKS_PER_REQUEST {
        return Err(NetworkError::Handshake(format!(
            "blocks {n} exceeds {MAX_BLOCKS_PER_REQUEST}"
        )));
    }
    let mut off = 4;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        if off + 4 > input.len() {
            return Err(NetworkError::Handshake(
                "truncated blocks-by-root length prefix".into(),
            ));
        }
        let len = u32::from_le_bytes(input[off..off + 4].try_into().unwrap_or([0; 4])) as usize;
        off += 4;
        if off + len > input.len() {
            return Err(NetworkError::Handshake(
                "truncated blocks-by-root block bytes".into(),
            ));
        }
        out.push(input[off..off + len].to_vec());
        off += len;
    }
    if off != input.len() {
        return Err(NetworkError::Handshake(
            "trailing bytes after blocks-by-root response".into(),
        ));
    }
    Ok(out)
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
        assert_eq!(decode_blocks_by_root_response(&enc).unwrap(), Vec::<Vec<u8>>::new());
    }
}
