//! Minimal Lean multiaddr parsing for `/ip4/.../udp/.../quic-v1`.

use crate::error::{NetworkError, Result};
use std::net::{Ipv4Addr, SocketAddrV4};

/// Parsed QUIC/UDP listen or dial target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuicUdpAddr {
    /// IPv4 host.
    pub ip: Ipv4Addr,
    /// UDP port.
    pub port: u16,
}

/// Parse `/ip4/A.B.C.D/udp/PORT/quic-v1` (optional trailing slash).
pub fn parse_quic_udp(multiaddr: &str) -> Result<QuicUdpAddr> {
    crate::transport::reject_non_quic(multiaddr)?;
    let parts: Vec<&str> = multiaddr.trim_matches('/').split('/').collect();
    // ip4, ADDR, udp, PORT, quic-v1
    if parts.len() < 5 {
        return Err(NetworkError::Handshake(format!(
            "expected /ip4/<addr>/udp/<port>/quic-v1, got {multiaddr}"
        )));
    }
    if parts[0] != "ip4" || parts[2] != "udp" {
        return Err(NetworkError::Handshake(format!(
            "unsupported multiaddr shape: {multiaddr}"
        )));
    }
    let proto = parts[4];
    if proto != "quic-v1" && proto != "quic" {
        return Err(NetworkError::Handshake(format!(
            "expected quic-v1 protocol, got {proto}"
        )));
    }
    let ip: Ipv4Addr = parts[1]
        .parse()
        .map_err(|_| NetworkError::Handshake(format!("bad ipv4: {}", parts[1])))?;
    let port: u16 = parts[3]
        .parse()
        .map_err(|_| NetworkError::Handshake(format!("bad udp port: {}", parts[3])))?;
    if port == 0 {
        return Err(NetworkError::Handshake("udp port 0 refused for dial".into()));
    }
    Ok(QuicUdpAddr { ip, port })
}

impl QuicUdpAddr {
    /// Socket address for `std::net` dials.
    pub fn socket_addr(&self) -> SocketAddrV4 {
        SocketAddrV4::new(self.ip, self.port)
    }

    /// Canonical multiaddr string.
    pub fn to_multiaddr(&self) -> String {
        format!("/ip4/{}/udp/{}/quic-v1", self.ip, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quic_v1() {
        let a = parse_quic_udp("/ip4/127.0.0.1/udp/9000/quic-v1").unwrap();
        assert_eq!(a.port, 9000);
        assert_eq!(a.to_multiaddr(), "/ip4/127.0.0.1/udp/9000/quic-v1");
    }

    #[test]
    fn rejects_tcp_shape() {
        assert!(parse_quic_udp("/ip4/127.0.0.1/tcp/9000").is_err());
    }
}
