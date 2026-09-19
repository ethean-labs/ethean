//! UDP Status probe dial (pre-libp2p QUIC crypto handshake).

use crate::error::{NetworkError, Result};
use crate::multiaddr::parse_quic_udp;
use crate::transport::{reject_non_quic, BoundTransport};
use ethean_network_wire::Status;
use std::net::UdpSocket;
use std::time::Duration;

/// Outcome of a UDP Status probe toward a peer multiaddr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UdpDialProbe {
    /// Remote multiaddr that was contacted.
    pub remote: String,
    /// Bytes of local Status that were sent.
    pub bytes_sent: usize,
    /// Optional decoded peer Status if a reply arrived in time.
    pub peer_status: Option<Status>,
}

/// Send local [`Status`] over UDP and optionally read one reply datagram.
///
/// This proves reachability of the `/udp/.../quic-v1` path. It is **not** a
/// TLS/QUIC cryptographic handshake; libp2p QUIC-v1 swarm dial remains separate.
pub fn probe_udp_status(
    bound: &BoundTransport,
    multiaddr: &str,
    local: &Status,
    timeout: Duration,
) -> Result<UdpDialProbe> {
    reject_non_quic(multiaddr)?;
    let addr = parse_quic_udp(multiaddr)?;
    let payload = local
        .encode()
        .map_err(|e| NetworkError::Wire(e.to_string()))?;

    // Dedicated connected socket so we do not disturb the listen socket's peers.
    let dial = UdpSocket::bind(("0.0.0.0", 0)).map_err(|e| {
        NetworkError::Handshake(format!("udp dial bind failed: {e}"))
    })?;
    dial.set_read_timeout(Some(timeout))
        .map_err(|e| NetworkError::Handshake(format!("set_read_timeout: {e}")))?;
    dial.connect(addr.socket_addr())
        .map_err(|e| NetworkError::Handshake(format!("udp connect {}: {e}", multiaddr)))?;
    let bytes_sent = dial
        .send(&payload)
        .map_err(|e| NetworkError::Handshake(format!("udp send: {e}")))?;

    let mut buf = [0u8; 2048];
    let peer_status = match dial.recv(&mut buf) {
        Ok(n) => match Status::decode(&buf[..n]) {
            Ok(s) => {
                s.compatible_with(local)
                    .map_err(|e| NetworkError::Handshake(e.to_string()))?;
                Some(s)
            }
            Err(e) => {
                return Err(NetworkError::Wire(format!("peer status decode: {e}")));
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
            let _ = bound.listen_port; // keep API using bound identity context
            None
        }
        Err(e) => return Err(NetworkError::Handshake(format!("udp recv: {e}"))),
    };

    Ok(UdpDialProbe {
        remote: addr.to_multiaddr(),
        bytes_sent,
        peer_status,
    })
}

/// True QUIC/libp2p dial without a bound swarm.
///
/// With feature `libp2p-quic`, bind [`crate::QuicSwarm`] (or
/// [`crate::SwarmFacade::bind_quic_swarm`]) and call `dial` / `dial_quic_peer`.
/// Without that feature, only [`probe_udp_status`] is available for path checks.
pub fn dial_quic_pending(multiaddr: &str) -> Result<()> {
    reject_non_quic(multiaddr)?;
    let _ = parse_quic_udp(multiaddr)?;
    #[cfg(feature = "libp2p-quic")]
    {
        Err(NetworkError::TransportPending(
            "no QuicSwarm bound; use SwarmFacade::bind_quic_swarm then dial_quic_peer",
        ))
    }
    #[cfg(not(feature = "libp2p-quic"))]
    {
        Err(NetworkError::TransportPending(
            "enable ethean-network/libp2p-quic for QuicSwarm dial; use probe_udp_status for path checks",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::NodeIdentity;
    use crate::transport::{prepare_transport, TransportConfig};
    use std::thread;

    #[test]
    fn probe_loopback_status_roundtrip() {
        let id = NodeIdentity::from_seed(b"dial-a");
        let listener = prepare_transport(
            &id,
            &TransportConfig {
                listen_port: 0,
                idle_timeout_ms: 1_000,
            },
        )
        .unwrap();
        // Blocking recv on a clone path: bind a blocking socket on same port is hard.
        // Instead: spawn echo server on its own socket.
        let server = UdpSocket::bind(("127.0.0.1", 0)).unwrap();
        let server_port = server.local_addr().unwrap().port();
        let status = Status {
            genesis_root: [9u8; 32],
            fork_segment: "lstar001".into(),
            head_slot: 1,
            head_root: [1u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        let echo = status.clone();
        let handle = thread::spawn(move || {
            let mut buf = [0u8; 2048];
            let (n, peer) = server.recv_from(&mut buf).unwrap();
            let got = Status::decode(&buf[..n]).unwrap();
            assert_eq!(got, echo);
            server
                .send_to(&echo.encode().unwrap(), peer)
                .unwrap();
        });

        let remote = format!("/ip4/127.0.0.1/udp/{server_port}/quic-v1");
        let probe = probe_udp_status(
            &listener,
            &remote,
            &status,
            Duration::from_millis(500),
        )
        .unwrap();
        assert_eq!(probe.peer_status.as_ref(), Some(&status));
        handle.join().unwrap();
    }

    #[test]
    fn quic_dial_still_pending() {
        assert!(dial_quic_pending("/ip4/127.0.0.1/udp/9/quic-v1").is_err());
    }
}
