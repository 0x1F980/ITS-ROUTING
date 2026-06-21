use its_transport::field_arith::FieldElement;
use its_transport::onion::{MorphicOnionPacket, PAYLOAD_SIZE};
use its_transport::SssPackedShare;

pub(crate) fn serialize_packet(packet: &MorphicOnionPacket) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(100); // 25 elements * 4 bytes = 100 bytes
    for i in 0..3 {
        bytes.extend_from_slice(&(packet.header_points[i].0.value() as u32).to_be_bytes());
        bytes.extend_from_slice(&(packet.header_points[i].1.value() as u32).to_be_bytes());
    }
    for i in 0..3 {
        bytes.extend_from_slice(&(packet.header_tags[i].value() as u32).to_be_bytes());
    }
    for i in 0..PAYLOAD_SIZE {
        bytes.extend_from_slice(&(packet.payload[i].value() as u32).to_be_bytes());
    }
    bytes
}

pub(crate) fn deserialize_packet(bytes: &[u8]) -> Result<MorphicOnionPacket, &'static str> {
    if bytes.len() < 100 {
        return Err("Packet too short");
    }
    let mut header_points = [(FieldElement::zero(), FieldElement::zero()); 3];
    let mut header_tags = [FieldElement::zero(); 3];
    let mut payload = [FieldElement::zero(); PAYLOAD_SIZE];

    let mut offset = 0;
    for i in 0..3 {
        let x = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        let y = u32::from_be_bytes([bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7]]);
        header_points[i] = (FieldElement::new(x), FieldElement::new(y));
        offset += 8;
    }
    for i in 0..3 {
        let tag = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        header_tags[i] = FieldElement::new(tag);
        offset += 4;
    }
    for i in 0..PAYLOAD_SIZE {
        let val = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        payload[i] = FieldElement::new(val);
        offset += 4;
    }

    Ok(MorphicOnionPacket {
        header_points,
        header_tags,
        payload,
    })
}

pub(crate) fn serialize_share(share: &SssPackedShare) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + share.data_points.len() * 4);
    bytes.extend_from_slice(&(share.id.value() as u32).to_be_bytes());
    bytes.extend_from_slice(&(share.data_points.len() as u32).to_be_bytes());
    for pt in &share.data_points {
        bytes.extend_from_slice(&(pt.value() as u32).to_be_bytes());
    }
    bytes
}

pub(crate) fn deserialize_share(bytes: &[u8]) -> Result<SssPackedShare, &'static str> {
    if bytes.len() < 8 {
        return Err("Share too short");
    }
    let id = FieldElement::new(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
    let num_points = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    const MAX_SHARE_POINTS: usize = 1_048_576;
    if num_points > MAX_SHARE_POINTS {
        return Err("Share point count out of bounds");
    }
    let expected = 8usize.saturating_add(num_points.saturating_mul(4));
    if expected != bytes.len() {
        return Err("Share length mismatch");
    }
    let mut data_points = Vec::with_capacity(num_points);
    let mut offset = 8;
    for _ in 0..num_points {
        if offset + 4 > bytes.len() {
            return Err("Share truncated");
        }
        let val = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        data_points.push(FieldElement::new(val));
        offset += 4;
    }
    Ok(SssPackedShare { id, data_points })
}

/// Extract opaque payload bytes from a peeled onion packet (field elements → LE u32 bytes).
pub(crate) fn payload_to_bytes(payload: &[FieldElement; PAYLOAD_SIZE]) -> Vec<u8> {
    let mut out = Vec::new();
    for fe in payload.iter() {
        let v = fe.value() as u32;
        if v == 0 && !out.is_empty() {
            break;
        }
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}
