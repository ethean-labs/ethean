//! Req/resp wire codec: varint length + snappy framed SSZ.

use crate::error::{Result, WireError};
use crate::limits::MAX_PAYLOAD_SIZE;
use crate::reqresp::ResponseCode;
use crate::snappy::{compress_frame, decompress_frame_prefix};
use crate::varint::{decode_varint, encode_varint};

/// One decoded req/resp response chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseChunk {
    /// Response code (unknown codes remapped per spec).
    pub code: ResponseCode,
    /// Decompressed SSZ (or UTF-8 error for failure codes).
    pub payload: Vec<u8>,
}

fn encode_framed(ssz_data: &[u8]) -> Result<Vec<u8>> {
    if ssz_data.len() > MAX_PAYLOAD_SIZE {
        return Err(WireError::PayloadTooLarge {
            got: ssz_data.len(),
            max: MAX_PAYLOAD_SIZE,
        });
    }
    let compressed = compress_frame(ssz_data)?;
    let mut out = encode_varint(ssz_data.len() as u64);
    out.extend_from_slice(&compressed);
    Ok(out)
}

fn decode_framed(data: &[u8], offset: usize) -> Result<Vec<u8>> {
    let (declared, varint_size) = decode_varint(data, offset)?;
    if declared as usize > MAX_PAYLOAD_SIZE {
        return Err(WireError::PayloadTooLarge {
            got: declared as usize,
            max: MAX_PAYLOAD_SIZE,
        });
    }
    let compressed = &data[offset + varint_size..];
    let (decompressed, consumed) = decompress_frame_prefix(compressed, declared as usize)?;
    if consumed != compressed.len() {
        return Err(WireError::TrailingBytes);
    }
    Ok(decompressed)
}

/// Encode an SSZ request: `[varint uncompressed_len][snappy framed]`.
pub fn encode_request(ssz_data: &[u8]) -> Result<Vec<u8>> {
    encode_framed(ssz_data)
}

/// Decode a wire-format request to SSZ bytes.
pub fn decode_request(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(WireError::Codec("empty request".into()));
    }
    decode_framed(data, 0)
}

/// Encode a response chunk: `[code][varint uncompressed_len][snappy framed]`.
pub fn encode_response(code: ResponseCode, ssz_data: &[u8]) -> Result<Vec<u8>> {
    let mut out = vec![code.as_u8()];
    out.extend_from_slice(&encode_framed(ssz_data)?);
    Ok(out)
}

/// Decode a single response chunk occupying the whole buffer.
pub fn decode_response(data: &[u8]) -> Result<(ResponseCode, Vec<u8>)> {
    let (chunk, consumed) = decode_response_chunk(data)?;
    if consumed != data.len() {
        return Err(WireError::TrailingBytes);
    }
    Ok((chunk.code, chunk.payload))
}

/// Decode one response chunk from the start of `data`. Returns `(chunk, bytes_consumed)`.
pub fn decode_response_chunk(data: &[u8]) -> Result<(ResponseChunk, usize)> {
    let (chunk, rest) = split_one_chunk(data)?;
    Ok((chunk, data.len() - rest.len()))
}

/// Encode several response chunks concatenated (one SignedBlock per chunk).
pub fn encode_response_stream(chunks: &[(ResponseCode, &[u8])]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for (code, payload) in chunks {
        out.extend_from_slice(&encode_response(*code, payload)?);
    }
    Ok(out)
}

/// Iterate response chunks from a concatenated stream until EOF.
pub fn decode_response_stream(mut data: &[u8]) -> Result<Vec<ResponseChunk>> {
    let mut chunks = Vec::new();
    while !data.is_empty() {
        let (chunk, rest) = split_one_chunk(data)?;
        chunks.push(chunk);
        data = rest;
    }
    Ok(chunks)
}

