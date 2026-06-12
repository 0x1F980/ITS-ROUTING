use core_logic::field_arith::FieldElement;
use core_logic::masking::{encapsulate, decapsulate};
use core_logic::otm::{
    generate_tag, verify_tag, derive_forward_secret,
    verify_forward_share, verify_backward_share,
};
use core_logic::poly::Polynomial;
use core_logic::trapdoor::Trapdoor;

// En simpel, hurtig og deterministisk Xorshift32 PRNG til at simulere en højkvalitets TRNG i testen.
struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        SimpleRng { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    #[allow(dead_code)]
    fn next_u64(&mut self) -> u64 {
        let low = self.next_u32() as u64;
        let high = self.next_u32() as u64;
        (high << 32) | low
    }

    fn next_field_element(&mut self) -> FieldElement {
        #[cfg(not(feature = "m61"))]
        {
            FieldElement::new(self.next_u32() % 2147483647)
        }
        #[cfg(feature = "m61")]
        {
            FieldElement::from_u64(self.next_u64() % 2305843009213693951)
        }
    }

    fn next_non_zero_field_element(&mut self) -> FieldElement {
        #[cfg(not(feature = "m61"))]
        {
            FieldElement::new((self.next_u32() % 2147483646) + 1)
        }
        #[cfg(feature = "m61")]
        {
            FieldElement::from_u64((self.next_u64() % 2305843009213693950) + 1)
        }
    }
}

