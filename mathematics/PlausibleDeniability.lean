import Adversary
import AEH.StegoIndistinguishability

/-!
# Plausible deniability — AEH mode

Alice/Bob indistinguishable from benign E-mass consumers under φ ~ 𝒟_benign.
-/

namespace ITS

open AEH

/-- Deniability claim: ITS actors blend into benign E population. -/
def plausibleDeniability : Prop :=
  stegoPosteriorUniform ∧ benignEmbedSupport = Transport.fieldPrime

theorem plausible_deniability : plausibleDeniability :=
  ⟨stego_posterior_uniform, benign_embed_support_eq⟩

/-- AEH culpability: "normal E use" is mathematically consistent. -/
def aehDeniabilityZeroBits (alice bob : Nat) (obs : Nat) : Prop :=
  mutualInfo (alice + bob) obs = 0

theorem aeh_deniability_zero_bits (alice bob obs : Nat) :
    aehDeniabilityZeroBits alice bob obs :=
  mutual_info_zero (alice + bob) obs

end ITS
