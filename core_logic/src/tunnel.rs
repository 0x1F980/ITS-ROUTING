use crate::field_arith::FieldElement;
use crate::masking::{encapsulate, decapsulate};
use crate::otm::{generate_tag, verify_tag, derive_forward_secret, verify_forward_share, verify_backward_share};
use crate::poly::Polynomial;
use crate::trapdoor::{Trapdoor, lagrange_interpolate};
use hal_abstraction::SecureRandom;
use subtle::{ConditionallySelectable, ConstantTimeEq};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A lightweight error type for bare-metal tunnel operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TunnelError {
    /// The received packet failed verification (integrity, authority, or tag check).
    InvalidPacket,
    /// An error occurred in the underlying hardware/entropy source.
    HardwareError,
}

/// A static, pre-allocated packet structure representing a single transmission
/// over the secure SCPST tunnel.
///
/// This struct has a fixed size and is completely stack-allocated, making it
/// perfect for bare-metal, FPGA, and seL4 IPC payloads.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct ScpstPacket {
    /// Alice's masked point containing the OTP key: `(x_i, y_masked)`
    pub masked_point: (FieldElement, FieldElement),
    /// Alice's forward SSS share for integrity: `(M_i, y_forward)`
    pub forward_point: (FieldElement, FieldElement),
    /// Alice's backward SSS share for authority: `(x_i, y_backward)`
    pub backward_point: (FieldElement, FieldElement),
    /// The Wegman-Carter authentication tag.
    pub tag: FieldElement,
    /// The OTP-encrypted ciphertext: `C = M_i + K_pool`
    pub ciphertext: FieldElement,
}

/// Alice's Endpoint (Sender) for the SCPST Tunnel.
///
/// Manages her local geometric state, steps her SSS-chains forward,
/// and encapsulates messages into static packets.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct AliceEndpoint<const K: usize> {
    /// Alice's static backward SSS polynomial Q(x).
    pub poly_backward: Polynomial<K>,
    /// The public point published by Bob.
    pub public_point: (FieldElement, FieldElement),
    /// The backward SSS point from the previous step.
    pub prev_back_point: (FieldElement, FieldElement),
    /// The message sent in the previous step.
    pub prev_msg: FieldElement,
    /// Monotonically increasing message/step counter.
    pub step_counter: u64,
}

impl<const K: usize> AliceEndpoint<K> {
    /// Creates a new `AliceEndpoint` with the initial parameters.
    pub fn new(
        poly_backward: Polynomial<K>,
        public_point: (FieldElement, FieldElement),
        initial_back_point: (FieldElement, FieldElement),
        initial_msg: FieldElement,
    ) -> Self {
        AliceEndpoint {
            poly_backward,
            public_point,
            prev_back_point: initial_back_point,
            prev_msg: initial_msg,
            step_counter: 1,
        }
    }

    /// Encapsulates a secret message into a static, secure packet in constant-time.
    ///
    /// # Arguments
    /// * `message` - The secret message to send (must be non-zero to prevent SSS x-coordinate collision).
    /// * `rng` - A secure random number generator (TRNG) to supply fresh entropy.
    pub fn send_packet<R: SecureRandom>(
        &mut self,
        message: FieldElement,
        rng: &mut R,
    ) -> Result<ScpstPacket, TunnelError> {
        let mut entropy = [0u8; 4];
        rng.fill_bytes(&mut entropy).map_err(|_| TunnelError::HardwareError)?;

        // A. Baglæns SSS (Autoritet)
        // We select x_i uniquely based on the step counter to rotate the points.
        // Z_65521 has 65521 elements. We avoid x=0 (Master Root) and x=1 (Bob's public point).
        let x_val = ((self.step_counter % 65519) + 2) as u16;
        let x_i = FieldElement::new(x_val);
        let y_back = self.poly_backward.evaluate(x_i);
        let backward_point = (x_i, y_back);

        // Nonce is derived deterministically from Q(65520)
        let nonce = self.poly_backward.evaluate(FieldElement::new(65520));

        // B. Forlæns SSS (Integritet)
        let s_forw = derive_forward_secret(self.prev_back_point, self.prev_msg);

        // Fresh random slope b_i from TRNG (must be non-zero)
        let b_i_raw = FieldElement::new((entropy[0] as u16) | ((entropy[1] as u16) << 8));
        let is_zero = b_i_raw.ct_eq(&FieldElement::zero());
        let b_i = FieldElement::conditional_select(&b_i_raw, &FieldElement::one(), is_zero);

        let poly_forward = Polynomial::new([s_forw, b_i]);
        let y_forw = poly_forward.evaluate(message);
        let forward_point = (message, y_forw);

        // MAC key is derived from P_i(65519)
        let k_mac = poly_forward.evaluate(FieldElement::new(65519));

        // C. OTP Maskering & Tag-generering
        let k_pool = FieldElement::new((entropy[2] as u16) | ((entropy[3] as u16) << 8));
        let masked_point = encapsulate(self.public_point, k_pool);
        let tag = generate_tag(k_mac, masked_point.1, nonce);
        let ciphertext = message + k_pool;

        // Update internal state for the next step
        self.prev_back_point = backward_point;
        self.prev_msg = message;
        self.step_counter = self.step_counter.wrapping_add(1);

        Ok(ScpstPacket {
            masked_point,
            forward_point,
            backward_point,
            tag,
            ciphertext,
        })
    }
}

