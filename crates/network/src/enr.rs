//! Minimal ENR (EIP-778) decoding for Lean bootnodes.
//!
//! Hive and lean-quickstart publish bootnodes as secp256k1-signed ENRs whose
//! key/value pairs carry `ip`, `udp`, `quic` (and the v6 variants). Ethean only
//! needs the QUIC dial address and the libp2p PeerId, so this decoder reads the
//! record and skips discv5 signature verification (peers are authenticated by
//! the QUIC handshake against the PeerId in the multiaddr).

use crate::error::{NetworkError, Result};
use base64::Engine;
use std::net::{Ipv4Addr, Ipv6Addr};

/// Fields Ethean reads from an ENR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrRecord {
    /// Record sequence number.
    pub seq: u64,
    /// Compressed secp256k1 public key (33 bytes).
    pub secp256k1: [u8; 33],
    pub ip4: Option<Ipv4Addr>,
    pub ip6: Option<Ipv6Addr>,
    pub udp: Option<u16>,
    pub udp6: Option<u16>,
    pub quic: Option<u16>,
    pub quic6: Option<u16>,
}

impl EnrRecord {
    /// libp2p PeerId (base58) for the record's secp256k1 key.
    pub fn peer_id(&self) -> String {
        peer_id_from_secp256k1(&self.secp256k1)
    }

    /// `/ip4/<ip>/udp/<quic|udp>/quic-v1/p2p/<peerid>` (IPv6 when only `ip6` is set).
    pub fn quic_multiaddr(&self) -> Result<String> {
        let peer = self.peer_id();
        if let Some(ip) = self.ip4 {
            let port = self.quic.or(self.udp).ok_or_else(|| {
                NetworkError::Handshake("ENR has ip but neither quic nor udp port".into())
            })?;
            return Ok(format!("/ip4/{ip}/udp/{port}/quic-v1/p2p/{peer}"));
        }
        if let Some(ip) = self.ip6 {
            let port = self.quic6.or(self.udp6).ok_or_else(|| {
                NetworkError::Handshake("ENR has ip6 but neither quic6 nor udp6 port".into())
            })?;
            return Ok(format!("/ip6/{ip}/udp/{port}/quic-v1/p2p/{peer}"));
        }
        Err(NetworkError::Handshake("ENR carries no ip / ip6".into()))
    }
}

/// Decode `enr:<base64url>` (prefix optional) into its Lean-relevant fields.
pub fn decode_enr(text: &str) -> Result<EnrRecord> {
    let t = text.trim();
    let body = t.strip_prefix("enr:").unwrap_or(t);
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(body.trim_end_matches('='))
        .map_err(|e| NetworkError::Handshake(format!("ENR base64: {e}")))?;
    let rlp = rlp::Rlp::new(&bytes);
    if !rlp.is_list() {
        return Err(NetworkError::Handshake("ENR is not an RLP list".into()));
    }
    let items: Vec<rlp::Rlp<'_>> = rlp.iter().collect();
    if items.len() < 2 || items.len() % 2 != 0 {
        return Err(NetworkError::Handshake(format!(
            "ENR list has {} items (expected signature, seq, key/value pairs)",
            items.len()
        )));
    }
    let seq: u64 = items[1]
        .as_val()
        .map_err(|e| NetworkError::Handshake(format!("ENR seq: {e}")))?;

    let mut rec = EnrRecord {
        seq,
        secp256k1: [0u8; 33],
        ip4: None,
        ip6: None,
        udp: None,
        udp6: None,
        quic: None,
        quic6: None,
    };
    let mut have_key = false;
    for pair in items[2..].chunks(2) {
        let key = pair[0]
            .data()
            .map_err(|e| NetworkError::Handshake(format!("ENR key: {e}")))?;
        let value = pair[1]
            .data()
            .map_err(|e| NetworkError::Handshake(format!("ENR value: {e}")))?;
        match key {
            b"id" => {
                if value != b"v4" {
                    return Err(NetworkError::Handshake(format!(
                        "unsupported ENR id scheme {:?}",
                        String::from_utf8_lossy(value)
                    )));
                }
            }
            b"secp256k1" => {
                if value.len() != 33 {
                    return Err(NetworkError::Handshake(format!(
                        "ENR secp256k1 key is {} bytes (expected 33 compressed)",
                        value.len()
                    )));
                }
                rec.secp256k1.copy_from_slice(value);
                have_key = true;
            }
            b"ip" => rec.ip4 = Some(Ipv4Addr::from(fixed::<4>(value, "ip")?)),
            b"ip6" => rec.ip6 = Some(Ipv6Addr::from(fixed::<16>(value, "ip6")?)),
            b"udp" => rec.udp = Some(port(value)?),
            b"udp6" => rec.udp6 = Some(port(value)?),
            b"quic" => rec.quic = Some(port(value)?),
            b"quic6" => rec.quic6 = Some(port(value)?),
            _ => {}
        }
    }
    if !have_key {
        return Err(NetworkError::Handshake("ENR has no secp256k1 key".into()));
    }
    Ok(rec)
}

/// Decode an ENR straight into a dialable QUIC multiaddr.
pub fn enr_to_multiaddr(text: &str) -> Result<String> {
    decode_enr(text)?.quic_multiaddr()
}

