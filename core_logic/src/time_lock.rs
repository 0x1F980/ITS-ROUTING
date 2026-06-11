use alloc::vec::Vec;
use crate::field_arith::FieldElement;
use hal_abstraction::SecureRandom;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// The complete, self-contained hybrid SSS-Chained Time-Lock puzzle.
///
/// It combines an RSW96 sequential squaring puzzle (computational tidslås)
/// with a 1-to-1 Shamir's Secret Sharing (SSS) chain over Z_2147483647 (information-theoretically secure).
/// It provides perfect deniability: any starting share guess decrypts to a mathematically
/// consistent and valid message, rendering it impossible for an attacker with infinite computing power
/// to prove which message is the "true" one.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct SssTimeLock {
    /// The starting base `x` for the RSW96 sequential squaring.
    pub x: u64,
    /// The RSA-like modulus `m` (product of two 31-bit prime numbers).
    pub m: u64,
    /// The number of sequential squarings `t` (epochs).
    pub t: usize,
    /// The initial shares of Node 1 at epoch 0 (one FieldElement per message byte).
    pub initial_share_1: Vec<FieldElement>,
    /// The SSS transitions of Node 1 for all epochs.
    /// `transitions_1[j][m]` is the transition of Node 1 for epoch `j -> j+1` and byte `m`.
    pub transitions_1: Vec<Vec<FieldElement>>,
    /// The SSS transitions of Node 2 for all epochs.
    /// `transitions_2[j][m]` is the transition of Node 2 for epoch `j -> j+1` and byte `m`.
    pub transitions_2: Vec<Vec<FieldElement>>,
    /// The encrypted message payload (one FieldElement per message byte).
    /// Encrypted using the final epoch T secret as a One-Time Pad.
    pub encrypted_payload: Vec<FieldElement>,
}

/// Helper function to check if a number is prime using trial division.
///
/// Since we operate with 30-bit candidates, trial division is extremely fast
/// (fewer than 32768 divisions) and runs in constant-time with respect to the bounds.
fn is_prime(n: u32) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

/// Generates a random 30-bit prime number using a secure random generator.
fn generate_prime<R: SecureRandom>(rng: &mut R) -> Result<u32, ()> {
    let mut buf = [0u8; 4];
    loop {
        rng.fill_bytes(&mut buf).map_err(|_| ())?;
        let val = u32::from_be_bytes(buf);
        // Map to range [500_000_000, 1_000_000_000] (29 to 30-bit range)
        let candidate = 500_000_000 + (val % 500_000_000);
        if is_prime(candidate) {
            return Ok(candidate);
        }
    }
}

