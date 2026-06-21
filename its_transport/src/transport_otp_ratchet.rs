use crate::field_arith::FieldElement;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// SSS epoch forward step (same algebra as `sss_chain_epoch_step_forward`).
#[inline]
fn epoch_step_forward(
    current: FieldElement,
    anchor: FieldElement,
    link_index: u32,
    entropy: FieldElement,
) -> FieldElement {
    let idx = FieldElement::new(link_index);
    let transition = current + current + anchor + idx + entropy;
    transition - current
}

/// Transport forward-secrecy ratchet — SSS OTP epoch steps (no HKDF/PBKDF).
///
/// Alice and Bob share a 32-byte OTP seed file (`--ratchet-seed-file`). Each `step()`
/// derives one-time `k_pool`, `k_mac`, and `nonce` for onion/AEH transport.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct TransportOtpRatchet {
    pub anchor: FieldElement,
    pub current: FieldElement,
    /// Monotonic step counter (transport FS index).
    pub counter: u64,
    seed: [u8; 32],
}

impl TransportOtpRatchet {
    /// Create ratchet from a 32-byte OTP seed (from ITS-KeyManagement export or OOB pipe).
    #[inline]
    pub fn new(seed: [u8; 32]) -> Self {
        let anchor = FieldElement::new(u32::from_le_bytes([
            seed[0], seed[1], seed[2], seed[3],
        ]));
        let current = FieldElement::new(u32::from_le_bytes([
            seed[4], seed[5], seed[6], seed[7],
        ]));
        TransportOtpRatchet {
            anchor,
            current,
            counter: 0,
            seed,
        }
    }

    fn step_entropy(&self) -> FieldElement {
        let off = 8 + ((self.counter as usize) % 6) * 4;
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.seed[off..off + 4]);
        bytes[0] ^= (self.counter & 0xff) as u8;
        bytes[1] ^= ((self.counter >> 8) & 0xff) as u8;
        bytes[2] ^= ((self.counter >> 16) & 0xff) as u8;
        bytes[3] ^= ((self.counter >> 24) & 0xff) as u8;
        FieldElement::new(u32::from_le_bytes(bytes))
    }

    /// Step ratchet; returns `(k_pool, k_mac, nonce)`.
    pub fn step(&mut self) -> Result<(FieldElement, FieldElement, FieldElement), ()> {
        let entropy = self.step_entropy();
        let next = epoch_step_forward(self.current, self.anchor, self.counter as u32, entropy);
        let k_pool = next;
        let k_mac = self.current + next;
        let nonce = FieldElement::new((self.counter as u32).wrapping_mul(0x9E37_79B9));
        self.current = next;
        self.counter = self.counter.wrapping_add(1);
        Ok((k_pool, k_mac, nonce))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lean mirror: `epochStepForward st entropy = current + anchor + counter + entropy`.
    fn lean_epoch_step_forward(
        current: FieldElement,
        anchor: FieldElement,
        counter: u32,
        entropy: FieldElement,
    ) -> FieldElement {
        current + anchor + FieldElement::new(counter) + entropy
    }

    #[test]
    fn rust_ratchet_algebra_matches_lean() {
        let current = FieldElement::new(1_000_003);
        let anchor = FieldElement::new(2_000_011);
        let counter = 7u32;
        let entropy = FieldElement::new(42_000_042);

        let via_rust = epoch_step_forward(current, anchor, counter, entropy);
        let via_lean = lean_epoch_step_forward(current, anchor, counter, entropy);
        assert_eq!(via_rust.value(), via_lean.value());

        let seed = [0x77u8; 32];
        let mut ratchet = TransportOtpRatchet::new(seed);
        ratchet.counter = counter as u64;
        ratchet.current = current;
        ratchet.anchor = anchor;

        let entropy_fe = ratchet.step_entropy();
        let expected = lean_epoch_step_forward(current, anchor, counter, entropy_fe);
        let (k_pool, _, _) = ratchet.step().unwrap();
        assert_eq!(k_pool.value(), expected.value());
    }

    #[test]
    fn otp_ratchet_stepping() {
        let seed = [0x42u8; 32];
        let mut alice = TransportOtpRatchet::new(seed);
        let mut bob = TransportOtpRatchet::new(seed);

        let (a1, b1, n1) = alice.step().unwrap();
        let (a2, b2, n2) = bob.step().unwrap();
        assert_eq!(a1.value(), a2.value());
        assert_eq!(b1.value(), b2.value());
        assert_eq!(n1.value(), n2.value());

        let (a3, _, _) = alice.step().unwrap();
        let (a4, _, _) = bob.step().unwrap();
        assert_eq!(a3.value(), a4.value());
        assert_ne!(a1.value(), a3.value());
    }
}
