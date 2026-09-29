# XMSS window prepared in the background (2026-09-29)

## Problem

A native XMSS secret key keeps two PROD bottom trees (2 × 65536 epochs) in
memory. When a signing epoch left that window, `ProductionBackend::sign` called
`prepare_for_epoch` while holding the key lock and built the next bottom tree
inline. Measured in release on this machine:

- one PROD bottom tree takes **10.4 s** on all cores
  (`timing_probe_prod_bottom_tree`, ignored by default).

In lean one epoch is one slot, so this happens about every 65536 slots (roughly
three days at 4 s slots). Each time, the attestation and proposal signatures
waited around 2.5 slots, and the build competed with the leanVM prover for
every core.

## What changed

| Piece | Change |
| --- | --- |
| `native::pending_bottom_tree` | Once `epoch` reaches the right half of the window, returns the next tree index. The left tree then holds only past epochs, which slashing protection never signs again |
| `native::install_bottom_tree` | Slides the window onto a tree built elsewhere. A tree built for a window that has since moved is dropped |
| `SecretKeyMaterial::prepare_ahead(epoch, max_workers)` | Reads the PRF key and parameter under the lock, builds the tree without the lock, then installs it under the lock. Loops until nothing is pending |
| `native::parallel::with_worker_cap` | Thread-local fan-out cap, so background builds use fewer cores |
| `key_prep::KeyPreparer` | Tracks installed registry keys (attestation and proposal). On interval 0 of each slot it reaps a finished worker and starts one `xmss-prepare` thread when any key is due; it uses a quarter of the cores |

The signing path keeps `prepare_for_epoch` as a fallback, for example right after
a restart deep into a key's lifetime before the worker catches up.

Timeline: the build starts when the slot enters the right half of the window, and
the tree is not needed for another 65536 slots. A slow, capped build therefore
costs nothing on the duty path.

## Tests

- `window_slides_on_a_tree_built_off_the_key` (TEST scheme): nothing is pending
  in the left half; the tree is built off the key and installed; a stale install
  is dropped; signing works without a signing-path build; nothing is pending at
  the activation end.
- `worker_cap_is_scoped_to_the_call`
- `empty_preparer_never_spawns`

```bash
cargo test -p ethean-crypto --lib xmss
cargo test -p ethean-crypto --release --lib -- --ignored prod_bottom_tree --nocapture
```
