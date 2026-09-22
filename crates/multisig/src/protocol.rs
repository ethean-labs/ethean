//! Wire protocol between the node and the `ethean-prover` child process.
//!
//! Each message is a frame: `u32` little-endian payload length, then the
//! payload `magic(4) | version(1) | kind(1) | request_id(8) | body`. Integers
//! are little-endian, lists are `u32` count-prefixed. Every length is bounded
//! before allocation.

use ethean_crypto::{PublicKey, Signature, PUBLIC_KEY_BYTES, SIGNATURE_BYTES};

use crate::error::{MultisigError, Result};
use crate::limits::{MAX_CHILDREN, MAX_COMPONENTS, MAX_KEYS_PER_COMPONENT, MAX_PROOF_BYTES};
use crate::prove::KeyedProof;

pub const MAGIC: [u8; 4] = *b"EPRV";
pub const VERSION: u8 = 1;
/// Largest payload either side accepts (keys, signatures and proofs fit well below).
pub const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;

const KIND_AGGREGATE: u8 = 0x01;
const KIND_MERGE: u8 = 0x02;
const KIND_SPLIT: u8 = 0x03;
const KIND_PING: u8 = 0x04;
const KIND_PROOF: u8 = 0x81;
const KIND_ERROR: u8 = 0x82;
const KIND_PONG: u8 = 0x83;

/// Work the prover process performs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    AggregateType1 {
        message: [u8; 32],
        slot: u64,
        raw: Vec<(PublicKey, Signature)>,
        children: Vec<KeyedProof>,
    },
    MergeType2 {
        components: Vec<KeyedProof>,
    },
    SplitType2 {
        message: [u8; 32],
        proof: Vec<u8>,
        public_keys_per_component: Vec<Vec<PublicKey>>,
    },
    Ping,
}

/// Prover reply to one request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Proof(Vec<u8>),
    Error(String),
    /// Health check reply carrying the prover's leanVM revision.
    Pong(String),
}

struct Writer(Vec<u8>);

impl Writer {
    fn header(kind: u8, id: u64) -> Self {
        let mut buf = Vec::with_capacity(64);
        buf.extend_from_slice(&MAGIC);
        buf.push(VERSION);
        buf.push(kind);
        buf.extend_from_slice(&id.to_le_bytes());
        Self(buf)
    }
    fn u32(&mut self, v: usize) {
        self.0.extend_from_slice(&(v as u32).to_le_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        self.u32(b.len());
        self.0.extend_from_slice(b);
    }
    fn keys(&mut self, keys: &[PublicKey]) {
        self.u32(keys.len());
        for k in keys {
            self.0.extend_from_slice(k.as_bytes());
        }
    }
    fn keyed(&mut self, list: &[KeyedProof]) {
        self.u32(list.len());
        for p in list {
            self.keys(&p.public_keys);
            self.bytes(&p.proof);
        }
    }
}

struct Reader<'a> {
    buf: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).ok_or(MultisigError::Malformed)?;
        let out = self.buf.get(self.at..end).ok_or(MultisigError::Malformed)?;
        self.at = end;
        Ok(out)
    }
    fn u32(&mut self, max: usize) -> Result<usize> {
        let v = u32::from_le_bytes(self.take(4)?.try_into().expect("4 bytes")) as usize;
        if v > max {
            return Err(MultisigError::Malformed);
        }
        Ok(v)
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 bytes"),
        ))
    }
    fn root(&mut self) -> Result<[u8; 32]> {
        Ok(self.take(32)?.try_into().expect("32 bytes"))
    }
    fn bytes(&mut self, max: usize) -> Result<Vec<u8>> {
        let n = self.u32(max)?;
        Ok(self.take(n)?.to_vec())
    }
    fn keys(&mut self) -> Result<Vec<PublicKey>> {
        let n = self.u32(MAX_KEYS_PER_COMPONENT)?;
        let mut keys = Vec::with_capacity(n.min(1024));
        for _ in 0..n {
            let bytes = self.take(PUBLIC_KEY_BYTES)?;
            keys.push(PublicKey::try_from_slice(bytes).map_err(|_| MultisigError::Malformed)?);
        }
        Ok(keys)
    }
    fn keyed(&mut self, max: usize) -> Result<Vec<KeyedProof>> {
        let n = self.u32(max)?;
        (0..n)
            .map(|_| {
                Ok(KeyedProof {
                    public_keys: self.keys()?,
                    proof: self.bytes(MAX_PROOF_BYTES)?,
                })
            })
            .collect()
    }
    fn finish(&self) -> Result<()> {
        (self.at == self.buf.len())
            .then_some(())
            .ok_or(MultisigError::Malformed)
    }
}

