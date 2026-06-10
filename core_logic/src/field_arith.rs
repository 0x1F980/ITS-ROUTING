use core::ops::{Add, Mul, Neg, Sub};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::Zeroize;

/// The prime modulus for our finite field Z_p (p = 17).
pub const MODULUS: u8 = 17;

/// A transparent wrapper representing an element of the finite field Z_17.
///
/// This struct implements constant-time modular arithmetic. All operations
/// are guaranteed to execute in a constant number of clock cycles, independent
/// of the input values, to prevent timing and power side-channel attacks.
///
/// Under the hood, containers holding this element can be zeroized to ensure sensitive
/// cryptographic material does not linger in the cache or stack.
#[derive(Clone, Copy, Debug, Default, Zeroize)]
pub struct FieldElement(pub u8);

impl FieldElement {
    /// Creates a new `FieldElement` from a raw `u8`.
    ///
    /// This function performs modular reduction modulo 17 in constant-time
    /// using a highly optimized 16-bit Barrett reduction, avoiding 32-bit
    /// multiplication and division. This is extremely efficient on 8-bit (AVR)
    /// and 16-bit (MSP430) microcontrollers, as well as 32-bit/64-bit CPUs.
    ///
    /// # Side-Channel Resistance
    /// - No conditional branching.
    /// - No division instruction.
    /// - Uses constant-time 16-bit multiplication and bit-shifts.
    #[inline]
    pub fn new(val: u8) -> Self {
        // Barrett reduction for u8 modulo 17 using 16-bit precision:
        // 2^12 / 17 = 240.9411...
        // We round up to 241.
        // This is 100% mathematically exact for all inputs in 0..=255.
        let q = ((val as u16) * 241) >> 12;
        let r = (val as u16) - q * (MODULUS as u16);
        FieldElement(r as u8)
    }

    /// Creates a `FieldElement` from a `u16` value in constant-time.
    ///
    /// Useful for reducing intermediate products or larger integers.
    ///
    /// This function is optimized to use 32-bit precision instead of 64-bit precision,
    /// making it extremely fast on 32-bit ARM Cortex-M microcontrollers and larger CPUs.
    ///
    /// # Side-Channel Resistance
    /// - Constant-time multiplication and bit-shifts using 32-bit precision
    ///   to prevent overflow and timing leaks.
    #[inline]
    pub fn from_u16(val: u16) -> Self {
        // Barrett reduction for u16 modulo 17 using 32-bit precision:
        // 2^20 / 17 = 61680.9411...
        // We round up to 61681.
        // Since 65535 * 61681 = 4042264335, the product fits perfectly
        // within a standard 32-bit unsigned integer (max 4294967295).
        // This is 100% mathematically exact for all inputs in 0..=65535.
        let q = ((val as u32) * 61681) >> 20;
        let r = (val as u32) - q * (MODULUS as u32);
        FieldElement(r as u8)
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

    /// Computes the modular inverse of `self` modulo 17.
    ///
    /// By Fermat's Little Theorem, for any prime p, x^(p-2) = x^(15) = x^-1 (mod 17).
    /// If `self` is 0, this returns 0.
    ///
    /// # Side-Channel Resistance
    /// - Uses a fixed, deterministic sequence of 5 multiplications to compute x^15.
    /// - No branches or conditional execution based on the input value.
    #[inline]
    pub fn invert(&self) -> Self {
        let x = *self;
        let x2 = x * x;       // x^2
        let x3 = x2 * x;      // x^3
        let x6 = x3 * x3;     // x^6
        let x12 = x6 * x6;    // x^12
        let x15 = x12 * x3;   // x^15
        x15
    }

    /// Returns the raw `u8` value of this field element.
    #[inline]
    pub fn value(&self) -> u8 {
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
        let val = u8::conditional_select(&a.0, &b.0, choice);
        FieldElement(val)
    }
}

impl Add for FieldElement {
    type Output = Self;

    /// Constant-time addition modulo 17.
    ///
    /// # Side-Channel Resistance
    /// - Avoids `if sum >= 17` branching.
    /// - Uses bitwise masking to conditionally subtract 17.
    #[inline]
    fn add(self, other: Self) -> Self {
        let sum = self.0 + other.0;
        let sub = sum.wrapping_sub(MODULUS);
        // If sub is negative (high bit set), sum < 17.
        let is_negative = (sub >> 7) & 1;
        let mask = 0u8.wrapping_sub(is_negative); // 0xFF if negative, 0x00 if non-negative
        let r = (sub & !mask) | (sum & mask);
        FieldElement(r)
    }
}

impl Sub for FieldElement {
    type Output = Self;

    /// Constant-time subtraction modulo 17.
    ///
    /// # Side-Channel Resistance
    /// - Avoids `if a < b` branching.
    /// - Uses bitwise masking to conditionally add 17 on underflow.
    #[inline]
    fn sub(self, other: Self) -> Self {
        let diff = self.0.wrapping_sub(other.0);
        // If diff is negative (high bit set), we underflowed.
        let is_negative = (diff >> 7) & 1;
        let mask = 0u8.wrapping_sub(is_negative); // 0xFF if negative, 0x00 if non-negative
        let r = diff.wrapping_add(MODULUS & mask);
        FieldElement(r)
    }
}

impl Mul for FieldElement {
    type Output = Self;

    /// Constant-time multiplication modulo 17.
    ///
    /// # Side-Channel Resistance
    /// - Uses `from_u16` to perform Barrett reduction without division or branching.
    #[inline]
    fn mul(self, other: Self) -> Self {
        let prod = (self.0 as u16) * (other.0 as u16);
        FieldElement::from_u16(prod)
    }
}

impl Neg for FieldElement {
    type Output = Self;

    /// Constant-time negation modulo 17.
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
        for i in 0..=255 {
            let expected = i % MODULUS;
            assert_eq!(FieldElement::new(i).value(), expected, "Failed for {}", i);
        }
    }

    #[test]
    fn test_from_u16_reduction() {
        for i in 0..=1000 {
            let expected = (i % MODULUS as u16) as u8;
            assert_eq!(FieldElement::from_u16(i).value(), expected, "Failed for {}", i);
        }
    }

    #[test]
    fn test_addition() {
        for a in 0..MODULUS {
            for b in 0..MODULUS {
                let expected = (a + b) % MODULUS;
                let res = FieldElement(a) + FieldElement(b);
                assert_eq!(res.value(), expected, "{} + {} failed", a, b);
            }
        }
    }

    #[test]
    fn test_subtraction() {
        for a in 0..MODULUS {
            for b in 0..MODULUS {
                let expected = (a + MODULUS - b) % MODULUS;
                let res = FieldElement(a) - FieldElement(b);
                assert_eq!(res.value(), expected, "{} - {} failed", a, b);
            }
        }
    }

    #[test]
    fn test_multiplication() {
        for a in 0..MODULUS {
            for b in 0..MODULUS {
                let expected = ((a as u16 * b as u16) % MODULUS as u16) as u8;
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

        for a in 1..MODULUS {
            let inv = FieldElement(a).invert();
            let prod = FieldElement(a) * inv;
            assert_eq!(prod.value(), 1, "Inverse of {} (which is {}) failed", a, inv.value());
        }
    }
}