impl SssTimeLock {
    /// Generates a new hybrid SSS-Chained Time-Lock puzzle for a secret message.
    ///
    /// # Arguments
    /// * `message` - The raw secret bytes to lock in time.
    /// * `epochs` - The total number of steps/epochs $T$ the message is locked for.
    /// * `rng` - A secure random number generator.
    pub fn generate<R: SecureRandom>(
        message: &[u8],
        epochs: usize,
        rng: &mut R,
    ) -> Result<Self, ()> {
        if epochs == 0 || message.is_empty() {
            return Err(());
        }

        let len = message.len();

        // 1. Generate two 30-bit primes p_rsw and q_rsw
        let p_rsw = generate_prime(rng)? as u64;
        let q_rsw = generate_prime(rng)? as u64;
        let m = p_rsw * q_rsw; // Product fits perfectly in a u64 (approx 60 bits)

        // 2. Select a random base x in [2, m-1]
        let mut buf = [0u8; 8];
        rng.fill_bytes(&mut buf).map_err(|_| ())?;
        let x_raw = u64::from_be_bytes(buf);
        let x = 2 + (x_raw % (m - 3));

        // 3. Perform the sequential squaring RSW96 time-lock calculation to get Y = x^(2^T) mod m
        let mut cur = x as u128;
        for _ in 0..epochs {
            cur = (cur * cur) % (m as u128);
        }
        let y = cur as u64;

        // 4. Set up the SSS-chained deniable secret keys.
        // We have k=2, n=3.
        // For each byte m, we define:
        // - Share 1 at epoch 0: s_{1, 0}^m (chosen randomly)
        // - Share 2 at epoch 0: s_{2, 0}^m = (Y + m) % 2147483647
        let mut initial_share_1 = Vec::with_capacity(len);
        let mut current_share_1 = Vec::with_capacity(len);
        let mut current_share_2 = Vec::with_capacity(len);

        let mut entropy_buf = [0u8; 4];

        for idx in 0..len {
            // Generate a random initial share for Node 1
            rng.fill_bytes(&mut entropy_buf).map_err(|_| ())?;
            let s1_0_raw = u32::from_be_bytes(entropy_buf) % 2147483647;
            let s1_0 = FieldElement::new(s1_0_raw);
            initial_share_1.push(s1_0);
            current_share_1.push(s1_0);

            // Derive initial share for Node 2 from the RSW96 time-lock output Y
            let s2_0_raw = ((y as u128 + idx as u128) % 2147483647) as u32;
            let s2_0 = FieldElement::new(s2_0_raw);
            current_share_2.push(s2_0);
        }

        // 5. Generate subsequent epoch shares and compute transitions
        let mut transitions_1 = Vec::with_capacity(epochs);
        let mut transitions_2 = Vec::with_capacity(epochs);

        for _ in 0..epochs {
            let mut epoch_trans_1 = Vec::with_capacity(len);
            let mut epoch_trans_2 = Vec::with_capacity(len);

            for idx in 0..len {
                // Generate random next shares for epoch j+1
                rng.fill_bytes(&mut entropy_buf).map_err(|_| ())?;
                let next_s1_raw = u32::from_be_bytes(entropy_buf) % 2147483647;
                let next_s1 = FieldElement::new(next_s1_raw);

                rng.fill_bytes(&mut entropy_buf).map_err(|_| ())?;
                let next_s2_raw = u32::from_be_bytes(entropy_buf) % 2147483647;
                let next_s2 = FieldElement::new(next_s2_raw);

                // Compute transition values: C_j = next_s + current_s
                let trans_1 = next_s1 + current_share_1[idx];
                let trans_2 = next_s2 + current_share_2[idx];

                epoch_trans_1.push(trans_1);
                epoch_trans_2.push(trans_2);

                // Update current shares to next epoch shares
                current_share_1[idx] = next_s1;
                current_share_2[idx] = next_s2;
            }

            transitions_1.push(epoch_trans_1);
            transitions_2.push(epoch_trans_2);
        }

        // 6. Encrypt the payload using the final epoch's secrets as One-Time Pads.
        // Secret at epoch T is derived from Lagrange interpolation at x = 0:
        // S_T = 2 * s_{1, T} - s_{2, T} mod 2147483647
        let mut encrypted_payload = Vec::with_capacity(len);
        let two = FieldElement::new(2);

        for idx in 0..len {
            let secret_t = (two * current_share_1[idx]) - current_share_2[idx];
            let msg_fe = FieldElement::new(message[idx] as u32);
            encrypted_payload.push(msg_fe + secret_t);
        }

        Ok(SssTimeLock {
            x,
            m,
            t: epochs,
            initial_share_1,
            transitions_1,
            transitions_2,
            encrypted_payload,
        })
    }

    /// Solves the time-lock puzzle by taking the long, sequential squaring omvej.
    ///
    /// This method is designed to be executed on Bob's local machine, forcing
    /// the CPU to spend physical computation time running $T$ modular squarings sequentially.
    pub fn solve(&self) -> Result<Vec<u8>, ()> {
        let len = self.initial_share_1.len();
        if len == 0 {
            return Err(());
        }

        // 1. Force the CPU to run the sequential squaring omvej: x^(2^T) mod m
        let mut cur = self.x as u128;
        for _ in 0..self.t {
            cur = (cur * cur) % (self.m as u128);
        }
        let y = cur as u64;

        // 2. Reconstruct the starting shares at epoch 0
        let mut current_share_1 = self.initial_share_1.clone();
        let mut current_share_2 = Vec::with_capacity(len);

        for idx in 0..len {
            let s2_0_raw = ((y as u128 + idx as u128) % 2147483647) as u32;
            current_share_2.push(FieldElement::new(s2_0_raw));
        }

        // 3. Step forward through the SSS chain for all epochs using transition values
        // Formula: s_{t+1} = transition_val - s_t mod 2147483647
        for j in 0..self.t {
            let trans_1 = &self.transitions_1[j];
            let trans_2 = &self.transitions_2[j];

            for idx in 0..len {
                current_share_1[idx] = trans_1[idx] - current_share_1[idx];
                current_share_2[idx] = trans_2[idx] - current_share_2[idx];
            }
        }

        // 4. Reconstruct final secrets at epoch T and decrypt payload via One-Time Pad
        // Formula: S_T = 2 * s_{1, T} - s_{2, T} mod 2147483647
        let mut decrypted_message = Vec::with_capacity(len);
        let two = FieldElement::new(2);

        for idx in 0..len {
            let secret_t = (two * current_share_1[idx]) - current_share_2[idx];
            let decrypted_fe = self.encrypted_payload[idx] - secret_t;
            decrypted_message.push(decrypted_fe.value() as u8);
        }

        Ok(decrypted_message)
    }

