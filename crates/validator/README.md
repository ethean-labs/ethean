# ethean-validator

Validator **XMSS signer safety** (Phase 07) and **duty scheduling** (Phase 09) for Ethean Lean Consensus.

- Signer: reserve → flush → sign; role-separated keys; burn uncertain leaves
- Duties: profile-driven `(slot, interval, generation)` ticks; duty gate; attester / proposer / aggregator flows
- Networking and gossip publication arrive in Phase 10
