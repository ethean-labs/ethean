//! Durable `--data-dir` open / resume / flush for [`EtheanClient`].

use crate::chain_persist::{self, PersistPaths};
use crate::client::EtheanClient;
use crate::start_config::LocalRoles;
use crate::Result;
use ethean_profile::lstar_devnet;
use ethean_storage::Database;
use ethean_types::State;
use std::path::PathBuf;
use tracing::{info, warn};

impl EtheanClient {
    /// Path-backed durable mode: fixed genesis + head resume.
    pub async fn open_data_dir(path: &str) -> Result<Self> {
        Self::open_data_dir_with_roles(path, LocalRoles::default()).await
    }

    /// Same as [`Self::open_data_dir`] with explicit local roles.
    pub async fn open_data_dir_with_roles(path: &str, roles: LocalRoles) -> Result<Self> {
        let profile = lstar_devnet()?;
        let paths = PersistPaths::new(path);
        let genesis = chain_persist::open_or_init(&paths, &profile, roles.validators)?;
        let db = open_store(path)?;
        let mut client =
            Self::with_genesis_store(profile.clone(), genesis.state.clone(), db).await?;
        client.resume_head(&paths, &genesis.state)?;
        client.persist_dir = Some(PathBuf::from(path));
        client.apply_local_roles(LocalRoles {
            validators: genesis.validators,
            ..roles
        });
        info!(
            data_dir = path,
            genesis_time = genesis.genesis_time,
            "durable data-dir mode ready"
        );
        Ok(client)
    }

    /// Durable mode on a genesis from the network (`config.yaml`): pin that
    /// genesis under `path`, resume the head stored there, and flush from now
    /// on. Without this a restarted mesh node starts again from genesis.
    pub fn attach_network_data_dir(&mut self, path: &str, genesis: &State) -> Result<()> {
        let paths = PersistPaths::new(path);
        chain_persist::pin_network_genesis(&paths, &self.profile, genesis)?;
        self.resume_head(&paths, genesis)?;
        self.persist_dir = Some(PathBuf::from(path));
        info!(
            data_dir = path,
            "durable data-dir attached to network genesis"
        );
        Ok(())
    }

    fn resume_head(&mut self, paths: &PersistPaths, genesis: &State) -> Result<()> {
        let Some(head) = chain_persist::load_head(paths)? else {
            return Ok(());
        };
        // Drop the genesis-only store so restore does not sync the tip back
        // to the genesis FC head.
        self.owner.fc = None;
        chain_persist::restore_owner(&mut self.owner, head)?;
        let blobs = crate::serve_cache_seed::load_persisted_block_blobs(paths);
        let profile = self.profile.clone();
        self.owner
            .rebuild_fork_choice_from_durable(genesis.clone(), &profile, &blobs);
        if self.owner.fc.is_none() {
            self.owner.try_init_fork_choice();
        }
        Ok(())
    }

    /// Write SSZ + `ethean.redb` when durable mode is active.
    pub(crate) fn flush_chain_persist(&mut self) {
        let Some(ref dir) = self.persist_dir else {
            return;
        };
        let head = self
            .owner
            .head_state
            .as_ref()
            .map(|s| (self.owner.head_root, s.latest_finalized.slot.get()));
        let unchanged = head.is_some()
            && self.persisted_head == head
            && self.owner.durable_blocks.is_empty()
            && self.owner.pending_block_gossip.is_none();
        if unchanged {
            return;
        }
        let paths = PersistPaths::new(dir.clone());
        let flushed = self.owner.durable_blocks.len() as u64;
        match chain_persist::save_head(&paths, &self.owner) {
            Ok(()) => {
                self.owner.clear_durable_blocks();
                self.persisted_head = head;
                let finalized = head.map(|(_, slot)| slot).unwrap_or(0);
                let floor = crate::block_prune::prune_floor(finalized, self.prune_keep_slots);
                let (files_removed, redb_removed) =
                    if crate::block_prune::prune_due(self.pruned_floor, floor) {
                        match crate::block_prune::prune_below_floor(&paths, floor) {
                            Ok(report) => {
                                self.pruned_floor = floor;
                                (report.files_removed as u64, report.redb_removed as u64)
                            }
                            Err(e) => {
                                warn!(error = %e, "durable block prune failed");
                                (0, 0)
                            }
                        }
                    } else {
                        (0, 0)
                    };
                if let Err(e) = self.observability.record_durable_persist(
                    flushed,
                    floor,
                    files_removed,
                    redb_removed,
                    self.prune_keep_slots,
                ) {
                    warn!(error = %e, "durable persist metrics failed");
                }
            }
            Err(e) => warn!(error = %e, "failed to flush durable chain files"),
        }
    }
}

fn open_store(path: &str) -> Result<Database> {
    match ethean_storage::open_path(path) {
        Ok(db) => {
            info!(path, rocks = db.is_rocks(), "opened path-backed store");
            Ok(db)
        }
        Err(e) => {
            warn!(
                path,
                error = %e,
                "optional path store unavailable; ethean.redb and SSZ files still persist"
            );
            Ok(Database::open()?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn flush_skips_an_unchanged_head() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let mut client = EtheanClient::open_data_dir(path).await.unwrap();
        let paths = PersistPaths::new(path);

        client.flush_chain_persist();
        assert!(paths.state_ssz().exists());
        std::fs::remove_file(paths.state_ssz()).unwrap();

        client.flush_chain_persist();
        assert!(!paths.state_ssz().exists());

        client.owner.head_root = [5u8; 32];
        client.flush_chain_persist();
        assert!(paths.state_ssz().exists());
    }
}
