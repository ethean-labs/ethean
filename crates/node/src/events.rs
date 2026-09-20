//! Events emitted by the chain owner to workers and API observers.

use ethean_primitives::Hash32;
use ethean_validator::{DutyTick, SuppressReason};

/// Outbound events (non-mutating observers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainEvent {
    /// A new duty tick was accepted.
    TickAccepted(DutyTick),
    /// A duplicate tick was ignored.
    TickDuplicate(DutyTick),
    /// A duty was suppressed before signing.
    DutySuppressed { tick: DutyTick, reason: SuppressReason },
    /// Head advanced after local import.
    HeadUpdated { root: Hash32, slot: u64 },
    /// Validated gossip ingested (content root recorded; SSZ decode still open).
    GossipIngested {
        /// Topic path that carried the payload.
        topic: String,
        /// SHA-256 of the decompressed payload (provisional content id).
        content_root: Hash32,
    },
    /// Local proposal planned from pool + structural transition.
    ProposalPlanned {
        /// `hash_tree_root` of the planned block.
        root: Hash32,
        /// Proposal slot.
        slot: u64,
        /// Number of attestations packed into the body.
        attestations: usize,
        /// Whether `decide_publish` allows gossiping this plan now.
        publish_allowed: bool,
    },
    /// SignedBlock SSZ ready for `/block/` gossip (publish window open).
    ProposalGossipReady {
        /// Inner block tree root.
        root: Hash32,
        /// Lean gossip topic string.
        topic: String,
        /// Encoded payload length in bytes.
        payload_len: usize,
        /// True when the envelope carries a non-empty Type-2 proof.
        has_type2_proof: bool,
        /// Local proposer signature length when signed this tick.
        proposer_sig_len: usize,
    },
    /// Local proposer signed the planned block root (XMSS/HMAC binding).
    ProposalSigned {
        /// Inner block tree root that was signed.
        root: Hash32,
        /// Signature wire length in bytes.
        signature_len: usize,
    },
    /// Proposer signature re-verified against the local key before gossip encode.
    ProposalBindingVerified {
        /// Inner block tree root that was verified.
        root: Hash32,
    },
    /// Local Type-2 aggregate proof attached to the planned block.
    Type2ProofAttached {
        /// Inner block tree root.
        root: Hash32,
        /// Proof byte length.
        proof_len: usize,
        /// Body attestations that already had Type-1 proofs in the pool.
        type1_hits: u32,
        /// Body attestation count that needed Type-1 cache coverage.
        type1_needed: u32,
    },
    /// Aggregator duty saw enough pool coverage to dispatch Type-1 prove.
    AggregatorReady {
        /// Attestation-data root being aggregated.
        data_root: Hash32,
        /// Assigned attestation subnet.
        subnet: u16,
        /// Observed participant coverage.
        coverage: u32,
    },
    /// Local Type-1 proof produced and re-verified for an aggregator-ready root.
    AggregatorType1Proved {
        /// Attestation-data root that was proved.
        data_root: Hash32,
        /// Proof byte length.
        proof_len: usize,
    },
    /// Type-1 aggregate SSZ queued for Lean aggregation / attestation gossip.
    AggregationGossipReady {
        /// Lean gossip topic string.
        topic: String,
        /// Attestation-data tree root.
        data_root: Hash32,
        /// Uncompressed payload length.
        payload_len: usize,
        /// Proof byte length.
        proof_len: usize,
    },
    /// Type-1 aggregate published on QuicSwarm gossip.
    AggregationPublished {
        /// Lean gossip topic string.
        topic: String,
        /// Attestation-data tree root.
        data_root: Hash32,
        /// Uncompressed payload length.
        payload_len: usize,
    },
    /// Pending proposal was published on QuicSwarm gossip.
    ProposalPublished {
        /// Lean gossip topic string.
        topic: String,
        /// Uncompressed SSZ payload length.
        payload_len: usize,
        /// True when the envelope carried a non-empty Type-2 proof.
        has_type2_proof: bool,
    },
    /// Syncing flag changed on the chain owner.
    SyncingUpdated(bool),
    /// Shutdown acknowledged; no new duties.
    ShutdownComplete,
}
