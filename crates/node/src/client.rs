//! Lean Consensus client shell: profile, genesis, chain owner, sync status.

use crate::{
    chain_owner::ChainOwner,
    clock::{clock_from_genesis, SlotClock},
    duty_loop::{run_duty_loop, DutyLoopConfig},
    events::ChainEvent,
    observability::{smoke_health_route, NodeObservability},
    shutdown::ShutdownState,
    signal_loop::run_until_signal,
    start_config::{RunMode, StartConfig},
    wall_loop::{run_wall_duty_loop, WallLoopConfig},
    wall_tick::tick_from_wall,
    Error, Result, VERSION,
};
use ethean_genesis::{
    local_smoke_genesis, BuiltGenesis, FakeTime, GenesisBuilder, GenesisError,
};
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
    shutdown: ShutdownState,
    observability: NodeObservability,
}

impl EtheanClient {
    /// Start with an explicit pinned profile and verified genesis state.
    pub async fn with_genesis(profile: ChainProfile, genesis: State) -> Result<Self> {
        Self::with_genesis_store(profile, genesis, Database::open()?).await
    }

    /// Same as [`Self::with_genesis`] but with a caller-provided store handle.
    pub async fn with_genesis_store(
        profile: ChainProfile,
        genesis: State,
        db: Database,
    ) -> Result<Self> {
        require_lstar_fork(&profile)?;
        if genesis.validators.is_empty() {
            return Err(Error::Genesis(GenesisError::EmptyValidators));
        }

        let clock = clock_from_genesis(&genesis, profile.clone())?;
        let observability = NodeObservability::new(VERSION)?;

        info!(
            fork = profile.fork_name,
            seconds_per_slot = profile.seconds_per_slot,
            genesis_time = genesis.genesis_time(),
            validators = genesis.validators.len(),
            rocks = db.is_rocks(),
            "Loaded chain profile and genesis"
        );

        let mut owner = ChainOwner::new(32);
        owner.generation = 1;
        owner.head_state = Some(genesis.clone());

        Ok(Self {
            profile,
            genesis,
            clock,
            owner,
            db,
            sync: SyncStatus::new(Slot::new(0), Slot::new(0)),
            shutdown: ShutdownState::default(),
            observability,
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

    /// Smoke genesis with a path-backed store (`ethean_storage::open_path`).
    pub async fn open_data_dir(path: &str) -> Result<Self> {
        let profile = lstar_devnet()?;
        let built = local_smoke_genesis(1_700_000_000)?;
        let db = ethean_storage::open_path(path)?;
        Self::with_genesis_store(profile, built.state, db).await
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

    /// Shutdown tracker.
    pub fn shutdown(&self) -> &ShutdownState {
        &self.shutdown
    }

    /// Metrics / readiness view.
    pub fn observability(&self) -> &NodeObservability {
        &self.observability
    }

    /// Default smoke start (`StartConfig::default()`).
    pub async fn start(self) -> Result<()> {
        self.start_with(StartConfig::default()).await
    }

    /// Verify schema, smoke health, run the configured duty loop, record metrics.
    pub async fn start_with(mut self, cfg: StartConfig) -> Result<()> {
        self.boot_gates()?;
        let events = match cfg.mode {
            RunMode::SmokeElapsed { ticks } => run_duty_loop(
                &self.profile,
                &mut self.owner,
                &mut self.shutdown,
                &mut self.sync,
                DutyLoopConfig {
                    max_ticks: ticks,
                    start_elapsed_ms: 0,
                },
            ),
            RunMode::WallClock {
                ticks,
                enable_sleep,
            } => {
                run_wall_duty_loop(
                    &self.clock,
                    &mut self.owner,
                    &mut self.shutdown,
                    &mut self.sync,
                    WallLoopConfig {
                        max_ticks: ticks,
                        enable_sleep,
                    },
                )
                .await?
            }
            RunMode::UntilSignal { enable_sleep } => {
                run_until_signal(
                    &self.clock,
                    &mut self.owner,
                    &mut self.shutdown,
                    &mut self.sync,
                    enable_sleep,
                )
                .await?
            }
        };
        self.finish_observability(&events)
    }

    fn boot_gates(&mut self) -> Result<()> {
        self.db.verify_schema()?;
        self.observability.mark_storage_ok();
        self.observability.mark_signer_ok();
        self.observability
            .apply_ffi_status(ethean_crypto::FfiStatus::probe());
        // Smoke crypto path is loaded even when production FFI is off.
        self.observability.mark_crypto_ok();

        let identity = crate::network::NodeIdentity::from_seed(b"ethean-local");
        let bound = crate::network::prepare_transport(
            &identity,
            &crate::network::TransportConfig {
                listen_port: 0,
                idle_timeout_ms: 30_000,
            },
        )?;
        info!(port = bound.listen_port, "UDP listen bind for QUIC facade");
        self.observability.mark_network_ok();

        let health = smoke_health_route()?;
        info!(?health, "Lean health route smoke ok");

        let genesis_ms = self.clock.genesis_time_millis()?;
        let wall = tick_from_wall(
            &self.clock,
            &FakeTime::new(genesis_ms),
            self.owner.generation.max(1),
        )?;
        info!(
            slot = wall.slot.get(),
            interval = wall.interval,
            fork = self.profile.fork_name,
            ready = self.observability.readiness.is_ready(),
            ffi_leansig = ethean_crypto::FfiStatus::probe().leansig,
            ffi_leanvm = ethean_crypto::FfiStatus::probe().leanvm,
            "Ethean Lean Consensus client starting duties"
        );
        Ok(())
    }

    fn finish_observability(&mut self, events: &[ChainEvent]) -> Result<()> {
        let accepted = events
            .iter()
            .filter(|e| matches!(e, ChainEvent::TickAccepted(_)))
            .count();
        let head_slot = self.owner.last_tick.map(|t| t.slot.get()).unwrap_or(0);
        self.observability
            .record_chain(head_slot, self.sync.lag())?;
        self.observability.refresh_ready_gauge()?;
        info!(
            ticks_accepted = accepted,
            events = events.len(),
            ready = self.observability.readiness.is_ready(),
            "Duty loop finished"
        );
        Ok(())
    }
}
