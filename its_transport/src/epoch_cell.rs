//! UES Pool epoch cell — `step()` yields `(K_{e+1}, C_e)` with `C_e` fixed size `L` drawn uniformly.
//!
//! Optional SSS `(k,n)` interleaving spreads share bytes across epochs for A-resilience.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use crate::field_arith::FieldElement;
use crate::otm::{generate_tag, verify_tag};
use crate::sss_fragment::{fragment_data, SssPackedShare};
use crate::transport_otp_ratchet::TransportOtpRatchet;
use crate::SecureRandom;
use zeroize::{Zeroize, ZeroizeOnDrop};

const CELL_HDR: usize = 4 + 2 + 4; // share_id + payload_len + otm_tag

/// Ratchet-backed epoch cell generator for UES Monocell Pool.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct EpochCellState {
    ratchet: TransportOtpRatchet,
    cell_size_l: usize,
    sss_k: usize,
    sss_n: usize,
    sss_interleave: bool,
    pending: Vec<PendingCellPayload>,
    pending_idx: usize,
    /// Reassemble SSS share wire chunks split across multiple epochs.
    #[zeroize(skip)]
    partial_share_wires: BTreeMap<u32, Vec<u8>>,
}

#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
struct PendingCellPayload {
    share_id: u32,
    bytes: Vec<u8>,
}

impl EpochCellState {
    /// Create state from a 32-byte OTP seed and pool parameters.
    pub fn new(seed: [u8; 32], cell_size_l: usize, sss_k: usize, sss_n: usize) -> Result<Self, ()> {
        if cell_size_l < CELL_HDR + 8 {
            return Err(());
        }
        let sss_interleave = sss_k > 0 && sss_n >= sss_k;
        Ok(EpochCellState {
            ratchet: TransportOtpRatchet::new(seed),
            cell_size_l,
            sss_k,
            sss_n,
            sss_interleave,
            pending: Vec::new(),
            pending_idx: 0,
            partial_share_wires: BTreeMap::new(),
        })
    }

    /// Current epoch index (monotonic step counter).
    pub fn epoch(&self) -> u64 {
        self.ratchet.counter
    }

    /// Queue a secret for SSS interleaved transmission across upcoming `step()` calls.
    pub fn queue_sss_payload<R: SecureRandom>(
        &mut self,
        secret: &[u8],
        rng: &mut R,
    ) -> Result<(), ()> {
        self.pending.clear();
        self.pending_idx = 0;
        if !self.sss_interleave {
            self.pending.push(PendingCellPayload {
                share_id: 1,
                bytes: secret.to_vec(),
            });
            return Ok(());
        }
        let shares = fragment_data(secret, self.sss_k, self.sss_n, rng)?;
        let max_payload = self.max_payload_len();
        for share in shares {
            let id = share.id.value();
            let wire = serialize_share_wire(&share);
            for chunk in wire.chunks(max_payload) {
                self.pending.push(PendingCellPayload {
                    share_id: id,
                    bytes: chunk.to_vec(),
                });
            }
        }
        if self.sss_interleave && self.pending.len() > 1 {
            interleave_in_place(&mut self.pending);
        }
        Ok(())
    }

    /// Epochs required to drain queued payloads (0 if nothing queued).
    pub fn queued_epochs(&self) -> usize {
        self.pending.len().saturating_sub(self.pending_idx)
    }

    /// Extra chaff epochs when fountain coding is enabled (v1.6).
    pub fn fountain_extra_chaff_epochs(&self, fountain_enabled: bool) -> usize {
        if fountain_enabled {
            self.sss_n.saturating_sub(self.sss_k) + 2
        } else {
            2
        }
    }

    /// Max verified shares to collect before giving up on fountain reconstruction.
    pub fn fountain_max_shares(&self, fountain_enabled: bool) -> usize {
        if fountain_enabled {
            self.sss_n
        } else {
            self.sss_k
        }
    }

    /// Advance ratchet one epoch; returns `(K_{e+1}, C_e)` where `C_e` has fixed length `L`.
    pub fn step<R: SecureRandom>(&mut self, rng: &mut R) -> Result<(FieldElement, Vec<u8>), ()> {
        let (k_pool, k_mac, nonce) = self.ratchet.step()?;
        let mut cell = Vec::with_capacity(self.cell_size_l);
        cell.resize(self.cell_size_l, 0);
        rng.fill_bytes(&mut cell).map_err(|_| ())?;

        if self.pending_idx < self.pending.len() {
            let payload = self.pending[self.pending_idx].clone();
            self.pending_idx += 1;
            embed_payload(&mut cell, &payload, k_mac, nonce);
        } else {
            write_chaff_header(&mut cell);
        }

        Ok((k_pool, cell))
    }

