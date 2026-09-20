//! Versioned leanVM process-IPC request/response frames (B2).
//!
//! Codec only: prove/verify still refuse to spawn until a sandbox round-trip
//! against [`crate::aggregation::LEANVM_REV`] lands (`protocol_ready`).

use crate::aggregation::{AggregateStatement, ProofKind, LEANVM_REV};
use crate::error::{CryptoError, Result};

/// ASCII magic for Ethean leanVM IPC frames.
pub const FRAME_MAGIC: &[u8; 4] = b"ELVM";

/// Current frame ABI version (bump on breaking layout changes).
pub const FRAME_VERSION: u16 = 1;

/// True when this crate can encode/decode [`IpcFrame`] (not the same as spawn-ready).
pub const FRAME_CODEC_READY: bool = true;

/// IPC operation tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IpcOp {
    /// Client → prover: prove this statement.
    ProveRequest = 1,
    /// Prover → client: proof bytes (or error).
    ProveResponse = 2,
    /// Client → prover: verify statement + proof.
    VerifyRequest = 3,
    /// Prover → client: accept/reject.
    VerifyResponse = 4,
    /// Client → prover: split Type-2 proof into Type-1 leaf descriptors.
    SplitRequest = 5,
    /// Prover → client: encoded Type-1 leaves (may be structural-only).
    SplitResponse = 6,
}

impl IpcOp {
    fn from_u8(v: u8) -> Result<Self> {
        match v {
            1 => Ok(Self::ProveRequest),
            2 => Ok(Self::ProveResponse),
            3 => Ok(Self::VerifyRequest),
            4 => Ok(Self::VerifyResponse),
            5 => Ok(Self::SplitRequest),
            6 => Ok(Self::SplitResponse),
            other => Err(CryptoError::InvalidAggregate(format!(
                "unknown leanVM IPC op {other}"
            ))),
        }
    }
}

/// One framed leanVM IPC message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcFrame {
    /// Operation.
    pub op: IpcOp,
    /// 40-char hex pin the peer must match (`LEANVM_REV`).
    pub pin_rev: String,
    /// Canonical statement wire bytes.
    pub statement: Vec<u8>,
    /// Proof blob (empty on prove-request).
    pub proof: Vec<u8>,
    /// Response success bit (ignored on requests).
    pub ok: bool,
}

impl IpcFrame {
    /// Build a prove-request frame for `statement` pinned to this crate's leanVM rev.
    pub fn prove_request(statement: &AggregateStatement) -> Result<Self> {
        statement.validate_shape()?;
        Ok(Self {
            op: IpcOp::ProveRequest,
            pin_rev: LEANVM_REV.to_string(),
            statement: statement.encode_wire(),
            proof: Vec::new(),
            ok: false,
        })
    }

    /// Build a verify-request frame.
    pub fn verify_request(statement: &AggregateStatement, proof: &[u8]) -> Result<Self> {
        statement.validate_shape()?;
        Ok(Self {
            op: IpcOp::VerifyRequest,
            pin_rev: LEANVM_REV.to_string(),
            statement: statement.encode_wire(),
            proof: proof.to_vec(),
            ok: false,
        })
    }

    /// Build a Type-2 → Type-1 split request (`proof` carries the Type-2 blob).
    pub fn split_request(statement: &AggregateStatement, type2_proof: &[u8]) -> Result<Self> {
        statement.validate_shape()?;
        if statement.kind != ProofKind::Type2 {
            return Err(CryptoError::InvalidAggregate(
                "leanVM IPC split_request requires ProofKind::Type2".into(),
            ));
        }
        Ok(Self {
            op: IpcOp::SplitRequest,
            pin_rev: LEANVM_REV.to_string(),
            statement: statement.encode_wire(),
            proof: type2_proof.to_vec(),
            ok: false,
        })
    }

