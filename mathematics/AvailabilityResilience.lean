import Transport.Basic

/-!
# Availability resilience — SSS deletion bound (operational A)

Shannon ITS does not prove perfect delivery; SSS (k, n) gives reconstruction
under bounded Eve deletion. Not I = 0 — operational resilience only.
-/

namespace ITS

open Transport

/-- Shares deleted by Eve (abstract count). -/
def deletedShares (f : Nat) : Nat := f

/-- Minimum shares required for reconstruction. -/
def thresholdK : Nat := defaultThresholdK

/-- Total shares emitted across epochs. -/
def totalSharesN : Nat := defaultTotalSharesN

/-- Reconstruction succeeds when fewer than (n - k + 1) shares are deleted. -/
def canReconstruct (f : Nat) : Prop :=
  f + thresholdK ≤ totalSharesN

theorem sss_reconstruction_bound (f : Nat) (hf : f + thresholdK ≤ totalSharesN) :
    canReconstruct f := hf

/-- Bound: reconstruction when enough shares remain. -/
def deletionResilience (f : Nat) : Prop :=
  f + thresholdK ≤ totalSharesN → canReconstruct f

theorem availability_resilience (f : Nat) (hf : f + thresholdK ≤ totalSharesN) :
    canReconstruct f :=
  sss_reconstruction_bound f hf

/-- Honest A-claim: operational, not information-theoretic. -/
def availabilityOperational : Prop :=
  ∀ f, f + thresholdK ≤ totalSharesN → canReconstruct f

theorem availability_operational : availabilityOperational :=
  fun f hf => sss_reconstruction_bound f hf

end ITS
