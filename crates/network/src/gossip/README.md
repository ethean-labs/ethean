# gossip

Application validation and Lean topic helpers for gossipsub.

- Topics: `/leanconsensus/{fork}/…/ssz_snappy` via [`LeanGossipTopics`](topics.rs)
- Codec: raw Snappy encode/decode
- Validation: ACCEPT / IGNORE / REJECT (rejects `/eth2/`)
- Transport mesh: `QuicSwarm::bind_for_fork` + `publish_gossip` / `pump_once`
