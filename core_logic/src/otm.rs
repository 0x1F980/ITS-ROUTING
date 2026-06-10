use crate::field_arith::FieldElement;
use crate::poly::Polynomial;
use crate::trapdoor::lagrange_interpolate;
use subtle::{Choice, ConstantTimeEq};
use zeroize::Zeroize;

/// Generates a Wegman-Carter One-Time MAC (OTM) tag for a given value `y`.
///
/// Formula: `T = (K_MAC * y + N) mod 17`
///
/// # Arguments
/// * `k_mac` - The shared one-time MAC key.
/// * `y` - The value/message to authenticate (e.g., the masked point's y-coordinate).
/// * `nonce` - The shared one-time nonce.
///
/// Returns the authentication tag `T`.
///
/// # Side-Channel Resistance
/// - Uses constant-time field multiplication and addition.
/// - No conditional branching or secret-dependent execution paths.
#[inline]
pub fn generate_tag(k_mac: FieldElement, y: FieldElement, nonce: FieldElement) -> FieldElement {
    (k_mac * y) + nonce
}

/// Verifies a Wegman-Carter One-Time MAC tag in constant-time.
///
/// # Arguments
/// * `k_mac` - The shared one-time MAC key.
/// * `y` - The value/message to authenticate.
/// * `nonce` - The shared one-time nonce.
/// * `tag` - The received authentication tag to verify.
///
/// Returns a `subtle::Choice` representing whether the tag is valid (1) or invalid (0).
///
/// # Side-Channel Resistance
/// - Verification is performed in constant-time using `subtle::ConstantTimeEq`.
/// - Avoids early-exit branching (e.g., `if tag == expected`) to prevent timing attacks.
#[inline]
pub fn verify_tag(
    k_mac: FieldElement,
    y: FieldElement,
    nonce: FieldElement,
    tag: FieldElement,
) -> Choice {
    let expected = generate_tag(k_mac, y, nonce);
    tag.ct_eq(&expected)
}

/// Combines forward and backward SSS-derived elements to produce a combined key or tag.
///
/// In finite field arithmetic, addition (+) is used as the secure, algebraic equivalent
/// of XOR (⊕) to preserve the field properties of Z_p.
///
/// # Arguments
/// * `sss_forward` - The forward SSS-derived element.
/// * `sss_backward` - The backward SSS-derived element.
///
/// Returns the combined field element.
#[inline]
pub fn combine_sss_chains(
    sss_forward: FieldElement,
    sss_backward: FieldElement,
) -> FieldElement {
    sss_forward + sss_backward
}

/// Deterministically connects the forward SSS chain to the backward SSS chain.
///
/// The forward secret is derived from the previous backward point and the previous message.
///
/// # Arguments
/// * `prev_backward_point` - The backward SSS point from the previous step.
/// * `prev_message` - The message from the previous step.
#[inline]
pub fn derive_forward_secret(
    prev_backward_point: (FieldElement, FieldElement),
    prev_message: FieldElement,
) -> FieldElement {
    prev_backward_point.1 + prev_message
}

/// Generates a combined Wegman-Carter tag using the SSS forward and backward points.
///
/// Formula: `T = (K_MAC * (y_forward + y_backward) + N) mod 17`
#[inline]
pub fn generate_chained_tag_with_points(
    forward_point: (FieldElement, FieldElement),
    backward_point: (FieldElement, FieldElement),
    k_mac: FieldElement,
    nonce: FieldElement,
) -> FieldElement {
    let mut y = combine_sss_chains(forward_point.1, backward_point.1);
    let tag = generate_tag(k_mac, y, nonce);
    y.zeroize();
    tag
}

/// Verifies a combined Wegman-Carter tag using the SSS forward and backward points in constant-time.
#[inline]
pub fn verify_chained_tag_with_points(
    forward_point: (FieldElement, FieldElement),
    backward_point: (FieldElement, FieldElement),
    k_mac: FieldElement,
    nonce: FieldElement,
    tag: FieldElement,
) -> Choice {
    let expected = generate_chained_tag_with_points(forward_point, backward_point, k_mac, nonce);
    tag.ct_eq(&expected)
}

/// Verifies a forward SSS share against a given forward polynomial in constant-time.
///
/// Checks that:
/// 1. The x-coordinate of the share matches the message.
/// 2. The y-coordinate of the share is the correct evaluation of the polynomial at the message.
pub fn verify_forward_share<const K: usize>(
    poly_forward: &Polynomial<K>,
    message: FieldElement,
    forward_point: (FieldElement, FieldElement),
) -> Choice {
    let x_matches = forward_point.0.ct_eq(&message);
    let expected_y = poly_forward.evaluate(message);
    let y_matches = forward_point.1.ct_eq(&expected_y);
    x_matches & y_matches
}

