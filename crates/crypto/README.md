# ethean-crypto

Lean Consensus XMSS wire surface and Type-1 / Type-2 aggregation for Ethean.

- PROD_CONFIG sizes: public key **52** bytes, signature **2536** bytes, dimension **46**, lifetime `2^32`.
- Aggregation: max proof **524288** bytes, `LOG_INV_RATE=2`, leanVM pin `e2592df4…`.
- Production backends fail closed unless `leansig-backend` / `leanvm-backend` compile.
- Default `test-hmac` / `test-aggregate` provide real (not always-true) verify for unit tests.