    /// Verify and extract an SSS share fragment from a harvested cell at `epoch`.
    pub fn verify_cell(
        &mut self,
        epoch: u64,
        cell: &[u8],
    ) -> Result<Option<SssPackedShare>, ()> {
        if cell.len() != self.cell_size_l {
            return Err(());
        }
        while self.ratchet.counter < epoch {
            self.ratchet.step()?;
        }
        if self.ratchet.counter != epoch {
            return Err(());
        }
        let (k_pool, k_mac, nonce) = self.ratchet.step()?;
        let _ = k_pool;

        let share_id = u32::from_be_bytes([cell[0], cell[1], cell[2], cell[3]]);
        let payload_len = u16::from_be_bytes([cell[4], cell[5]]) as usize;
        if share_id == 0 || payload_len == 0 {
            return Ok(None);
        }
        if CELL_HDR + payload_len > cell.len() {
            return Err(());
        }
        let payload = &cell[6..6 + payload_len];
        let tag_val = u32::from_be_bytes([
            cell[6 + payload_len],
            cell[6 + payload_len + 1],
            cell[6 + payload_len + 2],
            cell[6 + payload_len + 3],
        ]);
        let y = message_fe(payload);
        let tag = FieldElement::new(tag_val);
        if !bool::from(verify_tag(k_mac, y, nonce, tag)) {
            return Ok(None);
        }
        let buf = self
            .partial_share_wires
            .entry(share_id)
            .or_insert_with(Vec::new);
        buf.extend_from_slice(payload);
        match deserialize_share_wire(share_id, buf) {
            Ok(share) => {
                self.partial_share_wires.remove(&share_id);
                Ok(Some(share))
            }
            Err(()) => Ok(None),
        }
    }

    fn max_payload_len(&self) -> usize {
        self.cell_size_l.saturating_sub(CELL_HDR)
    }
}

fn write_chaff_header(cell: &mut [u8]) {
    cell[0..4].copy_from_slice(&0u32.to_be_bytes());
    cell[4..6].copy_from_slice(&0u16.to_be_bytes());
}

fn embed_payload(cell: &mut [u8], payload: &PendingCellPayload, k_mac: FieldElement, nonce: FieldElement) {
    let plen = payload.bytes.len();
    if CELL_HDR + plen > cell.len() {
        return;
    }
    cell[0..4].copy_from_slice(&payload.share_id.to_be_bytes());
    cell[4..6].copy_from_slice(&(plen as u16).to_be_bytes());
    cell[6..6 + plen].copy_from_slice(&payload.bytes);
    let y = message_fe(&payload.bytes);
    let tag = generate_tag(k_mac, y, nonce);
    let tag_off = 6 + plen;
    cell[tag_off..tag_off + 4].copy_from_slice(&(tag.value() as u32).to_be_bytes());
}

fn message_fe(data: &[u8]) -> FieldElement {
    let mut acc = [0u8; 4];
    for (i, &b) in data.iter().enumerate() {
        acc[i % 4] ^= b;
    }
    FieldElement::new(u32::from_be_bytes(acc))
}

