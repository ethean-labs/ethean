//! leanSpec `ssz` suite: decode the vector, re-encode it and hash it with
//! Ethean's own codecs; decode-rejection vectors must fail to decode.

use crate::driver::decode_hex_bytes;
use ethean_crypto::{Fp, PublicKey, Signature};
use ethean_network_wire::{BlocksByRootRequest, Status};
use ethean_ssz::{
    chunk_from_bytes, decode_bitlist, decode_u32, encode_bitlist, expect_exhausted,
    hash_tree_root_bytes, hash_tree_root_container, hash_tree_root_list, hash_tree_root_u64, Root,
};
use ethean_types::xmss_signature_root;
use ethean_types::{
    AggregatedAttestation, Attestation, AttestationData, Block, BlockBody, BlockHeader, Checkpoint,
    GenesisConfig, MultiMessageAggregate, SignedAggregatedAttestation, SignedAttestation,
    SignedBlock, SingleMessageAggregate, State, Validator,
};
use serde_json::Value;

/// Result of one ssz vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SszOutcome {
    /// Round trip and root matched.
    Checked,
    /// Decode failed as the vector requires.
    Rejected,
    /// Ethean has no codec for this type name.
    Unsupported(String),
}

type Decoded = Result<(Vec<u8>, Option<Root>), String>;

fn s<E: ToString>(e: E) -> String {
    e.to_string()
}

fn wire_checkpoint_root(cp: &ethean_network_wire::Checkpoint) -> Root {
    hash_tree_root_container(&[chunk_from_bytes(&cp.root), hash_tree_root_u64(cp.slot)])
}

fn decode_by_name(name: &str, bytes: &[u8]) -> Option<Decoded> {
    Some(match name {
        "Checkpoint" => Checkpoint::ssz_decode(bytes)
            .map_err(s)
            .map(|v| (v.ssz_encode(), Some(v.hash_tree_root()))),
        "AttestationData" => AttestationData::ssz_decode(bytes)
            .map_err(s)
            .map(|v| (v.ssz_encode(), Some(v.hash_tree_root()))),
        "Validator" => Validator::ssz_decode(bytes)
            .map_err(s)
            .map(|v| (v.ssz_encode(), Some(v.hash_tree_root()))),
        "BlockHeader" => BlockHeader::ssz_decode(bytes)
            .map_err(s)
            .map(|v| (v.ssz_encode(), Some(v.hash_tree_root()))),
        "Config" => GenesisConfig::ssz_decode(bytes)
            .map_err(s)
            .map(|v| (v.ssz_encode(), Some(v.hash_tree_root()))),
        "Attestation" => Attestation::ssz_decode(bytes)
            .map_err(s)
            .map(|v| (v.ssz_encode(), Some(v.hash_tree_root()))),
        "SignedAttestation" => SignedAttestation::ssz_decode(bytes)
            .map_err(s)
            .and_then(|v| Ok((v.ssz_encode(), Some(v.hash_tree_root().map_err(s)?)))),
        "AggregatedAttestation" => AggregatedAttestation::ssz_decode(bytes)
            .map_err(s)
            .and_then(|v| Ok((v.ssz_encode(), Some(v.hash_tree_root().map_err(s)?)))),
        "SignedAggregatedAttestation" => SignedAggregatedAttestation::ssz_decode(bytes)
            .map_err(s)
            .and_then(|v| {
                let root = hash_tree_root_container(&[
                    v.data.hash_tree_root(),
                    v.proof.hash_tree_root().map_err(s)?,
                ]);
                Ok((v.ssz_encode().map_err(s)?, Some(root)))
            }),
        "SingleMessageAggregate" => SingleMessageAggregate::ssz_decode(bytes)
            .map_err(s)
            .and_then(|v| {
                Ok((
                    v.ssz_encode().map_err(s)?,
                    Some(v.hash_tree_root().map_err(s)?),
                ))
            }),
        "MultiMessageAggregate" => MultiMessageAggregate::ssz_decode(bytes)
            .map_err(s)
            .and_then(|v| Ok((v.ssz_encode(), Some(v.hash_tree_root().map_err(s)?)))),
        "BlockBody" => BlockBody::ssz_decode(bytes).map_err(s).and_then(|v| {
            Ok((
                v.ssz_encode().map_err(s)?,
                Some(v.hash_tree_root().map_err(s)?),
            ))
        }),
        "Block" => Block::ssz_decode(bytes).map_err(s).and_then(|v| {
            Ok((
                v.ssz_encode().map_err(s)?,
                Some(v.hash_tree_root().map_err(s)?),
            ))
        }),
        "SignedBlock" => SignedBlock::ssz_decode(bytes).map_err(s).and_then(|v| {
            Ok((
                v.ssz_encode().map_err(s)?,
                Some(v.hash_tree_root().map_err(s)?),
            ))
        }),
        "State" => State::ssz_decode(bytes).map_err(s).and_then(|v| {
            Ok((
                v.ssz_encode().map_err(s)?,
                Some(v.hash_tree_root().map_err(s)?),
            ))
        }),
        "Status" => Status::decode(bytes).map_err(s).and_then(|v| {
            let root = hash_tree_root_container(&[
                wire_checkpoint_root(&v.finalized),
                wire_checkpoint_root(&v.head),
            ]);
            Ok((v.encode().map_err(s)?, Some(root)))
        }),
        "BlocksByRootRequest" => BlocksByRootRequest::decode(bytes).map_err(s).and_then(|v| {
            let root = hash_tree_root_list(&v.roots, 1024).map_err(s)?;
            Ok((v.encode(), Some(root)))
        }),
        "PublicKey" => PublicKey::try_from_slice(bytes).map_err(s).map(|k| {
            (
                k.as_bytes().to_vec(),
                Some(hash_tree_root_bytes(k.as_bytes())),
            )
        }),
        "Signature" => Signature::try_from_slice(bytes).map_err(s).and_then(|sig| {
            let b = sig.as_bytes();
            Ok((b.to_vec(), Some(xmss_signature_root(b).map_err(s)?)))
        }),
        "Fp" => {
            let arr: Result<[u8; 4], String> =
                bytes.try_into().map_err(|_| "Fp needs 4 bytes".to_string());
            arr.and_then(|a| Fp::from_le_bytes(a).map_err(s)).map(|f| {
                (
                    f.to_le_bytes().to_vec(),
                    Some(chunk_from_bytes(&f.to_le_bytes())),
                )
            })
        }
        "Uint32" => {
            let mut cursor = 0;
            decode_u32(bytes, &mut cursor)
                .and_then(|v| expect_exhausted(bytes, cursor).map(|_| v))
                .map_err(s)
                .map(|v| {
                    (
                        v.to_le_bytes().to_vec(),
                        Some(chunk_from_bytes(&v.to_le_bytes())),
                    )
                })
        }
        "Bytes4" => {
            if bytes.len() == 4 {
                Ok((bytes.to_vec(), Some(chunk_from_bytes(bytes))))
            } else {
                Err(format!("Bytes4 needs 4 bytes, got {}", bytes.len()))
            }
        }
        "Validators" => ethean_types::decode_validator_list(bytes).map_err(s).and_then(|list| {
            let roots: Vec<Root> = list.iter().map(Validator::hash_tree_root).collect();
            let mut enc = Vec::with_capacity(list.len() * 112);
            for v in &list {
                enc.extend_from_slice(&v.ssz_encode());
            }
            Ok((enc, Some(hash_tree_root_list(&roots, 1 << 12).map_err(s)?)))
        }),
        "DecodeBitlist8" => decode_bitlist(bytes, 8)
            .map_err(s)
            .map(|bits| (encode_bitlist(&bits), None)),
        _ => return None,
    })
}

