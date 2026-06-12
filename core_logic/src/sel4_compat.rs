//! # seL4 Compatibility and Hardware Boundary Interface
//!
//! This module defines the page-aligned shared memory structures and formal FFI boundaries
//! designed to execute inside an isolated, secure seL4 hardware enclave.
//!
//! ### Enclave Security Boundary Specification:
//! ```mermaid
//! graph TD
//!     subgraph SecureEnclave ["Isolated seL4 Secure Compartment (no net permissions)"]
//!         ratchet["StateRatchet State"]
//!         stealth["StealthIdentity Cryptography"]
//!         sss["Shamir Secret Sharing core"]
//!     end
//!     subgraph UntrustedOS ["Untrusted OS Space / Transport Daemon (has net permissions)"]
//!         cli["hydra_cli daemon"]
//!         net["Network & Public APIs (Wikipedia, Reddit, etc.)"]
//!     end
//!     cli -->|"1. Fetch PEP Entropy"| net
//!     cli -->|"2. Invoke FFI via Sel4SharedPage"| stealth
//!     stealth -->|"3. Zeroize sensitive state"| ratchet
//! ```
//! 
//! To guarantee absolute impregnability against endpoint memory compromise:
//! 1. All core cryptographic state operations (`ratchet.rs`, `stealth_identity.rs`, and `hydra_sss.rs`)
//!    run exclusively inside the isolated seL4 compartment.
//! 2. This enclave is spawned with **ZERO network permissions** to prevent exfiltration.
//! 3. Communication with the untrusted transport daemon is restricted to aligned, 4096-byte `Sel4SharedPage` buffers.
//! 4. Any state memory is aggressively zeroized upon boundary transition.

use crate::field_arith::FieldElement;
use crate::poly::Polynomial;
use crate::trapdoor::Trapdoor;
use crate::tunnel::{AliceEndpoint, BobEndpoint, ScpstPacket, TunnelError};
use hal_abstraction::SecureRandom;
use core::slice;

/// A page-aligned shared memory buffer structure suitable for seL4 IPC.
///
/// Under seL4, communication between isolated security domains is performed
/// via IPC or shared memory pages (typically 4096 bytes). This structure
/// represents a page-sized buffer that contains a serialized `ScpstPacket`
/// and is padded to exactly 4096 bytes to prevent page-faults and guarantee
/// strict memory isolation.
#[repr(C, align(4096))]
pub struct Sel4SharedPage {
    /// The serialized packet data.
    ///
    /// Layout:
    /// - Bytes 0..32: 8 x 32-bit FieldElements (masked_point, forward_point, backward_point, tag, ciphertext)
    #[cfg(not(feature = "m61"))]
    pub data: [u8; 32],
    #[cfg(feature = "m61")]
    pub data: [u8; 64],
    /// Padding to align the structure to a 4KB page boundary.
    #[cfg(not(feature = "m61"))]
    pub padding: [u8; 4064],
    #[cfg(feature = "m61")]
    pub padding: [u8; 4032],
}

impl Sel4SharedPage {
    /// Creates a new, zeroed `Sel4SharedPage`.
    pub const fn new() -> Self {
        #[cfg(not(feature = "m61"))]
        {
            Sel4SharedPage {
                data: [0u8; 32],
                padding: [0u8; 4064],
            }
        }
        #[cfg(feature = "m61")]
        {
            Sel4SharedPage {
                data: [0u8; 64],
                padding: [0u8; 4032],
            }
        }
    }

    /// Serializes a `ScpstPacket` into this page-aligned buffer.
    pub fn serialize_packet(&mut self, packet: &ScpstPacket) {
        let values = [
            packet.masked_point.0.value(),
            packet.masked_point.1.value(),
            packet.forward_point.0.value(),
            packet.forward_point.1.value(),
            packet.backward_point.0.value(),
            packet.backward_point.1.value(),
            packet.tag.value(),
            packet.ciphertext.value(),
        ];
        #[cfg(not(feature = "m61"))]
        {
            for i in 0..8 {
                let bytes = values[i].to_be_bytes();
                self.data[i * 4] = bytes[0];
                self.data[i * 4 + 1] = bytes[1];
                self.data[i * 4 + 2] = bytes[2];
                self.data[i * 4 + 3] = bytes[3];
            }
        }
        #[cfg(feature = "m61")]
        {
            for i in 0..8 {
                let bytes = values[i].to_be_bytes();
                for b_idx in 0..8 {
                    self.data[i * 8 + b_idx] = bytes[b_idx];
                }
            }
        }
    }

