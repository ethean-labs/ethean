//! JSON DTOs for Lean chain views and the hive `/lean/v0` interop surface.

use crate::hexutil::{hex_root_0x, parse_hex_root};
use ethean_primitives::{Hash32, Slot, HASH32_ZERO};
use serde::{Deserialize, Serialize};

/// Head view returned by GET `/lean/v1/chain/head`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadView {
    pub slot: Slot,
    pub root: Hash32,
}

/// Finalized checkpoint view with explicit trust label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizedView {
    pub slot: Slot,
    pub root: Hash32,
    /// Operator-visible trust source (never implied canonical by structure alone).
    pub trust_source: String,
}

/// Sync gate view for duties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncView {
    pub syncing: bool,
    pub head_slot: Slot,
    pub peer_horizon_slot: Slot,
}

/// Fork-choice view returned by GET /lean/v1/chain/fork_choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkChoiceStatsView {
    /// Live structural store is present on the node.
    pub live: bool,
    pub head_root: Hash32,
    pub safe_target_root: Hash32,
    pub safe_target_slot: u64,
    pub justified_root: Hash32,
    pub finalized_root: Hash32,
    pub reorg_total: u64,
    pub blocks: u64,
    pub pending_votes: u64,
    pub known_votes: u64,
}

/// Checkpoint on the hive `/lean/v0` wire (`slot` as JSON number, `root` as 0x-hex).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CheckpointBody {
    pub slot: u64,
    #[serde(serialize_with = "ser_root", deserialize_with = "de_root")]
    pub root: Hash32,
}

impl CheckpointBody {
    pub fn new(slot: Slot, root: Hash32) -> Self {
        Self {
            slot: slot.get(),
            root,
        }
    }

    pub fn genesis(root: Hash32) -> Self {
        Self { slot: 0, root }
    }
}

/// One fork-choice node (hive `ForkChoiceNodeResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ForkChoiceNodeBody {
    #[serde(serialize_with = "ser_root", deserialize_with = "de_root")]
    pub root: Hash32,
    pub slot: u64,
    #[serde(serialize_with = "ser_root", deserialize_with = "de_root")]
    pub parent_root: Hash32,
    pub proposer_index: u64,
    pub weight: u64,
}

/// Fork-choice snapshot (hive `ForkChoiceResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ForkChoiceBody {
    #[serde(default)]
    pub nodes: Vec<ForkChoiceNodeBody>,
    #[serde(serialize_with = "ser_root", deserialize_with = "de_root")]
    pub head: Hash32,
    pub justified: CheckpointBody,
    pub finalized: CheckpointBody,
    #[serde(
        default = "zero_root",
        serialize_with = "ser_root",
        deserialize_with = "de_root"
    )]
    pub safe_target: Hash32,
    #[serde(default)]
    pub validator_count: u64,
}

fn zero_root() -> Hash32 {
    HASH32_ZERO
}

fn ser_root<S: serde::Serializer>(root: &Hash32, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&hex_root_0x(root))
}

fn de_root<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Hash32, D::Error> {
    let s = String::deserialize(d)?;
    parse_hex_root(&s).map_err(serde::de::Error::custom)
}

/// GET `/lean/v0/health`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthBody {
    pub status: String,
    pub service: String,
}

/// GET `/lean/v0/admin/aggregator`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregatorStatusBody {
    pub is_aggregator: bool,
}

/// POST `/lean/v0/admin/aggregator` request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AggregatorToggleRequest {
    pub enabled: bool,
}

/// POST `/lean/v0/admin/aggregator` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregatorToggleBody {
    pub is_aggregator: bool,
    pub previous: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fork_choice_hive_field_names() {
        let root = [0xabu8; 32];
        let body = ForkChoiceBody {
            nodes: vec![ForkChoiceNodeBody {
                root,
                slot: 0,
                parent_root: HASH32_ZERO,
                proposer_index: 0,
                weight: 0,
            }],
            head: root,
            justified: CheckpointBody::genesis(root),
            finalized: CheckpointBody::genesis(root),
            safe_target: root,
            validator_count: 4,
        };
        let v = serde_json::to_value(&body).unwrap();
        assert!(v["head"].as_str().unwrap().starts_with("0xab"));
        assert_eq!(v["nodes"][0]["slot"], 0);
        assert_eq!(
            v["nodes"][0]["parent_root"],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(v["justified"]["slot"], 0);
        assert_eq!(v["validator_count"], 4);
        let back: ForkChoiceBody = serde_json::from_value(v).unwrap();
        assert_eq!(back, body);
    }
}
