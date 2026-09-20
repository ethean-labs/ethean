# Org profile README draft (`gitreadme.md`) — 2026-09-20

## What changed

Tightened the draft used for the GitHub organization profile README (`gitreadme.md`). The **Support** section and everything after it were left unchanged.

## Content

- Hero image: centered via `div align="center"`, width reduced from 484px to **220px** (portrait asset kept; no forced height).
- Tagline and section flow preserved (Who We Are → Focus → Guides → Path → Looking Ahead).
- **Our Focus** now names the concrete Lean Consensus targets: leanSig / XMSS-style signatures, leanMultisig / zkVM aggregation, 3SF and later heartbeat-style finality, ~4s slots / QUIC / Gossipsub, and lower stake floors / larger validator sets.
- Stated clearly that Ethean is a **Rust consensus client** (not execution) aligned with leanroadmap / pq-devnet work and client diversity — not a frozen Beacon clone.
- **What Guides Us** adds “Interop over imitation” (peer Lean clients + leanSpec pins; own architecture).
- **Our Path** mentions the arc from local finality and pq-devnet readiness toward mesh interop, without inventing unreleased claims.
- Reduced repetition between the experience narrative and the path/looking-ahead sections while keeping the long-horizon tone.

## Source of truth for claims

Aligned with root `README.md` Lean Consensus overview and project identity (consensus-only Rust client; leanroadmap research tracks).
