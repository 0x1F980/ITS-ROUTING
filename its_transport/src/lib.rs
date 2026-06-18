#![no_std]

//! SCPST transport core — migrated from legacy `core_logic` (ITS-session repo).
//! no_std + alloc; used by `its-routing` binary and `ITS-hardware`.

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
pub mod lorenz;
pub mod tunnel;
pub mod stealth_identity;
pub mod morphic_proof;

/// Onion routing (legacy module name `routing` in core_logic).
pub mod onion;

/// Payload fragment split/reconstruct (legacy `hydra_sss`).
pub mod sss_fragment;

/// SCPST transport ratchet (legacy `ratchet::StateRatchet`).
pub mod transport_ratchet;

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

// Compatibility re-exports (migration aliases).
pub use onion::{
    create_onion_packet, MorphicMixingNode, MorphicOnionPacket, PAYLOAD_SIZE,
};
pub use sss_fragment::{fragment_data, reconstruct_data, SssPackedShare};
pub use transport_ratchet::StateRatchet;

// Legacy module paths for hardware/tests during migration.
pub mod routing {
    pub use crate::onion::*;
}
pub mod hydra_sss {
    pub use crate::sss_fragment::*;
}
pub mod ratchet {
    pub use crate::transport_ratchet::*;
}
