#![no_std]

//! # HAL Abstraction Layer for SCPST
//!
//! This crate defines hardware-independent traits for secure communications,
//! entropy sources, and side-channel countermeasures (such as noise/jitter generation).
//! It is designed to be fully compatible with `no_std` environments, bare-metal MCUs,
//! FPGA interfaces, and seL4 microkernel capabilities.

use subtle::Choice;

/// A trait representing a secure, physical or logical communication channel.
///
/// This abstracts the actual I/O layer (e.g., UART, SPI, MMIO, seL4 IPC, or network sockets)
/// from the core cryptographic logic.
pub trait SecureChannel {
    /// The error type associated with this channel.
    type Error;

    /// Sends a slice of data over the secure channel.
    ///
    /// This operation must be implemented in a way that minimizes timing leaks,
    /// or delegates timing-independent transmission to the underlying hardware.
    fn send(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Receives data from the secure channel into the provided buffer.
    ///
    /// Returns the number of bytes successfully received.
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
}

/// A trait representing a cryptographically secure True Random Number Generator (TRNG).
///
/// Hardware platforms (such as MCU TRNG peripherals or FPGA entropy sources)
/// must implement this trait to provide high-quality entropy for key generation and masking.
pub trait SecureRandom {
    /// The error type associated with the random source.
    type Error;

    /// Fills the destination buffer with cryptographically secure random bytes.
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error>;
}

/// A trait for generating physical or computational noise to mitigate TEMPEST/LPI side-channels.
///
/// Implementations can perform dummy operations, toggle GPIO pins connected to noise circuits,
/// or execute variable/random delay loops to mask power and electromagnetic signatures.
pub trait NoiseGenerator {
    /// Generates a computational or physical jitter/noise event.
    ///
    /// This is called during sensitive cryptographic operations to blind the power/EM profile.
    fn generate_jitter(&mut self);
}

/// A helper trait for constant-time selection, mapping to `subtle::ConditionallySelectable`.
pub trait ConstantTimeSelect {
    /// Conditionally select between `self` and `other` in constant-time.
    ///
    /// If `choice` is 1, returns `other`. If `choice` is 0, returns `self`.
    fn ct_select(self, other: Self, choice: Choice) -> Self;
}