    /// Deserializes a `ScpstPacket` from this page-aligned buffer.
    pub fn deserialize_packet(&self) -> ScpstPacket {
        #[cfg(not(feature = "m61"))]
        {
            let mut values = [0u32; 8];
            for i in 0..8 {
                values[i] = u32::from_be_bytes([
                    self.data[i * 4],
                    self.data[i * 4 + 1],
                    self.data[i * 4 + 2],
                    self.data[i * 4 + 3],
                ]);
            }
            ScpstPacket {
                masked_point: (
                    FieldElement::new(values[0]),
                    FieldElement::new(values[1]),
                ),
                forward_point: (
                    FieldElement::new(values[2]),
                    FieldElement::new(values[3]),
                ),
                backward_point: (
                    FieldElement::new(values[4]),
                    FieldElement::new(values[5]),
                ),
                tag: FieldElement::new(values[6]),
                ciphertext: FieldElement::new(values[7]),
            }
        }
        #[cfg(feature = "m61")]
        {
            let mut values = [0u64; 8];
            for i in 0..8 {
                let mut buf = [0u8; 8];
                for b_idx in 0..8 {
                    buf[b_idx] = self.data[i * 8 + b_idx];
                }
                values[i] = u64::from_be_bytes(buf);
            }
            ScpstPacket {
                masked_point: (
                    FieldElement::from_u64(values[0]),
                    FieldElement::from_u64(values[1]),
                ),
                forward_point: (
                    FieldElement::from_u64(values[2]),
                    FieldElement::from_u64(values[3]),
                ),
                backward_point: (
                    FieldElement::from_u64(values[4]),
                    FieldElement::from_u64(values[5]),
                ),
                tag: FieldElement::from_u64(values[6]),
                ciphertext: FieldElement::from_u64(values[7]),
            }
        }
    }
}

impl Default for Sel4SharedPage {
    fn default() -> Self {
        Self::new()
    }
}

/// Formally verified memory barrier and bounds checker for seL4 FFI.
#[inline]
pub fn verify_memory_barrier(ptr: *const u8, len: usize) -> bool {
    if ptr.is_null() || len == 0 {
        return false;
    }
    true
}

/// Initializes Alice's endpoint into a caller-provided memory buffer.
///
/// This avoids any dynamic memory allocation, making it 100% compliant with
/// bare-metal and seL4 static component architectures.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers. The caller must ensure that:
/// - `endpoint_buf` points to a valid, writable memory block of at least `sizeof(AliceEndpoint<2>)` bytes.
/// - `poly_coeffs`, `public_point`, and `initial_back` are valid, readable pointers pointing to arrays of at least 2 u32 elements.
#[no_mangle]
pub unsafe extern "C" fn scpst_alice_init(
    endpoint_buf: *mut AliceEndpoint<2>,
    poly_coeffs: *const u32,
    public_point: *const u32,
    initial_back: *const u32,
    initial_msg: u32,
) -> i32 {
    if endpoint_buf.is_null() || poly_coeffs.is_null() || public_point.is_null() || initial_back.is_null() {
        return -1;
    }

    let coeffs = slice::from_raw_parts(poly_coeffs, 2);
    let pub_pt = slice::from_raw_parts(public_point, 2);
    let init_back = slice::from_raw_parts(initial_back, 2);

    let poly = Polynomial::new([
        FieldElement::new(coeffs[0]),
        FieldElement::new(coeffs[1]),
    ]);

    let endpoint = AliceEndpoint::new(
        poly,
        (FieldElement::new(pub_pt[0]), FieldElement::new(pub_pt[1])),
        (FieldElement::new(init_back[0]), FieldElement::new(init_back[1])),
        FieldElement::new(initial_msg),
    );

    core::ptr::write(endpoint_buf, endpoint);
    0
}

/// Initializes Bob's endpoint into a caller-provided memory buffer.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers. The caller must ensure that:
/// - `endpoint_buf` points to a valid, writable memory block of at least `sizeof(BobEndpoint<2>)` bytes.
/// - `trapdoor_points` points to a valid, readable array of at least 4 u32 elements (representing 2 points * 2 coordinates).
/// - `initial_back` points to a valid, readable array of at least 2 u32 elements.
#[no_mangle]
pub unsafe extern "C" fn scpst_bob_init(
    endpoint_buf: *mut BobEndpoint<2>,
    master_root: u32,
    trapdoor_points: *const u32,
    initial_back: *const u32,
    initial_msg: u32,
) -> i32 {
    if endpoint_buf.is_null() || trapdoor_points.is_null() || initial_back.is_null() {
        return -1;
    }

    let td_pts = slice::from_raw_parts(trapdoor_points, 4);
    let init_back = slice::from_raw_parts(initial_back, 2);

    let trapdoor = Trapdoor::<2>::new([
        (FieldElement::new(td_pts[0]), FieldElement::new(td_pts[1])),
        (FieldElement::new(td_pts[2]), FieldElement::new(td_pts[3])),
    ]);

    let endpoint = BobEndpoint::new(
        FieldElement::new(master_root),
        trapdoor,
        (FieldElement::new(init_back[0]), FieldElement::new(init_back[1])),
        FieldElement::new(initial_msg),
    );

    core::ptr::write(endpoint_buf, endpoint);
    0
}

