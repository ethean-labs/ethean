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
        /// Merged block proof length in bytes.
        proof_len: usize,
    },
    /// Local proposer signed the planned block root (XMSS/HMAC binding).
    ProposalSigned {
        /// Inner block tree root that was signed.
        root: Hash32,
        /// Signature wire length in bytes.
        signature_len: usize,
    },
    /// A prove job was queued with the proof service.
    ProofScheduled {
        /// "attestation" or "block".
        kind: &'static str,
        /// Attestation-data root or block root.
        root: Hash32,
    },
    /// A prove job failed or its proof did not verify.
    ProofFailed {
        /// "attestation" or "block".
        kind: &'static str,
        /// Reason.
        error: String,
    },
    /// The merged block proof verified and the block was imported locally.
    BlockProofAttached {
        /// Inner block tree root.
        root: Hash32,
        /// Proof length in bytes.
        proof_len: usize,
    },
    /// A Type-1 aggregate verified, entered the pool and was queued for gossip.
    AggregateProved {
        /// Attestation-data root.
        data_root: Hash32,
        /// Validators covered.
        coverage: u32,
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
    /// Local attester signed attestation data for an owned validator index.
    AttestationSigned {
        /// Attestation-data tree root that was signed.
        data_root: Hash32,
        /// Validator index that voted.
        validator_index: ethean_primitives::ValidatorIndex,
        /// Signature wire length in bytes.
        signature_len: usize,
        /// Attestation subnet.
        subnet: u16,
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
        /// Merged block proof length in bytes.
        proof_len: usize,
    },
    /// Syncing flag changed on the chain owner.
    SyncingUpdated(bool),
    /// Shutdown acknowledged; no new duties.
    ShutdownComplete,
}
