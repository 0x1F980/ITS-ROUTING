//! Fragment roundtrip regression (Gate 1 baseline).

use its_transport::field_arith::FieldElement;
use its_transport::{fragment_data, reconstruct_data, SecureRandom};

struct TestRng(u32);

impl SecureRandom for TestRng {
    type Error = ();
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        for b in dest.iter_mut() {
            self.0 = self.0.wrapping_mul(1103515245).wrapping_add(12345);
            *b = (self.0 >> 16) as u8;
        }
        Ok(())
    }
}

#[test]
fn sss_fragment_roundtrip() {
    let secret = b"Hello, Information-Theoretic Secrecy!";
    let mut rng = TestRng(0xBAADF00D);
    let shares = fragment_data(secret, 4, 10, &mut rng).expect("fragment");
    let got = reconstruct_data(&shares[0..4], 4).expect("reconstruct");
    assert_eq!(got.as_slice(), secret);
}

#[test]
fn field_element_modulus_nonzero() {
    let a = FieldElement::new(1);
    assert_ne!(a.value(), 0);
}
