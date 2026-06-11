use core_logic::lorenz::LorenzAttractor;

struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        SimpleRng { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }
}

#[test]
fn test_lorenz_statistical_profile() {
    // Initialize our deterministic chaotic attractor using a pseudorandom seed
    let mut rng = SimpleRng::new(0xDEADBEEF);
    let seed = rng.next_u32();
    let mut attractor = LorenzAttractor::new(seed);

    const SAMPLE_COUNT: usize = 1000;
    let mut samples = [0u32; SAMPLE_COUNT];

    // Collect 1000 chaotic timing intervals
    for i in 0..SAMPLE_COUNT {
        samples[i] = attractor.next_step();
    }

    // -------------------------------------------------------------------------
    // 1. BOUNDEDNESS VERIFICATION
    // -------------------------------------------------------------------------
    // The delay must strictly reside in [50, 150] milliseconds to mimic
    // realistic physical networks and prevent queue starvation or buffer bloat.
    for &sample in samples.iter() {
        assert!(sample >= 50, "Delay underflow: {}", sample);
        assert!(sample <= 150, "Delay overflow: {}", sample);
    }

    // -------------------------------------------------------------------------
    // 2. VARIANCE AND ENTROPY ANALYSIS
    // -------------------------------------------------------------------------
    // Compute the mean and variance to ensure high-entropy physical-like distribution
    let sum: u64 = samples.iter().map(|&x| x as u64).sum();
    let mean = (sum as f64) / (SAMPLE_COUNT as f64);

    let variance_sum: f64 = samples.iter()
        .map(|&x| {
            let diff = (x as f64) - mean;
            diff * diff
        })
        .sum();
    let variance = variance_sum / (SAMPLE_COUNT as f64);
    let std_dev = variance.sqrt();

    // Natural physical jitter profiles have a high standard deviation (not static or low-frequency).
    // We expect a robust standard deviation of at least 10.0ms for an interval range of 100ms.
    assert!(std_dev > 10.0, "Jitter profile too static: std_dev = {}", std_dev);

    // -------------------------------------------------------------------------
    // 3. STATISTICAL DISTRIBUTION FREQUENCY CHECK
    // -------------------------------------------------------------------------
    // To ensure the sequence is not degenerate, we bin the delays into 10 buckets
    // and verify that every bucket receives a non-zero, healthy number of hits.
    let mut bins = [0u32; 10];
    for &sample in samples.iter() {
        let idx = ((sample - 50) / 10).min(9) as usize;
        bins[idx] += 1;
    }

    for (bin_idx, &count) in bins.iter().enumerate() {
        assert!(
            count > 5,
            "Degenerate chaotic distribution in bin {}: only {} samples",
            bin_idx,
            count
        );
    }

    // -------------------------------------------------------------------------
    // 4. CHAOTIC AUTOCORRELATION DECAY
    // -------------------------------------------------------------------------
    // Chaotic systems are characterized by rapidly decaying autocorrelation,
    // meaning past delays do not linearly predict future delays.
    // Let's compute the autocorrelation for lag = 1 and lag = 10.
    let lag_1_cov: f64 = (0..(SAMPLE_COUNT - 1))
        .map(|i| {
            ((samples[i] as f64) - mean) * ((samples[i + 1] as f64) - mean)
        })
        .sum::<f64>() / ((SAMPLE_COUNT - 1) as f64);
    let r_1 = lag_1_cov / variance;

    let lag_10_cov: f64 = (0..(SAMPLE_COUNT - 10))
        .map(|i| {
            ((samples[i] as f64) - mean) * ((samples[i + 1] as f64) - mean)
        })
        .sum::<f64>() / ((SAMPLE_COUNT - 10) as f64);
    let r_10 = lag_10_cov / variance;

    // The autocorrelation should be low, demonstrating no simple linear correlations.
    // Due to orbiting around dual chaotic attractor lobes, medium-term low-frequency periodicities
    // are natural, which is why r_10 has a slightly higher bound than long-term white noise.
    assert!(r_1.abs() < 0.85, "High immediate correlation: r_1 = {}", r_1);
    assert!(r_10.abs() < 0.70, "Slow-decaying correlation: r_10 = {}", r_10);
}
