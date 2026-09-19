//! Domain-separation tags for digests (not Poseidon production claims).

/// Domain tag for signing-root hashing in tests and helpers.
pub const DOMAIN_SIGNING_ROOT: &[u8] = b"ethean-crypto/v1/signing-root";

/// Domain tag for key fingerprint material.
pub const DOMAIN_KEY_FINGERPRINT: &[u8] = b"ethean-crypto/v1/key-fingerprint";

/// Domain tag for test-scale HMAC scheme (cfg(test) / fallback only).
pub const DOMAIN_TEST_SCHEME: &[u8] = b"ethean-crypto/v1/test-hmac-xmss";

/// Domain tag for signature-hash journal entries.
pub const DOMAIN_SIG_HASH: &[u8] = b"ethean-crypto/v1/sig-hash";