/// Verifies a backward SSS share against the Master-Root and previous points in constant-time.
///
/// Reconstructs the polynomial using the Master-Root (at x=0) and the previous `K - 1` points,
/// then verifies that the new point lies on this polynomial.
pub fn verify_backward_share<const K: usize>(
    master_root: FieldElement,
    prev_points: &[(FieldElement, FieldElement)],
    new_point: (FieldElement, FieldElement),
) -> Choice {
    // We construct the K points: (0, master_root) and the K-1 prev_points.
    let mut points = [(FieldElement::zero(), FieldElement::zero()); K];
    points[0] = (FieldElement::zero(), master_root);
    for (i, pt) in points.iter_mut().enumerate().take(K).skip(1) {
        let idx = i - 1;
        if idx < prev_points.len() {
            *pt = prev_points[idx];
        }
    }

    let expected_y = lagrange_interpolate(&points, new_point.0);
    new_point.1.ct_eq(&expected_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otm_generation_and_verification() {
        let k_mac = FieldElement::new(5);
        let y = FieldElement::new(3);
        let nonce = FieldElement::new(10);

        // T = (5 * 3 + 10) = 25 = 8 mod 17
        let tag = generate_tag(k_mac, y, nonce);
        assert_eq!(tag.value(), 8);

        // Verification should succeed
        let is_valid = verify_tag(k_mac, y, nonce, tag);
        assert!(bool::from(is_valid));

        // Verification should fail if y is altered
        let is_valid_altered_y = verify_tag(k_mac, FieldElement::new(4), nonce, tag);
        assert!(!bool::from(is_valid_altered_y));

        // Verification should fail if tag is altered
        let is_valid_altered_tag = verify_tag(k_mac, y, nonce, FieldElement::new(9));
        assert!(!bool::from(is_valid_altered_tag));
    }

    #[test]
    fn test_combine_sss_chains() {
        let forward = FieldElement::new(12);
        let backward = FieldElement::new(7);
        // (12 + 7) = 19 = 2 mod 17
        let combined = combine_sss_chains(forward, backward);
        assert_eq!(combined.value(), 2);
    }

    #[test]
    fn test_chained_tag_with_points() {
        let forward_point = (FieldElement::new(3), FieldElement::new(12));
        let backward_point = (FieldElement::new(1), FieldElement::new(7));
        let k_mac = FieldElement::new(5);
        let nonce = FieldElement::new(10);

        // y = 12 + 7 = 19 = 2 mod 17
        // T = (5 * 2 + 10) = 20 = 3 mod 17
        let tag = generate_chained_tag_with_points(forward_point, backward_point, k_mac, nonce);
        assert_eq!(tag.value(), 3);

        let is_valid = verify_chained_tag_with_points(forward_point, backward_point, k_mac, nonce, tag);
        assert!(bool::from(is_valid));
    }

    #[test]
    fn test_verify_forward_share() {
        // P(x) = 5 + 3x (modulo 17)
        let poly = Polynomial::new([FieldElement::new(5), FieldElement::new(3)]);
        let message = FieldElement::new(2);
        // P(2) = 11
        let forward_point = (FieldElement::new(2), FieldElement::new(11));

        let is_valid = verify_forward_share(&poly, message, forward_point);
        assert!(bool::from(is_valid));

        // Should fail if message doesn't match x
        let is_invalid_x = verify_forward_share(&poly, FieldElement::new(3), forward_point);
        assert!(!bool::from(is_invalid_x));

        // Should fail if y is incorrect
        let is_invalid_y = verify_forward_share(&poly, message, (FieldElement::new(2), FieldElement::new(12)));
        assert!(!bool::from(is_invalid_y));
    }

    #[test]
    fn test_verify_backward_share() {
        // P(x) = 5 + 3x (modulo 17)
        // Master-Root: P(0) = 5
        // prev_points: [P(1) = 8]
        // new_point: P(2) = 11
        let master_root = FieldElement::new(5);
        let prev_points = [(FieldElement::new(1), FieldElement::new(8))];
        let new_point = (FieldElement::new(2), FieldElement::new(11));

        let is_valid = verify_backward_share::<2>(master_root, &prev_points, new_point);
        assert!(bool::from(is_valid));

        // Should fail if new_point is not on the line
        let invalid_point = (FieldElement::new(2), FieldElement::new(12));
        let is_invalid = verify_backward_share::<2>(master_root, &prev_points, invalid_point);
        assert!(!bool::from(is_invalid));
    }
}
