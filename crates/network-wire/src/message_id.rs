//! Gossipsub message-ID (leanSpec / Ethereum consensus P2P).

use sha2::{Digest, Sha256};

/// 4-byte domain when Snappy decompression of gossip data failed.
pub const MESSAGE_DOMAIN_INVALID_SNAPPY: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

/// 4-byte domain when Snappy decompression of gossip data succeeded.
pub const MESSAGE_DOMAIN_VALID_SNAPPY: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

/// 20-byte gossipsub message identifier.
pub type MessageId = [u8; 20];

/// Compute `SHA256(domain ‖ uint64_le(len(topic)) ‖ topic ‖ data)[:20]`.
pub fn compute_message_id(topic: &[u8], data: &[u8], domain: [u8; 4]) -> MessageId {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update((topic.len() as u64).to_le_bytes());
    hasher.update(topic);
    hasher.update(data);
    let digest = hasher.finalize();
    let mut out = [0u8; 20];
    out.copy_from_slice(&digest[..20]);
    out
}

/// Message-id for valid-snappy gossip (`data` is the decompressed payload).
pub fn message_id_valid_snappy(topic: &str, decompressed: &[u8]) -> MessageId {
    compute_message_id(
        topic.as_bytes(),
        decompressed,
        MESSAGE_DOMAIN_VALID_SNAPPY,
    )
}

/// Message-id for invalid-snappy gossip (`data` is the raw on-wire payload).
pub fn message_id_invalid_snappy(topic: &str, raw: &[u8]) -> MessageId {
    compute_message_id(topic.as_bytes(), raw, MESSAGE_DOMAIN_INVALID_SNAPPY)
}

/// Compute the gossip message-id, attempting raw Snappy decompression first.
///
/// On success the valid-snappy domain is used with the decompressed bytes;
/// otherwise the invalid-snappy domain is used with the raw bytes.
pub fn message_id_from_raw(topic: &str, raw: &[u8]) -> MessageId {
    match crate::snappy::decompress_raw(raw) {
        Ok(plain) => message_id_valid_snappy(topic, &plain),
        Err(_) => message_id_invalid_snappy(topic, raw),
    }
}

/// Back-compat alias: valid-snappy id over `topic` + `data`.
pub fn message_id(topic: &str, data: &[u8]) -> ethean_primitives::Hash32 {
    let id = message_id_valid_snappy(topic, data);
    let mut out = [0u8; 32];
    out[..20].copy_from_slice(&id);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn leanspec_message_id_vectors() {
        let topic = b"/leanconsensus/12345678/block/ssz_snappy";
        let deadbeef = [0xde, 0xad, 0xbe, 0xef];
        assert_eq!(
            hex(&compute_message_id(topic, &deadbeef, MESSAGE_DOMAIN_VALID_SNAPPY)),
            "8505fa518a9a5bb8376e4aba7bce5eae12a3dd48"
        );
        let large = vec![0xab; 256];
        assert_eq!(
            hex(&compute_message_id(topic, &large, MESSAGE_DOMAIN_VALID_SNAPPY)),
            "d962d4a07a499c918aa0dda5844ec09cdd14080b"
        );
        assert_eq!(
            hex(&compute_message_id(topic, &deadbeef, MESSAGE_DOMAIN_INVALID_SNAPPY)),
            "2aff5124db8064dd9ddcdd7fc251397ab02527c7"
        );
        assert_eq!(
            hex(&compute_message_id(topic, b"", MESSAGE_DOMAIN_VALID_SNAPPY)),
            "dec01e3e1997dcc2c46c0633116bc6a4ee521086"
        );
        assert_eq!(
            hex(&compute_message_id(b"", &deadbeef, MESSAGE_DOMAIN_VALID_SNAPPY)),
            "285f038cae99a63f9861ed5868790bf251ee0904"
        );
        assert_eq!(
            hex(&compute_message_id(b"", b"", MESSAGE_DOMAIN_VALID_SNAPPY)),
            "ca888f40c3caca805b37a5434c75de5550616e07"
        );
        assert_eq!(
            hex(&compute_message_id(
                b"test-topic",
                b"hello world",
                MESSAGE_DOMAIN_INVALID_SNAPPY
            )),
            "144f143f82dbaf88cf586116efe027443750b97c"
        );
        assert_eq!(
            hex(&compute_message_id(
                b"test-topic",
                b"hello world",
                MESSAGE_DOMAIN_VALID_SNAPPY
            )),
            "2e09dafa813949f06738e85887b351aa6c7dc3ad"
        );
    }
}
