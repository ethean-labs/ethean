# Remote proposer sidecar verify on gossip ingest (2026-09-19)

## Goal

When gossip carries a proposer XMSS binding outside `SignedBlock.proof` (Sidecar policy), verify it against the registry `proposal_public_key` before STF.

## Landed

- `ethean-transition::verify_proposer_signature` — length, registry lookup, crypto verify via caller backend.
- `DecodedBlockGossip.proposer_signature: Option<Vec<u8>>` (decode paths leave it `None` until a sidecar wire format exists).
- `import_decoded_block` rejects when a sidecar is present and `ProductionBackend` verify fails.

## Behaviour

| Sidecar | Result |
| --- | --- |
| Absent | unchanged (structural / Type-2 paths) |
| Present + verify fail | `Rejected` (fail closed) |
| Present + verify ok | continue to Type-2 / structural STF |

## Tests

- `proposer_verify::tests::*` (HMAC round-trip / tamper)
- `gossip_stf::tests::rejects_bad_proposer_sidecar_when_state_present`

## Next

Define how the sidecar is framed on the wire (or flip Type-2 embed policy when leanMultisig pins encoding). Until then decode keeps `proposer_signature = None`.
