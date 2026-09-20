//! Fork-choice store state (lstar essentials).

use std::collections::HashMap;

use ethean_primitives::{Hash32, ValidatorIndex};
use ethean_types::{AttestationData, Block, Checkpoint, State};

use crate::opts::ForkChoiceOpts;

/// One aggregated payload keyed by attestation-data root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregatedPayloadEntry {
    /// `hash_tree_root(AttestationData)`.
    pub data_root: Hash32,
    /// Shared attestation data.
    pub data: AttestationData,
    /// Distinct participant bit-sets observed for this data root.
    pub participant_sets: Vec<Vec<u64>>,
}

/// Local view of the chain for running fork choice (leanSpec `Store`).
#[derive(Debug, Clone)]
pub struct ForkChoiceStore {
    /// Interval counter since genesis: `slot * intervals_per_slot + interval`.
    pub time: u64,
    pub intervals_per_slot: u64,
    /// Wall-clock genesis (Unix seconds) for fixture `tick.time` conversion.
    pub genesis_time: u64,
    /// Interval length in milliseconds (profile timing).
    pub milliseconds_per_interval: u64,
    pub gossip_disparity_intervals: u64,
    pub historical_roots_limit: u64,
    pub opts: ForkChoiceOpts,
    pub head: Hash32,
    pub safe_target: Hash32,
    pub latest_justified: Checkpoint,
    pub latest_finalized: Checkpoint,
    pub blocks: HashMap<Hash32, Block>,
    pub block_states: HashMap<Hash32, State>,
    /// Pending votes awaiting promotion into the counted pool.
    pub latest_new_attestations: HashMap<ValidatorIndex, AttestationData>,
    /// Votes counted toward head selection.
    pub latest_known_attestations: HashMap<ValidatorIndex, AttestationData>,
    /// Pending gossip aggregates (leanSpec `latest_new_aggregated_payloads`).
    pub latest_new_payloads: HashMap<Hash32, AggregatedPayloadEntry>,
    /// Known on-chain / promoted aggregates (`latest_known_aggregated_payloads`).
    pub latest_known_payloads: HashMap<Hash32, AggregatedPayloadEntry>,
}

impl ForkChoiceStore {
    pub fn head(&self) -> Hash32 {
        self.head
    }

    pub fn safe_target(&self) -> Hash32 {
        self.safe_target
    }

    pub fn justified(&self) -> Checkpoint {
        self.latest_justified
    }

    pub fn finalized(&self) -> Checkpoint {
        self.latest_finalized
    }

    pub fn current_slot(&self) -> u64 {
        self.time / self.intervals_per_slot
    }
}
