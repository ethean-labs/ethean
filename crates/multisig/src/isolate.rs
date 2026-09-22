//! Isolation for leanMultisig calls on attacker-controlled bytes.
//!
//! Each call runs on a scoped thread with a large explicit stack: the verifier
//! and prover recurse deeply (debug builds overflow the default 2-8 MiB
//! stacks), and a thread boundary also contains panics. The library asserts on
//! some malformed inputs; a crafted gossip proof must be rejected, not take the
//! node down. The stack is reserved lazily, so the cost is one thread spawn
//! (tens of microseconds) against a verification of tens of milliseconds.

use crate::error::{MultisigError, Result};

/// Stack reserved for one leanMultisig call.
pub const CALL_STACK_BYTES: usize = 256 * 1024 * 1024;

/// Run `f` on an isolated large-stack thread; a panic becomes
/// [`MultisigError::Panicked`].
pub fn guarded<T: Send>(f: impl FnOnce() -> Result<T> + Send) -> Result<T> {
    std::thread::scope(|scope| {
        let handle = std::thread::Builder::new()
            .name("leanmultisig-call".into())
            .stack_size(CALL_STACK_BYTES)
            .spawn_scoped(scope, f)
            .map_err(|e| MultisigError::ProverFailed(format!("spawn: {e}")))?;
        handle.join().unwrap_or(Err(MultisigError::Panicked))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panics_become_errors() {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result: Result<()> = guarded(|| panic!("boom"));
        std::panic::set_hook(hook);
        assert_eq!(result, Err(MultisigError::Panicked));
    }

    #[test]
    fn deep_recursion_fits() {
        fn depth(n: u64) -> u64 {
            let pad = [n; 64];
            if n == 0 {
                0
            } else {
                depth(n - 1) + pad[0] % 2
            }
        }
        assert!(guarded(|| Ok(depth(100_000))).is_ok());
    }
}
