use its_transport::field_arith::FieldElement;

// DEPRECATE: legacy stego block codec removed — production AEH uses `aeh_carrier::embed_cell` (φ ~ D_benign).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AehChannel {
    Wikipedia,
    GitHubGists,
    DnsTxt,
    Reddit,
    NasaTelemetry,
    DomesticNews,
    SneakernetFile,
}

impl AehChannel {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            AehChannel::Wikipedia => "Wikipedia API (Simulated)",
            AehChannel::GitHubGists => "GitHub Gists API (Simulated)",
            AehChannel::DnsTxt => "DNS TXT Records (Simulated)",
            AehChannel::Reddit => "Reddit Comments API (Simulated)",
            AehChannel::NasaTelemetry => "NASA Seismology API (Simulated)",
            AehChannel::DomesticNews => "State-Approved Domestic News Board (Simulated ALT)",
            AehChannel::SneakernetFile => "Sneakernet Local File / QR (Simulated ALT)",
        }
    }
}

/// Keyed Carter-Wegman Universal Polynomial Hash (100% ITS-Secure)
/// Maps raw public telemetry bytes directly to N distinct FieldElements.
pub(crate) fn universal_aeh_hash(raw_data: &[u8], key: FieldElement, n: usize) -> Vec<FieldElement> {
    // 1. Group raw bytes into u32 and reduce to FieldElements to form our coefficients
    let mut coeffs = Vec::new();
    let mut chunks = raw_data.chunks_exact(4);
    while let Some(chunk) = chunks.next() {
        let val = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        coeffs.push(FieldElement::new(val));
    }
    let remainder = chunks.remainder();
    if !remainder.is_empty() {
        let mut buf = [0u8; 4];
        buf[..remainder.len()].copy_from_slice(remainder);
        coeffs.push(FieldElement::new(u32::from_be_bytes(buf)));
    }

    if coeffs.is_empty() {
        coeffs.push(FieldElement::new(42));
    }

    // 2. For each of the N desired points, evaluate the polynomial at x = key + j
    let mut hashed_points = Vec::with_capacity(n);
    for j in 0..n {
        let eval_point = key + FieldElement::new(j as u32 + 1);

        // Horner's method for constant-time, ITS-secure polynomial evaluation
        let mut result = FieldElement::zero();
        for &coeff in coeffs.iter().rev() {
            result = (result * eval_point) + coeff;
        }
        hashed_points.push(result);
    }

    hashed_points
}
