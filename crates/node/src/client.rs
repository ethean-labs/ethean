//! Lean Consensus client shell: profile, genesis, chain owner, sync status.

use crate::{
    chain_owner::ChainOwner,
    clock::{clock_from_genesis, SlotClock},
    observability::NodeObservability,
    shutdown::ShutdownState,
    Error, Result, VERSION,
};
use ethean_genesis::{BuiltGenesis, GenesisBuilder, GenesisError};
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
    /// When set, flush head snapshots under this directory after duty steps.
    pub(crate) persist_dir: Option<std::path::PathBuf>,
    /// Bootnode multiaddrs configured for this start (Grafana mesh expectation).
    pub(crate) bootnode_count: u64,
    /// Shared Lean HTTP API snapshot (set when `--http` listener is enabled).
    pub(crate) api: Option<std::sync::Arc<ethean_rpc::SharedApiState>>,
    /// Network label mirrored into `/lean/v1/node/identity`.
    pub(crate) network_label: String,
    /// libp2p PeerId (base58) once the swarm is bound; empty until then.
    pub(crate) peer_id: String,
    /// Slots retained below finalized before durable prune (see `--prune-keep-slots`).
    pub(crate) prune_keep_slots: u64,
    /// Set once the pre-genesis wait has been logged (one line, not per round).
    pub(crate) pre_genesis_logged: bool,
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
        crate::local_finality::seal_genesis_head(&mut owner);
        owner.try_init_fork_choice();
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
            persist_dir: None,
            bootnode_count: 0,
            api: None,
            network_label: String::new(),
            peer_id: String::new(),
            prune_keep_slots: crate::block_prune::KEEP_BELOW_FINALIZED,
            pre_genesis_logged: false,
        })
    }

    /// Build genesis via [`GenesisBuilder`], then start.
    pub async fn from_builder(profile: ChainProfile, builder: GenesisBuilder) -> Result<Self> {
        let BuiltGenesis { state, state_root } = builder.build()?;
        info!(?state_root, "Genesis built");
        Self::with_genesis(profile, state).await
    }

    /// Local smoke start: recent multi-validator genesis (4 validators).
    pub async fn new() -> Result<Self> {
        Self::with_local_roles(crate::start_config::LocalRoles::default()).await
    }

    /// Recent genesis sized by [`LocalRoles::validators`].
    pub async fn with_local_roles(roles: crate::start_config::LocalRoles) -> Result<Self> {
        let profile = lstar_devnet()?;
        let built =
            crate::local_genesis::local_devnet_genesis(roles.validators, profile.seconds_per_slot)?;
        let mut client = Self::with_genesis(profile, built.state).await?;
        client.apply_local_roles(roles);
        Ok(client)
    }

    /// Apply aggregator / local-finality flags after construction.
    pub fn apply_local_roles(&mut self, roles: crate::start_config::LocalRoles) {
        self.owner.is_aggregator = roles.is_aggregator;
        self.owner.local_finality = roles.local_finality;
        if roles.is_aggregator && !roles.local_finality {
            self.ensure_prover();
        }
        info!(
            validators = roles.validators,
            is_aggregator = roles.is_aggregator,
            local_finality = roles.local_finality,
            "local roles applied"
        );
    }

    /// Start the leanMultisig proof service once, when a prover binary exists.
    pub fn ensure_prover(&mut self) {
        if self.owner.prover.is_some() {
            return;
        }
        self.owner.prover = crate::proof_service::ProofService::discover();
        if self.owner.prover.is_none() {
            tracing::warn!(
                "no ethean-prover binary found (set ETHEAN_PROVER_BIN); \
                 aggregates and block proofs will not be produced"
            );
        }
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

    /// libp2p PeerId string after swarm bind (empty when identity is unknown).
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }
}
