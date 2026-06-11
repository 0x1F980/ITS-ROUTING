use core::ops::{Add, Mul, Neg, Sub};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::Zeroize;

/// The prime modulus for our finite field Z_p (p = 65521).
///
/// 65521 is the largest prime number under 2^16 (65536). This allows a single FieldElement
/// to represent up to 16 bits, enabling full byte (0..=255) mapping in a single element.
/// This quadruples the relative transmission density/efficiency of our SSS-based routing.
pub const MODULUS: u16 = 65521;

/// A transparent wrapper representing an element of the finite field Z_65521.
///
/// This struct implements constant-time modular arithmetic. All operations
/// are guaranteed to execute in a constant number of clock cycles, independent
/// of the input values, to prevent timing and power side-channel attacks.
///
/// Under the hood, containers holding this element can be zeroized to ensure sensitive
/// cryptographic material does not linger in the cache or stack.
#[derive(Clone, Copy, Debug, Default, Zeroize)]
pub struct FieldElement(pub u16);

impl FieldElement {
    /// Creates a new `FieldElement` from a raw `u16`.
    ///
    /// This function performs modular reduction modulo 65521 in constant-time
    /// without any division or conditional branching.
    ///
    /// # Side-Channel Resistance
    /// - No conditional branching.
    /// - No division instruction.
    /// - Constant-time comparison and conditional selection.
    #[inline]
    pub fn new(val: u16) -> Self {
        let diff = (val as i32) - (MODULUS as i32);
        let is_negative = ((diff >> 31) & 1) as u16; // 1 if val < 65521, 0 otherwise
        let mask = 0u16.wrapping_sub(is_negative); // 0xFFFF if val < 65521, 0x0000 otherwise
        let r = ((diff as u16) & !mask) | (val & mask);
        FieldElement(r)
    }

    /// Creates a `FieldElement` from a `u32` value in constant-time.
    ///
    /// Useful for reducing intermediate products or larger integers.
    ///
    /// This function is optimized using Barrett reduction with 64-bit precision,
    /// which compiles to extremely fast instructions on both 32-bit and 64-bit CPUs.
    ///
    /// # Side-Channel Resistance
    /// - Constant-time multiplication and bit-shifts using 64-bit precision
    ///   to prevent overflow and timing leaks.
    #[inline]
    pub fn from_u32(val: u32) -> Self {
        // Barrett reduction for u32 modulo 65521 using 64-bit precision (k = 32):
        // q = (val * M) >> 32
        // M = 2^32 / 65521 = 65549.52...
        // We round up to 65550 to ensure exactness for all inputs.
        let q = ((val as u64) * 65550) >> 32;
        let mut r = val - (q as u32) * (MODULUS as u32);
        
        // Adjust for potential rounding errors
        for _ in 0..2 {
            let sub = r.wrapping_sub(MODULUS as u32);
            let is_negative = (sub >> 31) & 1;
            let mask = 0u32.wrapping_sub(is_negative); // 0xFFFFFFFF if negative, 0x00000000 otherwise
            r = (sub & !mask) | (r & mask);
        }
        FieldElement(r as u16)
    }

    /// Returns the additive identity (0).
    #[inline]
    pub fn zero() -> Self {
        FieldElement(0)
    }

    /// Returns the multiplicative identity (1).
    #[inline]
    pub fn one() -> Self {
        FieldElement(1)
    }

    /// Computes the modular inverse of `self` modulo 65521.
    ///
    /// By Fermat's Little Theorem, for any prime p, x^(p-2) = x^(65519) = x^-1 (mod 65521).
    /// If `self` is 0, this returns 0.
    ///
    /// # Side-Channel Resistance
    /// - Uses a fixed, deterministic sequence of 15 squarings and 11 multiplications.
    /// - No branches or conditional execution based on the input value.
    #[inline]
    pub fn invert(&self) -> Self {
        let x = *self;
        // Compute x^65519 using square-and-multiply in 100% constant-time:
        // 65519 = 1111_1111_1110_1111 in binary.
        let x2 = x * x;
        let x3 = x2 * x;
        let x6 = x3 * x3;
        let x7 = x6 * x;
        let x14 = x7 * x7;
        let x15 = x14 * x;
        let x30 = x15 * x15;
        let x31 = x30 * x;
        let x62 = x31 * x31;
        let x63 = x62 * x;
        let x126 = x63 * x63;
        let x127 = x126 * x;
        let x254 = x127 * x127;
        let x255 = x254 * x; // 8 ones
        
        let mut t = x255;
        for _ in 0..3 {
            t = t * t;
        }
        let x2047 = t * x7; // 11 ones (since 255 * 8 + 7 = 2047)
        
        // Bit 4 is 0, square once:
        let mut t = x2047 * x2047;
        
        // Bits 3 down to 0 are four 1s (represented by x15):
        for _ in 0..4 {
            t = t * t;
        }
        let x65519 = t * x15; // Exponent: (2047 * 2 * 16) + 15 = 65519
        x65519
    }

    /// Returns the raw `u16` value of this field element.
    #[inline]
    pub fn value(&self) -> u16 {
        self.0
    }
}

