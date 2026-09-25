//! `EtheanClient::start_with` — metrics/HTTP bind, boot gates, duty loop.

use crate::client::EtheanClient;
use crate::duty_loop::{run_duty_loop, DutyLoopConfig};
use crate::start_config::{RunMode, StartConfig};
use crate::{Error, Result};
use tracing::info;

impl EtheanClient {
    /// Default smoke start (`StartConfig::default()`).
    pub async fn start(self) -> Result<()> {
        self.start_with(StartConfig::default()).await
    }

    /// Verify schema, smoke health, run the configured duty loop, record metrics.
    pub async fn start_with(mut self, cfg: StartConfig) -> Result<()> {
        self.apply_local_roles(cfg.roles);
        self.bootnode_count = cfg.network.bootnodes.len() as u64;
        self.network_label = cfg.network.id.as_str().to_string();
        self.prune_keep_slots = cfg.prune_keep_slots;
        if !cfg.aggregate_subnet_ids.is_empty() {
            info!(
                ids = ?cfg.aggregate_subnet_ids,
                "aggregate-subnet-ids recorded (Ethean already subscribes to every attestation subnet)"
            );
        }
        bind_listeners(&mut self, &cfg).await?;
        self.boot_gates(&cfg.network, cfg.listen_port, &cfg.listen_identity())
            .await?;
        if let Some(ref url) = cfg.checkpoint_sync_url {
            let url = url.clone();
            match tokio::task::spawn_blocking(move || {
                crate::checkpoint_sync::fetch_checkpoint(&url)
            })
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r)
            {
                Ok(anchor) => {
                    if crate::checkpoint_sync::apply_anchor(&mut self.owner, anchor) {
                        self.flush_chain_persist();
                    }
                }
                Err(error) => {
                    // leanSpec: startup aborts rather than falling back to genesis.
                    return Err(crate::Error::Config(format!(
                        "checkpoint sync failed: {error}"
                    )));
                }
            }
        }
        capture_peer_id(&mut self, &cfg);
        self.refresh_slot_metrics()?;
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
        self.finish_observability(&events)?;
        self.flush_chain_persist();
        Ok(())
    }
}

async fn bind_listeners(client: &mut EtheanClient, cfg: &StartConfig) -> Result<()> {
    if let Some(ref metrics) = cfg.metrics {
        let bound = ethean_metrics::spawn_metrics_server(
            metrics.addr,
            client.observability.registry.clone(),
            client.observability.ready_flag.clone(),
        )
        .await
        .map_err(|e| Error::Config(format!("metrics bind {}: {e}", metrics.addr)))?;
        info!(%bound, "Prometheus scrape endpoint ready");
    }
    if let Some(ref http) = cfg.http {
        let state = ethean_rpc::SharedApiState::new(http.admin_token.clone());
        state.set_aggregator(client.owner.is_aggregator);
        if crate::test_driver::enabled_by_env() {
            state.install_driver(std::sync::Arc::new(crate::test_driver::NodeTestDriver::default()));
            info!("hive test driver enabled (HIVE_LEAN_TEST_DRIVER=1)");
        }
        let bound = ethean_rpc::spawn_lean_http(http.addr, state.clone())
            .await
            .map_err(|e| Error::Config(format!("lean HTTP bind {}: {e}", http.addr)))?;
        client.api = Some(state);
        info!(%bound, "Lean HTTP API ready");
    }
    Ok(())
}

fn capture_peer_id(client: &mut EtheanClient, cfg: &StartConfig) {
    #[cfg(feature = "libp2p-quic")]
    {
        if let Some(ref facade) = client.swarm {
            if let Some(q) = facade.quic.as_ref() {
                client.peer_id = q.peer_id.to_string();
            }
        }
    }
    if client.peer_id.is_empty() {
        if let Some(ref key) = cfg.node_key {
            if let Some(id) = crate::cli_resolve::node_key_peer_id(key) {
                client.peer_id = id;
            }
        }
    }
}
