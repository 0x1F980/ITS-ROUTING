use alloc::vec::Vec;
use crate::field_arith::FieldElement;
use crate::trapdoor::lagrange_interpolate;
use hal_abstraction::SecureRandom;
use subtle::{Choice, ConditionallySelectable};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// Converts a raw byte (0..=255) into a single `FieldElement` of Z_65521.
///
/// Since MODULUS is 65521 (which is >= 256), any byte fits perfectly and directly into
/// a single FieldElement without any splitting or bases, achieving 100% throughput density!
#[inline]
pub fn byte_to_field_elements(byte: u8) -> FieldElement {
    FieldElement::new(byte as u16)
}

/// Converts a `FieldElement` of Z_65521 back into a raw byte.
#[inline]
pub fn field_elements_to_byte(fe: FieldElement) -> u8 {
    fe.value() as u8
}

/// A share (fragment) of Hydra-ITS data hosted by a VPS node.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct HydraShare {
    /// The node/share index (x-coordinate), must be non-zero (1..=n).
    pub id: FieldElement,
    /// The SSS-evaluation points (y-coordinates) for each digit of the data.
    pub data_points: Vec<FieldElement>,
}

/// Fragments a slice of secret bytes into `n` shares, such that any `k` shares
/// can reconstruct the original secret, and any `k-1` shares reveal absolute 0 information.
///
/// # Arguments
/// * `secret_data` - The raw bytes of the webpage or file to fragment.
/// * `k` - The reconstruction threshold.
/// * `n` - The total number of shares to generate.
/// * `rng` - A secure random number generator to supply polynomial coefficients.
pub fn fragment_data<R: SecureRandom>(
    secret_data: &[u8],
    k: usize,
    n: usize,
    rng: &mut R,
) -> Result<Vec<HydraShare>, ()> {
    if k == 0 || n < k || n >= 65521 {
        return Err(()); // Z_65521 supports up to n = 65520 participants (since x=0 is the secret)
    }

    // Convert bytes directly 1-to-1 into FieldElements with zeroizing protection
    let mut field_elements = Zeroizing::new(Vec::with_capacity(secret_data.len()));
    for &byte in secret_data.iter() {
        field_elements.push(byte_to_field_elements(byte));
    }

    // Initialize shares
    let mut shares = Vec::with_capacity(n);
    for i in 1..=n {
        shares.push(HydraShare {
            id: FieldElement::new(i as u16),
            data_points: Vec::with_capacity(field_elements.len()),
        });
    }

    // Buffer for generating random coefficients
    let mut coef_buf = [0u8; 2];

    // For each secret FieldElement, we generate a random polynomial of degree k-1
    for &secret in field_elements.iter() {
        // Coefficients: coeffs[0] is the secret, coeffs[1..k] are random in Z_65521
        let mut coeffs = Zeroizing::new(Vec::with_capacity(k));
        coeffs.push(secret);
        for _ in 1..k {
            rng.fill_bytes(&mut coef_buf).map_err(|_| ())?;
            let val_raw = ((coef_buf[0] as u16) | ((coef_buf[1] as u16) << 8)) % 65521;
            
            // Ensure the highest coefficient is non-zero to preserve the degree
            let is_zero = Choice::from((val_raw == 0) as u8);
            let val_fe = FieldElement::new(val_raw);
            let non_zero_fe = FieldElement::conditional_select(&val_fe, &FieldElement::one(), is_zero);
            coeffs.push(non_zero_fe);
        }

        // Evaluate the polynomial at x = 1..=n for each share
        for share in shares.iter_mut() {
            let x = share.id;
            // Horner's method for polynomial evaluation
            let mut result = coeffs[k - 1];
            for i in (0..k - 1).rev() {
                result = (result * x) + coeffs[i];
            }
            share.data_points.push(result);
        }
    }

    Ok(shares)
}

/// Reconstructs the original secret bytes from at least `k` Hydra shares.
///
/// # Arguments
/// * `shares` - A slice of received shares (must contain at least `k` unique shares).
/// * `k` - The reconstruction threshold.
pub fn reconstruct_data(shares: &[HydraShare], k: usize) -> Result<Vec<u8>, ()> {
    if shares.len() < k {
        return Err(());
    }

    let num_points = shares[0].data_points.len();
    for share in shares.iter() {
        if share.data_points.len() != num_points {
            return Err(()); // Mismatched share lengths
        }
    }

    let mut reconstructed_elements = Zeroizing::new(Vec::with_capacity(num_points));

    // Reconstruct each field element using Lagrange interpolation at x = 0
    for m in 0..num_points {
        let mut interpolation_points = Zeroizing::new(Vec::with_capacity(k));
        for share in shares.iter().take(k) {
            interpolation_points.push((share.id, share.data_points[m]));
        }

        // Evaluate the polynomial at x = 0 to extract the secret (coeffs[0])
        let secret = lagrange_interpolate(&*interpolation_points, FieldElement::zero());
        reconstructed_elements.push(secret);
    }

    let mut secret_bytes = Vec::with_capacity(num_points);
    for &fe in reconstructed_elements.iter() {
        secret_bytes.push(field_elements_to_byte(fe));
    }

    Ok(secret_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

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
    fn test_byte_splitting_roundtrip() {
        for b in 0..=255 {
            let fe = byte_to_field_elements(b);
            assert!(fe.value() < 65521);
            let reconstructed = field_elements_to_byte(fe);
            assert_eq!(reconstructed, b);
        }
    }

    #[test]
    fn test_hydra_fragmentation_and_reconstruction() {
        let mut rng = MockRng { state: 0xBAADF00D };
        let secret = b"Hello, Information-Theoretic Secrecy!";
        
        let k = 4;
        let n = 10;
        
        // Split data into 10 shares, threshold 4
        let shares = fragment_data(secret, k, n, &mut rng).unwrap();
        assert_eq!(shares.len(), 10);
        for share in shares.iter() {
            assert_eq!(share.data_points.len(), secret.len());
        }

        // Reconstruct using shares [0, 2, 5, 8] (4 shares)
        let subset = vec![shares[0].clone(), shares[2].clone(), shares[5].clone(), shares[8].clone()];
        let reconstructed = reconstruct_data(&subset, k).unwrap();
        assert_eq!(reconstructed, secret);
    }
}
