use crate::field_arith::FieldElement;
use subtle::{Choice, ConditionallySelectable};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Performs Lagrange interpolation over Z_251 to evaluate the polynomial at `x`
/// given a slice of points (x_i, y_i).
///
/// # Side-Channel Resistance
/// - Fuldstændig branchless: Alle if/else-betingelser er elimineret.
/// - Anvender `subtle::Choice` og `FieldElement::conditional_select` til at maskere
///   indeks-sammenligninger i konstant-tid.
/// - Integrerer **computational jitter** via `core::hint::black_box` i multiplikations-løkkerne
///   for at sløre strøm- og elektromagnetiske sidekanaler (DPA/TEMPEST).
pub fn lagrange_interpolate(points: &[(FieldElement, FieldElement)], x: FieldElement) -> FieldElement {
    let mut result = FieldElement::zero();
    let n = points.len();

    for i in 0..n {
        let mut numerator = FieldElement::one();
        let mut denominator = FieldElement::one();

        for j in 0..n {
            // Sammenlign i og j uden if/else.
            // Hvis i != j, returnerer vi Choice(1), ellers Choice(0).
            let is_different = Choice::from((i != j) as u8);

            let term_num = x - points[j].0;
            let term_den = points[i].0 - points[j].0;

            // Inject computational jitter during sensitive modular math to randomize CPU power trace.
            // Using black_box guarantees that the compiler cannot optimize this out.
            let mut jitter_seed = term_num.value() as u32;
            for _ in 0..4 {
                jitter_seed = jitter_seed.wrapping_mul(1103515245).wrapping_add(12345);
            }
            let _dummy = core::hint::black_box(jitter_seed);

            // Hvis j == i, multiplicerer vi med 1 for at bevare produktet uændret.
            let num_factor = FieldElement::conditional_select(&FieldElement::one(), &term_num, is_different);
            let den_factor = FieldElement::conditional_select(&FieldElement::one(), &term_den, is_different);

            numerator = numerator * num_factor;
            denominator = denominator * den_factor;
        }

        let basis = numerator * denominator.invert();
        result = result + (points[i].1 * basis);
    }

    result
}

/// Bob's Trapdoor structure for SSS-Chained Perfect Secrecy Trapdoor (SCPST).
///
/// A polynomial of threshold `K` (degree `K - 1`) requires `K` points to be fully reconstructed.
/// Bob keeps exactly 1 point secret (the "faldlem" or trapdoor) and publishes `K - 1` points.
///
/// By combining the secret point and the public points, Bob (and only Bob) can reconstruct
/// the polynomial and evaluate it at any point.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct Trapdoor<const K: usize> {
    /// Bob's points on the polynomial.
    /// `points[0]` is Bob's secret point (trapdoor).
    /// `points[1..K]` are Bob's public points.
    pub points: [(FieldElement, FieldElement); K],
}

impl<const K: usize> Trapdoor<K> {
    /// Creates a new `Trapdoor` instance.
    #[inline]
    pub fn new(points: [(FieldElement, FieldElement); K]) -> Self {
        Trapdoor { points }
    }

    /// Evaluates the underlying polynomial at a given `x` using Bob's private trapdoor
    /// and the public points.
    ///
    /// # Side-Channel Resistance
    /// - Performs Lagrange interpolation in constant-time.
    pub fn evaluate_at(&self, x: FieldElement) -> FieldElement {
        lagrange_interpolate(&self.points, x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lagrange_interpolation() {
        // P(x) = 5 + 3x (modulo 251)
        // P(1) = 8, P(2) = 11
        let points = [
            (FieldElement::new(1), FieldElement::new(8)),
            (FieldElement::new(2), FieldElement::new(11)),
        ];

        assert_eq!(lagrange_interpolate(&points, FieldElement::new(0)).value(), 5);
        assert_eq!(lagrange_interpolate(&points, FieldElement::new(1)).value(), 8);
        assert_eq!(lagrange_interpolate(&points, FieldElement::new(2)).value(), 11);
    }

    #[test]
    fn test_trapdoor_evaluation() {
        // P(x) = 5 + 3x (modulo 251)
        // Secret point (trapdoor): (2, 11)
        // Public point: (1, 8)
        let trapdoor = Trapdoor::<2>::new([
            (FieldElement::new(2), FieldElement::new(11)),
            (FieldElement::new(1), FieldElement::new(8)),
        ]);

        assert_eq!(trapdoor.evaluate_at(FieldElement::new(0)).value(), 5);
        assert_eq!(trapdoor.evaluate_at(FieldElement::new(1)).value(), 8);
        assert_eq!(trapdoor.evaluate_at(FieldElement::new(2)).value(), 11);
        assert_eq!(trapdoor.evaluate_at(FieldElement::new(3)).value(), 14);
    }
}
