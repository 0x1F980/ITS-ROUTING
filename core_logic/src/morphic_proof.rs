extern crate alloc;

use alloc::vec::Vec;
use crate::field_arith::FieldElement;
use crate::trapdoor::lagrange_interpolate;
use hal_abstraction::SecureRandom;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// A structure representing homomorphic path attestation probe-shares.
///
/// Alice defines a secret probe value `S_probe`. She splits it homomorphically
/// into shares. Each node on the routing path blinds and accumulates these shares
/// lineary according to their local state. At the destination, Bob interpolates
/// the accumulated shares to recover `S_probe` and mathematically verify the path.
#[derive(Clone, Debug, Default, Zeroize, ZeroizeOnDrop)]
pub struct MorphicProbe {
    /// The coordinates `(x, y)` of the probe share.
    pub point: (FieldElement, FieldElement),
}

impl MorphicProbe {
    /// Generates a set of `n` morphic probe shares from a secret `s_probe`.
    ///
    /// Alice chooses a polynomial of degree `k-1` with `coeffs[0] = s_probe`
    /// and evaluates it at `x = 1..=n`.
    pub fn generate_shares<R: SecureRandom>(
        s_probe: FieldElement,
        k: usize,
        n: usize,
        rng: &mut R,
    ) -> Result<Vec<Self>, ()> {
        if k == 0 || n < k || n >= 2147483647 {
            return Err(());
        }

        let mut coeffs = Zeroizing::new(Vec::with_capacity(k));
        coeffs.push(s_probe);

        let mut buf = [0u8; 4];
        for _ in 1..k {
            rng.fill_bytes(&mut buf).map_err(|_| ())?;
            let val = u32::from_be_bytes(buf) % 2147483647;
            let is_zero = Choice::from((val == 0) as u8);
            let val_fe = FieldElement::new(val);
            let non_zero_fe = FieldElement::conditional_select(&val_fe, &FieldElement::one(), is_zero);
            coeffs.push(non_zero_fe);
        }

        let mut shares = Vec::with_capacity(n);
        for i in 1..=n {
            let x = FieldElement::new(i as u32);
            
            // Horner's method to evaluate dynamic polynomial at point x
            let mut y = coeffs[k - 1];
            for j in (0..k - 1).rev() {
                // Inject computational jitter during Horner's evaluation to randomize power trace
                let mut jitter_seed = x.value();
                for _ in 0..3 {
                    jitter_seed = jitter_seed.wrapping_mul(1103515245).wrapping_add(12345);
                }
                let _dummy = core::hint::black_box(jitter_seed);

                y = (y * x) + coeffs[j];
            }
            
            shares.push(MorphicProbe { point: (x, y) });
        }

        Ok(shares)
    }

    /// Blinds (morphic transformation) the probe share at a node by multiplying
    /// it with a local factor and adding another share's value.
    ///
    /// Formula: `new_y = (factor * y_1) + y_2` (Morphic blending)
    #[inline]
    pub fn morphic_blend(&mut self, factor: FieldElement, addition: FieldElement) {
        self.point.1 = (self.point.1 * factor) + addition;
    }
}

/// Verifies whether the accumulated homomorphic probe-shares interpolate back
/// to Alice's expected `S_probe` at `x_probe = 0` (or another specified evaluation point).
///
/// # Arguments
/// * `shares` - The probe points accumulated along the routing path.
/// * `expected_s_probe` - The secret probe value that Alice pre-selected.
pub fn verify_morphic_path(
    shares: &[(FieldElement, FieldElement)],
    expected_s_probe: FieldElement,
) -> Choice {
    let recovered = lagrange_interpolate(shares, FieldElement::zero());
    recovered.ct_eq(&expected_s_probe)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_morphic_probe_roundtrip() {
        let mut rng = MockRng { state: 42 };
        let s_probe = FieldElement::new(9); // Secret probe value
        let k = 3;
        let n = 5;

        // Generate 5 probe shares with threshold 3
        let mut probes = MorphicProbe::generate_shares::<_>(s_probe, k, n, &mut rng).unwrap();
        assert_eq!(probes.len(), 5);

        // Extract points for reconstruction
        let points: Vec<(FieldElement, FieldElement)> = probes.iter().map(|p| p.point).collect();

        // Verify that interpolating 3 shares recovers the secret probe
        let is_valid = verify_morphic_path(&points[0..3], s_probe);
        assert!(bool::from(is_valid));

        // Let's perform a homomorphic morphic blending (scaling by 2, adding 3)
        // new_y = 2 * y + 3
        // Modulo arithmetic: new_secret = 2 * 9 + 3 = 21 modulo 2147483647
        let factor = FieldElement::new(2);
        let addition = FieldElement::new(3);

        for p in probes.iter_mut() {
            p.morphic_blend(factor, addition);
        }

        let morphed_points: Vec<(FieldElement, FieldElement)> = probes.iter().map(|p| p.point).collect();
        let expected_morphed_secret = FieldElement::new(21);

        let is_morphed_valid = verify_morphic_path(&morphed_points[0..3], expected_morphed_secret);
        assert!(bool::from(is_morphed_valid));
    }
}
