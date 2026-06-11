use crate::field_arith::FieldElement;
use crate::trapdoor::Trapdoor;
use hal_abstraction::SecureRandom;

/// Encapsulates (masks) a secret key `k_pool` using one of Bob's public points.
///
/// # Arguments
/// * `public_point` - One of Bob's public points `(x_i, P(x_i))`.
/// * `k_pool` - The secret key $K_{pool}$ to mask.
///
/// Returns the masked point `(x_i, y_masked)` where `y_masked = P(x_i) + K_{pool}`.
///
/// # Side-Channel Resistance
/// - Uses constant-time addition.
/// - No conditional branches or secret-dependent memory accesses.
#[inline]
pub fn encapsulate(
    public_point: (FieldElement, FieldElement),
    k_pool: FieldElement,
) -> (FieldElement, FieldElement) {
    let (x_i, y_i) = public_point;
    let y_masked = y_i + k_pool;
    (x_i, y_masked)
}

/// Decapsulates (unmasks) a masked point to extract the secret key `k_pool` using Bob's trapdoor.
///
/// # Arguments
/// * `trapdoor` - Bob's private trapdoor.
/// * `masked_point` - The masked point `(x_i, y_masked)` received from Alice.
///
/// Returns the extracted secret key $K_{pool}$.
///
/// # Side-Channel Resistance
/// - Performs polynomial evaluation via Lagrange interpolation in constant-time.
/// - Uses constant-time subtraction to isolate the key.
/// - No conditional branches or secret-dependent memory accesses.
#[inline]
pub fn decapsulate<const K: usize>(
    trapdoor: &Trapdoor<K>,
    masked_point: (FieldElement, FieldElement),
) -> FieldElement {
    let (x_i, y_masked) = masked_point;
    let y_i = trapdoor.evaluate_at(x_i);
    y_masked - y_i
}

/// Encapsulates a secret key generated directly from Alice's TRNG pool.
///
/// This integrates Alice's TRNG to generate a fresh, random `k_pool` and mask it.
///
/// # Errors
/// Returns the underlying TRNG error if entropy generation fails.
pub fn encapsulate_with_trng<R: SecureRandom>(
    rng: &mut R,
    public_point: (FieldElement, FieldElement),
) -> Result<((FieldElement, FieldElement), FieldElement), R::Error> {
    let mut buf = [0u8; 2];
    rng.fill_bytes(&mut buf)?;

    let val_raw = (buf[0] as u16) | ((buf[1] as u16) << 8);
    let k_pool = FieldElement::new(val_raw);
    let masked_point = encapsulate(public_point, k_pool);

    Ok((masked_point, k_pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRng {
        value: u8,
    }

    impl SecureRandom for MockRng {
        type Error = ();

        fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
            for byte in dest.iter_mut() {
                *byte = self.value;
            }
            Ok(())
        }
    }

    #[test]
    fn test_encapsulation_roundtrip() {
        // P(x) = 5 + 3x (modulo 65521)
        // Public point: (1, 8)
        // Secret point (trapdoor): (2, 11)
        let public_point = (FieldElement::new(1), FieldElement::new(8));
        let trapdoor = Trapdoor::<2>::new([
            (FieldElement::new(2), FieldElement::new(11)),
            public_point,
        ]);

        let k_pool = FieldElement::new(12);

        // Alice encapsulates k_pool
        let masked_point = encapsulate(public_point, k_pool);
        assert_eq!(masked_point.0.value(), 1);
        assert_eq!(masked_point.1.value(), 20); // 8 + 12 = 20 mod 65521

        // Bob decapsulates masked_point
        let recovered_k = decapsulate(&trapdoor, masked_point);
        assert_eq!(recovered_k.value(), 12);
    }

    #[test]
    fn test_encapsulate_with_trng() {
        let public_point = (FieldElement::new(1), FieldElement::new(8));
        let trapdoor = Trapdoor::<2>::new([
            (FieldElement::new(2), FieldElement::new(11)),
            public_point,
        ]);

        let mut rng = MockRng { value: 12 };
        let (masked_point, k_pool) = encapsulate_with_trng(&mut rng, public_point).unwrap();

        // 12 | (12 << 8) = 3084
        assert_eq!(k_pool.value(), 3084);
        assert_eq!(masked_point.0.value(), 1);
        assert_eq!(masked_point.1.value(), 3092); // 3084 + 8 = 3092 mod 65521

        let recovered_k = decapsulate(&trapdoor, masked_point);
        assert_eq!(recovered_k.value(), 3084);
    }
}
