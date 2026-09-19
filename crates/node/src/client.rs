//! Lean Consensus client shell: profile, genesis, chain owner, sync status.

use crate::{
    chain_owner::ChainOwner,
    clock::{clock_from_genesis, SlotClock},
    Error, Result,
};
use ethean_genesis::{local_smoke_genesis, BuiltGenesis, GenesisBuilder, GenesisError};
use ethean_profile::{lstar_devnet, require_lstar_fork, ChainProfile};
use ethean_primitives::Slot;
use ethean_storage::Database;
use ethean_sync::SyncStatus;
use ethean_types::State;
use tracing::info;

/// Main Ethean Lean Consensus client.
pub struct EtheanClient {
    profile: ChainProfile,
    genesis: State,
    clock: SlotClock,
    owner: ChainOwner,
    db: Database,
    sync: SyncStatus,
}

impl EtheanClient {
    /// Start with an explicit pinned profile and verified genesis state.
    pub async fn with_genesis(profile: ChainProfile, genesis: State) -> Result<Self> {
        require_lstar_fork(&profile)?;
        if genesis.validators.is_empty() {
            return Err(Error::Genesis(GenesisError::EmptyValidators));
        }

        let clock = clock_from_genesis(&genesis, profile.clone())?;
        let db = Database::open()?;

        info!(
            fork = profile.fork_name,
            seconds_per_slot = profile.seconds_per_slot,
            genesis_time = genesis.genesis_time(),
            validators = genesis.validators.len(),
            "Loaded chain profile and genesis"
        );

        Ok(Self {
            profile,
            genesis,
            clock,
            owner: ChainOwner::new(32),
            db,
            sync: SyncStatus::new(Slot::new(0), Slot::new(0)),
        })
    }

    /// Build genesis via [`GenesisBuilder`], then start.
    pub async fn from_builder(profile: ChainProfile, builder: GenesisBuilder) -> Result<Self> {
        let BuiltGenesis { state, state_root } = builder.build()?;
        info!(?state_root, "Genesis built");
        Self::with_genesis(profile, state).await
    }

    /// Local smoke start: `lstar_devnet` + single zero-key validator genesis.
    pub async fn new() -> Result<Self> {
        let profile = lstar_devnet()?;
        let built = local_smoke_genesis(1_700_000_000)?;
        Self::with_genesis(profile, built.state).await
    }

    /// Access the chain owner (sole writer of head/sync flags).
    pub fn owner(&self) -> &ChainOwner {
        &self.owner
    }

    /// Mutable chain owner for command application.
    pub fn owner_mut(&mut self) -> &mut ChainOwner {
        &mut self.owner
    }

    /// Pinned profile.
    pub fn profile(&self) -> &ChainProfile {
        &self.profile
    }

    /// Genesis state.
    pub fn genesis(&self) -> &State {
        &self.genesis
    }

    /// Slot clock.
    pub fn clock(&self) -> &SlotClock {
        &self.clock
    }

    /// Durable store handle.
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Sync status view.
    pub fn sync(&self) -> &SyncStatus {
        &self.sync
    }

    /// Smoke start: verify schema and log readiness (no Beacon HTTP).
    pub async fn start(self) -> Result<()> {
        self.db.verify_schema()?;
        info!(
            fork = self.profile.fork_name,
            syncing = self.owner.syncing,
            "Ethean Lean Consensus client ready"
        );
        Ok(())
    }
}
