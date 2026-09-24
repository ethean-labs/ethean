//! leanSpec `networking_codec` suite against `ethean-network-wire` and the ENR decoder.

pub use crate::suite_networking_wire::NetOutcome;
use crate::suite_networking_wire::{
    base64url, compress_frame, compress_raw, decode_request, decode_response,
    decode_response_stream, decode_varint, decompress_frame, decompress_raw, encode_request,
    encode_response, hex_field, same,
};
use ethean_network::decode_enr;
use ethean_network_wire::{
    compute_message_id, encode_varint, topic_aggregation, topic_attestation, topic_block,
    ResponseCode,
};
use serde_json::Value;

/// Snappy encoders differ in block choices; the vector is checked by
/// decompression in both directions plus the uncompressed length.
fn snappy_block(codec: &Value, out: &Value) -> Result<(), String> {
    let data = hex_field(codec, "data")?;
    let want = hex_field(out, "compressed")?;
    same(
        "snappy_block decompress(vector)",
        &decompress_raw(&want)?,
        &data,
    )?;
    same(
        "snappy_block roundtrip",
        &decompress_raw(&compress_raw(&data)?)?,
        &data,
    )?;
    let len = out
        .get("uncompressedLength")
        .and_then(Value::as_u64)
        .unwrap_or(data.len() as u64);
    if len != data.len() as u64 {
        return Err("snappy_block uncompressedLength mismatch".into());
    }
    Ok(())
}

fn snappy_frame(codec: &Value, out: &Value) -> Result<(), String> {
    let data = hex_field(codec, "data")?;
    let want = hex_field(out, "framed")?;
    same(
        "snappy_frame decompress(vector)",
        &decompress_frame(&want)?,
        &data,
    )?;
    same(
        "snappy_frame roundtrip",
        &decompress_frame(&compress_frame(&data)?)?,
        &data,
    )
}

fn varint(codec: &Value, out: &Value) -> Result<(), String> {
    let value = codec
        .get("value")
        .and_then(Value::as_u64)
        .ok_or("varint value")?;
    let want = hex_field(out, "encoded")?;
    same("varint encode", &encode_varint(value), &want)?;
    let (decoded, used) = decode_varint(&want, 0)?;
    if decoded != value || used != want.len() {
        return Err("varint decode mismatch".into());
    }
    Ok(())
}

fn reqresp_request(codec: &Value, out: &Value) -> Result<(), String> {
    let ssz = hex_field(codec, "sszData")?;
    let want = hex_field(out, "encoded")?;
    same(
        "reqresp_request decode(vector)",
        &decode_request(&want)?,
        &ssz,
    )?;
    same(
        "reqresp_request roundtrip",
        &decode_request(&encode_request(&ssz)?)?,
        &ssz,
    )
}

fn reqresp_response(codec: &Value, out: &Value) -> Result<(), String> {
    let ssz = hex_field(codec, "sszData")?;
    let code = codec
        .get("responseCode")
        .and_then(Value::as_u64)
        .ok_or("responseCode")? as u8;
    let want = hex_field(out, "encoded")?;
    let (got_code, got_ssz) = decode_response(&want)?;
    if got_code.as_u8() != code {
        return Err(format!(
            "reqresp_response code {} != {code}",
            got_code.as_u8()
        ));
    }
    same("reqresp_response decode(vector)", &got_ssz, &ssz)?;
    let ours = encode_response(ResponseCode::from_wire(code), &ssz)?;
    same(
        "reqresp_response roundtrip",
        &decode_response(&ours)?.1,
        &ssz,
    )
}

fn reqresp_stream(codec: &Value, out: &Value) -> Result<(), String> {
    let chunks = codec
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or("chunks")?;
    let want = hex_field(out, "encoded")?;
    let decoded = decode_response_stream(&want)?;
    if decoded.len() != chunks.len() {
        return Err(format!(
            "stream chunk count {} != {}",
            decoded.len(),
            chunks.len()
        ));
    }
    for (got, spec) in decoded.iter().zip(chunks) {
        let code = spec
            .get("responseCode")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u8;
        if got.code.as_u8() != code {
            return Err("stream chunk code mismatch".into());
        }
        same(
            "stream chunk payload",
            &got.payload,
            &hex_field(spec, "sszData")?,
        )?;
    }
    Ok(())
}