/// Split the first complete response chunk off `data`.
pub fn split_one_chunk(data: &[u8]) -> Result<(ResponseChunk, &[u8])> {
    if data.is_empty() {
        return Err(WireError::Codec("empty response".into()));
    }
    if data.len() < 2 {
        return Err(WireError::Codec("response too short".into()));
    }
    let code = ResponseCode::from_wire(data[0]);
    let (declared, varint_size) = decode_varint(data, 1)?;
    if declared as usize > MAX_PAYLOAD_SIZE {
        return Err(WireError::PayloadTooLarge {
            got: declared as usize,
            max: MAX_PAYLOAD_SIZE,
        });
    }
    let frame_start = 1 + varint_size;
    if frame_start > data.len() {
        return Err(WireError::Codec("truncated response frame".into()));
    }
    let (payload, frame_len) = decompress_frame_prefix(&data[frame_start..], declared as usize)?;
    let consumed = frame_start + frame_len;
    Ok((ResponseChunk { code, payload }, &data[consumed..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_roundtrip_empty_and_small() {
        let empty = encode_request(b"").unwrap();
        assert_eq!(empty[0], 0x00);
        assert_eq!(decode_request(&empty).unwrap(), b"");
        let small = encode_request(&[1, 2, 3, 4]).unwrap();
        assert_eq!(decode_request(&small).unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn response_codes_roundtrip() {
        let body = encode_response(ResponseCode::Success, &[1, 2, 3, 4]).unwrap();
        assert_eq!(body[0], 0);
        let (code, data) = decode_response(&body).unwrap();
        assert_eq!(code, ResponseCode::Success);
        assert_eq!(data, vec![1, 2, 3, 4]);

        let err = encode_response(ResponseCode::InvalidRequest, b"bad request").unwrap();
        let (code, data) = decode_response(&err).unwrap();
        assert_eq!(code, ResponseCode::InvalidRequest);
        assert_eq!(data, b"bad request");
    }

    #[test]
    fn response_stream_two_chunks() {
        let stream = encode_response_stream(&[
            (ResponseCode::Success, b"\xde\xad\xbe\xef".as_slice()),
            (ResponseCode::Success, b"\xca\xfe\xba\xbe".as_slice()),
        ])
        .unwrap();
        let chunks = decode_response_stream(&stream).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].payload, b"\xde\xad\xbe\xef");
        assert_eq!(chunks[1].payload, b"\xca\xfe\xba\xbe");
    }

    #[test]
    fn response_stream_success_then_unavailable() {
        let payload = vec![0x11u8; 16];
        let stream = encode_response_stream(&[
            (ResponseCode::Success, payload.as_slice()),
            (ResponseCode::ResourceUnavailable, b"not found".as_slice()),
        ])
        .unwrap();
        let chunks = decode_response_stream(&stream).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].code, ResponseCode::Success);
        assert_eq!(chunks[0].payload, payload);
        assert_eq!(chunks[1].code, ResponseCode::ResourceUnavailable);
        assert_eq!(chunks[1].payload, b"not found");
    }

    #[test]
    fn response_stream_three_compressible() {
        let a = vec![0u8; 64];
        let b = vec![0xffu8; 64];
        let c = vec![0xaau8; 64];
        let stream = encode_response_stream(&[
            (ResponseCode::Success, a.as_slice()),
            (ResponseCode::Success, b.as_slice()),
            (ResponseCode::Success, c.as_slice()),
        ])
        .unwrap();
        let chunks = decode_response_stream(&stream).unwrap();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].payload, a);
        assert_eq!(chunks[1].payload, b);
        assert_eq!(chunks[2].payload, c);
    }

    #[test]
    fn status_ssz_request_roundtrip() {
        let ssz = crate::status::Status {
            finalized: crate::status::Checkpoint {
                root: [0x01; 32],
                slot: 100,
            },
            head: crate::status::Checkpoint {
                root: [0x02; 32],
                slot: 150,
            },
        }
        .encode()
        .unwrap();
        let wire = encode_request(&ssz).unwrap();
        assert_eq!(decode_request(&wire).unwrap(), ssz);
        let resp = encode_response(ResponseCode::Success, &ssz).unwrap();
        let (code, data) = decode_response(&resp).unwrap();
        assert_eq!(code, ResponseCode::Success);
        assert_eq!(data, ssz);
    }

    #[test]
    fn leanspec_request_payload_roundtrips() {
        roundtrip_request(&[]);
        roundtrip_request(&[1, 2, 3, 4]);
        roundtrip_request(&vec![0xab; 127]);
        roundtrip_request(&vec![0xcd; 128]);
        roundtrip_request(&vec![0xde, 0xad, 0xbe, 0xef].repeat(256));
        roundtrip_request(&(0u8..=255).collect::<Vec<_>>());
        roundtrip_request(&[0u8; 32]);
        roundtrip_request(&crate::BlocksByRootRequest::new(vec![]).unwrap().encode());
        roundtrip_request(
            &crate::BlocksByRootRequest::new(vec![[0xaa; 32], [0xbb; 32]])
                .unwrap()
                .encode(),
        );
    }

    #[test]
    fn leanspec_response_payload_roundtrips() {
        roundtrip_response(ResponseCode::Success, &[1, 2, 3, 4]);
        roundtrip_response(ResponseCode::Success, &[]);
        roundtrip_response(ResponseCode::InvalidRequest, b"bad request");
        roundtrip_response(ResponseCode::ServerError, b"internal error");
        roundtrip_response(ResponseCode::ResourceUnavailable, b"block not found");
        roundtrip_response(ResponseCode::Success, &(0u8..=255).collect::<Vec<_>>());
        roundtrip_response(ResponseCode::Success, &vec![0xca, 0xfe, 0xba, 0xbe].repeat(256));
        roundtrip_response(ResponseCode::Success, &[0xff; 32]);
    }

    fn roundtrip_request(ssz: &[u8]) {
        let wire = encode_request(ssz).unwrap();
        assert_eq!(decode_request(&wire).unwrap(), ssz);
    }

    fn roundtrip_response(code: ResponseCode, ssz: &[u8]) {
        let wire = encode_response(code, ssz).unwrap();
        let (got, data) = decode_response(&wire).unwrap();
        assert_eq!(got, code);
        assert_eq!(data, ssz);
    }
}
