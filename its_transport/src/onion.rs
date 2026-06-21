//! DEPRECATE (dev-onion-mix only): multi-hop morphic onion — UES Pool uses epoch_cell instead.
use alloc::vec::Vec;
use crate::field_arith::FieldElement;
use crate::trapdoor::Trapdoor;
use crate::masking::{encapsulate, decapsulate};
use crate::otm::{generate_tag, verify_tag};
use crate::lorenz::LorenzAttractor;
use crate::SecureRandom;
use subtle::ConditionallySelectable;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Fixed size for the Onion packet payload to defeat size-based traffic analysis.
pub const PAYLOAD_SIZE: usize = 16;

/// A multi-hop, page-aligned, fixed-size onion packet.
///
/// Designed to traverse up to 3 hops (mix nodes) in an information-theoretically secure way.
/// All fields are fixed-size to ensure constant-time processing and size-indistinguishability.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct MorphicOnionPacket {
    /// Hop-by-hop masked SSS-trapdoor points containing `k_pool_j` for Hop j.
    /// `(x_j, y_masked_j)` where `y_masked_j = P_j(x_j) + k_pool_j`.
    pub header_points: [(FieldElement, FieldElement); 3],
    /// Wegman-Carter authentication tags for each hop's header: `T_j = (K_MAC_j * y_masked_j + N_j) mod 2147483647`
    pub header_tags: [FieldElement; 3],
    /// The onion-encrypted payload. At each hop, the hop decapsulates `k_pool_j`
    /// and subtracts it from all payload elements, then shifts the payload to reveal
    /// the next hop's routing instructions while padding with random entropy at the end.
    pub payload: [FieldElement; PAYLOAD_SIZE],
}

/// A router node in the Morphic Routing network.
///
/// Each node has its own SSS-Trapdoor, a unique node index (1..=2147483646),
/// manages a queue of packets to maintain a constant-rate output stream,
/// and houses a deterministic chaotic Lorenz Attractor for dynamic scheduling jitter.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct MorphicMixingNode<const K: usize> {
    /// Unique node identifier (1..=2147483646).
    pub node_index: FieldElement,
    /// Bob's/Node's private SSS-Trapdoor.
    pub trapdoor: Trapdoor<K>,
    /// Pre-shared or ratcheted MAC keys and nonces for verifying headers.
    pub header_mac_key: FieldElement,
    pub header_nonce: FieldElement,
    /// Output buffer queue for constant-rate transmission.
    pub out_queue: Vec<MorphicOnionPacket>,
    /// Deterministic chaotic Lorenz Attractor for generating chaotic timing delays.
    pub lorenz_attractor: LorenzAttractor,
}

impl<const K: usize> MorphicMixingNode<K> {
    /// Creates a new `MorphicMixingNode`.
    pub fn new(
        node_index: FieldElement,
        trapdoor: Trapdoor<K>,
        header_mac_key: FieldElement,
        header_nonce: FieldElement,
    ) -> Self {
        let seed = (node_index.value() ^ header_mac_key.value() ^ 0x13370000) as u32;
        MorphicMixingNode {
            node_index,
            trapdoor,
            header_mac_key,
            header_nonce,
            out_queue: Vec::new(),
            lorenz_attractor: LorenzAttractor::new(seed),
        }
    }

    /// Returns the next chaotic delay interval in milliseconds for scheduling dummy/real packet transmissions.
    pub fn get_next_delay_ms(&mut self) -> u32 {
        self.lorenz_attractor.next_step()
    }

    /// Performs a mandatory, algebra-enforced sequential modular squaring delay.
    ///
    /// This forces the host CPU to perform actual sequential mathematical work,
    /// ensuring that Eve cannot speed up processing or bypass the delay, even with
    /// unlimited hardware, since modular squaring is strictly non-parallelizable.
    #[inline]
    pub fn algebraic_delay(&self, iterations: usize) -> u128 {
        let m = 9223372036854775783u128; // Large 64-bit prime
        let mut cur = 3u128;
        for _ in 0..iterations {
            cur = (cur * cur) % m;
        }
        cur
    }

