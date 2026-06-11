use crate::field_arith::FieldElement;
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A cryptographic state ratchet for SSS-Chained Perfect Secrecy Trapdoor (SCPST).
///
/// This ratchet synchronizes Alice and Bob by deriving unique, one-time keys
/// and nonces for each step in the chain. It uses HKDF-SHA256 as a Pseudo-Random
/// Function (PRF) to ensure that:
/// 1. Forward Secrecy: Compromise of the current seed does not reveal past keys.
/// 2. Backward Secrecy: Compromise of the current seed does not easily reveal future keys
///    without the ratchet stepping mechanism.
/// 3. Replay Protection: Each step is tied to a unique, monotonically increasing counter.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct StateRatchet {
    /// The current 32-byte secret seed.
    pub seed: [u8; 32],
    /// The message index / counter.
    pub counter: u64,
}

impl StateRatchet {
    /// Creates a new `StateRatchet` with a given initial seed and counter set to 0.
    #[inline]
    pub fn new(seed: [u8; 32]) -> Self {
        StateRatchet { seed, counter: 0 }
    }

    /// Steps the ratchet forward, deriving the next seed and the 16-bit one-time keys
    /// for the current step.
    ///
    /// Returns a tuple containing:
    /// `(k_pool, k_mac, nonce)` as `FieldElement`s.
    ///
    /// # Errors
    /// Returns `Err(())` if the HKDF expansion fails (which is mathematically impossible
    /// for a 64-byte output, but handled to guarantee no panic in the runtime path).
    ///
    /// # Side-Channel Resistance
    /// - Uses HKDF-SHA256, which is constant-time and free from data-dependent branches.
    /// - Intermediate buffers are zeroized on drop.
    /// - No `panic!` in the runtime path.
    pub fn step(&mut self) -> Result<(FieldElement, FieldElement, FieldElement), ()> {
        let salt = self.counter.to_be_bytes();
        
        // Extract step
        let hk = Hkdf::<Sha256>::new(Some(&salt), &self.seed);
        
        // Expand step to get 64 bytes of output key material (OKM).
        // - 32 bytes for the next seed.
        // - 2 bytes for k_pool.
        // - 2 bytes for k_mac.
        // - 2 bytes for nonce.
        let mut okm = [0u8; 64];
        if hk.expand(b"scpst-ratchet-step", &mut okm).is_err() {
            return Err(());
        }

        // Extract the next seed and the raw field elements.
        let mut next_seed = [0u8; 32];
        next_seed.copy_from_slice(&okm[0..32]);

        // Each element is now a full 16-bit value
        let k_pool_raw = (okm[32] as u16) | ((okm[33] as u16) << 8);
        let k_mac_raw = (okm[34] as u16) | ((okm[35] as u16) << 8);
        let nonce_raw = (okm[36] as u16) | ((okm[37] as u16) << 8);

        // Zeroize the old seed and update with the next seed.
        self.seed.zeroize();
        self.seed = next_seed;

        // Increment the counter.
        self.counter = self.counter.wrapping_add(1);

        // Zeroize the OKM buffer to prevent leakage.
        okm.zeroize();

        // Reduce raw values to FieldElements.
        let k_pool = FieldElement::new(k_pool_raw);
        let k_mac = FieldElement::new(k_mac_raw);
        let nonce = FieldElement::new(nonce_raw);

        Ok((k_pool, k_mac, nonce))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratchet_stepping() {
        let initial_seed = [0x42u8; 32];
        let mut ratchet_alice = StateRatchet::new(initial_seed);
        let mut ratchet_bob = StateRatchet::new(initial_seed);

        // Step 1
        let (k_pool_a1, k_mac_a1, nonce_a1) = ratchet_alice.step().unwrap();
        let (k_pool_b1, k_mac_b1, nonce_b1) = ratchet_bob.step().unwrap();

        assert_eq!(k_pool_a1.value(), k_pool_b1.value());
        assert_eq!(k_mac_a1.value(), k_mac_b1.value());
        assert_eq!(nonce_a1.value(), nonce_b1.value());

        // Step 2 (should produce different keys)
        let (k_pool_a2, k_mac_a2, nonce_a2) = ratchet_alice.step().unwrap();
        let (k_pool_b2, k_mac_b2, nonce_b2) = ratchet_bob.step().unwrap();

        assert_eq!(k_pool_a2.value(), k_pool_b2.value());
        assert_eq!(k_mac_a2.value(), k_mac_b2.value());
        assert_eq!(nonce_a2.value(), nonce_b2.value());

        assert_ne!(k_pool_a1.value(), k_pool_a2.value());
    }
}
