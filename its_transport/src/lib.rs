#![no_std]

//! ITS transport core — epoch cell pool, SSS fragment, transport ratchet.
//! Dev-only onion/mix modules require `dev-onion-mix` feature.
//! `no_std` + alloc; consumed by `its-routing` and `ITS-hardware`.

extern crate alloc;

/// TRNG trait for key generation and masking.
pub trait SecureRandom {
    type Error;
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error>;
}

pub mod field_arith;
pub mod poly;
pub mod trapdoor;
pub mod masking;
#[cfg(feature = "dev-onion-mix")]
pub mod lorenz;
#[cfg(feature = "dev-onion-mix")]
pub mod stealth_identity;
#[cfg(feature = "dev-onion-mix")]
pub mod morphic_proof;
#[cfg(feature = "dev-onion-mix")]
pub mod onion;
pub mod sss_fragment;
pub mod transport_otp_ratchet;
pub mod epoch_cell;
pub mod tunnel;

pub mod otm {
    pub use its_otm_public_attestation::otm::{
        combine_sss_chains, derive_forward_secret, generate_chained_tag_with_points,
        generate_tag, verify_backward_share, verify_chained_tag_with_points,
        verify_forward_share, verify_tag,
    };
    pub use its_otm_public_attestation::{
        create_public_attestation, verify_public_attestation, OtmChainSigner,
        PublicAttestationBundle,
    };
}

#[cfg(feature = "dev-onion-mix")]
pub use onion::{
    create_chaff_onion_packet, create_onion_packet, MorphicMixingNode, MorphicOnionPacket,
    PAYLOAD_SIZE,
};
pub use sss_fragment::{fragment_data, reconstruct_data, SssPackedShare};
pub use transport_otp_ratchet::TransportOtpRatchet;
pub use epoch_cell::EpochCellState;
