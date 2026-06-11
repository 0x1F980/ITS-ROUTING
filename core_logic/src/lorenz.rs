use zeroize::{Zeroize, ZeroizeOnDrop};

/// A deterministic, fixed-point chaotic Lorenz Attractor generator.
///
/// Under the hood, this uses 16.16 fixed-point arithmetic to guarantee identical
/// mathematical trajectories across all CPU architectures (ARM Cortex-M, x86_64, RISC-V, etc.)
/// without floating-point variability or non-deterministic CPU scheduling.
///
/// The Lorenz Attractor operates in a chaotic state space and is used to modulate
/// timing intervals (chaff ticks) dynamically, making them appear completely natural
/// and indistinguishable from ambient biological/physical noise, while remaining
/// perfectly reproducible for synchronized sender/receiver endpoints.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct LorenzAttractor {
    /// State variable X in Q16.16 fixed-point format.
    pub x: i32,
    /// State variable Y in Q16.16 fixed-point format.
    pub y: i32,
    /// State variable Z in Q16.16 fixed-point format.
    pub z: i32,
}

#[inline]
fn q_mul(a: i32, b: i32) -> i32 {
    (((a as i64) * (b as i64)) >> 16) as i32
}

impl LorenzAttractor {
    /// Creates a new `LorenzAttractor` instance seeded with a shared secret.
    ///
    /// The seed is used to project initial states safely into standard starting points
    /// far from the equilibrium points, ensuring high chaotic oscillation right from the start.
    pub fn new(seed: u32) -> Self {
        // We start farther from the foci to trigger rapid butterfly oscillations.
        // Base starting point: x = 655360 (10.0), y = 65536 (1.0), z = 589824 (9.0)
        let x_perturb = (((seed & 0xFF) as i32) - 128) * 200; 
        let y_perturb = ((((seed >> 8) & 0xFF) as i32) - 128) * 200;
        let z_perturb = ((((seed >> 16) & 0xFF) as i32) - 128) * 200;

        LorenzAttractor {
            x: 655360 + x_perturb,
            y: 65536 + y_perturb,
            z: 589824 + z_perturb,
        }
    }

    /// Evaluates one multi-step chaotic iteration of the Lorenz equations
    /// and returns the next deterministically jitted timing interval in milliseconds.
    ///
    /// Standard chaotic parameters:
    /// \sigma = 10, \rho = 28, \beta = 8/3
    pub fn next_step(&mut self) -> u32 {
        let dt = 655;         // dt = 0.01 in Q16.16 (0.01 * 65536 = 655.36)
        let sigma = 655360;   // \sigma = 10.0 in Q16.16
        let rho = 1835008;    // \rho = 28.0 in Q16.16
        let beta = 174762;    // \beta = 8/3 in Q16.16 (2.6666 * 65536 = 174762.6)

        // Run 10 internal Euler integration steps to allow chaotic mixing,
        // which eliminates short-term linear autocorrelation and matches natural timing profiles.
        for _ in 0..10 {
            let dx = q_mul(q_mul(sigma, self.y - self.x), dt);
            let dy = q_mul(q_mul(self.x, rho - self.z) - self.y, dt);
            let dz = q_mul(q_mul(self.x, self.y) - q_mul(beta, self.z), dt);

            // Saturated addition prevents chaotic boundary overflows from creating undefined states
            self.x = self.x.saturating_add(dx);
            self.y = self.y.saturating_add(dy);
            self.z = self.z.saturating_add(dz);
        }

        // Map state variable Z to a delay in milliseconds.
        // In the standard Lorenz Attractor, Z oscillates chaotically within [12.0, 45.0].
        let z_int = (self.z >> 16).abs();
        let z_clamped = z_int.clamp(12, 45);
        let normalized = (z_clamped - 12) as u32;

        // Map the physical bounds [12, 45] of the Lorenz butterfly lobes directly to [50ms, 150ms]
        50 + (normalized * 100 / 33)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::println;

    #[test]
    fn test_lorenz_chaotic_divergence() {
        // Two attractors with extremely close initial seeds (butterfly effect)
        let mut l1 = LorenzAttractor::new(123456);
        let mut l2 = LorenzAttractor::new(123457);

        println!("Initial l1: {:?}", l1);
        println!("Initial l2: {:?}", l2);

        // Initially they should yield identical or extremely close delays
        let d1_0 = l1.next_step();
        let d2_0 = l2.next_step();
        assert!((d1_0 as i32 - d2_0 as i32).abs() < 10);

        // After 100 chaotic steps (with 10 internal steps each), they should diverge wildly due to the butterfly effect
        for i in 0..100 {
            let s1 = l1.next_step();
            let s2 = l2.next_step();
            if i % 10 == 0 {
                println!("Step {}: l1={:?} (delay={}), l2={:?} (delay={})", i, l1, s1, l2, s2);
            }
        }

        let d1_final = l1.next_step();
        let d2_final = l2.next_step();

        println!("Final step: d1={}, d2={}", d1_final, d2_final);

        // They are highly likely to have diverged chaotically
        assert_ne!(d1_final, d2_final);
    }
}
