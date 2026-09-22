# aggregation

Node-side pools feeding leanMultisig aggregation.

- `AttestationSignaturePool`: verified individual `SignedAttestation` votes,
  grouped by attestation data root, kept only by aggregators and bounded.
- `AggregatePool`: verified Type-1 proofs (from aggregation gossip or local
  aggregation) keyed by attestation data root; block bodies draw only from it.
- Every proof is verified in-process (`ethean-multisig`) before it enters the
  pool. Proving never runs on the chain-owner task: `proof_service` owns the
  `ethean-prover` child process, and `aggregation_duty` / `duty_propose`
  submit jobs and collect results at the start of each step.