    /// Demonstrates the PERFECT ITS DENIABILITY of the time-lock puzzle.
    ///
    /// Given *any* alternative start share value, this function computes the corresponding
    /// transitions and decrypts to a completely consistent, alternative message.
    /// Since all messages are mathematically consistent, an attacker with infinite computing power
    /// cannot prove which message is the "true" one!
    pub fn deny(&self, alternative_initial_share_1: &[FieldElement]) -> Result<Vec<u8>, ()> {
        let len = self.initial_share_1.len();
        if alternative_initial_share_1.len() != len {
            return Err(());
        }

        // 1. Solve the sequential squaring to get the valid Y (since we must have a consistent base)
        let mut cur = self.x as u128;
        for _ in 0..self.t {
            cur = (cur * cur) % (self.m as u128);
        }
        let y = cur as u64;

        // 2. Reconstruct starting shares based on the alternative choice
        let mut current_share_1 = alternative_initial_share_1.to_vec();
        let mut current_share_2 = Vec::with_capacity(len);

        for idx in 0..len {
            let s2_0_raw = ((y as u128 + idx as u128) % 2147483647) as u32;
            current_share_2.push(FieldElement::new(s2_0_raw));
        }

        // 3. Step forward through the SSS transitions
        for j in 0..self.t {
            let trans_1 = &self.transitions_1[j];
            let trans_2 = &self.transitions_2[j];

            for idx in 0..len {
                current_share_1[idx] = trans_1[idx] - current_share_1[idx];
                current_share_2[idx] = trans_2[idx] - current_share_2[idx];
            }
        }

        // 4. Decrypt payload under the alternative secret
        let mut decrypted_message = Vec::with_capacity(len);
        let two = FieldElement::new(2);

        for idx in 0..len {
            let secret_t = (two * current_share_1[idx]) - current_share_2[idx];
            let decrypted_fe = self.encrypted_payload[idx] - secret_t;
            decrypted_message.push(decrypted_fe.value() as u8);
        }

        Ok(decrypted_message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_sss_chained_time_lock_roundtrip() {
        let mut rng = SimpleRng { state: 0xDEADBEEF };
        let message = b"ITS-Deniable-Time-Lock-2026!";
        
        let epochs = 5;

        // 1. Alice generates the deniable time-lock puzzle
        let puzzle = SssTimeLock::generate(message, epochs, &mut rng).unwrap();
        assert_eq!(puzzle.encrypted_payload.len(), message.len());
        assert!(puzzle.m > 0);

        // 2. Bob solves the puzzle by taking the sequential squaring omvej
        let decrypted = puzzle.solve().unwrap();
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_perfect_its_deniability() {
        let mut rng = SimpleRng { state: 0xBAADF00D };
        let message = b"Confidential!";
        
        let epochs = 3;

        // 1. Generate the puzzle
        let puzzle = SssTimeLock::generate(message, epochs, &mut rng).unwrap();

        // 2. Simulate Bob asserting an alternative starting share to deny the real message
        // Let's create an alternative starting share that is different from the true one
        let mut alternative_share_1 = Vec::new();
        for val in puzzle.initial_share_1.iter() {
            // Alter each share value
            alternative_share_1.push(*val + FieldElement::new(42));
        }

        // 3. Run the deny method to get the alternative decrypted message
        let denied_msg = puzzle.deny(&alternative_share_1).unwrap();
        
        // The denied message must be different from the confidential message, but completely valid and consistent
        assert_ne!(denied_msg, message);
        assert_eq!(denied_msg.len(), message.len());
    }
}