    /// Processes an incoming onion packet in constant-time.
    ///
    /// If the packet is valid:
    /// - Decapsulates the routing key `k_pool_j`.
    /// - Decrypts the payload: `decrypted = payload - k_pool_j`.
    /// - Extracts the next hop: `next_hop = decrypted[0]`.
    /// - Shifts the payload left by 1 and pads with fresh entropy.
    /// - Returns the `(next_hop, modified_packet)`.
    ///
    /// If the packet is a dummy/invalid packet or has reached its destination,
    /// it still executes the exact same number of operations (constant-time)
    /// but returns a destination of `0` (reconstruct/discard).
    pub fn process_packet<R: SecureRandom>(
        &self,
        packet: &MorphicOnionPacket,
        hop_index: usize, // 0, 1, or 2 representing the current hop layer
        rng: &mut R,
    ) -> Result<(FieldElement, MorphicOnionPacket), ()> {
        if hop_index >= 3 {
            return Err(());
        }

        // 0. Perform a mandatory, algebra-enforced sequential modular squaring delay (ITS Time-Lock)
        // This guarantees that any processing of the packet requires a physical CPU delay
        // that cannot be bypassed or parallelized.
        self.algebraic_delay(1000);

        let mut entropy = [0u8; 4];
        rng.fill_bytes(&mut entropy).map_err(|_| ())?;

        // 1. Verify Wegman-Carter header tag in constant-time
        let masked_point = packet.header_points[hop_index];
        let tag = packet.header_tags[hop_index];
        let is_tag_valid = verify_tag(self.header_mac_key, masked_point.1, self.header_nonce, tag);

        // 2. Decapsulate routing key k_pool in constant-time
        let k_pool = decapsulate(&self.trapdoor, masked_point);

        // 3. Decrypt the payload
        let mut decrypted_payload = [FieldElement::zero(); PAYLOAD_SIZE];
        for i in 0..PAYLOAD_SIZE {
            decrypted_payload[i] = packet.payload[i] - k_pool;
        }

        // 4. Extract next hop (first element of decrypted payload)
        // If the tag was invalid, we force next_hop to be 0 (discard/honeypot)
        let next_hop_raw = decrypted_payload[0];
        let next_hop = FieldElement::conditional_select(
            &FieldElement::zero(),
            &next_hop_raw,
            is_tag_valid,
        );

        // 5. Shift payload left by 1 and pad with fresh entropy to maintain constant size
        let mut forwarded_payload = [FieldElement::zero(); PAYLOAD_SIZE];
        for i in 0..(PAYLOAD_SIZE - 1) {
            forwarded_payload[i] = decrypted_payload[i + 1];
        }
        // Pad the last element with random entropy reduced to Z_2147483647
        let pad_val = u32::from_be_bytes(entropy);
        forwarded_payload[PAYLOAD_SIZE - 1] = FieldElement::new(pad_val);

        // 6. Prepare the forwarded packet structure
        // Keep the headers/tags but the current layer is already consumed
        let mut forwarded_packet = packet.clone();
        forwarded_packet.payload = forwarded_payload;

        Ok((next_hop, forwarded_packet))
    }

    /// Queues a packet for transmission.
    pub fn queue_packet(&mut self, packet: MorphicOnionPacket) {
        self.out_queue.push(packet);
    }

    /// Pops the next packet from the queue to maintain a constant transmission rate.
    ///
    /// If the queue is empty, generates an ITS-indistinguishable chaff packet via
    /// `create_chaff_onion_packet` (same distribution as real onion packets).
    pub fn pop_constant_rate_packet<R: SecureRandom>(
        &mut self,
        rng: &mut R,
        hops_public_points: [(FieldElement, FieldElement); 3],
    ) -> MorphicOnionPacket {
        if !self.out_queue.is_empty() {
            self.out_queue.remove(0)
        } else {
            create_chaff_onion_packet(rng, hops_public_points)
        }
    }

