//! Background proving: a worker thread that owns the `ethean-prover` client.
//!
//! The chain owner never waits on a proof. It submits jobs and collects
//! finished outcomes at the start of each duty step, then re-verifies every
//! proof in-process before using it.
//!
//! Queue order is **Block > Attestation > Split** so Type-2 publish work is
//! not stuck behind post-block Split recovery on a busy mesh host.

use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use ethean_crypto::{PublicKey, Signature};
use ethean_multisig::{KeyedProof, ProverClient, ProverConfig};
use ethean_primitives::Hash32;
use ethean_types::AttestationData;

use crate::block_builder::PlanTransition;

/// Jobs queued beyond this are refused (the prover serves one at a time).
const QUEUE_DEPTH: usize = 16;

/// Work for the prover.
#[derive(Debug, Clone)]
pub enum ProofJob {
    /// Single-message aggregate for one attestation data.
    Attestation {
        data: AttestationData,
        /// Union of validators the resulting proof covers.
        participants: Vec<bool>,
        raw: Vec<(PublicKey, Signature)>,
        children: Vec<KeyedProof>,
    },
    /// Block proof: body attestation proofs in order, then the proposer.
    Block {
        plan: PlanTransition,
        attestation_proofs: Vec<KeyedProof>,
        proposer_key: PublicKey,
        proposer_signature: Signature,
    },
    /// Recover the Type-1 proof of one block vote from a block proof.
    Split {
        data: AttestationData,
        participants: Vec<bool>,
        block_proof: Vec<u8>,
        public_keys_per_component: Vec<Vec<PublicKey>>,
    },
}

impl ProofJob {
    fn kind_label(&self) -> &'static str {
        match self {
            ProofJob::Block { .. } => "block",
            ProofJob::Attestation { .. } => "attestation",
            ProofJob::Split { .. } => "split",
        }
    }
}

/// Finished job; proofs still need in-process verification by the caller.
#[derive(Debug, Clone)]
pub enum ProofOutcome {
    Attestation {
        data: AttestationData,
        participants: Vec<bool>,
        proof: Result<Vec<u8>, String>,
        /// Wall time the prover spent on this job.
        elapsed: Duration,
    },
    Block {
        plan: PlanTransition,
        proof: Result<Vec<u8>, String>,
        /// Wall time the prover spent on this job.
        elapsed: Duration,
    },
    /// A Type-1 recovered from a block proof (never re-gossiped).
    Split {
        data: AttestationData,
        participants: Vec<bool>,
        proof: Result<Vec<u8>, String>,
        elapsed: Duration,
    },
}

/// Priority inbox: Block jumps ahead of Attestation, which jumps ahead of Split.
#[derive(Default)]
struct PriorityInbox {
    blocks: VecDeque<ProofJob>,
    attestations: VecDeque<ProofJob>,
    splits: VecDeque<ProofJob>,
}

impl PriorityInbox {
    fn len(&self) -> usize {
        self.blocks.len() + self.attestations.len() + self.splits.len()
    }

    fn push(&mut self, job: ProofJob) {
        match &job {
            ProofJob::Block { .. } => self.blocks.push_back(job),
            ProofJob::Attestation { .. } => self.attestations.push_back(job),
            ProofJob::Split { .. } => self.splits.push_back(job),
        }
    }

    fn pop_next(&mut self) -> Option<ProofJob> {
        self.blocks
            .pop_front()
            .or_else(|| self.attestations.pop_front())
            .or_else(|| self.splits.pop_front())
    }
}

struct SharedInbox {
    inner: Mutex<(PriorityInbox, bool)>,
    cv: Condvar,
}

