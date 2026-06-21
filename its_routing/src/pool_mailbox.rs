//! PoolMailbox — hidden addressing without `.i2p`.
//!
//! Recipient hint lives inside Shannon ITS wire ciphertext (`body`), not in pool cell headers.
//! Bob harvests all epochs; per-cell OTM verify (epoch_cell) + post-reconstruct wire gate.

/// Minimal ITS wire header parse (no `its_asymmetric` dep — routing boundary).
const WIRE_MIN: usize = 8 + 4;

/// Pool mailbox filter derived from contact OTM / public fingerprint (KM contact graph).
#[derive(Clone, Debug, Default)]
pub struct PoolMailbox {
    /// Namespace derived from fingerprint (logging / optional strict pre-filter).
    pub namespace: u32,
    /// 16-byte fingerprint prefix (hex-decoded or raw pubkey hash).
    pub fingerprint: [u8; 16],
    /// When true, reject reconstructions that fail ITS wire structure or OTM tag layout.
    pub strict: bool,
}

impl PoolMailbox {
    pub fn open() -> Self {
        Self::default()
    }

    pub fn from_fingerprint_bytes(fp: &[u8]) -> Self {
        let mut fingerprint = [0u8; 16];
        let n = fp.len().min(16);
        fingerprint[..n].copy_from_slice(&fp[..n]);
        Self {
            namespace: mailbox_namespace(&fingerprint),
            fingerprint,
            strict: false,
        }
    }

    pub fn from_fingerprint_hex(hex: &str) -> Option<Self> {
        let bytes = decode_hex(hex)?;
        Some(Self::from_fingerprint_bytes(&bytes))
    }

    /// Per-cell gate after epoch_cell OTM verify: share IDs are not mailbox hints (plan: ID ∈ ciphertext only).
    pub fn accept_verified_share(&self, _share_id: u32) -> bool {
        true
    }

    /// Post-reconstruct gate before writing `message.wire` to Bob.
    pub fn accept_reconstructed_payload(&self, payload: &[u8]) -> bool {
        if payload.len() < WIRE_MIN {
            return false;
        }
        if !self.strict {
            return true;
        }
        if !wire_is_valid_its_seal(payload) {
            return false;
        }
        if self.namespace == 0 {
            return true;
        }
        wire_otm_contact_hint_matches(payload, self.namespace)
    }
}

/// Derive a stable mailbox namespace from contact fingerprint bytes.
pub fn mailbox_namespace(fingerprint: &[u8]) -> u32 {
    if fingerprint.len() >= 4 {
        u32::from_be_bytes([fingerprint[0], fingerprint[1], fingerprint[2], fingerprint[3]])
    } else {
        0
    }
}

/// Observable pool cells carry no recipient ID; provenance ∉ O (ParticipationTheorem).
pub fn provenance_in_observable_feed() -> bool {
    false
}

fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    let h: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    if h.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(h.len() / 2);
    let bytes = h.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = from_hex_digit(bytes[i])?;
        let lo = from_hex_digit(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

fn from_hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Structural ITS wire parse (msg_id, body, sigma, morphic_blend, otm_tags).
pub fn wire_is_valid_its_seal(data: &[u8]) -> bool {
    parse_wire_layout(data).is_some()
}

/// Mailbox hint correlation: sealed `body` length tag + first OTM tag (inside wire, not pool header).
fn wire_otm_contact_hint_matches(data: &[u8], namespace: u32) -> bool {
    let Some(layout) = parse_wire_layout(data) else {
        return false;
    };
    if layout.body.is_empty() {
        return false;
    }
    let body_hint = u32::from_be_bytes([
        layout.body[0],
        layout.body.get(1).copied().unwrap_or(0),
        layout.body.get(2).copied().unwrap_or(0),
        layout.body.get(3).copied().unwrap_or(0),
    ]);
    let tag_hint = layout
        .otm_tags
        .first()
        .copied()
        .unwrap_or(0);
    let combined = body_hint ^ tag_hint;
    combined == namespace || namespace == 0
}

struct WireLayout {
    body: Vec<u8>,
    otm_tags: Vec<u32>,
}

fn parse_wire_layout(data: &[u8]) -> Option<WireLayout> {
    if data.len() < WIRE_MIN {
        return None;
    }
    let mut off = 0usize;
    let _msg_id = read_u64(data, &mut off)?;
    let body_len = read_u32(data, &mut off)? as usize;
    if off + body_len > data.len() {
        return None;
    }
    let body = data[off..off + body_len].to_vec();
    off += body_len;
    let n = read_u32(data, &mut off)? as usize;
    if n != body_len {
        return None;
    }
    let sigma_end = off.checked_add(n.checked_mul(4)?)?;
    let morphic_end = sigma_end.checked_add(n.checked_mul(4)?)?;
    if morphic_end > data.len() {
        return None;
    }
    off = morphic_end;
    let tag_count = read_u32(data, &mut off)? as usize;
    if tag_count != n {
        return None;
    }
    let mut otm_tags = Vec::with_capacity(tag_count);
    for _ in 0..tag_count {
        otm_tags.push(read_u32(data, &mut off)?);
    }
    if off != data.len() {
        return None;
    }
    Some(WireLayout { body, otm_tags })
}

fn read_u32(data: &[u8], off: &mut usize) -> Option<u32> {
    if *off + 4 > data.len() {
        return None;
    }
    let mut b = [0u8; 4];
    b.copy_from_slice(&data[*off..*off + 4]);
    *off += 4;
    Some(u32::from_be_bytes(b))
}

fn read_u64(data: &[u8], off: &mut usize) -> Option<u64> {
    if *off + 8 > data.len() {
        return None;
    }
    let mut b = [0u8; 8];
    b.copy_from_slice(&data[*off..*off + 8]);
    *off += 8;
    Some(u64::from_be_bytes(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_from_fingerprint() {
        let fp = [0xAB, 0xCD, 0x12, 0x34, 0x00];
        assert_eq!(mailbox_namespace(&fp), 0xABCD1234);
    }

    #[test]
    fn rejects_truncated_wire() {
        let mb = PoolMailbox::open();
        assert!(!mb.accept_reconstructed_payload(b"\x00\x01"));
    }
}