fn parse_header(payload: &[u8]) -> Result<(u8, u64, Reader<'_>)> {
    let mut r = Reader {
        buf: payload,
        at: 0,
    };
    if r.take(4)? != MAGIC || r.take(1)?[0] != VERSION {
        return Err(MultisigError::Malformed);
    }
    let kind = r.take(1)?[0];
    let id = r.u64()?;
    Ok((kind, id, r))
}

impl Request {
    pub fn encode(&self, id: u64) -> Vec<u8> {
        match self {
            Request::AggregateType1 {
                message,
                slot,
                raw,
                children,
            } => {
                let mut w = Writer::header(KIND_AGGREGATE, id);
                w.0.extend_from_slice(message);
                w.0.extend_from_slice(&slot.to_le_bytes());
                w.u32(raw.len());
                for (k, s) in raw {
                    w.0.extend_from_slice(k.as_bytes());
                    w.0.extend_from_slice(s.as_bytes());
                }
                w.keyed(children);
                w.0
            }
            Request::MergeType2 { components } => {
                let mut w = Writer::header(KIND_MERGE, id);
                w.keyed(components);
                w.0
            }
            Request::SplitType2 {
                message,
                proof,
                public_keys_per_component,
            } => {
                let mut w = Writer::header(KIND_SPLIT, id);
                w.0.extend_from_slice(message);
                w.bytes(proof);
                w.u32(public_keys_per_component.len());
                for keys in public_keys_per_component {
                    w.keys(keys);
                }
                w.0
            }
            Request::Ping => Writer::header(KIND_PING, id).0,
        }
    }

    pub fn decode(payload: &[u8]) -> Result<(u64, Self)> {
        let (kind, id, mut r) = parse_header(payload)?;
        let req = match kind {
            KIND_AGGREGATE => {
                let message = r.root()?;
                let slot = r.u64()?;
                let n = r.u32(MAX_KEYS_PER_COMPONENT)?;
                let mut raw = Vec::with_capacity(n.min(1024));
                for _ in 0..n {
                    let key = PublicKey::try_from_slice(r.take(PUBLIC_KEY_BYTES)?);
                    let sig = Signature::try_from_slice(r.take(SIGNATURE_BYTES)?);
                    raw.push((
                        key.map_err(|_| MultisigError::Malformed)?,
                        sig.map_err(|_| MultisigError::Malformed)?,
                    ));
                }
                let children = r.keyed(MAX_CHILDREN)?;
                Request::AggregateType1 {
                    message,
                    slot,
                    raw,
                    children,
                }
            }
            KIND_MERGE => Request::MergeType2 {
                components: r.keyed(MAX_COMPONENTS)?,
            },
            KIND_SPLIT => {
                let message = r.root()?;
                let proof = r.bytes(MAX_PROOF_BYTES)?;
                let n = r.u32(MAX_COMPONENTS)?;
                let public_keys_per_component = (0..n).map(|_| r.keys()).collect::<Result<_>>()?;
                Request::SplitType2 {
                    message,
                    proof,
                    public_keys_per_component,
                }
            }
            KIND_PING => Request::Ping,
            _ => return Err(MultisigError::Malformed),
        };
        r.finish()?;
        Ok((id, req))
    }
}

impl Response {
    pub fn encode(&self, id: u64) -> Vec<u8> {
        let (kind, body) = match self {
            Response::Proof(p) => (KIND_PROOF, p.as_slice()),
            Response::Error(e) => (KIND_ERROR, e.as_bytes()),
            Response::Pong(rev) => (KIND_PONG, rev.as_bytes()),
        };
        let mut w = Writer::header(kind, id);
        w.bytes(body);
        w.0
    }

    pub fn decode(payload: &[u8]) -> Result<(u64, Self)> {
        let (kind, id, mut r) = parse_header(payload)?;
        let body = r.bytes(MAX_FRAME_BYTES)?;
        r.finish()?;
        let text = || String::from_utf8(body.clone()).map_err(|_| MultisigError::Malformed);
        let resp = match kind {
            KIND_PROOF => Response::Proof(body.clone()),
            KIND_ERROR => Response::Error(text()?),
            KIND_PONG => Response::Pong(text()?),
            _ => return Err(MultisigError::Malformed),
        };
        Ok((id, resp))
    }
}

pub use crate::frame::{read_frame, write_frame};
