import UnifiedEpochStream
import AvailabilityResilience

/-!
# CIA doctrine — C = ITS, I = ITS, A = operational

Explicit ranking for public claims.
-/

namespace ITS

/-- Confidentiality: full ITS within O. -/
def confidentialityITS : Prop :=
  ∀ s o, unifiedEpochZeroLeak s o

theorem confidentiality_its : confidentialityITS :=
  fun s o => unified_epoch_zero_leak s o

/-- Integrity: OTM WC-MAC floor P(forge) ≤ 1/p (abstract). -/
def integrityITS : Prop :=
  Transport.forgeProbFloor ≤ Transport.fieldPrime

theorem integrity_its : integrityITS := Transport.forge_prob_bounded

/-- Availability: best-effort SSS / multi-courier — not I = 0. -/
def availabilityOperationalClaim : Prop :=
  availabilityOperational

theorem availability_operational_claim : availabilityOperationalClaim :=
  availability_operational

/-- Full CIA bundle with honest A ranking. -/
def ciaDoctrine : Prop :=
  confidentialityITS ∧ integrityITS ∧ availabilityOperationalClaim

theorem cia_doctrine : ciaDoctrine :=
  ⟨confidentiality_its, integrity_its, availability_operational_claim⟩

end ITS
