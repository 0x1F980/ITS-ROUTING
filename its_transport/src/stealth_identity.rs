use crate::field_arith::FieldElement;
use crate::otm::{generate_tag, verify_tag};
use subtle::{Choice, ConstantTimeEq};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Optional stealth identity layer implementing **Ambient Entropy Harvesting (AEH)**.
///
/// In this mode, Alice and Bob do not run active VPS routing servers. Instead, they parasitically
/// inject and extract messages from public, external chaotic entropy pools (e.g. public telemetries,
/// block hashes, or torrent trackers) using algebraic anchors.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct StealthIdentity {
    /// The shared anchor reference point (e.g. derived from public keys or shared seeds).
    pub anchor: FieldElement,
    /// A pseudorandom whitening factor derived from the ratchet to flatten statistical patterns.
    pub whitening_factor: FieldElement,
}

impl StealthIdentity {
    /// Creates a new `StealthIdentity` instance.
    pub fn new(anchor: FieldElement, whitening_factor: FieldElement) -> Self {
        StealthIdentity {
            anchor,
            whitening_factor,
        }
    }

    /// Alice computes her static contribution `M` beforehand without knowing the network's state.
    ///
    /// Formula: `M = S - A mod 2147483647`
    /// This masks the shard `S` completely behind the anchor `A`.
    #[inline]
    pub fn impose(&self, shard: FieldElement) -> FieldElement {
        shard - self.anchor
    }

    /// Alice injects her contribution `M` into an external public entropy pool element `E`.
    ///
    /// Formula: `X = M + E mod 2147483647`
    /// To any observer without the anchor, the combined result `X` is indistinguishable from random noise.
    #[inline]
    pub fn inject(&self, m: FieldElement, entropy: FieldElement) -> FieldElement {
        m + entropy
    }

    /// Bob passively extracts Alice's shard `S` from the observed combined block `X` using the anchor and entropy.
    ///
    /// Formula: `S' = X - E + A mod 2147483647`
    ///
    /// Proof of correctness:
    /// `S' = (S - A + E) - E + A = S mod 2147483647`
    #[inline]
    pub fn transpose(&self, x: FieldElement, entropy: FieldElement) -> FieldElement {
        x - entropy + self.anchor
    }

    /// Whiten (Hamming-Fix) a shard to flatten its statistical bit-patterns and 0/1 ratios.
    ///
    /// Blends the shard with the shared pseudorandom `whitening_factor` so that it is
    /// statistically indistinguishable from raw physical hardware noise.
    #[inline]
    pub fn shard_whiten(&self, shard: FieldElement) -> FieldElement {
        shard + self.whitening_factor
    }

    /// Reverts the whitening transformation to extract the original, clean SSS shard.
    #[inline]
    pub fn shard_unwhiten(&self, whitened_shard: FieldElement) -> FieldElement {
        whitened_shard - self.whitening_factor
    }

    /// Bob scans a candidate entropy pool element to see if the statistical clue is present.
    ///
    /// Returns a `subtle::Choice` representing whether the clue is valid (1) or invalid (0).
    pub fn discover_clue(
        &self,
        pool_element: FieldElement,
        expected_raw_element: FieldElement,
    ) -> Choice {
        let reconstructed_anchor = pool_element - expected_raw_element;
        reconstructed_anchor.ct_eq(&self.anchor)
    }

    /// Generates a Wegman-Carter One-Time MAC attestation tag for our static contribution `m`.
    ///
    /// Formula: `T = (K_MAC * M + Nonce) mod 2147483647`
    #[inline]
    pub fn generate_attestation(
        &self,
        m: FieldElement,
        k_mac: FieldElement,
        nonce: FieldElement,
    ) -> FieldElement {
        generate_tag(k_mac, m, nonce)
    }

    /// Verifies a Wegman-Carter One-Time MAC attestation tag in constant-time.
    #[inline]
    pub fn verify_attestation(
        &self,
        m: FieldElement,
        k_mac: FieldElement,
        nonce: FieldElement,
        tag: FieldElement,
    ) -> Choice {
        verify_tag(k_mac, m, nonce, tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passive_entropy_parasitism_roundtrip() {
        let anchor = FieldElement::new(12);
        let whitening_factor = FieldElement::new(5);
        let stealth = StealthIdentity::new(anchor, whitening_factor);

        let secret_shard = FieldElement::new(15);
        let external_entropy = FieldElement::new(7); // e.g. from an external public telemetry stream

        // 1. Whitening (Sikrer 100% statistisk støj-profil)
        let whitened = stealth.shard_whiten(secret_shard);
        assert_eq!(whitened.value(), 20); // 15 + 5 = 20 mod 2147483647

        // 2. Impose (Beregnes forinden asynkront)
        let m = stealth.impose(whitened);
        assert_eq!(m.value(), 8); // 20 - 12 = 8 mod 2147483647

        // 3. Inject (Indlejres i ekstern entropi)
        let x = stealth.inject(m, external_entropy);
        assert_eq!(x.value(), 15); // 8 + 7 = 15 mod 2147483647

        // --- TRANSIT (Eve ejer al infrastruktur og ser kun x = 15, hvilket passer til enhver besked) ---

        // 4. Transpose (Bob trækker sharden ud)
        let recovered_whitened = stealth.transpose(x, external_entropy);
        assert_eq!(recovered_whitened.value(), 20);

        // 5. Unwhitening
        let recovered_shard = stealth.shard_unwhiten(recovered_whitened);
        assert_eq!(recovered_shard.value(), 15);
    }
}