    /// Encode to bytes: magic | ver | op | pin40 | stmt_len | stmt | proof_len | proof | ok.
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.pin_rev.len() != 40 {
            return Err(CryptoError::InvalidAggregate(
                "leanVM IPC pin_rev must be 40 hex chars".into(),
            ));
        }
        let mut out = Vec::with_capacity(
            4 + 2 + 1 + 40 + 4 + self.statement.len() + 4 + self.proof.len() + 1,
        );
        out.extend_from_slice(FRAME_MAGIC);
        out.extend_from_slice(&FRAME_VERSION.to_le_bytes());
        out.push(self.op as u8);
        out.extend_from_slice(self.pin_rev.as_bytes());
        out.extend_from_slice(&(self.statement.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.statement);
        out.extend_from_slice(&(self.proof.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.proof);
        out.push(u8::from(self.ok));
        Ok(out)
    }

    /// Decode bytes produced by [`Self::encode`].
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut i = 0usize;
        let magic = bytes.get(i..i + 4).ok_or_else(short)?;
        if magic != FRAME_MAGIC {
            return Err(CryptoError::InvalidAggregate(
                "leanVM IPC magic mismatch".into(),
            ));
        }
        i += 4;
        let ver = read_u16(bytes, &mut i)?;
        if ver != FRAME_VERSION {
            return Err(CryptoError::InvalidAggregate(format!(
                "leanVM IPC version {ver} unsupported (want {FRAME_VERSION})"
            )));
        }
        let op = IpcOp::from_u8(*bytes.get(i).ok_or_else(short)?)?;
        i += 1;
        let pin_bytes = bytes.get(i..i + 40).ok_or_else(short)?;
        let pin_rev = std::str::from_utf8(pin_bytes)
            .map_err(|_| CryptoError::InvalidAggregate("leanVM IPC pin not utf8".into()))?
            .to_string();
        i += 40;
        let stmt_len = read_u32(bytes, &mut i)? as usize;
        let statement = bytes
            .get(i..i + stmt_len)
            .ok_or_else(short)?
            .to_vec();
        i += stmt_len;
        let proof_len = read_u32(bytes, &mut i)? as usize;
        let proof = bytes.get(i..i + proof_len).ok_or_else(short)?.to_vec();
        i += proof_len;
        let ok = *bytes.get(i).ok_or_else(short)? != 0;
        i += 1;
        if i != bytes.len() {
            return Err(CryptoError::InvalidAggregate(
                "trailing bytes after leanVM IPC frame".into(),
            ));
        }
        Ok(Self {
            op,
            pin_rev,
            statement,
            proof,
            ok,
        })
    }
}

fn short() -> CryptoError {
    CryptoError::InvalidAggregate("truncated leanVM IPC frame".into())
}

fn read_u16(bytes: &[u8], i: &mut usize) -> Result<u16> {
    let end = i.saturating_add(2);
    let slice = bytes.get(*i..end).ok_or_else(short)?;
    let mut arr = [0u8; 2];
    arr.copy_from_slice(slice);
    *i = end;
    Ok(u16::from_le_bytes(arr))
}

fn read_u32(bytes: &[u8], i: &mut usize) -> Result<u32> {
    let end = i.saturating_add(4);
    let slice = bytes.get(*i..end).ok_or_else(short)?;
    let mut arr = [0u8; 4];
    arr.copy_from_slice(slice);
    *i = end;
    Ok(u32::from_le_bytes(arr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::{ParticipantSet, ProofKind};

    fn sample() -> AggregateStatement {
        AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
            slot: 9,
            participants: ParticipantSet::try_from_ordered(vec![0, 1]).unwrap(),
            components: vec![],
        }
    }

    #[test]
    fn prove_request_roundtrip() {
        assert!(FRAME_CODEC_READY);
        let stmt = sample();
        let frame = IpcFrame::prove_request(&stmt).unwrap();
        assert_eq!(frame.op, IpcOp::ProveRequest);
        assert_eq!(frame.pin_rev, LEANVM_REV);
        let bytes = frame.encode().unwrap();
        let decoded = IpcFrame::decode(&bytes).unwrap();
        assert_eq!(decoded, frame);
        let back = AggregateStatement::decode_wire(&decoded.statement).unwrap();
        assert_eq!(back, stmt);
    }

    #[test]
    fn verify_request_carries_proof() {
        let frame = IpcFrame::verify_request(&sample(), &[9, 9, 9]).unwrap();
        let decoded = IpcFrame::decode(&frame.encode().unwrap()).unwrap();
        assert_eq!(decoded.op, IpcOp::VerifyRequest);
        assert_eq!(decoded.proof, vec![9, 9, 9]);
    }

    #[test]
    fn split_request_requires_type2() {
        assert!(IpcFrame::split_request(&sample(), &[1]).is_err());
        let type2 = AggregateStatement {
            kind: ProofKind::Type2,
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
            slot: 3,
            participants: ParticipantSet::empty(),
            components: vec![crate::aggregation::Type2ComponentRef {
                message_root: [3u8; 32],
                slot: 3,
            }],
        };
        let frame = IpcFrame::split_request(&type2, &[7, 7]).unwrap();
        assert_eq!(frame.op, IpcOp::SplitRequest);
        assert_eq!(frame.proof, vec![7, 7]);
        let decoded = IpcFrame::decode(&frame.encode().unwrap()).unwrap();
        assert_eq!(decoded.op, IpcOp::SplitRequest);
    }

    #[test]
    fn rejects_bad_magic() {
        assert!(IpcFrame::decode(b"XXXX").is_err());
    }
}
