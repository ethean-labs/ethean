//! SSZ codec primitives for the XMSS containers (offsets are validated, field
//! elements must be canonical).

use super::merkle::{HashSubTree, HashTreeLayer, HashTreeOpening};
use super::tweak_hash::Digest;
use crate::error::{CryptoError, Result};
use crate::field::Fp;

pub const DIGEST_BYTES: usize = 32;

pub fn write_fps(buf: &mut Vec<u8>, elements: &[Fp]) {
    for fe in elements {
        buf.extend_from_slice(&fe.to_le_bytes());
    }
}

pub fn read_fps<const N: usize>(bytes: &[u8]) -> Result<[Fp; N]> {
    if bytes.len() != N * 4 {
        return Err(CryptoError::MalformedEncoding("field vector length"));
    }
    let mut out = [Fp::ZERO; N];
    for (fe, chunk) in out.iter_mut().zip(bytes.as_chunks::<4>().0) {
        *fe = Fp::from_le_bytes(*chunk)?;
    }
    Ok(out)
}

pub fn read_digests(bytes: &[u8]) -> Result<Vec<Digest>> {
    if !bytes.len().is_multiple_of(DIGEST_BYTES) {
        return Err(CryptoError::MalformedEncoding("digest list length"));
    }
    bytes
        .as_chunks::<DIGEST_BYTES>()
        .0
        .iter()
        .map(|chunk| read_fps::<8>(chunk))
        .collect()
}

pub fn write_digests(buf: &mut Vec<u8>, digests: &[Digest]) {
    for d in digests {
        write_fps(buf, d);
    }
}

pub fn read_u32(bytes: &[u8], at: usize) -> Result<u32> {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")))
        .ok_or(CryptoError::MalformedEncoding("truncated u32"))
}

pub fn read_u64(bytes: &[u8], at: usize) -> Result<u64> {
    bytes
        .get(at..at + 8)
        .map(|b| u64::from_le_bytes(b.try_into().expect("8 bytes")))
        .ok_or(CryptoError::MalformedEncoding("truncated u64"))
}

fn expect_offset(bytes: &[u8], at: usize, expected: usize) -> Result<()> {
    if read_u32(bytes, at)? as usize != expected {
        return Err(CryptoError::MalformedEncoding("unexpected SSZ offset"));
    }
    Ok(())
}

impl HashTreeOpening {
    pub fn ssz_len(&self) -> usize {
        4 + self.siblings.len() * DIGEST_BYTES
    }

    pub fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&4u32.to_le_bytes());
        write_digests(buf, &self.siblings);
    }

    pub fn from_ssz(bytes: &[u8]) -> Result<Self> {
        expect_offset(bytes, 0, 4)?;
        Ok(Self {
            siblings: read_digests(&bytes[4..])?,
        })
    }
}

impl HashTreeLayer {
    pub fn ssz_len(&self) -> usize {
        12 + self.nodes.len() * DIGEST_BYTES
    }

    pub fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.start_index.to_le_bytes());
        buf.extend_from_slice(&12u32.to_le_bytes());
        write_digests(buf, &self.nodes);
    }

    pub fn from_ssz(bytes: &[u8]) -> Result<Self> {
        let start_index = read_u64(bytes, 0)?;
        expect_offset(bytes, 8, 12)?;
        Ok(Self {
            start_index,
            nodes: read_digests(&bytes[12..])?,
        })
    }
}

impl HashSubTree {
    pub fn ssz_len(&self) -> usize {
        20 + self.layers.iter().map(|l| 4 + l.ssz_len()).sum::<usize>()
    }

    pub fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.depth.to_le_bytes());
        buf.extend_from_slice(&self.lowest_layer.to_le_bytes());
        buf.extend_from_slice(&20u32.to_le_bytes());
        let mut offset = 4 * self.layers.len();
        for layer in &self.layers {
            buf.extend_from_slice(&(offset as u32).to_le_bytes());
            offset += layer.ssz_len();
        }
        for layer in &self.layers {
            layer.ssz_append(buf);
        }
    }

    pub fn from_ssz(bytes: &[u8]) -> Result<Self> {
        let depth = read_u64(bytes, 0)?;
        let lowest_layer = read_u64(bytes, 8)?;
        expect_offset(bytes, 16, 20)?;
        let list = &bytes[20..];
        let layers = decode_variable_list(list, HashTreeLayer::from_ssz)?;
        if layers.is_empty() || lowest_layer >= depth || depth > 32 {
            return Err(CryptoError::MalformedEncoding("subtree shape"));
        }
        Ok(Self {
            depth,
            lowest_layer,
            layers,
        })
    }
}

/// Decode an SSZ list of variable-size items (offset table then items).
pub fn decode_variable_list<T>(bytes: &[u8], item: fn(&[u8]) -> Result<T>) -> Result<Vec<T>> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let first = read_u32(bytes, 0)? as usize;
    if !first.is_multiple_of(4) || first == 0 || first > bytes.len() {
        return Err(CryptoError::MalformedEncoding("list offset table"));
    }
    let count = first / 4;
    let mut offsets = Vec::with_capacity(count + 1);
    for i in 0..count {
        offsets.push(read_u32(bytes, i * 4)? as usize);
    }
    offsets.push(bytes.len());
    let mut items = Vec::with_capacity(count);
    for w in offsets.windows(2) {
        if w[0] > w[1] || w[1] > bytes.len() {
            return Err(CryptoError::MalformedEncoding("list item offsets"));
        }
        items.push(item(&bytes[w[0]..w[1]])?);
    }
    Ok(items)
}
