//! Ethean Lean Consensus validator — signer safety and duty scheduling.

#![forbid(unsafe_code)]

pub mod aggregator;
pub mod attester;
pub mod duty_gate;
pub mod error;
pub mod proposer;
pub mod scheduler;
pub mod signer;

pub use aggregator::{run_aggregator, AggregatorOutcome, AggregatorPlan};
pub use attester::{run_attester, AttesterOutcome, AttesterPlan};
pub use duty_gate::{evaluate_gate, DutyView, SuppressReason};
pub use error::{Result, SignerError};
pub use proposer::{run_proposer, ProposerOutcome, ProposerPlan};
pub use scheduler::{advance_tick, should_process, tick_from_elapsed_ms, DutyTick};
pub use signer::{
    record_from_keygen, InMemorySignerStore, KeyId, KeyRecord, ReservationStatus, Signer,
    SignerStore, SigningDuty, SigningRole, SigningRoot,
};
