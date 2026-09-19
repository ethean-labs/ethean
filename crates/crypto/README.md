# ethean-crypto

Lean Consensus XMSS wire surface for Ethean.

- PROD_CONFIG sizes: public key **52** bytes, signature **2536** bytes, dimension **46**, lifetime `2^32`.
- Production backend fails closed unless `leansig-backend` compiles against pinned leanSig.
- Default feature `test-hmac` provides a real (not always-true) verify for unit tests and signer journal tests.

Aggregate proof APIs are deferred to Phase 08.