#[test]
fn test_pure_geometric_tunnel_1000_steps() {
    // =========================================================================
    // 1. SETUP PHASE (Ren SSS & Geometrisk Ratchet)
    // =========================================================================
    // Modulus p = 2147483647. Threshold K = 2 (lineære polynomier).
    // Alice's Master Root R = 5.
    let master_root = FieldElement::new(5);

    // Bob's private trapdoor point: (2, P_trap(2) = 11)
    // Bob's public point: (1, P_trap(1) = 8)
    let public_point = (FieldElement::new(1), FieldElement::new(8));
    let trapdoor = Trapdoor::<2>::new([
        (FieldElement::new(2), FieldElement::new(11)),
        public_point,
    ]);

    // Alice og Bob deler det statiske baglæns-polynomium Q(x) = 5 + 3x (modulo 2147483647)
    // Alice kender hele polynomiet. Bob kender kun Master Root R = 5 og det indledende punkt.
    let poly_backward = Polynomial::new([FieldElement::new(5), FieldElement::new(3)]);

    // Indledende synkroniserings-tilstand (Step 0)
    let mut alice_prev_back = (FieldElement::new(2), FieldElement::new(11)); // Q(2) = 11
    let mut bob_prev_back = (FieldElement::new(2), FieldElement::new(11));

    let mut alice_prev_msg = FieldElement::new(7);
    let mut bob_prev_msg = FieldElement::new(7);

    // Initialiser vores simulerede TRNG
    let mut rng = SimpleRng::new(0x1337_C0DE);

    // Buffer til statistisk ensartethedstest (Chi-i-anden)
    let mut ciphertexts = [0 as core_logic::field_arith::FieldStorage; 1000];

    // Vi simulerer 1.000 kontinuerlige pakker over den rene geometriske tunnel
    for i in 1..=1000 {
        // Generer en tilfældig besked fra TRNG (undgår M_i = 0 for at sikre gyldige x-koordinater i SSS)
        let secret_msg = rng.next_non_zero_field_element();

        // =========================================================================
        // 2. ALICE: GEOMETRISK GENERERING & TRANSMISSION
        // =========================================================================
        // A. Baglæns SSS (Autoritet)
        // Alice evaluerer det statiske Q(x) ved et unikt x_i for at generere sit autoritetspunkt.
        // Vi roterer x_i i feltet Z_2147483647 (undgår x=0 og x=1 for at forhindre overlap med Master Root og public point)
        let x_i = FieldElement::new((i % 2147483645) + 2);
        let y_back = poly_backward.evaluate(x_i);
        let alice_back_point = (x_i, y_back);

        // Alice udleder det hemmelige nonce N_i ved at evaluere Q(2147483646)
        let nonce = poly_backward.evaluate(FieldElement::new(2147483646));

        // B. Forlæns SSS (Integritet)
        // Alice udleder den forlæns hemmelighed s_forw fra det foregående baglæns punkt og besked
        let s_forw = derive_forward_secret(alice_prev_back, alice_prev_msg);

        // Alice opretter et nyt forlæns polynomium P_i(x) = s_forw + b_i * x
        // b_i is a fresh slope generated from her TRNG
        let b_i = rng.next_non_zero_field_element();
        let poly_forward = Polynomial::new([s_forw, b_i]);

        // Alice genererer det forlæns integritetspunkt ved x = M_i (beskeden selv!)
        let y_forw = poly_forward.evaluate(secret_msg);
        let alice_forw_point = (secret_msg, y_forw);

        // Alice udleder den hemmelige MAC-nøgle K_MAC ved at evaluere P_i(2147483645)
        let k_mac = poly_forward.evaluate(FieldElement::new(2147483645));

        // C. OTP Maskering & Tag-generering
        // Alice genererer en éngangs-nøgle K_pool fra sin TRNG
        let k_pool = rng.next_field_element();

        // Alice maskerer k_pool med Bobs offentlige punkt
        let masked_point = encapsulate(public_point, k_pool);

        // Alice genererer Wegman-Carter tagget over den maskerede y-koordinat
        let tag = generate_tag(k_mac, masked_point.1, nonce);

        // Alice krypterer beskeden med OTP: C = M + K_pool
        let ciphertext = secret_msg + k_pool;
        ciphertexts[(i - 1) as usize] = ciphertext.value();

        // =========================================================================
        // 3. BOB: VERIFIKATION & DEKRYPTERING (100% Constant-Time)
        // =========================================================================
        // Bob modtager: (masked_point, alice_forw_point, alice_back_point, tag, ciphertext)

        // A. Verificer baglæns autoritetspunkt
        let is_back_valid = verify_backward_share::<2>(master_root, &[bob_prev_back], alice_back_point);

        // Bob rekonstruerer det baglæns polynomium Q(x) for at udlede det hemmelige nonce N_i
        // Da K = 2, kan han interpolere ud fra (0, R) og det modtagne alice_back_point
        let points_back_reconstructed = [
            (FieldElement::zero(), master_root),
            alice_back_point,
        ];
        let bob_nonce = core_logic::trapdoor::lagrange_interpolate(&points_back_reconstructed, FieldElement::new(2147483646));

        // B. Dekapsling & Dekryptering
        // Bob dekapsler K_pool ved hjælp af sin private trapdoor
        let bob_k_pool = decapsulate(&trapdoor, masked_point);
        let decrypted_msg = ciphertext - bob_k_pool;

        // C. Verificer forlæns integritetspunkt
        // Bob udleder den forlæns hemmelighed s_forw
        let bob_s_forw = derive_forward_secret(bob_prev_back, bob_prev_msg);

        // Bob rekonstruerer det forlæns polynomium P_i(x) ud fra (0, s_forw) og det modtagne alice_forw_point
        // (alice_forw_point.0 er den dekrypterede besked, og alice_forw_point.1 er y_forw)
        let points_forw_reconstructed = [
            (FieldElement::zero(), bob_s_forw),
            (decrypted_msg, alice_forw_point.1),
        ];

        // Bob udleder den hemmelige MAC-nøgle K_MAC ved at evaluere P_i(2147483645)
        let bob_k_mac = core_logic::trapdoor::lagrange_interpolate(&points_forw_reconstructed, FieldElement::new(2147483645));

        // Bob genskaber det forlæns polynomium som en Polynomial struct til verifikation
        // Hældningen b_i = (y_forw - s_forw) * decrypted_msg^-1
        let slope_num = alice_forw_point.1 - bob_s_forw;
        let slope_den = decrypted_msg.invert();
        let bob_b_i = slope_num * slope_den;
        let bob_poly_forward = Polynomial::new([bob_s_forw, bob_b_i]);

        let is_forw_valid = verify_forward_share::<2>(&bob_poly_forward, decrypted_msg, alice_forw_point);

        // D. Verificer Wegman-Carter tag
        let is_tag_valid = verify_tag(bob_k_mac, masked_point.1, bob_nonce, tag);

        // E. Kombiner alle tjek i konstant-tid (ingen branch/early return)
        let is_packet_valid = is_back_valid & is_forw_valid & is_tag_valid;

        assert!(
            bool::from(is_packet_valid),
            "Tunnel verifikation fejlede ved trin {}",
            i
        );

        assert_eq!(
            decrypted_msg.value(),
            secret_msg.value(),
            "Dekrypteret besked mismatch ved trin {}",
            i
        );

        // Opdater tilstande for næste rotation
        alice_prev_back = alice_back_point;
        alice_prev_msg = secret_msg;

        bob_prev_back = alice_back_point;
        bob_prev_msg = decrypted_msg;
    }

    // =========================================================================
    // 4. STATISTISK ENSARTETHEDSTEST (Chi-i-anden test på ciphertexts)
    // =========================================================================
    // Vi tjekker, om de 1.000 ciphertexts er uniformt fordelt over Z_2147483647.
    // Vi grupperer dem i 10 statistiske bøtter.
    let mut counts = [0u32; 10];
    for &c in ciphertexts.iter() {
        let bin = (((c as u128) * 10) / (core_logic::field_arith::MODULUS as u128)).min(9) as usize;
        counts[bin] += 1;
    }

    let expected = 1000.0 / 10.0;
    let mut chi_squared = 0.0;
    for &count in counts.iter() {
        let diff = (count as f64) - expected;
        chi_squared += (diff * diff) / expected;
    }

    // For df = 9 og alpha = 0.01 er den kritiske værdi ~21.67.
    // Hvis chi_squared < 25.0, kan vi ikke skelne ciphertexts fra perfekt uniform støj!
    println!("Chi-squared statistisk værdi: {}", chi_squared);
    assert!(
        chi_squared < 25.0,
        "Statistisk ensartethedstest fejlede! Chi-squared: {}",
        chi_squared
    );
}