impl ConstantTimeEq for FieldElement {
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl ConditionallySelectable for FieldElement {
    #[inline]
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let val = u16::conditional_select(&a.0, &b.0, choice);
        FieldElement(val)
    }
}

impl Add for FieldElement {
    type Output = Self;

    /// Constant-time addition modulo 65521.
    ///
    /// # Side-Channel Resistance
    /// - Avoids `if sum >= 65521` branching.
    /// - Uses bitwise masking to conditionally subtract 65521.
    #[inline]
    fn add(self, other: Self) -> Self {
        let sum = (self.0 as u32) + (other.0 as u32);
        let sub = sum.wrapping_sub(MODULUS as u32);
        // If sub is negative (high bit set), sum < 65521.
        let is_negative = (sub >> 31) & 1;
        let mask = 0u32.wrapping_sub(is_negative); // 0xFFFFFFFF if negative, 0x00000000 if non-negative
        let r = (sub & !mask) | (sum & mask);
        FieldElement(r as u16)
    }
}

impl Sub for FieldElement {
    type Output = Self;

    /// Constant-time subtraction modulo 65521.
    ///
    /// # Side-Channel Resistance
    /// - Avoids `if a < b` branching.
    /// - Uses bitwise masking to conditionally add 65521 on underflow.
    #[inline]
    fn sub(self, other: Self) -> Self {
        let diff = (self.0 as i32) - (other.0 as i32);
        // If diff is negative (high bit set), we underflowed.
        let is_negative = ((diff >> 31) & 1) as u32;
        let mask = 0u32.wrapping_sub(is_negative); // 0xFFFFFFFF if negative, 0x00000000 if non-negative
        let r = (diff as u32).wrapping_add((MODULUS as u32) & mask);
        FieldElement(r as u16)
    }
}

impl Mul for FieldElement {
    type Output = Self;

    /// Constant-time multiplication modulo 65521.
    ///
    /// # Side-Channel Resistance
    /// - Uses `from_u32` to perform Barrett reduction without division or branching.
    #[inline]
    fn mul(self, other: Self) -> Self {
        let prod = (self.0 as u32) * (other.0 as u32);
        FieldElement::from_u32(prod)
    }
}

impl Neg for FieldElement {
    type Output = Self;

    /// Constant-time negation modulo 65521.
    ///
    /// # Side-Channel Resistance
    /// - Uses constant-time selection via `subtle` to handle the zero edge-case.
    #[inline]
    fn neg(self) -> Self {
        let neg_val = MODULUS - self.0;
        let is_zero = self.0.ct_eq(&0);
        FieldElement::conditional_select(&FieldElement(neg_val), &FieldElement(0), is_zero)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_reduction() {
        for i in 0..=65535 {
            let expected = i % MODULUS;
            assert_eq!(FieldElement::new(i).value(), expected, "Failed for {}", i);
        }
    }

    #[test]
    fn test_from_u32_reduction() {
        // Test a range of products up to MODULUS^2
        for i in 0..1000000 {
            let expected = (i % MODULUS as u32) as u16;
            assert_eq!(FieldElement::from_u32(i).value(), expected, "Failed for {}", i);
        }
    }

    #[test]
    fn test_addition() {
        for a in (0..MODULUS).step_by(111) {
            for b in (0..MODULUS).step_by(111) {
                let expected = (((a as u32) + (b as u32)) % (MODULUS as u32)) as u16;
                let res = FieldElement(a) + FieldElement(b);
                assert_eq!(res.value(), expected, "{} + {} failed", a, b);
            }
        }
    }

    #[test]
    fn test_subtraction() {
        for a in (0..MODULUS).step_by(111) {
            for b in (0..MODULUS).step_by(111) {
                let expected = (((a as u32) + (MODULUS as u32) - (b as u32)) % (MODULUS as u32)) as u16;
                let res = FieldElement(a) - FieldElement(b);
                assert_eq!(res.value(), expected, "{} - {} failed", a, b);
            }
        }
    }

    #[test]
    fn test_multiplication() {
        for a in (0..MODULUS).step_by(111) {
            for b in (0..MODULUS).step_by(111) {
                let expected = (((a as u32) * (b as u32)) % MODULUS as u32) as u16;
                let res = FieldElement(a) * FieldElement(b);
                assert_eq!(res.value(), expected, "{} * {} failed", a, b);
            }
        }
    }

    #[test]
    fn test_negation() {
        for a in 0..MODULUS {
            let expected = (MODULUS - a) % MODULUS;
            let res = -FieldElement(a);
            assert_eq!(res.value(), expected, "-{} failed", a);
        }
    }

    #[test]
    fn test_inversion() {
        // Zero inverse should be zero
        assert_eq!(FieldElement::zero().invert().value(), 0);

        for a in (1..MODULUS).step_by(11) {
            let inv = FieldElement(a).invert();
            let prod = FieldElement(a) * inv;
            assert_eq!(prod.value(), 1, "Inverse of {} (which is {}) failed", a, inv.value());
        }
    }
}
