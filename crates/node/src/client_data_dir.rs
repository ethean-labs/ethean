//! Durable `--data-dir` open / resume / flush for [`EtheanClient`].

use crate::chain_persist::{self, PersistPaths};
use crate::client::EtheanClient;
use crate::start_config::LocalRoles;
use crate::Result;
use ethean_profile::lstar_devnet;
use ethean_storage::Database;
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
        let mut client = Self::with_genesis_store(profile, genesis.state, db).await?;
        if let Some(head) = chain_persist::load_head(&paths)? {
            chain_persist::restore_owner(&mut client.owner, head)?;
        }
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

    /// Write SSZ + `ethean.redb` when durable mode is active.
    pub(crate) fn flush_chain_persist(&mut self) {
        let Some(ref dir) = self.persist_dir else {
            return;
        };
        let paths = PersistPaths::new(dir.clone());
        if let Err(e) = chain_persist::save_head(&paths, &self.owner) {
            warn!(error = %e, "failed to flush durable chain files");
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
