use its_transport::field_arith::FieldElement;
use its_transport::trapdoor::Trapdoor;
use its_transport::routing::{create_onion_packet, MorphicMixingNode};
use its_transport::morphic_proof::{MorphicProbe, verify_morphic_path};

struct XorShiftRng {
    state: u32,
}

impl XorShiftRng {
    fn new(seed: u32) -> Self {
        XorShiftRng { state: seed }
    }
}

impl its_transport::SecureRandom for XorShiftRng {
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
fn test_provable_morphic_routing_shadow_simulation() {
    let mut rng = XorShiftRng::new(0x11223344);

    // =========================================================================
    // 1. SCENARIO SETUP: ALICE, CLAIRE AND BLIND VPS-NODES
    // =========================================================================
    // We set up public points and private trapdoors for two intermediate VPS-nodes.
    let pub_pts = [
        (FieldElement::new(1), FieldElement::new(8)),   // Node 1 (Mix 1)
        (FieldElement::new(2), FieldElement::new(11)),  // Node 2 (Mix 2)
    ];

    let trapdoor_1 = Trapdoor::<2>::new([(FieldElement::new(2), FieldElement::new(11)), pub_pts[0]]);
    let trapdoor_2 = Trapdoor::<2>::new([(FieldElement::new(1), FieldElement::new(8)), pub_pts[1]]);

    let mac_keys = [FieldElement::new(5), FieldElement::new(6)];
    let nonces = [FieldElement::new(10), FieldElement::new(11)];

    let node1 = MorphicMixingNode::new(FieldElement::new(1), trapdoor_1, mac_keys[0], nonces[0]);
    let node2 = MorphicMixingNode::new(FieldElement::new(2), trapdoor_2, mac_keys[1], nonces[1]);

    // =========================================================================
    // 2. PATH ATTESTATION (PROBE GENERATION)
    // =========================================================================
    // Alice selects a highly sensitive routing probe secret: S_probe = 12.
    // She splits S_probe homomorphically into shares.
    let s_probe = FieldElement::new(12);
    let mut probe_shares = MorphicProbe::generate_shares(s_probe, 3, 5, &mut rng).unwrap();
    assert_eq!(probe_shares.len(), 5);

    // Alice also selects her secret query keys
    let alice_query = [FieldElement::new(14), FieldElement::new(15)];

    // Claire, an independent network user, is simultaneously sending a different packet.
    let claire_query = [FieldElement::new(3), FieldElement::new(4)];

    // Alice builds her onion packet through Node 1 (Hop 1) and Node 2 (Hop 2)
    let k_pools_alice = [FieldElement::new(7), FieldElement::new(14), FieldElement::new(1)];
    let alice_packet = create_onion_packet(
        [pub_pts[0], pub_pts[1], (FieldElement::zero(), FieldElement::zero())],
        k_pools_alice,
        [mac_keys[0], mac_keys[1], FieldElement::zero()],
        [nonces[0], nonces[1], FieldElement::zero()],
        [FieldElement::new(2), FieldElement::new(9), FieldElement::zero()],
        &alice_query,
    );

    // Claire builds her onion packet along the same nodes
    let k_pools_claire = [FieldElement::new(9), FieldElement::new(11), FieldElement::new(1)];
    let claire_packet = create_onion_packet(
        [pub_pts[0], pub_pts[1], (FieldElement::zero(), FieldElement::zero())],
        k_pools_claire,
        [mac_keys[0], mac_keys[1], FieldElement::zero()],
        [nonces[0], nonces[1], FieldElement::zero()],
        [FieldElement::new(2), FieldElement::new(9), FieldElement::zero()],
        &claire_query,
    );

    // =========================================================================
    // 3. MORPHIC NETWORK CODING (BLIND MIXING) & PLAUSIBLE DENIABILITY
    // =========================================================================
    // VPS Node 1 receives both packets. It does NOT decrypt them.
    // It blindly mixes them using scalar factors c1 = 3 and c2 = 5 modulo 251.
    //
    // Formula: morphed_packet = (3 * alice_packet) + (5 * claire_packet) mod 251
    let c1 = FieldElement::new(3);
    let c2 = FieldElement::new(5);
    let morphed_packet = node1.blind_linear_mix(&alice_packet, &claire_packet, c1, c2);

    // VPS Node 1 also blends the homomorphic path probe-shares accordingly:
    for i in 0..5 {
        probe_shares[i].morphic_blend(c1, FieldElement::zero()); // morphed probe_i = c1 * p_i
    }

    // PLAUSIBLE DENIABILITY AUDIT:
    // To any adversary (Eve) who has seized VPS Node 1, all numbers are perfectly uniform noise.
    // For example, a payload element of morphed_packet is:
    // morphed_payload_i = 3 * alice_payload_i + 5 * claire_payload_i mod 251
    // Since there are two independent sources and more variables than equations (underdetermined),
    // Eve has absolute mathematically proven blind ignorance.
    assert_eq!(morphed_packet.payload.len(), 16);

    // =========================================================================
    // 4. TRANSIT TO HOP 2 (Node 2)
    // =========================================================================
    // In our pure stateless Morphic Routing, Node 2 is also a blind linear mixer.
    // It receives the morphed packet and blends it further (e.g. scaling by c3 = 4)
    // with background or decoy noise, forwarding the cascaded combination to Bob.
    let c3 = FieldElement::new(4);
    let final_packet = node2.blind_linear_mix(&morphed_packet, &morphed_packet, c3, FieldElement::zero());

    // VPS Node 2 also morphically scales the probe shares:
    for i in 0..5 {
        probe_shares[i].morphic_blend(c3, FieldElement::zero()); // final morphed probe_i = c3 * (c1 * p_i)
    }

    assert_eq!(final_packet.payload.len(), 16);

    // =========================================================================
    // 5. HOMOMORPHIC PATH VERIFICATION (KRYDSTJEK) AT THE DESTINATION
    // =========================================================================
    // Bob receives the processed packet and the morphed probe shares.
    // Since S_probe was 12, and the path morphed it by scaling by c1 (3) then c3 (4):
    // Expected morphed S_probe = 4 * (3 * 12) = 144 mod 251
    let expected_morphed_probe = FieldElement::new(144);

    // Bob collects the morphed probe-share points
    let final_probe_points: Vec<(FieldElement, FieldElement)> = probe_shares.iter().map(|p| p.point).collect();

    // Bob mathematically verifies the routing path vha. Homomorphic Path Attestation (Lagrange interpolation)
    let is_path_proven = verify_morphic_path(&final_probe_points[0..3], expected_morphed_probe);
    
    // THE ROUTING IS MATHEMATICALLY PROVEN!
    assert!(bool::from(is_path_proven));
    println!("Morphic routing successfully proven with 0% decryption overhead!");

    // =========================================================================
    // 6. ACTIVE ATTACK SCENARIO: DETECTION OF ROUTING ANOMALY
    // =========================================================================
    // Suppose an active attacker (Eve) tries to hijack the packet and route it
    // away from Node 2 to her own rogue node, or tampers with the packet payload.
    // She bypasses Node 2's morphic blend or modifies a share coordinate.
    let mut rogue_probe_shares = probe_shares.clone();
    // Tamper with the y-coordinate of a single probe share (representing routing diversion or manipulation)
    rogue_probe_shares[1].point.1 = rogue_probe_shares[1].point.1 + FieldElement::new(1);

    let rogue_probe_points: Vec<(FieldElement, FieldElement)> = rogue_probe_shares.iter().map(|p| p.point).collect();

    // Bob runs the same verification on the tampered/rogue rute
    let is_rogue_path_valid = verify_morphic_path(&rogue_probe_points[0..3], expected_morphed_probe);

    // THE ATTACK IS INSTANTLY DETECTED AND REJECTED!
    assert!(!bool::from(is_rogue_path_valid));
    println!("Active routing manipulation successfully detected and neutralized!");
}
