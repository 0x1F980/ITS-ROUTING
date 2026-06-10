#![no_std]

//! # SCPST Core Cryptographic Logic
//!
//! This crate implements the core mathematical and cryptographic logic for the
//! SSS-Chained Perfect Secrecy Trapdoor (SCPST) protocol.
//!
//! All operations in this crate are designed to be:
//! 1. **no_std compatible** (fully running in core/alloc).
//! 2. **Constant-Time** (no data-dependent branching, loops, or memory accesses).
//! 3. **Memory Safe** (zeroizing sensitive data on drop).

pub mod field_arith;
pub mod poly;
pub mod trapdoor;
pub mod masking;
pub mod ratchet;
pub mod otm;
pub mod tunnel;