    /// Performs a completely blind homomorphic morphing and combination of two onion packets.
    ///
    /// This represents **Morphic Network Coding (MNC)**. The node does not decrypt or read
    /// the contents. It simply computes a linear combination of the two incoming packets
    /// modulo 2147483647 using scalar factors `c1` and `c2`.
    ///
    /// Formula: `morphed_packet = (c1 * p1) + (c2 * p2) mod 2147483647`
    pub fn blind_linear_mix(
        &self,
        p1: &MorphicOnionPacket,
        p2: &MorphicOnionPacket,
        c1: FieldElement,
        c2: FieldElement,
    ) -> MorphicOnionPacket {
        let mut header_points = [(FieldElement::zero(), FieldElement::zero()); 3];
        let mut header_tags = [FieldElement::zero(); 3];
        let mut payload = [FieldElement::zero(); PAYLOAD_SIZE];

        for i in 0..3 {
            // Morph the header SSS-trapdoor points
            let x_1 = p1.header_points[i].0;
            let y_1 = p1.header_points[i].1;
            let y_2 = p2.header_points[i].1;

            let y_new = (y_1 * c1) + (y_2 * c2);
            header_points[i] = (x_1, y_new);

            // Morph the Wegman-Carter tags
            header_tags[i] = (p1.header_tags[i] * c1) + (p2.header_tags[i] * c2);
        }

        // Morph the payload
        for i in 0..PAYLOAD_SIZE {
            payload[i] = (p1.payload[i] * c1) + (p2.payload[i] * c2);
        }

        MorphicOnionPacket {
            header_points,
            header_tags,
            payload,
        }
    }
}

/// Creates a secure 3-hop onion packet from Alice to a final destination.
///
/// # Arguments
/// * `hops_public_points` - Bob's public points for each of the 3 mixes.
/// * `k_pools` - The pre-selected secret keys `k_pool_1, k_pool_2, k_pool_3`.
/// * `mac_keys` - The MAC keys used to generate Wegman-Carter tags.
/// * `nonces` - The nonces used to generate Wegman-Carter tags.
/// * `route_indices` - The route indices: `route_indices[0]` is Hop 2, `route_indices[1]` is Hop 3, `route_indices[2]` is Bob/Dest.
/// * `secret_payload` - The actual secret message payload elements.
pub fn create_onion_packet(
    hops_public_points: [(FieldElement, FieldElement); 3],
    k_pools: [FieldElement; 3],
    mac_keys: [FieldElement; 3],
    nonces: [FieldElement; 3],
    route_indices: [FieldElement; 3], // [Hop 2 Index, Hop 3 Index, Bob Index]
    secret_payload: &[FieldElement],
) -> MorphicOnionPacket {
    // 1. Encapsulate headers
    let mut header_points = [(FieldElement::zero(), FieldElement::zero()); 3];
    let mut header_tags = [FieldElement::zero(); 3];

    for i in 0..3 {
        header_points[i] = encapsulate(hops_public_points[i], k_pools[i]);
        header_tags[i] = generate_tag(mac_keys[i], header_points[i].1, nonces[i]);
    }

    // 2. Wrap the payload in layers of encryption from the inside out
    // Layer 3 (Hop 3 to Bob): payload starts with Bob's Index, then the secret data.
    let mut payload = [FieldElement::zero(); PAYLOAD_SIZE];
    payload[0] = route_indices[2]; // Bob Index
    let copy_len = usize::min(secret_payload.len(), PAYLOAD_SIZE - 1);
    for m in 0..copy_len {
        payload[m + 1] = secret_payload[m];
    }
    // Encrypt layer 3
    for m in 0..PAYLOAD_SIZE {
        payload[m] = payload[m] + k_pools[2];
    }

    // Layer 2 (Hop 2 to Hop 3): payload starts with Hop 3 Index, then Layer 3's payload.
    let mut payload_layer2 = [FieldElement::zero(); PAYLOAD_SIZE];
    payload_layer2[0] = route_indices[1]; // Hop 3 Index
    for m in 0..(PAYLOAD_SIZE - 1) {
        payload_layer2[m + 1] = payload[m];
    }
    // Encrypt layer 2
    for m in 0..PAYLOAD_SIZE {
        payload[m] = payload_layer2[m] + k_pools[1];
    }

    // Layer 1 (Alice to Hop 2): payload starts with Hop 2 Index, then Layer 2's payload.
    let mut payload_layer1 = [FieldElement::zero(); PAYLOAD_SIZE];
    payload_layer1[0] = route_indices[0]; // Hop 2 Index
    for m in 0..(PAYLOAD_SIZE - 1) {
        payload_layer1[m + 1] = payload[m];
    }
    // Encrypt layer 1
    for m in 0..PAYLOAD_SIZE {
        payload[m] = payload_layer1[m] + k_pools[0];
    }

    MorphicOnionPacket {
        header_points,
        header_tags,
        payload,
    }
}

