//! Background proving: a worker thread that owns the `ethean-prover` client.
//!
//! The chain owner never waits on a proof. It submits jobs and collects
//! finished outcomes at the start of each duty step, then re-verifies every
//! proof in-process before using it.

use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
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
    jobs: SyncSender<ProofJob>,
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
        let (jobs, job_rx) = mpsc::sync_channel::<ProofJob>(QUEUE_DEPTH);
        let (outcome_tx, outcomes) = mpsc::channel();
        std::thread::Builder::new()
            .name("ethean-proof-service".into())
            .spawn(move || {
                let client = ProverClient::new(config);
                while let Ok(job) = job_rx.recv() {
                    if outcome_tx.send(run(&client, job)).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            jobs,
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
    pub fn submit(&mut self, job: ProofJob) -> bool {
        let key = match &job {
            ProofJob::Attestation { data, .. } | ProofJob::Split { data, .. } => {
                (Some(data.hash_tree_root()), None)
            }
            ProofJob::Block { plan, .. } => (None, plan.block_root().ok()),
        };
        match self.jobs.try_send(job) {
            Ok(()) => {
                if let Some(root) = key.0 {
                    self.attestations_in_flight.insert(root);
                }
                if key.1.is_some() {
                    self.block_in_flight = key.1;
                }
                true
            }
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => false,
        }
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
