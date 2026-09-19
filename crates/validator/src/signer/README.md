# Validator signer

Role-separated durable signing for Lean Consensus XMSS keys.

- Reservation is flushed before any signature leaves the process.
- Exact retries are idempotent; conflicting roots for the same role/slot are rejected.
- Uncertain leaves after crash are burned; counters never rewind.
- Secrets have redacted `Debug`.

Crypto verify uses `ethean-crypto` (test-hmac in unit tests; production fails closed until leanSig backend builds).
