//! OTM verify path used by `its-routing` AEH receive (`verify_aeh_otm` → `verify_tag`).

use its_otm_public_attestation::field_arith::FieldElement;
use its_otm_public_attestation::otm::verify_tag;

/// Mirrors `its_routing/src/main.rs`: first argument is the MAC input `y`.
fn verify_aeh_otm(m: FieldElement, k_mac: FieldElement, nonce: FieldElement, tag: FieldElement) -> bool {
    bool::from(verify_tag(k_mac, m, nonce, tag))
}

#[test]
fn verify_aeh_otm_m31_section5_golden() {
    // ITS-OTM mathematics §5: T = 11 * 415 + 13 = 4578
    let y = FieldElement::new(415);
    let k_mac = FieldElement::new(11);
    let nonce = FieldElement::new(13);
    let tag = FieldElement::new(4578);

    assert!(verify_aeh_otm(y, k_mac, nonce, tag));
    assert!(!verify_aeh_otm(
        FieldElement::new(416),
        k_mac,
        nonce,
        tag
    ));
}