#[test]
fn test_active_attacker_mitm_and_replay() {
    let master_root = FieldElement::new(5);
    let public_point = (FieldElement::new(1), FieldElement::new(8));
    let trapdoor = Trapdoor::<2>::new([
        (FieldElement::new(2), FieldElement::new(11)),
        public_point,
    ]);

    let poly_backward = Polynomial::new([FieldElement::new(5), FieldElement::new(3)]);

    // Initial tilstand
    let alice_prev_back = (FieldElement::new(2), FieldElement::new(11));
    let bob_prev_back = (FieldElement::new(2), FieldElement::new(11));

    let alice_prev_msg = FieldElement::new(7);
    let bob_prev_msg = FieldElement::new(7);

    // Alice genererer en legitim pakke
    let secret_msg = FieldElement::new(4);
    let x_i = FieldElement::new(3);
    let y_back = poly_backward.evaluate(x_i);
    let alice_back_point = (x_i, y_back);
    let nonce = poly_backward.evaluate(FieldElement::new(2147483646));

    let s_forw = derive_forward_secret(alice_prev_back, alice_prev_msg);
    let b_i = FieldElement::new(2);
    let poly_forward = Polynomial::new([s_forw, b_i]);
    let y_forw = poly_forward.evaluate(secret_msg);
    let alice_forw_point = (secret_msg, y_forw);
    let k_mac = poly_forward.evaluate(FieldElement::new(2147483645));

    let k_pool = FieldElement::new(12);
    let masked_point = encapsulate(public_point, k_pool);
    let tag = generate_tag(k_mac, masked_point.1, nonce);
    let ciphertext = secret_msg + k_pool;

    // =========================================================================
    // MITM CASE 1: Eve ændrer ciphertext (Bit-flip angreb)
    // =========================================================================
    {
        let tampered_ciphertext = ciphertext + FieldElement::new(1); // Eve ændrer ciphertext

        // Bob modtager og dekrypterer
        let bob_k_pool = decapsulate(&trapdoor, masked_point);
        let decrypted_msg = tampered_ciphertext - bob_k_pool;

        // Bob forsøger at verificere
        let bob_s_forw = derive_forward_secret(bob_prev_back, bob_prev_msg);
        let points_forw_reconstructed = [
            (FieldElement::zero(), bob_s_forw),
            (decrypted_msg, alice_forw_point.1),
        ];
        let _bob_k_mac = core_logic::trapdoor::lagrange_interpolate(&points_forw_reconstructed, FieldElement::new(2147483645));

        let slope_num = alice_forw_point.1 - bob_s_forw;
        let slope_den = decrypted_msg.invert();
        let bob_b_i = slope_num * slope_den;
        let bob_poly_forward = Polynomial::new([bob_s_forw, bob_b_i]);

        let is_forw_valid = verify_forward_share::<2>(&bob_poly_forward, decrypted_msg, alice_forw_point);

        // Da decrypted_msg != secret_msg, skal forlæns share verifikation fejle!
        assert!(!bool::from(is_forw_valid));
    }

    // =========================================================================
    // MITM CASE 2: Eve ændrer det forlæns integritetspunkt (y-koordinat)
    // =========================================================================
    {
        let tampered_forw_point = (alice_forw_point.0, alice_forw_point.1 + FieldElement::new(1));

        // Bob dekrypterer den korrekte besked
        let bob_k_pool = decapsulate(&trapdoor, masked_point);
        let decrypted_msg = ciphertext - bob_k_pool;

        // Bob forsøger at rekonstruere og verificere
        let bob_s_forw = derive_forward_secret(bob_prev_back, bob_prev_msg);
        let points_forw_reconstructed = [
            (FieldElement::zero(), bob_s_forw),
            (decrypted_msg, tampered_forw_point.1),
        ];
        let bob_k_mac = core_logic::trapdoor::lagrange_interpolate(&points_forw_reconstructed, FieldElement::new(2147483645));

        // Da Eve ændrede det forlæns punkt, vil den rekonstruerede k_mac være helt forkert,
        // hvilket får Wegman-Carter tag-verifikationen to at fejle!
        let points_back_reconstructed = [
            (FieldElement::zero(), master_root),
            alice_back_point,
        ];
        let bob_nonce = core_logic::trapdoor::lagrange_interpolate(&points_back_reconstructed, FieldElement::new(2147483646));

        let is_tag_valid = verify_tag(bob_k_mac, masked_point.1, bob_nonce, tag);
        assert!(!bool::from(is_tag_valid));
    }

    // =========================================================================
    // MITM CASE 3: Replay angreb (Eve gensender en gammel pakke)
    // =========================================================================
    {
        // Vi simulerer, at Bob har flyttet sin tilstand et skridt fremad
        let bob_advanced_prev_back = (FieldElement::new(4), FieldElement::new(17)); // Bob har bevæget sig til et nyt punkt
        let bob_advanced_prev_msg = FieldElement::new(9);

        // Eve forsøger nu at gensende den gamle pakke
        let bob_s_forw = derive_forward_secret(bob_advanced_prev_back, bob_advanced_prev_msg);
        let bob_k_pool = decapsulate(&trapdoor, masked_point);
        let decrypted_msg = ciphertext - bob_k_pool;

        let points_forw_reconstructed = [
            (FieldElement::zero(), bob_s_forw),
            (decrypted_msg, alice_forw_point.1),
        ];
        let bob_k_mac = core_logic::trapdoor::lagrange_interpolate(&points_forw_reconstructed, FieldElement::new(2147483645));

        // Da Bobs s_forw is different, the reconstructed k_mac is completely corrupt, and tag verification fails promptly!
        let points_back_reconstructed = [
            (FieldElement::zero(), master_root),
            alice_back_point,
        ];
        let bob_nonce = core_logic::trapdoor::lagrange_interpolate(&points_back_reconstructed, FieldElement::new(2147483646));

        let is_tag_valid = verify_tag(bob_k_mac, masked_point.1, bob_nonce, tag);
        assert!(!bool::from(is_tag_valid));
    }
}