/// libp2p PeerId for a compressed secp256k1 public key.
///
/// Protobuf `PublicKey{Type=Secp256k1(2), Data}` is 37 bytes, so the PeerId uses
/// the identity multihash (`0x00`, length, bytes) rendered as base58btc.
pub fn peer_id_from_secp256k1(pubkey: &[u8; 33]) -> String {
    let mut proto = Vec::with_capacity(37);
    proto.extend_from_slice(&[0x08, 0x02, 0x12, 0x21]);
    proto.extend_from_slice(pubkey);
    let mut mh = Vec::with_capacity(39);
    mh.push(0x00);
    mh.push(proto.len() as u8);
    mh.extend_from_slice(&proto);
    bs58::encode(mh).into_string()
}

fn fixed<const N: usize>(value: &[u8], key: &str) -> Result<[u8; N]> {
    if value.len() != N {
        return Err(NetworkError::Handshake(format!(
            "ENR {key} is {} bytes (expected {N})",
            value.len()
        )));
    }
    let mut out = [0u8; N];
    out.copy_from_slice(value);
    Ok(out)
}

fn port(value: &[u8]) -> Result<u16> {
    if value.is_empty() || value.len() > 2 {
        return Err(NetworkError::Handshake(format!(
            "ENR port is {} bytes",
            value.len()
        )));
    }
    Ok(value.iter().fold(0u16, |acc, b| (acc << 8) | *b as u16))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// First entry of the lean-quickstart README `nodes.yaml` example.
    const QUICKSTART_ENR: &str = "enr:-IW4QMn2QUYENcnsEpITZLph3YZee8Y3B92INUje_riQUOFQQ5Zm5kASi7E_IuQoGCWgcmCYrH920Q52kH7tQcWcPhEBgmlkgnY0gmlwhH8AAAGEcXVpY4IjKIlzZWNwMjU2azGhAhMMnGF1rmIPQ9tWgqfkNmvsG-aIyc9EJU5JFo3Tegys";
    const QUICKSTART_PUBKEY: &str =
        "02130c9c6175ae620f43db5682a7e4366bec1be688c9cf44254e49168dd37a0cac";

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn decodes_quickstart_enr_fields() {
        let rec = decode_enr(QUICKSTART_ENR).unwrap();
        assert_eq!(rec.seq, 1);
        assert_eq!(rec.ip4, Some(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(rec.quic, Some(9000));
        assert_eq!(rec.udp, None);
        assert_eq!(rec.secp256k1.to_vec(), hex(QUICKSTART_PUBKEY));
        let addr = rec.quic_multiaddr().unwrap();
        assert_eq!(
            addr,
            "/ip4/127.0.0.1/udp/9000/quic-v1/p2p/16Uiu2HAkvi2sxT75Bpq1c7yV2FjnSQJJ432d6jeshbmfdJss1i6f"
        );
        assert_eq!(enr_to_multiaddr(QUICKSTART_ENR).unwrap(), addr);
    }

    #[test]
    fn peer_id_is_stable_for_known_key() {
        let mut pk = [0u8; 33];
        pk.copy_from_slice(&hex(QUICKSTART_PUBKEY));
        let id = peer_id_from_secp256k1(&pk);
        // Same value lean-quickstart prints for node zeam_0 in validator-config.yaml.
        assert_eq!(id, "16Uiu2HAkvi2sxT75Bpq1c7yV2FjnSQJJ432d6jeshbmfdJss1i6f");
    }

    #[cfg(feature = "libp2p-quic")]
    #[test]
    fn peer_id_matches_libp2p_derivation() {
        let rec = decode_enr(QUICKSTART_ENR).unwrap();
        let pk = libp2p::identity::secp256k1::PublicKey::try_from_bytes(&rec.secp256k1).unwrap();
        let expected = libp2p::identity::PublicKey::from(pk).to_peer_id().to_string();
        assert_eq!(rec.peer_id(), expected);
        // The record was signed by lean-quickstart node zeam_0; its secret must yield the same id.
        let key = crate::node_key::NodeKey::from_hex(
            "bdf953adc161873ba026330c56450453f582e3c4ee6cb713644794bcfdd85fe5",
        )
        .unwrap();
        assert_eq!(key.peer_id().unwrap().to_string(), expected);
    }

    #[test]
    fn rejects_garbage() {
        assert!(decode_enr("enr:not-base64!").is_err());
        assert!(decode_enr("enr:AAAA").is_err());
        let mut pk = [0u8; 33];
        pk[0] = 2;
        let rec = EnrRecord {
            seq: 1,
            secp256k1: pk,
            ip4: None,
            ip6: None,
            udp: None,
            udp6: None,
            quic: None,
            quic6: None,
        };
        assert!(rec.quic_multiaddr().is_err());
    }

    #[test]
    fn falls_back_to_udp_when_quic_missing() {
        let rec = EnrRecord {
            seq: 0,
            secp256k1: [2u8; 33],
            ip4: Some(Ipv4Addr::new(10, 0, 0, 2)),
            ip6: None,
            udp: Some(9001),
            udp6: None,
            quic: None,
            quic6: None,
        };
        assert!(rec
            .quic_multiaddr()
            .unwrap()
            .starts_with("/ip4/10.0.0.2/udp/9001/quic-v1/p2p/"));
    }
}
