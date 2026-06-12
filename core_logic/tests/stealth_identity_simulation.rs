use core_logic::field_arith::FieldElement;
use core_logic::stealth_identity::StealthIdentity;
use core_logic::hydra_sss::{fragment_data, reconstruct_data, HydraShare};
use core_logic::ratchet::StateRatchet;
use hal_abstraction::SecureRandom;

struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        SimpleRng { state: seed }
    }
}

impl hal_abstraction::SecureRandom for SimpleRng {
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
fn test_passive_entropy_parasitism_complete_simulation() {
    let mut rng = SimpleRng::new(0x7331_C0DE);

    // =========================================================================
    // 1. SCENARIO SETUP: THE HIGHLY SENSITIVE WEBPAGE
    // =========================================================================
    // Alice wants to store/share a classified document anonymously on the internet
    // without running any active routing servers and leaving 0 trace of sender culpability.
    let webpage_bytes = b"<html><body><h2>MUTANT-X DECENTRALIZED ARCHIVE</h2></body></html>";
    let k = 3;
    let n = 5;

    // Split the webpage into SSS-shares over Z_2147483647 (1-to-1 mapping)
    let initial_shares = fragment_data(webpage_bytes, k, n, &mut rng).unwrap();
    assert_eq!(initial_shares.len(), 5);

    // =========================================================================
    // 2. PARASITIC AMBIENT ENTROPY SETUP
    // =========================================================================
    // We simulate an external, public, un-trusted internet telemetry stream (e.g. NASA telemetry, Bitcoin blocks).
    // This is the "Entropy Pool" E.
    let mut external_public_telemetry = vec![FieldElement::zero(); webpage_bytes.len()];
    for val in external_public_telemetry.iter_mut() {
        let mut buf = [0u8; 4];
        let _ = rng.fill_bytes(&mut buf);
        *val = FieldElement::new(u32::from_be_bytes(buf));
    }

    // =========================================================================
    // 3. STEALTH ANCHOR & RATCHET SETUP
    // =========================================================================
    // Alice and Bob share an anchor and whitening seed offline.
    let anchor = FieldElement::new(13);
    let mut seed = [0u8; 32];
    seed[0..4].copy_from_slice(&13u32.to_be_bytes()); // Anchor
    seed[4..8].copy_from_slice(&7u32.to_be_bytes());  // Whitening seed
    let ratchet = StateRatchet::new(seed);

    // =========================================================================
    // 4. ALICE: IMPOSE, INJECT & ATTEST
    // =========================================================================
    // For each of her shares, Alice derives dynamic whitening factor k_pool,
    // k_mac, and nonce from her StateRatchet, then whitens, imposes, and attests
    // to her static contribution beforehand.
    let mut parasitised_shares = Vec::with_capacity(n);
    let mut parasitised_tags = Vec::with_capacity(n); // Wegman-Carter tags

    for share in initial_shares.iter() {
        let mut mutated_data_points = Vec::with_capacity(share.data_points.len());
        let mut data_tags = Vec::with_capacity(share.data_points.len());

        let share_idx = share.id.value() as u64;
        let mut share_ratchet = ratchet.clone();
        share_ratchet.counter = share_idx;
        let (k_pool, k_mac, nonce) = share_ratchet.step().unwrap();

        // Construct StealthIdentity using k_pool dynamically as the whitening factor
        let stealth = StealthIdentity::new(anchor, k_pool);

        for (idx, &s) in share.data_points.iter().enumerate() {
            // A. Whitening (Sikrer 100% statistisk hvid støj-profil)
            let s_whitened = stealth.shard_whiten(s);
            // B. Impose (Beregnes helt uafhængigt af netværket)
            let m = stealth.impose(s_whitened);
            // C. Inject (Indlejres i den uvidende eksterne entropi-strøm E)
            let x = stealth.inject(m, external_public_telemetry[idx]);
            // D. Attest (Generer Wegman-Carter tag over M)
            let tag = stealth.generate_attestation(m, k_mac, nonce);

            mutated_data_points.push(x);
            data_tags.push(tag);
        }

        parasitised_shares.push(core_logic::hydra_sss::HydraShare {
            id: share.id,
            data_points: mutated_data_points,
        });
        parasitised_tags.push(data_tags);
    }

    assert_eq!(parasitised_shares.len(), 5);

    // =========================================================================
    // 5. UNWITTING PARTICIPANTS AUDIT (PLAUSIBLE DENIABILITY)
    // =========================================================================
    // Eve seizes all public files and pools.
    // Since the database contains 100% uniform data, Eve cannot prove that Alice
    // uploaded any specific webpage, nor that Bob downloaded anything.

    // =========================================================================
    // 6. EVE'S ACTIVE MITM ATTACK
    // =========================================================================
    // Eve tampers with the data points of share 1 by changing a coordinate value,
    // and she blocks share 3 entirely!
    println!("[EVE ATTACK]: Eve blocks share 3, and tampers with share 1!");
    
    // Tamper with share 1
    parasitised_shares[0].data_points[5] = parasitised_shares[0].data_points[5] + FieldElement::new(999);

    // =========================================================================
    // 7. BOB: PASSIVE ATTESATION VERIFICATION & RECONSTRUCTION
    // =========================================================================
    // Bob fetches the remaining unblocked shares (1, 2, 4, 5). He verifies each
    // share's Wegman-Carter tags before transposing.
    let mut verified_shares = Vec::new();

    for (s_idx, share) in parasitised_shares.iter().enumerate() {
        // Skip blocked share (index 2 corresponds to share ID 3)
        if s_idx == 2 {
            println!("Bob: Share ID 3 is blocked/censored. Skipping.");
            continue;
        }

        let share_idx = share.id.value() as u64;
        let mut share_ratchet = ratchet.clone();
        share_ratchet.counter = share_idx;
        let (k_pool, k_mac, nonce) = share_ratchet.step().unwrap();

        let stealth = StealthIdentity::new(anchor, k_pool);
        let mut reconstructed_points = Vec::with_capacity(share.data_points.len());
        let mut share_valid = true;

        for (idx, &x) in share.data_points.iter().enumerate() {
            let tag = parasitised_tags[s_idx][idx];
            
            // Reconstruct M: M = X - E
            let m = x - external_public_telemetry[idx];

            // Verify attestation
            let is_valid = stealth.verify_attestation(m, k_mac, nonce, tag);
            if bool::from(is_valid) {
                let recovered_whitened = stealth.transpose(x, external_public_telemetry[idx]);
                let s_recovered = stealth.shard_unwhiten(recovered_whitened);
                reconstructed_points.push(s_recovered);
            } else {
                share_valid = false;
                break;
            }
        }

        if share_valid {
            println!("Bob: Share ID {} successfully VERIFIED. Extracting.", share.id.value());
            verified_shares.push(HydraShare {
                id: share.id,
                data_points: reconstructed_points,
            });
        } else {
            println!("Bob: Share ID {} TAMPERING DETECTED! Wegman-Carter attestation failed. Discarding.", share.id.value());
        }
    }

    // =========================================================================
    // 8. WEBPAGE RECONSTRUCTION
    // =========================================================================
    // Bob should have discarded share 1, skipped share 3, and successfully verified shares 2, 4, and 5.
    // 3 verified shares satisfies the reconstruction threshold k = 3!
    assert_eq!(verified_shares.len(), 3);
    assert_eq!(verified_shares[0].id.value(), 2);
    assert_eq!(verified_shares[1].id.value(), 4);
    assert_eq!(verified_shares[2].id.value(), 5);

    let reconstructed_webpage = reconstruct_data(&verified_shares, k).unwrap();
    assert_eq!(reconstructed_webpage, webpage_bytes);

    println!("Simulation Successful: Wegman-Carter OTM Attestation completely blocked Eve's tampering and censorship!");
}