impl SharedInbox {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new((PriorityInbox::default(), false)),
            cv: Condvar::new(),
        })
    }

    fn try_push(&self, job: ProofJob) -> bool {
        let mut guard = self.inner.lock().expect("proof inbox");
        let (inbox, closed) = &mut *guard;
        if *closed || inbox.len() >= QUEUE_DEPTH {
            return false;
        }
        inbox.push(job);
        self.cv.notify_one();
        true
    }

    fn pop_blocking(&self) -> Option<ProofJob> {
        let mut guard = self.inner.lock().expect("proof inbox");
        loop {
            if let Some(job) = guard.0.pop_next() {
                return Some(job);
            }
            if guard.1 {
                return None;
            }
            guard = self.cv.wait(guard).expect("proof inbox wait");
        }
    }
}

fn run(client: &ProverClient, job: ProofJob) -> ProofOutcome {
    let started = Instant::now();
    match job {
        ProofJob::Attestation {
            data,
            participants,
            raw,
            children,
        } => {
            let proof = client
                .aggregate_type1(children, raw, data.hash_tree_root(), data.slot.get())
                .map_err(|e| e.to_string());
            ProofOutcome::Attestation {
                data,
                participants,
                proof,
                elapsed: started.elapsed(),
            }
        }
        ProofJob::Block {
            plan,
            mut attestation_proofs,
            proposer_key,
            proposer_signature,
        } => {
            let proof = plan
                .block_root()
                .and_then(|root| {
                    client
                        .aggregate_type1(
                            Vec::new(),
                            vec![(proposer_key, proposer_signature)],
                            root,
                            plan.block.slot.get(),
                        )
                        .map_err(|e| e.to_string())
                })
                .and_then(|proposer_proof| {
                    attestation_proofs.push(KeyedProof {
                        public_keys: vec![proposer_key],
                        proof: proposer_proof,
                    });
                    client
                        .merge_type2(attestation_proofs)
                        .map_err(|e| e.to_string())
                });
            ProofOutcome::Block {
                plan,
                proof,
                elapsed: started.elapsed(),
            }
        }
        ProofJob::Split {
            data,
            participants,
            block_proof,
            public_keys_per_component,
        } => {
            let proof = client
                .split_type2(block_proof, public_keys_per_component, data.hash_tree_root())
                .map_err(|e| e.to_string());
            ProofOutcome::Split {
                data,
                participants,
                proof,
                elapsed: started.elapsed(),
            }
        }
    }
}

/// Owner-side handle to the proving thread.
pub struct ProofService {
    inbox: Arc<SharedInbox>,
    outcomes: Receiver<ProofOutcome>,
    attestations_in_flight: HashSet<Hash32>,
    block_in_flight: Option<Hash32>,
}

impl std::fmt::Debug for ProofService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProofService")
            .field("attestations_in_flight", &self.attestations_in_flight.len())
            .field("block_in_flight", &self.block_in_flight.is_some())
            .finish()
    }
}