fn interleave_in_place(items: &mut Vec<PendingCellPayload>) {
    if items.len() <= 1 {
        return;
    }
    let taken = core::mem::take(items);
    let max_id = taken.iter().map(|p| p.share_id).max().unwrap_or(0) as usize;
    let mut buckets = vec![Vec::new(); max_id + 1];
    for p in taken {
        buckets[p.share_id as usize].push(p);
    }
    let mut out = Vec::with_capacity(buckets.iter().map(|b| b.len()).sum());
    loop {
        let mut progressed = false;
        for b in buckets.iter_mut() {
            if let Some(next) = b.first().cloned() {
                b.remove(0);
                out.push(next);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    items.extend(out);
}

fn serialize_share_wire(share: &SssPackedShare) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + share.data_points.len() * 4);
    bytes.extend_from_slice(&(share.id.value() as u32).to_be_bytes());
    bytes.extend_from_slice(&(share.data_points.len() as u32).to_be_bytes());
    for pt in &share.data_points {
        bytes.extend_from_slice(&(pt.value() as u32).to_be_bytes());
    }
    bytes
}

fn deserialize_share_wire(share_id: u32, payload: &[u8]) -> Result<SssPackedShare, ()> {
    if payload.len() < 8 {
        return Err(());
    }
    let id = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
    if id != share_id {
        return Err(());
    }
    let num_points = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]) as usize;
    let expected = 8 + num_points * 4;
    if payload.len() != expected {
        return Err(());
    }
    let mut data_points = Vec::with_capacity(num_points);
    let mut offset = 8;
    for _ in 0..num_points {
        let val = u32::from_be_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
        ]);
        data_points.push(FieldElement::new(val));
        offset += 4;
    }
    Ok(SssPackedShare {
        id: FieldElement::new(share_id),
        data_points,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reconstruct_data;

    struct MockRng {
        state: u32,
    }

    impl SecureRandom for MockRng {
        type Error = ();

        fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
            for byte in dest.iter_mut() {
                self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
                *byte = (self.state >> 16) as u8;
            }
            Ok(())
        }
    }

    #[test]
    fn epoch_cell_step_fixed_size() {
        let seed = [0x11u8; 32];
        let mut state = EpochCellState::new(seed, 256, 2, 3).unwrap();
        let mut rng = MockRng { state: 0xDEADBEEF };
        let (k1, c1) = state.step(&mut rng).unwrap();
        assert_eq!(c1.len(), 256);
        assert_eq!(state.epoch(), 1);
        let (k2, c2) = state.step(&mut rng).unwrap();
        assert_ne!(k1.value(), k2.value());
        assert_eq!(c2.len(), 256);
    }

    #[test]
    fn epoch_cell_sss_interleave_roundtrip() {
        let seed = [0x42u8; 32];
        let mut alice = EpochCellState::new(seed, 512, 2, 3).unwrap();
        let mut bob = EpochCellState::new(seed, 512, 2, 3).unwrap();
        let mut rng = MockRng { state: 0xCAFEBABE };
        let secret = b"UES pool epoch cell interleave";
        alice.queue_sss_payload(secret, &mut rng).unwrap();
        let epochs = alice.queued_epochs();
        assert!(epochs >= 3);

        let mut cells = Vec::new();
        for e in 0..epochs {
            let (_, cell) = alice.step(&mut rng).unwrap();
            cells.push((e as u64, cell));
        }

        let mut shares = Vec::new();
        for (epoch, cell) in cells {
            if let Some(share) = bob.verify_cell(epoch, &cell).unwrap() {
                if !shares.iter().any(|s: &SssPackedShare| s.id == share.id) {
                    shares.push(share);
                }
            }
        }
        assert!(shares.len() >= 2);
        let recovered = reconstruct_data(&shares, 2).unwrap();
        assert_eq!(recovered, secret);
    }

    #[test]
    fn epoch_cell_sss_multi_chunk_roundtrip() {
        let seed = [0x99u8; 32];
        let mut alice = EpochCellState::new(seed, 512, 2, 3).unwrap();
        let mut bob = EpochCellState::new(seed, 512, 2, 3).unwrap();
        let mut rng = MockRng { state: 0xFEEDFACE };
        // ~200 bytes forces multiple wire chunks per share (max_payload = 502).
        let secret: Vec<u8> = (0..200).map(|i| (i as u8).wrapping_mul(7)).collect();
        alice.queue_sss_payload(&secret, &mut rng).unwrap();
        let epochs = alice.queued_epochs();
        assert!(
            epochs > 5,
            "expected >5 payload epochs for multi-chunk shares, got {epochs}"
        );

        let mut cells = Vec::new();
        for e in 0..epochs {
            let (_, cell) = alice.step(&mut rng).unwrap();
            cells.push((e as u64, cell));
        }

        let mut shares = Vec::new();
        for (epoch, cell) in cells {
            if let Some(share) = bob.verify_cell(epoch, &cell).unwrap() {
                if !shares.iter().any(|s: &SssPackedShare| s.id == share.id) {
                    shares.push(share);
                }
            }
        }
        assert!(shares.len() >= 2);
        let recovered = reconstruct_data(&shares, 2).unwrap();
        assert_eq!(recovered, secret);
    }

    #[test]
    fn rust_epoch_cell_refines_ideal() {
        // M17 / X4: counter aligns with Lean idealStep.1 (= e + 1); fixed cell size L.
        const L: usize = 256;
        let seed = [0x55u8; 32];
        let mut state = EpochCellState::new(seed, L, 2, 3).unwrap();
        let mut rng = MockRng { state: 0x1234 };

        assert_eq!(state.epoch(), 0, "initial epoch index");

        for e in 0u64..8 {
            assert_eq!(state.epoch(), e, "epoch index before step");
            let (_, cell) = state.step(&mut rng).unwrap();
            assert_eq!(cell.len(), L, "fixed cell size L");
            assert_eq!(state.epoch(), e + 1, "idealStep counter component (e + 1)");
        }

        // Deterministic replay: fresh state + same RNG seed ⇒ identical ratchet keys and cells.
        let mut replay = EpochCellState::new(seed, L, 2, 3).unwrap();
        let mut rng_replay = MockRng { state: 0x1234 };
        let (k_orig, c_orig) = EpochCellState::new(seed, L, 2, 3)
            .unwrap()
            .step(&mut MockRng { state: 0x1234 })
            .unwrap();
        let (k_rep, c_rep) = replay.step(&mut rng_replay).unwrap();
        assert_eq!(k_orig.value(), k_rep.value(), "deterministic ratchet replay");
        assert_eq!(c_orig, c_rep, "deterministic cell draw under same RNG");
    }
}
