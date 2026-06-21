//! ITS-routing library surface (daemon, client, config) for tests and hardware re-export.

// Prod transport spine — always available with default `pool` build.
pub use its_transport::SecureRandom;
pub use its_transport::{fragment_data, reconstruct_data, EpochCellState, SssPackedShare};
pub use its_transport::TransportOtpRatchet;
pub use its_transport::field_arith;
pub use its_transport::sss_fragment;
pub use its_transport::transport_otp_ratchet;
pub use its_transport::epoch_cell;

#[cfg(feature = "dev-onion-mix")]
pub use its_transport::{
    create_chaff_onion_packet, create_onion_packet, MorphicMixingNode, MorphicOnionPacket,
    PAYLOAD_SIZE,
};
#[cfg(feature = "dev-onion-mix")]
pub use its_transport::{lorenz, morphic_proof, onion, stealth_identity};

pub mod stdio;
pub mod rng;
pub mod aeh;
pub mod ridges;
pub mod config;
pub mod courier;
#[cfg(feature = "dev-onion-mix")]
pub mod packet;
pub mod aeh_channel;
pub mod aeh_carrier;
pub mod ratchet;
pub mod pool_mailbox;
#[cfg(feature = "pool")]
pub mod cover_transport;
#[cfg(feature = "dev-onion-mix")]
pub mod daemon;
pub mod client;
pub mod cli;
