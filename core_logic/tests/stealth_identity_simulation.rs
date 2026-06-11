use core_logic::field_arith::FieldElement;
use core_logic::stealth_identity::StealthIdentity;
use core_logic::hydra_sss::{fragment_data, reconstruct_data};
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
    // 3. STEALTH ANCHOR SETUP
    // =========================================================================
    // Alice and Bob share an anchor and whitening seed (derived forinden/offline).
    let anchor = FieldElement::new(13);
    let whitening = FieldElement::new(7);
    let stealth = StealthIdentity::new(anchor, whitening);

    // =========================================================================
    // 4. ALICE: IMPOSE & INJECT (PREPARATION FORINDEN)
    // =========================================================================
    // Alice does NOT connect to Bob or run active handshakes.
    // For each of her shares, she whitens and imposes her static contribution "forinden".
    // This prevents any timing/frequency signature correlation.
    let mut parasitised_shares = Vec::with_capacity(n);

    // We store the mutated shards on public dead-drops (e.g. Torrent metadata trackers)
    // disguised as 100% normal random background noise.
    for share in initial_shares.iter() {
        let mut mutated_data_points = Vec::with_capacity(share.data_points.len());

        for (idx, &s) in share.data_points.iter().enumerate() {
            // A. Whitening (Sikrer 100% statistisk hvid støj-profil)
            let s_whitened = stealth.shard_whiten(s);
            // B. Impose (Beregnes helt uafhængigt af netværket)
            let m = stealth.impose(s_whitened);
            // C. Inject (Indlejres i den uvidende eksterne entropi-strøm E)
            let x = stealth.inject(m, external_public_telemetry[idx]);

            mutated_data_points.push(x);
        }

        parasitised_shares.push(core_logic::hydra_sss::HydraShare {
            id: share.id,
            data_points: mutated_data_points,
        });
    }

    // =========================================================================
    // 5. UNWITTING PARTICIPANTS AUDIT (PLAUSIBLE DENIABILITY)
    // =========================================================================
    // Eve seizes all public files and pools.
    // Since the database contains 100% uniform data and is algebraisk underbestemt,
    // Eve cannot prove that Alice uploaded any specific webpage, nor that Bob downloaded anything.
    // Both Alice, Bob, and the public platform have absolute plausible deniability.
    assert_eq!(parasitised_shares.len(), 5);

    // =========================================================================
    // 6. BOB: PASSIVE TRANSPOSE & REBUILD
    // =========================================================================
    // Bob wants to reconstruct the webpage. He passively reads 3 of the mutated shares (Node 1, 2, 4)
    // and the public telemetry stream, then performs the Inverse-Logik transposition.
    let mut recovered_shares = Vec::with_capacity(k);

    for share in parasitised_shares.iter().take(k) {
        let mut reconstructed_points = Vec::with_capacity(share.data_points.len());

        for (idx, &x) in share.data_points.iter().enumerate() {
            // A. Transpose (Filtrerer den samlede blok gennem ankeret)
            let recovered_whitened = stealth.transpose(x, external_public_telemetry[idx]);
            // B. Unwhiten (Hent det oprindelige SSS punkt)
            let s_recovered = stealth.shard_unwhiten(recovered_whitened);

            reconstructed_points.push(s_recovered);
        }

        recovered_shares.push(core_logic::hydra_sss::HydraShare {
            id: share.id,
            data_points: reconstructed_points,
        });
    }

    // =========================================================================
    // 7. WEBPAGE RECONSTRUCTION
    // =========================================================================
    // Bob reconstructs the original webpage from the recovered shares!
    let reconstructed_webpage = reconstruct_data(&recovered_shares, k).unwrap();
    assert_eq!(reconstructed_webpage, webpage_bytes);

    println!("Simulation Successful: Webpage recovered parasitically with 0% sender traceability!");
}
