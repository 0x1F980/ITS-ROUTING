//! In-process SssTimeLock roundtrip (same glue as `its-routing time-lock` / `time-unlock`).

use its_self_enclosed_timelock::{GenerateError, SecureRandom, SssTimeLock};

struct SimpleRng {
    state: u32,
}

impl SecureRandom for SimpleRng {
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
fn timelock_generate_solve_roundtrip() {
    let mut rng = SimpleRng { state: 0xC0FFEE };
    let message = b"ROUTING timelock integration";

    let puzzle = SssTimeLock::generate(message, 4, &mut rng).expect("generate");
    let decrypted = puzzle.solve().expect("solve");
    assert_eq!(decrypted, message);
}

#[test]
fn timelock_rejects_empty_message() {
    let mut rng = SimpleRng { state: 1 };
    let err = SssTimeLock::generate(b"", 3, &mut rng).unwrap_err();
    assert!(matches!(err, GenerateError::InvalidInput));
}
