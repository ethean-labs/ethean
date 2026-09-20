//! Type-2 → Type-1 split over leanVM process IPC (structural until crypto_split).

use crate::aggregation::{split_type2_to_type1, AggregateStatement, Type1Leaf, LEANVM_REV};
use crate::error::{CryptoError, Result};
use crate::leanvm_ipc_frame::{IpcFrame, IpcOp};
use crate::leanvm_ipc_spawn::{exchange_frame, DEFAULT_IPC_WALL};
use crate::leanvm_ipc::LeanVmIpcStatus;
use std::path::Path;

/// Encode Type-1 leaves into the SplitResponse `proof` blob.
pub fn encode_type1_leaves(leaves: &[Type1Leaf]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(leaves.len() as u32).to_le_bytes());
    for leaf in leaves {
        out.extend_from_slice(&leaf.message_root);
        out.extend_from_slice(&leaf.slot.to_le_bytes());
        out.extend_from_slice(&(leaf.proof.len() as u32).to_le_bytes());
        out.extend_from_slice(&leaf.proof);
        out.push(u8::from(leaf.crypto_split));
    }
    out
}

/// Decode leaves produced by [`encode_type1_leaves`].
pub fn decode_type1_leaves(bytes: &[u8]) -> Result<Vec<Type1Leaf>> {
    let mut i = 0usize;
    let count = read_u32(bytes, &mut i)? as usize;
    let mut leaves = Vec::with_capacity(count);
    for _ in 0..count {
        let root = bytes
            .get(i..i + 32)
            .ok_or_else(|| CryptoError::InvalidAggregate("short Type-1 leaf root".into()))?;
        let mut message_root = [0u8; 32];
        message_root.copy_from_slice(root);
        i += 32;
        let slot = read_u64(bytes, &mut i)?;
        let proof_len = read_u32(bytes, &mut i)? as usize;
        let proof = bytes
            .get(i..i + proof_len)
            .ok_or_else(|| CryptoError::InvalidAggregate("short Type-1 leaf proof".into()))?
            .to_vec();
        i += proof_len;
        let crypto_split = *bytes
            .get(i)
            .ok_or_else(|| CryptoError::InvalidAggregate("short Type-1 leaf flag".into()))?
            != 0;
        i += 1;
        leaves.push(Type1Leaf {
            message_root,
            slot,
            proof,
            crypto_split,
        });
    }
    if i != bytes.len() {
        return Err(CryptoError::InvalidAggregate(
            "trailing bytes after Type-1 leaf list".into(),
        ));
    }
    Ok(leaves)
}

/// Split via process IPC when `ETHEAN_LEANVM_PROVER` is present; else structural local split.
///
/// Mock peers return structural leaves (`crypto_split = false`). Real SNARK decomposition
/// stays fail-closed until a production leanVM implements SplitResponse with proofs.
pub fn split_ipc(statement: &AggregateStatement, type2_proof: &[u8]) -> Result<Vec<Type1Leaf>> {
    let status = LeanVmIpcStatus::probe();
    let Some(path) = status.binary_path.as_ref() else {
        return split_type2_to_type1(statement, type2_proof);
    };
    if !status.binary_present || !status.spawn_exchange_wired {
        return Err(CryptoError::BackendUnavailable(
            "leanVM split IPC needs a present prover (set ETHEAN_LEANVM_PROVER)",
        ));
    }
    split_ipc_at(path, statement, type2_proof)
}

/// Split against an explicit prover path (tests / CI).
pub fn split_ipc_at(
    prover: &Path,
    statement: &AggregateStatement,
    type2_proof: &[u8],
) -> Result<Vec<Type1Leaf>> {
    let request = IpcFrame::split_request(statement, type2_proof)?;
    let response = exchange_frame(prover, &request, DEFAULT_IPC_WALL)?;
    accept_split_response(&request, response)
}

fn accept_split_response(request: &IpcFrame, response: IpcFrame) -> Result<Vec<Type1Leaf>> {
    if response.op != IpcOp::SplitResponse {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC expected SplitResponse".into(),
        ));
    }
    if response.pin_rev != LEANVM_REV {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC split response pin mismatch".into(),
        ));
    }
    if response.statement != request.statement {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC split response statement mismatch".into(),
        ));
    }
    if !response.ok {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC split response not ok".into(),
        ));
    }
    decode_type1_leaves(&response.proof)
}

fn read_u32(bytes: &[u8], i: &mut usize) -> Result<u32> {
    let end = i.saturating_add(4);
    let slice = bytes
        .get(*i..end)
        .ok_or_else(|| CryptoError::InvalidAggregate("truncated u32".into()))?;
    let mut arr = [0u8; 4];
    arr.copy_from_slice(slice);
    *i = end;
    Ok(u32::from_le_bytes(arr))
}

fn read_u64(bytes: &[u8], i: &mut usize) -> Result<u64> {
    let end = i.saturating_add(8);
    let slice = bytes
        .get(*i..end)
        .ok_or_else(|| CryptoError::InvalidAggregate("truncated u64".into()))?;
    let mut arr = [0u8; 8];
    arr.copy_from_slice(slice);
    *i = end;
    Ok(u64::from_le_bytes(arr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::{ParticipantSet, ProofKind, Type2ComponentRef};

    fn sample_type2() -> AggregateStatement {
        AggregateStatement {
            kind: ProofKind::Type2,
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
            slot: 7,
            participants: ParticipantSet::empty(),
            components: vec![
                Type2ComponentRef {
                    message_root: [10u8; 32],
                    slot: 7,
                },
                Type2ComponentRef {
                    message_root: [11u8; 32],
                    slot: 7,
                },
            ],
        }
    }

    #[test]
    fn leaf_wire_roundtrip() {
        let leaves = split_type2_to_type1(&sample_type2(), &[9u8; 16]).unwrap();
        let encoded = encode_type1_leaves(&leaves);
        let decoded = decode_type1_leaves(&encoded).unwrap();
        assert_eq!(decoded, leaves);
    }

    #[test]
    fn split_request_response_accept() {
        let stmt = sample_type2();
        let req = IpcFrame::split_request(&stmt, &[1, 2, 3]).unwrap();
        let leaves = split_type2_to_type1(&stmt, &[1, 2, 3]).unwrap();
        let resp = IpcFrame {
            op: IpcOp::SplitResponse,
            pin_rev: LEANVM_REV.to_string(),
            statement: req.statement.clone(),
            proof: encode_type1_leaves(&leaves),
            ok: true,
        };
        let out = accept_split_response(&req, resp).unwrap();
        assert_eq!(out.len(), 2);
        assert!(!out[0].crypto_split);
    }
}