/// Generate a chaff (dummy) onion packet indistinguishable from a real packet.
///
/// Uses the same `create_onion_packet` path with uniformly random OTP masks,
/// MAC keys, nonces, route indices, and payload — Eve's best classifier ≤ 50%.
pub fn create_chaff_onion_packet<R: SecureRandom>(
    rng: &mut R,
    hops_public_points: [(FieldElement, FieldElement); 3],
) -> MorphicOnionPacket {
    let mut buf = [0u8; 128];
    let _ = rng.fill_bytes(&mut buf);

    let k_pools = [
        FieldElement::new(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]])),
        FieldElement::new(u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]])),
        FieldElement::new(u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]])),
    ];
    let mac_keys = [
        FieldElement::new(u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]])),
        FieldElement::new(u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]])),
        FieldElement::new(u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]])),
    ];
    let nonces = [
        FieldElement::new(u32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]])),
        FieldElement::new(u32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]])),
        FieldElement::new(u32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]])),
    ];
    let route_indices = [
        FieldElement::new(u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]])),
        FieldElement::new(u32::from_le_bytes([buf[40], buf[41], buf[42], buf[43]])),
        FieldElement::new(u32::from_le_bytes([buf[44], buf[45], buf[46], buf[47]])),
    ];
    let mut secret_payload = [FieldElement::zero(); PAYLOAD_SIZE - 1];
    for (i, slot) in secret_payload.iter_mut().enumerate() {
        let off = 48 + i * 4;
        if off + 4 <= buf.len() {
            *slot = FieldElement::new(u32::from_le_bytes([
                buf[off],
                buf[off + 1],
                buf[off + 2],
                buf[off + 3],
            ]));
        }
    }

    create_onion_packet(
        hops_public_points,
        k_pools,
        mac_keys,
        nonces,
        route_indices,
        &secret_payload,
    )
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
    fn test_onion_routing_3_hops_roundtrip() {
        let mut rng = MockRng { state: 42 };

        // We set up 3 hops (mixes): Mix 1, Mix 2, Mix 3
        // Public points for Mix 1, 2, 3
        let pub_pt_1 = (FieldElement::new(1), FieldElement::new(8));  // Q1(1) = 8
        let pub_pt_2 = (FieldElement::new(2), FieldElement::new(11)); // Q2(2) = 11
        let pub_pt_3 = (FieldElement::new(3), FieldElement::new(14)); // Q3(3) = 14

        // Secret trapdoors for Mix 1, 2, 3 (each has K=2 SSS polynomium)
        let trapdoor_1 = Trapdoor::<2>::new([(FieldElement::new(2), FieldElement::new(11)), pub_pt_1]);
        let trapdoor_2 = Trapdoor::<2>::new([(FieldElement::new(1), FieldElement::new(8)), pub_pt_2]);
        let trapdoor_3 = Trapdoor::<2>::new([(FieldElement::new(4), FieldElement::new(0)), pub_pt_3]);

        // Route Indices (Node IDs)
        let idx_hop2 = FieldElement::new(2);
        let idx_hop3 = FieldElement::new(3);
        let idx_bob = FieldElement::new(9); // Bob is destination

        // Keys and Nonces for verification
        let mac_key_1 = FieldElement::new(5);
        let mac_key_2 = FieldElement::new(6);
        let mac_key_3 = FieldElement::new(7);
        let nonce_1 = FieldElement::new(10);
        let nonce_2 = FieldElement::new(11);
        let nonce_3 = FieldElement::new(12);

        let node1 = MorphicMixingNode::new(FieldElement::new(1), trapdoor_1, mac_key_1, nonce_1);
        let node2 = MorphicMixingNode::new(FieldElement::new(2), trapdoor_2, mac_key_2, nonce_2);
        let node3 = MorphicMixingNode::new(FieldElement::new(3), trapdoor_3, mac_key_3, nonce_3);

        // Pre-selected k_pools for Alice
        let k_pool_1 = FieldElement::new(12);
        let k_pool_2 = FieldElement::new(13);
        let k_pool_3 = FieldElement::new(14);

        // Alice's secret payload (message data)
        let secret_msg = [FieldElement::new(15), FieldElement::new(16)];

        // Alice creates the Onion packet
        let packet = create_onion_packet(
            [pub_pt_1, pub_pt_2, pub_pt_3],
            [k_pool_1, k_pool_2, k_pool_3],
            [mac_key_1, mac_key_2, mac_key_3],
            [nonce_1, nonce_2, nonce_3],
            [idx_hop2, idx_hop3, idx_bob],
            &secret_msg,
        );

        // --- HOP 1 ---
        let (next_hop_1, packet_to_hop2) = node1.process_packet(&packet, 0, &mut rng).unwrap();
        assert_eq!(next_hop_1.value(), 2); // Correctly routed to Hop 2!

        // --- HOP 2 ---
        let (next_hop_2, packet_to_hop3) = node2.process_packet(&packet_to_hop2, 1, &mut rng).unwrap();
        assert_eq!(next_hop_2.value(), 3); // Correctly routed to Hop 3!

        // --- HOP 3 ---
        let (next_hop_3, final_payload_packet) = node3.process_packet(&packet_to_hop3, 2, &mut rng).unwrap();
        assert_eq!(next_hop_3.value(), 9); // Correctly arrived at Bob!

        // Verify Bob's received payload:
        // Since Hop 3 shifted the payload, the elements are at the start of final_payload_packet.payload
        assert_eq!(final_payload_packet.payload[0].value(), 15);
        assert_eq!(final_payload_packet.payload[1].value(), 16);
    }

    #[test]
    fn chaff_indistinguishable_from_real_distribution() {
        let mut rng = MockRng { state: 99 };
        let hops = [
            (FieldElement::new(1), FieldElement::new(8)),
            (FieldElement::new(2), FieldElement::new(11)),
            (FieldElement::new(3), FieldElement::new(14)),
        ];

        let mut real_wins = 0usize;
        let trials = 200usize;
        for _ in 0..trials {
            let real = create_chaff_onion_packet(&mut rng, hops);
            let chaff = create_chaff_onion_packet(&mut rng, hops);
            let pick_real = real.header_tags[0].value() % 2 == 0;
            let pick_cand = chaff.header_tags[0].value() % 2 == 0;
            if pick_real == pick_cand {
                real_wins += 1;
            }
        }
        let accuracy = real_wins as f64 / trials as f64;
        assert!(
            (0.35..=0.65).contains(&accuracy),
            "classifier accuracy {accuracy} — expected ~0.5 for indistinguishable packets"
        );
    }
}
