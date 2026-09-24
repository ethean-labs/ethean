//! Status exchange against local chain identity.

use ethean_network_wire::Status;

use crate::error::{NetworkError, Result};

/// Local and remote Status after a successful compatibility check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusExchange {
    pub local: Status,
    pub remote: Status,
}

/// Accept remote Status; keep head claims untrusted.
pub fn handle_status(local: &Status, remote: Status) -> Result<StatusExchange> {
    remote
        .compatible_with(local)
        .map_err(|e| NetworkError::Handshake(e.to_string()))?;
    Ok(StatusExchange {
        local: local.clone(),
        remote,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_network_wire::Checkpoint;

    #[test]
    fn accepts_distinct_heads() {
        let local = Status {
            finalized: Checkpoint {
                root: [1u8; 32],
                slot: 0,
            },
            head: Checkpoint {
                root: [2u8; 32],
                slot: 1,
            },
        };
        let remote = Status {
            finalized: Checkpoint {
                root: [1u8; 32],
                slot: 0,
            },
            head: Checkpoint {
                root: [9u8; 32],
                slot: 20,
            },
        };
        let ex = handle_status(&local, remote.clone()).unwrap();
        assert_eq!(ex.remote.head_slot(), 20);
    }
}
