//! Apply [`ChainCommand`] to the chain owner and shutdown state.

use crate::chain_owner::ChainOwner;
use crate::commands::ChainCommand;
use crate::events::ChainEvent;
use crate::shutdown::ShutdownState;

/// Dispatch one command; returns the observer event.
pub fn apply_command(
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    cmd: ChainCommand,
) -> ChainEvent {
    match cmd {
        ChainCommand::Tick(tick) => {
            if !shutdown.accepts_new_duties() {
                return ChainEvent::ShutdownComplete;
            }
            if owner.on_tick(tick) {
                ChainEvent::TickAccepted(tick)
            } else {
                ChainEvent::TickDuplicate(tick)
            }
        }
        ChainCommand::ImportBlock { root, parent } => {
            if shutdown.phase() == crate::shutdown::ShutdownPhase::Stopped {
                return ChainEvent::ShutdownComplete;
            }
            if parent != owner.head_root {
                return ChainEvent::HeadUpdated {
                    root: owner.head_root,
                    slot: owner.last_tick.map(|t| t.slot.get()).unwrap_or(0),
                };
            }
            owner.head_root = root;
            ChainEvent::HeadUpdated {
                root,
                slot: owner.last_tick.map(|t| t.slot.get()).unwrap_or(0),
            }
        }
        ChainCommand::SetSyncing(syncing) => {
            owner.syncing = syncing;
            ChainEvent::SyncingUpdated(syncing)
        }
        ChainCommand::Shutdown => {
            shutdown.begin_drain();
            shutdown.finish();
            ChainEvent::ShutdownComplete
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;
    use ethean_validator::DutyTick;

    #[test]
    fn tick_then_shutdown() {
        let mut owner = ChainOwner::new(2);
        let mut shutdown = ShutdownState::default();
        let tick = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 1,
        };
        assert!(matches!(
            apply_command(&mut owner, &mut shutdown, ChainCommand::Tick(tick)),
            ChainEvent::TickAccepted(_)
        ));
        assert_eq!(
            apply_command(&mut owner, &mut shutdown, ChainCommand::Shutdown),
            ChainEvent::ShutdownComplete
        );
        assert!(!shutdown.accepts_new_duties());
    }

    #[test]
    fn import_requires_parent() {
        let mut owner = ChainOwner::new(2);
        owner.head_root = [1u8; 32];
        let mut shutdown = ShutdownState::default();
        let ev = apply_command(
            &mut owner,
            &mut shutdown,
            ChainCommand::ImportBlock {
                root: [2u8; 32],
                parent: [9u8; 32],
            },
        );
        assert_eq!(owner.head_root, [1u8; 32]);
        assert!(matches!(ev, ChainEvent::HeadUpdated { root, .. } if root == [1u8; 32]));
        let ev = apply_command(
            &mut owner,
            &mut shutdown,
            ChainCommand::ImportBlock {
                root: [2u8; 32],
                parent: [1u8; 32],
            },
        );
        assert_eq!(owner.head_root, [2u8; 32]);
        assert!(matches!(ev, ChainEvent::HeadUpdated { root, .. } if root == [2u8; 32]));
    }
}
