use core::ops::{Add, Mul, Neg, Sub};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::Zeroize;

/// The prime modulus for our finite field Z_p (p = 2^31 - 1 = 2147483647).
///
/// 2147483647 is the 8th Mersenne prime. Using a Mersenne prime allows extremely fast
/// modular reduction without any division or multiplication, using simple bit-shifts and additions.
/// This also reduces the forgery probability of our Wegman-Carter tags to virtually zero (1 in 2.1 billion).
pub const MODULUS: u32 = 2147483647;

/// A transparent wrapper representing an element of the finite field Z_2147483647.
///
/// This struct implements constant-time modular arithmetic. All operations
/// are guaranteed to execute in a constant number of clock cycles, independent
/// of the input values, to prevent timing and power side-channel attacks.
///
/// Under the hood, containers holding this element can be zeroized to ensure sensitive
/// cryptographic material does not linger in the cache or stack.
#[derive(Clone, Copy, Debug, Default, Zeroize)]
pub struct FieldElement(pub u32);

impl FieldElement {
    /// Creates a new `FieldElement` from a raw `u32` value, performing modular reduction in constant-time.
    ///
    /// # Side-Channel Resistance
    /// - No conditional branching.
    /// - No division instruction.
    /// - Constant-time comparison and conditional selection.
    #[inline]
    pub fn new(val: u32) -> Self {
        // Since MODULUS is 2^31 - 1, we can perform a fast Mersenne reduction:
        // val mod (2^31 - 1) = (val & 0x7FFFFFFF) + (val >> 31)
        let sum = (val & 0x7FFFFFFF) + (val >> 31);
        let sub = sum.wrapping_sub(MODULUS);
        let is_negative = (sub >> 31) & 1;
        let mask = 0u32.wrapping_sub(is_negative); // 0xFFFFFFFF if negative, 0 otherwise
        let r = (sub & !mask) | (sum & mask);
        FieldElement(r)
    }

    /// Creates a `FieldElement` from a `u32` value. Same as `new` for the M31 field.
    #[inline]
    pub fn from_u32(val: u32) -> Self {
        Self::new(val)
    }

    /// Creates a `FieldElement` from a `u64` value in constant-time.
    ///
    /// Useful for reducing intermediate products.
    #[inline]
    pub fn from_u64(val: u64) -> Self {
        // Double-fold Mersenne reduction for u64 values (up to 2^62)
        let mut sum = (val & 0x7FFFFFFF) + (val >> 31);
        sum = (sum & 0x7FFFFFFF) + (sum >> 31);
        let mut r = sum as u32;
        for _ in 0..2 {
            let sub = r.wrapping_sub(MODULUS);
            let is_negative = (sub >> 31) & 1;
            let mask = 0u32.wrapping_sub(is_negative);
            r = (sub & !mask) | (r & mask);
        }
        FieldElement(r)
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

    /// Computes the modular inverse of `self` modulo 2147483647.
    ///
    /// By Fermat's Little Theorem, for any prime p, x^(p-2) = x^(2147483645) = x^-1 (mod 2147483647).
    /// If `self` is 0, this returns 0.
    ///
    /// # Side-Channel Resistance
    /// - Uses a fixed, deterministic sequence of 31 squarings and conditional multiplications.
    /// - No branches or conditional execution based on the input value.
    #[inline]
    pub fn invert(&self) -> Self {
        let x = *self;
        let mut res = FieldElement::one();
        let mut base = x;
        let mut exp = 2147483645u32; // 2^31 - 3
        for _ in 0..31 {
            let bit = Choice::from((exp & 1) as u8);
            let multiplied = res * base;
            res = FieldElement::conditional_select(&res, &multiplied, bit);
            base = base * base;
            exp >>= 1;
        }
        
        // If the input was 0, Fermat's Little Theorem yields 0, which is correct.
        let is_zero = x.0.ct_eq(&0);
        FieldElement::conditional_select(&res, &FieldElement::zero(), is_zero)
    }

    /// Returns the raw `u32` value of this field element.
    #[inline]
    pub fn value(&self) -> u32 {
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
        let val = u32::conditional_select(&a.0, &b.0, choice);
        FieldElement(val)
    }
}

impl Add for FieldElement {
    type Output = Self;

    /// Constant-time addition modulo 2147483647.
    #[inline]
    fn add(self, other: Self) -> Self {
        let sum = (self.0 as u64) + (other.0 as u64);
        let sub = sum.wrapping_sub(MODULUS as u64);
        let is_negative = (sub >> 63) & 1;
        let mask = 0u64.wrapping_sub(is_negative);
        let r = (sub & !mask) | (sum & mask);
        FieldElement(r as u32)
    }
}

impl Sub for FieldElement {
    type Output = Self;

    /// Constant-time subtraction modulo 2147483647.
    #[inline]
    fn sub(self, other: Self) -> Self {
        let diff = (self.0 as i64) - (other.0 as i64);
        let is_negative = ((diff >> 63) & 1) as u64;
        let mask = 0u64.wrapping_sub(is_negative);
        let r = (diff as u64).wrapping_add((MODULUS as u64) & mask);
        FieldElement(r as u32)
    }
}

impl Mul for FieldElement {
    type Output = Self;

    /// Constant-time multiplication modulo 2147483647.
    #[inline]
    fn mul(self, other: Self) -> Self {
        let prod = (self.0 as u64) * (other.0 as u64);
        FieldElement::from_u64(prod)
    }
}

impl Neg for FieldElement {
    type Output = Self;

    /// Constant-time negation modulo 2147483647.
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
        assert_eq!(FieldElement::new(0).value(), 0);
        assert_eq!(FieldElement::new(2147483646).value(), 2147483646);
        assert_eq!(FieldElement::new(2147483647).value(), 0);
        assert_eq!(FieldElement::new(2147483648).value(), 1);
        assert_eq!(FieldElement::new(4294967294).value(), 0);
    }

    #[test]
    fn test_from_u64_reduction() {
        assert_eq!(FieldElement::from_u64(4294967295).value(), 1);
        assert_eq!(FieldElement::from_u64(9223372036854775807).value(), 1);
    }

    #[test]
    fn test_addition() {
        let a = FieldElement::new(2147483640);
        let b = FieldElement::new(10);
        assert_eq!((a + b).value(), 3);
    }

    #[test]
    fn test_subtraction() {
        let a = FieldElement::new(5);
        let b = FieldElement::new(10);
        assert_eq!((a - b).value(), 2147483642);
    }

    #[test]
    fn test_multiplication() {
        let a = FieldElement::new(1073741824);
        let b = FieldElement::new(2);
        assert_eq!((a * b).value(), 1); // 2^31 mod (2^31 - 1) = 1
    }

    #[test]
    fn test_negation() {
        assert_eq!((-FieldElement::zero()).value(), 0);
        assert_eq!((-FieldElement::new(1)).value(), 2147483646);
    }

    #[test]
    fn test_inversion() {
        assert_eq!(FieldElement::zero().invert().value(), 0);
        let a = FieldElement::new(42);
        let inv = a.invert();
        assert_eq!((a * inv).value(), 1);
    }
}