/// Bob's Endpoint (Receiver) for the SCPST Tunnel.
///
/// Manages his local trapdoor state, verifies incoming packets in constant-time,
/// and decapsulates/decrypts messages.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct BobEndpoint<const K: usize> {
    /// Bob's secret Master-Root R = Q(0).
    pub master_root: FieldElement,
    /// Bob's private trapdoor.
    pub trapdoor: Trapdoor<K>,
    /// The backward SSS point from the previous step.
    pub prev_back_point: (FieldElement, FieldElement),
    /// The message received in the previous step.
    pub prev_msg: FieldElement,
}

impl<const K: usize> BobEndpoint<K> {
    /// Creates a new `BobEndpoint` with the initial parameters.
    pub fn new(
        master_root: FieldElement,
        trapdoor: Trapdoor<K>,
        initial_back_point: (FieldElement, FieldElement),
        initial_msg: FieldElement,
    ) -> Self {
        BobEndpoint {
            master_root,
            trapdoor,
            prev_back_point: initial_back_point,
            prev_msg: initial_msg,
        }
    }

    /// Receives, verifies, and decrypts a packet in 100% constant-time.
    ///
    /// # Arguments
    /// * `packet` - The incoming static packet.
    pub fn receive_packet(&mut self, packet: ScpstPacket) -> Result<FieldElement, TunnelError> {
        // A. Verificer baglæns autoritetspunkt (Choice-based constant-time check)
        let is_back_valid = verify_backward_share::<K>(self.master_root, &[self.prev_back_point], packet.backward_point);

        // Bob rekonstruerer det baglæns polynomium Q(x) for at udlede det hemmelige nonce N_i
        let points_back_reconstructed = [
            (FieldElement::zero(), self.master_root),
            packet.backward_point,
        ];
        let bob_nonce = lagrange_interpolate(&points_back_reconstructed, FieldElement::new(65520));

        // B. Dekapsling & Dekryptering
        let bob_k_pool = decapsulate(&self.trapdoor, packet.masked_point);
        let decrypted_msg = packet.ciphertext - bob_k_pool;

        // C. Verificer forlæns integritetspunkt
        let bob_s_forw = derive_forward_secret(self.prev_back_point, self.prev_msg);

        // Bob rekonstruerer det forlæns polynomium P_i(x) ud fra (0, s_forw) og det modtagne forlæns punkt
        let points_forw_reconstructed = [
            (FieldElement::zero(), bob_s_forw),
            (decrypted_msg, packet.forward_point.1),
        ];

        // Bob udleder den hemmelige MAC-nøgle K_MAC ved at evaluere P_i(65519)
        let bob_k_mac = lagrange_interpolate(&points_forw_reconstructed, FieldElement::new(65519));

        // Bob genskaber det forlæns polynomium som en Polynomial struct til verifikation
        let slope_num = packet.forward_point.1 - bob_s_forw;
        let slope_den = decrypted_msg.invert();
        let bob_b_i = slope_num * slope_den;
        let bob_poly_forward = Polynomial::new([bob_s_forw, bob_b_i]);

        let is_forw_valid = verify_forward_share::<2>(&bob_poly_forward, decrypted_msg, packet.forward_point);

        // D. Verificer Wegman-Carter tag
        let is_tag_valid = verify_tag(bob_k_mac, packet.masked_point.1, bob_nonce, packet.tag);

        // E. Kombiner alle tjek i konstant-tid (ingen branch/early return)
        let is_packet_valid = is_back_valid & is_forw_valid & is_tag_valid;

        // Betinget valg af returværdi: Hvis pakken er ugyldig, returnerer vi Error.
        // For at bevare absolut konstant-tid, udfører vi denne kontrol til allersidst.
        if bool::from(is_packet_valid) {
            // Opdater tilstande for næste rotation
            self.prev_back_point = packet.backward_point;
            self.prev_msg = decrypted_msg;
            Ok(decrypted_msg)
        } else {
            Err(TunnelError::InvalidPacket)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRng {
        state: u32,
    }

    impl SecureRandom for MockRng {
        type Error = ();

        fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
            for byte in dest.iter_mut() {
                self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
                *byte = (self.state >> 16) as u8;
            }
            Ok(())
        }
    }

    #[test]
    fn test_bare_metal_endpoints_roundtrip() {
        let master_root = FieldElement::new(5);
        let public_point = (FieldElement::new(1), FieldElement::new(8));
        let trapdoor = Trapdoor::<2>::new([
            (FieldElement::new(2), FieldElement::new(11)),
            public_point,
        ]);

        let poly_backward = Polynomial::new([FieldElement::new(5), FieldElement::new(3)]);
        let initial_back_point = (FieldElement::new(2), FieldElement::new(11));
        let initial_msg = FieldElement::new(7);

        let mut alice = AliceEndpoint::new(poly_backward, public_point, initial_back_point, initial_msg);
        let mut bob = BobEndpoint::new(master_root, trapdoor, initial_back_point, initial_msg);

        let mut rng = MockRng { state: 0xDEADBEEF };

        for _ in 0..10 {
            let msg = FieldElement::new(12);
            let packet = alice.send_packet(msg, &mut rng).unwrap();
            let decrypted = bob.receive_packet(packet).unwrap();
            assert_eq!(decrypted.value(), 12);
        }
    }
}
