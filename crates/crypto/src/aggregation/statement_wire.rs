//! Canonical wire encoding for aggregate statements (prover IPC / FFI).
//!
//! Matches the byte layout hashed by [`AggregateStatement::digest`].

use crate::aggregation::statement::{
    AggregateStatement, ParticipantSet, ProofKind, Type2ComponentRef,
};
use crate::error::{CryptoError, Result};

impl AggregateStatement {
    /// Encode public inputs for leanVM prove/verify IPC (little-endian lengths).
    pub fn encode_wire(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(128 + self.participants.as_slice().len() * 4);
        buf.push(match self.kind {
            ProofKind::Type1 => 1,
            ProofKind::Type2 => 2,
        });
        buf.extend_from_slice(&self.profile_digest);
        buf.extend_from_slice(&self.message_root);
        buf.extend_from_slice(&self.slot.to_le_bytes());
        buf.extend_from_slice(&(self.participants.as_slice().len() as u32).to_le_bytes());
        for idx in self.participants.as_slice() {
            buf.extend_from_slice(&idx.to_le_bytes());
        }
        buf.extend_from_slice(&(self.components.len() as u32).to_le_bytes());
        for c in &self.components {
            buf.extend_from_slice(&c.message_root);
            buf.extend_from_slice(&c.slot.to_le_bytes());
        }
        buf
    }

    /// Decode wire bytes produced by [`Self::encode_wire`].
    pub fn decode_wire(bytes: &[u8]) -> Result<Self> {
        let mut i = 0usize;
        let kind = match *bytes.get(i).ok_or_else(|| short())? {
            1 => ProofKind::Type1,
            2 => ProofKind::Type2,
            other => {
                return Err(CryptoError::InvalidAggregate(format!(
                    "unknown proof kind tag {other}"
                )))
            }
        };
        i += 1;
        let profile_digest = read_array32(bytes, &mut i)?;
        let message_root = read_array32(bytes, &mut i)?;
        let slot = read_u64(bytes, &mut i)?;
        let n_part = read_u32(bytes, &mut i)? as usize;
        let mut parts = Vec::with_capacity(n_part);
        for _ in 0..n_part {
            parts.push(read_u32(bytes, &mut i)?);
        }
        let n_comp = read_u32(bytes, &mut i)? as usize;
        let mut components = Vec::with_capacity(n_comp);
        for _ in 0..n_comp {
            components.push(Type2ComponentRef {
                message_root: read_array32(bytes, &mut i)?,
                slot: read_u64(bytes, &mut i)?,
            });
        }
        if i != bytes.len() {
            return Err(CryptoError::InvalidAggregate(
                "trailing bytes after statement wire".into(),
            ));
        }
        let statement = Self {
            kind,
            profile_digest,
            message_root,
            slot,
            participants: ParticipantSet::try_from_ordered(parts)?,
            components,
        };
        statement.validate_shape()?;
        Ok(statement)
    }
}

fn short() -> CryptoError {
    CryptoError::InvalidAggregate("truncated statement wire".into())
}

fn read_array32(bytes: &[u8], i: &mut usize) -> Result<[u8; 32]> {
    let end = i.saturating_add(32);
    let slice = bytes.get(*i..end).ok_or_else(short)?;
    let mut out = [0u8; 32];
    out.copy_from_slice(slice);
    *i = end;
    Ok(out)
}

fn read_u32(bytes: &[u8], i: &mut usize) -> Result<u32> {
    let end = i.saturating_add(4);
    let slice = bytes.get(*i..end).ok_or_else(short)?;
    let mut arr = [0u8; 4];
    arr.copy_from_slice(slice);
    *i = end;
    Ok(u32::from_le_bytes(arr))
}

fn read_u64(bytes: &[u8], i: &mut usize) -> Result<u64> {
    let end = i.saturating_add(8);
    let slice = bytes.get(*i..end).ok_or_else(short)?;
    let mut arr = [0u8; 8];
    arr.copy_from_slice(slice);
    *i = end;
    Ok(u64::from_le_bytes(arr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_roundtrip_type1() {
        let s = AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [9u8; 32],
            message_root: [3u8; 32],
            slot: 42,
            participants: ParticipantSet::try_from_ordered(vec![0, 2, 7]).unwrap(),
            components: vec![],
        };
        let wire = s.encode_wire();
        let back = AggregateStatement::decode_wire(&wire).unwrap();
        assert_eq!(s, back);
        assert_eq!(s.digest(), back.digest());
    }
}