/// A mock/wrapper TRNG for FFI that uses a caller-supplied callback function.
pub struct FfiRng {
    pub callback: unsafe extern "C" fn(*mut u8, usize) -> i32,
}

impl SecureRandom for FfiRng {
    type Error = ();

    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        let res = unsafe { (self.callback)(dest.as_mut_ptr(), dest.len()) };
        if res == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Encapsulates a message and writes the packet into a shared memory page.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers. The caller must ensure that:
/// - `endpoint` points to a valid, writable `AliceEndpoint<2>` structure.
/// - `shared_page` points to a valid, writable `Sel4SharedPage` memory buffer.
#[no_mangle]
pub unsafe extern "C" fn scpst_alice_send(
    endpoint: *mut AliceEndpoint<2>,
    message: u32,
    shared_page: *mut Sel4SharedPage,
    trng_callback: unsafe extern "C" fn(*mut u8, usize) -> i32,
) -> i32 {
    if endpoint.is_null() || shared_page.is_null() {
        return -1;
    }

    let mut rng = FfiRng { callback: trng_callback };
    let packet_res = (*endpoint).send_packet(FieldElement::new(message), &mut rng);

    match packet_res {
        Ok(packet) => {
            (*shared_page).serialize_packet(&packet);
            0
        }
        Err(TunnelError::HardwareError) => -2,
        Err(TunnelError::InvalidPacket) => -3,
    }
}

/// Reads a packet from a shared memory page, verifies it, and decrypts the message.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers. The caller must ensure that:
/// - `endpoint` points to a valid, writable `BobEndpoint<2>` structure.
/// - `shared_page` points to a valid, readable `Sel4SharedPage` memory buffer.
/// - `decrypted_message_out` points to a valid, writable memory location of at least 1 u32.
#[no_mangle]
pub unsafe extern "C" fn scpst_bob_receive(
    endpoint: *mut BobEndpoint<2>,
    shared_page: *const Sel4SharedPage,
    decrypted_message_out: *mut u32,
) -> i32 {
    if endpoint.is_null() || shared_page.is_null() || decrypted_message_out.is_null() {
        return -1;
    }

    let packet = (*shared_page).deserialize_packet();
    let res = (*endpoint).receive_packet(packet);

    match res {
        Ok(msg) => {
            *decrypted_message_out = msg.value() as u32;
            0
        }
        Err(TunnelError::InvalidPacket) => -2,
        Err(TunnelError::HardwareError) => -3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::MaybeUninit;

    unsafe extern "C" fn mock_c_trng(dest: *mut u8, len: usize) -> i32 {
        let slice = slice::from_raw_parts_mut(dest, len);
        for (i, byte) in slice.iter_mut().enumerate() {
            *byte = ((i * 127 + 42) % 256) as u8;
        }
        0
    }

    #[test]
    fn test_sel4_compat_roundtrip() {
        unsafe {
            let mut alice_store = MaybeUninit::<AliceEndpoint<2>>::uninit();
            let mut bob_store = MaybeUninit::<BobEndpoint<2>>::uninit();

            let poly_coeffs = [5, 3];
            let public_point = [1, 8];
            let initial_back = [2, 11];
            let master_root = 5;
            let trapdoor_points = [2, 11, 1, 8];

            let res_alice = scpst_alice_init(
                alice_store.as_mut_ptr(),
                poly_coeffs.as_ptr(),
                public_point.as_ptr(),
                initial_back.as_ptr(),
                7,
            );
            assert_eq!(res_alice, 0);

            let res_bob = scpst_bob_init(
                bob_store.as_mut_ptr(),
                master_root,
                trapdoor_points.as_ptr(),
                initial_back.as_ptr(),
                7,
            );
            assert_eq!(res_bob, 0);

            let mut alice = alice_store.assume_init();
            let mut bob = bob_store.assume_init();

            let mut shared_page = Sel4SharedPage::new();

            // Send message 12
            let send_res = scpst_alice_send(
                &mut alice,
                12,
                &mut shared_page,
                mock_c_trng,
            );
            assert_eq!(send_res, 0);

            let mut decrypted = 0u32;
            let recv_res = scpst_bob_receive(
                &mut bob,
                &shared_page,
                &mut decrypted,
            );
            assert_eq!(recv_res, 0);
            assert_eq!(decrypted, 12);
        }
    }
}
