//! Lean Consensus client shell: profile, genesis, chain owner, sync status.

use crate::{
    chain_owner::ChainOwner,
    clock::{clock_from_genesis, SlotClock},
    duty_loop::{run_duty_loop, DutyLoopConfig},
    observability::NodeObservability,
    shutdown::ShutdownState,
    start_config::{RunMode, StartConfig},
    Error, Result, VERSION,
};
use ethean_genesis::{local_smoke_genesis, BuiltGenesis, GenesisBuilder, GenesisError};
use ethean_network::StatusSessionBook;
use ethean_network_wire::Status;
use ethean_primitives::Slot;
use ethean_profile::{lstar_devnet, require_lstar_fork, ChainProfile};
use ethean_storage::Database;
use ethean_sync::SyncStatus;
use ethean_types::State;
use ethean_validator::SYNC_LAG_THRESHOLD_SLOTS;
use tracing::info;

/// Main Ethean Lean Consensus client.
pub struct EtheanClient {
    pub(crate) profile: ChainProfile,
    pub(crate) genesis: State,
    pub(crate) clock: SlotClock,
    /// Owns chain mutation; workers only read snapshots / send commands.
    pub(crate) owner: ChainOwner,
    pub(crate) db: Database,
    pub(crate) sync: SyncStatus,
    pub(crate) shutdown: ShutdownState,
    pub(crate) observability: NodeObservability,
    /// Pending / completed Lean Status handshakes.
    pub(crate) status_sessions: StatusSessionBook,
    /// Last advertised local Status (set during boot).
    pub(crate) local_status: Option<Status>,
    /// Durable libp2p QUIC facade (feature `libp2p-quic`).
    #[cfg(feature = "libp2p-quic")]
    pub(crate) swarm: Option<crate::network::SwarmFacade>,
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

        let mut owner = ChainOwner::new(SYNC_LAG_THRESHOLD_SLOTS);
        owner.generation = 1;
        owner.head_state = Some(genesis.clone());
        owner.profile = Some(profile.clone());
        match crate::local_proposer::LocalProposer::prefer_production() {
            Ok(prop) => {
                let production = prop.is_production();
                owner.proposer = Some(prop);
                info!(production, "Local proposer key loaded");
            }
            Err(e) => {
                info!(error = %e, "Local proposer unavailable; proposals stay unsigned");
            }
        }

        Ok(Self {
            profile,
            genesis,
            clock,
            owner,
            db,
            sync: SyncStatus::new(Slot::new(0), Slot::new(0)),
            shutdown: ShutdownState::default(),
            observability,
            status_sessions: StatusSessionBook::default(),
            local_status: None,
            #[cfg(feature = "libp2p-quic")]
            swarm: None,
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
        self.boot_gates(&cfg.network).await?;
        #[cfg(feature = "libp2p-quic")]
        {
            self.boot_pump_status_and_gossip().await?;
        }
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
            } => self.run_wall_with_flush(ticks, enable_sleep).await?,
            RunMode::UntilSignal { enable_sleep } => {
                self.run_until_signal_with_flush(enable_sleep).await?
            }
        };
        self.finish_observability(&events)
    }
}
