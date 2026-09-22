# ethean-multisig

leanMultisig aggregate proofs for Ethean, pinned to leanVM `e2592df4` (the
pq-devnet-4 revision ream, ethlambda and zeam link), so proofs are
byte-compatible across clients.

## Split between node and prover

| Where | What | Why |
| --- | --- | --- |
| Node process | `verify_single` / `verify_multi` (`LeanMultisigVerifier`) | ~30 ms per proof; needs only `setup_verifier` |
| `ethean-prover` process | `aggregate_type1`, `merge_type2`, `split_type2` | `setup_prover` disables glibc heap trimming and large-block mmap process-wide, precomputes large DFT tables, and leanVM forbids concurrent proofs in one process |

`ProverClient` supervises the child: one request in flight, per-request
timeout, revision handshake (`Pong` must carry the same leanVM revision), and
kill-and-respawn on timeout or a broken pipe. The wire protocol (`protocol.rs`)
is a length-prefixed binary frame with every count and length bounded before
allocation.

## Hardening around leanMultisig

- **Bounded decompression.** `decompress_without_pubkeys` trusts the LZ4 size
  prefix. A 512 KiB gossip proof could otherwise request gigabytes; the prefix
  is capped at 8x the proof length before the call.
- **Panic and stack isolation.** Every leanMultisig call runs on a scoped
  thread with a 256 MiB reserved stack. The verifier and bytecode compiler
  recurse deeply, and the library asserts on some malformed inputs; a hostile
  proof is rejected instead of crashing the node. This needs
  `panic = "unwind"` in the release profile.
- **Binding checks.** Each component's `(message, slot)` inside the proof must
  equal the consensus-derived binding, and the component count must match.
- **Limits.** Proofs 1..=512 KiB, at most 16 components or children, at most
  2^15 keys per component, slots below 2^32.

## Tests

```bash
cargo build --release -p ethean-prover
cargo test --release -p ethean-multisig -p ethean-prover
cargo test --release -p ethean-node --test aggregation_flow
```

The end-to-end tests read the PROD keys in `../leansig-test-keys/prod_scheme`
(next to the workspace) and skip when it is absent.