fn message_id(codec: &Value, out: &Value) -> Result<(), String> {
    let topic = hex_field(codec, "topic")?;
    let data = hex_field(codec, "data")?;
    let domain: [u8; 4] = hex_field(codec, "domain")?
        .try_into()
        .map_err(|_| "domain is not 4 bytes")?;
    let want = hex_field(out, "messageId")?;
    same(
        "gossip_message_id",
        &compute_message_id(&topic, &data, domain),
        &want,
    )
}

fn topic(codec: &Value, out: &Value) -> Result<(), String> {
    let kind = codec
        .get("topicKind")
        .and_then(Value::as_str)
        .ok_or("topicKind")?;
    let name = codec
        .get("networkName")
        .and_then(Value::as_str)
        .ok_or("networkName")?;
    let want = out
        .get("topicString")
        .and_then(Value::as_str)
        .ok_or("topicString")?;
    let got = match kind {
        "block" => topic_block(name),
        "aggregation" => topic_aggregation(name),
        "attestation" => {
            let subnet = codec.get("subnetId").and_then(Value::as_u64).unwrap_or(0) as u16;
            topic_attestation(name, subnet)
        }
        other => return Err(format!("unknown topicKind {other}")),
    }
    .map_err(|e| e.to_string())?;
    if got != want {
        return Err(format!("gossip_topic {got} != {want}"));
    }
    // `forkValid` compares the topic's network name with the expected one.
    if let (Some(valid), Some(expected)) = (
        out.get("forkValid").and_then(Value::as_bool),
        codec.get("expectedNetworkName").and_then(Value::as_str),
    ) {
        if (name == expected) != valid {
            return Err("gossip_topic forkValid mismatch".into());
        }
    }
    Ok(())
}

fn enr(codec: &Value, out: &Value) -> Result<(), String> {
    let text = codec
        .get("enrString")
        .and_then(Value::as_str)
        .ok_or("enrString")?;
    let record = decode_enr(text).map_err(|e| e.to_string())?;
    if let Some(seq) = out.get("seq").and_then(Value::as_u64) {
        if record.seq != seq {
            return Err(format!("enr seq {} != {seq}", record.seq));
        }
    }
    if let Some(valid) = out.get("isValid").and_then(Value::as_bool) {
        if !valid {
            return Err("enr decoded although the vector marks it invalid".into());
        }
    }
    Ok(())
}

fn decode_failure(codec: &Value) -> Result<NetOutcome, String> {
    let decoder = codec
        .get("decoder")
        .and_then(Value::as_str)
        .ok_or("decoder")?;
    let raw = hex_field(codec, "rawBytes")?;
    let rejected = match decoder {
        "varint" => decode_varint(&raw, 0).is_err(),
        "snappy_frame" => decompress_frame(&raw).is_err(),
        "reqresp_request" => decode_request(&raw).is_err(),
        "enr" => decode_enr(&format!("enr:{}", base64url(&raw))).is_err(),
        other => return Ok(NetOutcome::Unsupported(format!("decode_failure/{other}"))),
    };
    if rejected {
        Ok(NetOutcome::Rejected)
    } else {
        Err(format!("{decoder}: accepted bytes the vector rejects"))
    }
}

/// Run one networking vector.
pub fn run_networking_case(case: &Value) -> Result<NetOutcome, String> {
    let codec = case.get("codec").ok_or("missing codec")?;
    let out = case.get("output").cloned().unwrap_or(Value::Null);
    let kind = codec
        .get("kind")
        .and_then(Value::as_str)
        .ok_or("codec.kind")?;
    let checked = match kind {
        "snappy_block" => snappy_block(codec, &out),
        "snappy_frame" => snappy_frame(codec, &out),
        "varint" => varint(codec, &out),
        "reqresp_request" => reqresp_request(codec, &out),
        "reqresp_response" => reqresp_response(codec, &out),
        "reqresp_response_stream" => reqresp_stream(codec, &out),
        "gossip_message_id" => message_id(codec, &out),
        "gossip_topic" => topic(codec, &out),
        "enr" => enr(codec, &out),
        "decode_failure" => return decode_failure(codec),
        other => return Ok(NetOutcome::Unsupported(other.to_string())),
    };
    checked.map(|_| NetOutcome::Checked)
}
