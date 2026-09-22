//! Prover protocol round-trips and rejection of malformed frames.

use ethean_crypto::{PublicKey, Signature, PUBLIC_KEY_BYTES, SIGNATURE_BYTES};
use ethean_multisig::protocol::{read_frame, write_frame, Request, Response};
use ethean_multisig::KeyedProof;

fn key(b: u8) -> PublicKey {
    PublicKey::from_bytes([b; PUBLIC_KEY_BYTES])
}

#[test]
fn requests_round_trip() {
    let requests = vec![
        Request::AggregateType1 {
            message: [7; 32],
            slot: 42,
            raw: vec![(key(1), Signature::from_bytes([9; SIGNATURE_BYTES]))],
            children: vec![KeyedProof {
                public_keys: vec![key(2), key(3)],
                proof: vec![1, 2, 3],
            }],
        },
        Request::MergeType2 {
            components: vec![KeyedProof {
                public_keys: vec![key(4)],
                proof: vec![5; 10],
            }],
        },
        Request::SplitType2 {
            message: [1; 32],
            proof: vec![8; 20],
            public_keys_per_component: vec![vec![key(5)], vec![key(6), key(7)]],
        },
        Request::Ping,
    ];
    for (id, req) in requests.into_iter().enumerate() {
        let bytes = req.encode(id as u64);
        assert_eq!(Request::decode(&bytes).unwrap(), (id as u64, req));
    }
}

#[test]
fn responses_round_trip_through_frames() {
    let mut stream = Vec::new();
    for (id, resp) in [
        Response::Proof(vec![1, 2, 3]),
        Response::Error("bad".into()),
        Response::Pong("rev".into()),
    ]
    .iter()
    .enumerate()
    {
        write_frame(&mut stream, &resp.encode(id as u64)).unwrap();
    }
    let mut cursor = std::io::Cursor::new(stream);
    for expected in 0..3u64 {
        let frame = read_frame(&mut cursor).unwrap().unwrap();
        assert_eq!(Response::decode(&frame).unwrap().0, expected);
    }
    assert!(read_frame(&mut cursor).unwrap().is_none());
}

#[test]
fn rejects_truncated_trailing_and_foreign_frames() {
    let bytes = Request::Ping.encode(1);
    assert!(Request::decode(&bytes[..bytes.len() - 1]).is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(Request::decode(&trailing).is_err());
    let mut foreign = bytes.clone();
    foreign[0] = b'X';
    assert!(Request::decode(&foreign).is_err());
    let mut huge_count = Request::MergeType2 { components: vec![] }.encode(1);
    let n = huge_count.len();
    huge_count[n - 4..].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Request::decode(&huge_count).is_err());
    let mut oversized = Vec::new();
    oversized.extend_from_slice(&u32::MAX.to_le_bytes());
    assert!(read_frame(&mut std::io::Cursor::new(oversized)).is_err());
}
