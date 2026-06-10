use crate::field_arith::FieldElement;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A polynomial over Z_17 of degree at most K - 1.
///
/// Represented as a fixed-size array of coefficients:
/// `P(x) = c_0 + c_1 * x + c_2 * x^2 + ... + c_{K-1} * x^{K-1}`.
///
/// Using const-generics ensures that the polynomial's degree is fixed at compile-time,
/// completely avoiding dynamic memory allocation (`alloc`) and ensuring deterministic
/// stack-allocated memory layout suitable for bare-metal, FPGA, and seL4 targets.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct Polynomial<const K: usize> {
    /// The coefficients of the polynomial, from lowest degree (c_0) to highest (c_{K-1}).
    pub coeffs: [FieldElement; K],
}

impl<const K: usize> Polynomial<K> {
    /// Creates a new `Polynomial` from a fixed array of coefficients.
    #[inline]
    pub fn new(coeffs: [FieldElement; K]) -> Self {
        Polynomial { coeffs }
    }

    /// Evaluates the polynomial at point `x` in constant-time using Horner's method.
    ///
    /// Horner's method nests operations:
    /// `P(x) = c_0 + x * (c_1 + x * (c_2 + ... + x * c_{K-1}))`
    ///
    /// # Side-Channel Resistance
    /// - The loop always executes exactly `K - 1` times, which is a compile-time constant.
    /// - No conditional branching or early exits based on secret values.
    /// - Memory access pattern is completely independent of the coefficients or the evaluation point.
    pub fn evaluate(&self, x: FieldElement) -> FieldElement {
        if K == 0 {
            return FieldElement::zero();
        }

        let mut result = self.coeffs[K - 1];

        // The loop bounds are entirely determined by the compile-time constant K.
        // This ensures identical instruction execution counts and branch behavior.
        for i in (0..K - 1).rev() {
            result = (result * x) + self.coeffs[i];
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial_evaluation() {
        // P(x) = 5 + 3x (modulo 17)
        // P(0) = 5
        // P(1) = 8
        // P(2) = 11
        // P(3) = 14
        // P(4) = 17 = 0
        let p = Polynomial::new([FieldElement::new(5), FieldElement::new(3)]);

        assert_eq!(p.evaluate(FieldElement::new(0)).value(), 5);
        assert_eq!(p.evaluate(FieldElement::new(1)).value(), 8);
        assert_eq!(p.evaluate(FieldElement::new(2)).value(), 11);
        assert_eq!(p.evaluate(FieldElement::new(3)).value(), 14);
        assert_eq!(p.evaluate(FieldElement::new(4)).value(), 0);
    }

    #[test]
    fn test_higher_degree_evaluation() {
        // P(x) = 1 + 2x + 3x^2 (modulo 17)
        // P(0) = 1
        // P(1) = 6
        // P(2) = 1 + 4 + 12 = 17 = 0
        // P(3) = 1 + 6 + 27 = 34 = 0
        let p = Polynomial::new([
            FieldElement::new(1),
            FieldElement::new(2),
            FieldElement::new(3),
        ]);

        assert_eq!(p.evaluate(FieldElement::new(0)).value(), 1);
        assert_eq!(p.evaluate(FieldElement::new(1)).value(), 6);
        assert_eq!(p.evaluate(FieldElement::new(2)).value(), 0);
        assert_eq!(p.evaluate(FieldElement::new(3)).value(), 0);
    }
}