impl ProofService {
    /// Start the worker thread; the prover process starts on the first job.
    pub fn spawn(config: ProverConfig) -> std::io::Result<Self> {
        let inbox = SharedInbox::new();
        let worker_inbox = Arc::clone(&inbox);
        let (outcome_tx, outcomes) = mpsc::channel();
        std::thread::Builder::new()
            .name("ethean-proof-service".into())
            .spawn(move || {
                let client = ProverClient::new(config);
                while let Some(job) = worker_inbox.pop_blocking() {
                    let kind = job.kind_label();
                    tracing::debug!(kind, "proof worker starting job");
                    if outcome_tx.send(run(&client, job)).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            inbox,
            outcomes,
            attestations_in_flight: HashSet::new(),
            block_in_flight: None,
        })
    }

    /// Discover the prover binary and start the service, if available.
    pub fn discover() -> Option<Self> {
        let config = ProverConfig::discover()?;
        tracing::info!(prover = %config.binary.display(), "leanMultisig prover found");
        Self::spawn(config).ok()
    }

    /// True while an aggregate for `data_root` is being proved.
    pub fn attestation_in_flight(&self, data_root: &Hash32) -> bool {
        self.attestations_in_flight.contains(data_root)
    }

    /// True while a block proof is being produced.
    pub fn block_in_flight(&self) -> bool {
        self.block_in_flight.is_some()
    }

    /// Queue a job; false when the queue is full or the worker is gone.
    ///
    /// Block jobs jump ahead of attestations and splits already waiting.
    pub fn submit(&mut self, job: ProofJob) -> bool {
        let key = match &job {
            ProofJob::Attestation { data, .. } | ProofJob::Split { data, .. } => {
                (Some(data.hash_tree_root()), None)
            }
            ProofJob::Block { plan, .. } => (None, plan.block_root().ok()),
        };
        if !self.inbox.try_push(job) {
            return false;
        }
        if let Some(root) = key.0 {
            self.attestations_in_flight.insert(root);
        }
        if key.1.is_some() {
            self.block_in_flight = key.1;
        }
        true
    }

    /// Collect every finished outcome without blocking.
    pub fn drain(&mut self) -> Vec<ProofOutcome> {
        let mut out = Vec::new();
        loop {
            match self.outcomes.try_recv() {
                Ok(outcome) => {
                    match &outcome {
                        ProofOutcome::Attestation { data, .. }
                        | ProofOutcome::Split { data, .. } => {
                            self.attestations_in_flight.remove(&data.hash_tree_root());
                        }
                        ProofOutcome::Block { .. } => self.block_in_flight = None,
                    }
                    out.push(outcome);
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => return out,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_types::{AttestationData, Block, BlockBody, Checkpoint};

    fn att_job(slot: u64) -> ProofJob {
        let data = AttestationData {
            slot: Slot::new(slot),
            head: Checkpoint::genesis(),
            target: Checkpoint::genesis(),
            source: Checkpoint::genesis(),
        };
        ProofJob::Attestation {
            data,
            participants: vec![true],
            raw: Vec::new(),
            children: Vec::new(),
        }
    }

    fn split_job(slot: u64) -> ProofJob {
        let data = AttestationData {
            slot: Slot::new(slot),
            head: Checkpoint::genesis(),
            target: Checkpoint::genesis(),
            source: Checkpoint::genesis(),
        };
        ProofJob::Split {
            data,
            participants: vec![true],
            block_proof: vec![1],
            public_keys_per_component: Vec::new(),
        }
    }

    fn block_job(slot: u64) -> ProofJob {
        let plan = PlanTransition {
            parent_root: HASH32_ZERO,
            block: Block {
                slot: Slot::new(slot),
                proposer_index: ValidatorIndex::ZERO,
                parent_root: HASH32_ZERO,
                state_root: HASH32_ZERO,
                body: BlockBody::default(),
            },
            aggregate_proof: Vec::new(),
            attestation_proofs: Vec::new(),
        };
        ProofJob::Block {
            plan,
            attestation_proofs: Vec::new(),
            proposer_key: PublicKey::from_bytes([0u8; 52]),
            proposer_signature: Signature::from_bytes([0u8; 2536]),
        }
    }

    #[test]
    fn priority_pops_block_before_split() {
        let mut inbox = PriorityInbox::default();
        inbox.push(split_job(1));
        inbox.push(split_job(2));
        inbox.push(att_job(3));
        inbox.push(block_job(4));
        assert!(matches!(inbox.pop_next(), Some(ProofJob::Block { .. })));
        assert!(matches!(
            inbox.pop_next(),
            Some(ProofJob::Attestation { .. })
        ));
        assert!(matches!(inbox.pop_next(), Some(ProofJob::Split { .. })));
        assert!(matches!(inbox.pop_next(), Some(ProofJob::Split { .. })));
        assert!(inbox.pop_next().is_none());
    }

    #[test]
    fn shared_inbox_respects_depth() {
        let inbox = SharedInbox::new();
        for i in 0..QUEUE_DEPTH {
            assert!(inbox.try_push(split_job(i as u64)));
        }
        assert!(!inbox.try_push(block_job(99)));
    }
}
