//! Boot gates and observability finish hooks for [`EtheanClient`].

use crate::client::EtheanClient;
use crate::events::ChainEvent;
use crate::fork_digest_policy::ForkDigestSource;
use crate::observability::smoke_health_route;
use crate::wall_tick::tick_from_wall;
use crate::Result;
use ethean_genesis::FakeTime;
use tracing::{info, warn};

impl EtheanClient {
    pub(crate) async fn boot_gates(
        &mut self,
        network: &crate::network_target::NetworkTarget,
    ) -> Result<()> {
        self.db.verify_schema()?;
        self.observability.mark_storage_ok();
        self.observability.mark_signer_ok();
        self.observability
            .apply_ffi_status(ethean_crypto::FfiStatus::probe());
        let leanvm_gate = ethean_crypto::LeanVmGate::probe();
        let leansig_gate = ethean_crypto::LeanSigGate::probe();
        info!(
            network = network.id.as_str(),
            bootnodes = network.bootnodes.len(),
            fork_digest = network.fork_digest.as_deref().unwrap_or(""),
            leanvm_feature = leanvm_gate.feature_enabled,
            leanvm_ffi = leanvm_gate.ffi_linked,
            leanvm_ipc_binary = leanvm_gate.ipc_binary_present,
            leanvm_ipc_frame = leanvm_gate.ipc_frame_abi_ready,
            leanvm_ipc_ready = leanvm_gate.ipc_protocol_ready,
            leanvm_pin = leanvm_gate.pinned_rev,
            leansig_feature = leansig_gate.feature_enabled,
            leansig_pin = leansig_gate.pinned_rev,
            "crypto backend gate probe"
        );
        if network.has_bootnodes() && !leansig_gate.ready() {
            warn!(
                network = network.id.as_str(),
                pin = leansig_gate.pinned_rev,
                reason = leansig_gate.refuse_reason().unwrap_or("unavailable"),
                "mesh dial without production leanSig; peer XMSS verify stays fail-closed"
            );
        }
        if network.has_bootnodes() && !leanvm_gate.ready() {
            warn!(
                network = network.id.as_str(),
                pin = leanvm_gate.pinned_rev,
                reason = leanvm_gate.refuse_reason().unwrap_or("unavailable"),
                "mesh dial without production leanVM; Type-2 proof verify stays fail-closed"
            );
        }
        if let Some(stem) = network.id.config_stem() {
            let path = crate::agg_pin::aggpin_path_for_stem(stem);
            if let Some(pin) = crate::agg_pin::AggPin::load_file(&path) {
                for m in pin.mismatches() {
                    warn!(
                        network = network.id.as_str(),
                        field = m.field,
                        operator = %m.operator,
                        local = %m.local,
                        path = %path.display(),
                        "operator aggregation pin mismatches this build"
                    );
                }
                if pin.mismatches().is_empty()
                    && (pin.log_inv_rate.is_some()
                        || pin.leanvm_rev.is_some()
                        || pin.leansig_rev.is_some())
                {
                    info!(
                        network = network.id.as_str(),
                        path = %path.display(),
                        "operator aggregation pin matches local LOG_INV_RATE / leanVM / leanSig"
                    );
                }
            }
        }
        // Smoke crypto path is loaded even when production FFI is off.
        self.observability.mark_crypto_ok();

        let fork_segment = ethean_network_wire::fork_segment_resolve(
            self.profile.fork_name,
            network.fork_digest.as_deref(),
        )
        .map_err(|e| crate::Error::Config(format!("fork segment: {e}")))?;
        match network.fork_digest_source() {
            ForkDigestSource::OperatorOverride => {
                info!(%fork_segment, "resolved gossip fork segment from operator digest");
            }
            ForkDigestSource::InterimNameHash => {
                info!(%fork_segment, "resolved interim gossip fork segment (SHA-256 name hash)");
            }
        }
        if network.mesh_isolation_risk() {
            warn!(
                network = network.id.as_str(),
                %fork_segment,
                "dialing bootnodes without operator --fork-digest; Lean topics will not match peer mesh"
            );
        }

        let (_port, swarm) = crate::boot_network::prepare_boot_network(&fork_segment).await?;
        #[cfg(feature = "libp2p-quic")]
        {
            self.swarm = swarm;
            crate::boot_network::dial_bootnodes(network, self.swarm.as_mut());
        }
        #[cfg(not(feature = "libp2p-quic"))]
        {
            let _ = swarm;
            crate::boot_network::dial_bootnodes(network, None);
        }

        let genesis_root = self
            .genesis
            .hash_tree_root()
            .map_err(crate::Error::Types)?;
        let status =
            crate::local_status::local_status(&self.owner, genesis_root, &fork_segment);
        info!(
            head_slot = status.head_slot,
            finalized_slot = status.finalized_slot,
            fork = %status.fork_segment,
            "local Lean Status ready for peer handshake"
        );
        self.local_status = Some(status);

        #[cfg(feature = "libp2p-quic")]
        if let Some(facade) = self.swarm.as_mut() {
            if let Some(ref st) = self.local_status {
                if let Ok(bytes) = st.encode() {
                    let _ = facade.set_local_status_bytes(bytes);
                }
            }
        }

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
            fork_segment = %fork_segment,
            network = network.id.as_str(),
            ready = self.observability.readiness.is_ready(),
            ffi_leansig = ethean_crypto::FfiStatus::probe().leansig,
            ffi_leanvm = ethean_crypto::FfiStatus::probe().leanvm,
            "Ethean Lean Consensus client starting duties"
        );
        Ok(())
    }

    pub(crate) fn finish_observability(&mut self, events: &[ChainEvent]) -> Result<()> {
        let accepted = events
            .iter()
            .filter(|e| matches!(e, ChainEvent::TickAccepted(_)))
            .count();
        self.refresh_slot_metrics()?;
        info!(
            ticks_accepted = accepted,
            events = events.len(),
            ready = self.observability.readiness.is_ready(),
            "Duty loop finished"
        );
        Ok(())
    }
}