/// Run one ssz vector.
pub fn run_ssz_case(case: &Value) -> Result<SszOutcome, String> {
    let name = case
        .get("typeName")
        .and_then(Value::as_str)
        .ok_or("missing typeName")?;
    let serialized = decode_hex_bytes(
        case.get("serialized")
            .and_then(Value::as_str)
            .ok_or("missing serialized")?,
    )?;
    let expect_reject = case.get("rejectionReason").is_some();
    // Rejection vectors carry an empty root.
    let want_root = match case.get("root").and_then(Value::as_str) {
        Some(h) if h.trim_start_matches("0x").len() == 64 => {
            let v = decode_hex_bytes(h)?;
            Some(<[u8; 32]>::try_from(v.as_slice()).map_err(|_| "root is not 32 bytes")?)
        }
        _ => None,
    };
    let Some(decoded) = decode_by_name(name, &serialized) else {
        return Ok(SszOutcome::Unsupported(name.to_string()));
    };
    match (decoded, expect_reject) {
        (Err(_), true) => Ok(SszOutcome::Rejected),
        (Err(e), false) => Err(format!("{name}: decode failed: {e}")),
        (Ok(_), true) => Err(format!("{name}: decoded, but DECODE_ERROR expected")),
        (Ok((encoded, root)), false) => {
            if encoded != serialized {
                return Err(format!("{name}: re-encoding differs from the vector"));
            }
            if let (Some(want), Some(got)) = (want_root, root) {
                if want != got {
                    return Err(format!("{name}: hash_tree_root differs from the vector"));
                }
            }
            Ok(SszOutcome::Checked)
        }
    }
}
