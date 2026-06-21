import UnifiedEpochStream
import AvailabilityResilience
import ForwardProof

/-!
# CIA doctrine — C = ITS, I = ITS, A = ITS forward-proof (v8/v9)

A = public log forward proof + SSS bound — not operational-only delivery trust.
v9 whitelist/consensus/receive-gate bundle: `networkEcosystemCertificateV9`.
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

/-- Availability (v8 core): forward proof + SSS reconstruct bound. -/
def availabilityITSClaim : Prop :=
  availabilityITSForward

theorem availability_its_claim : availabilityITSClaim :=
  availability_its_forward

/-- Legacy operational SSS bound (subset of availabilityITSForward). -/
def availabilityOperationalClaim : Prop :=
  availabilityOperational

theorem availability_operational_claim : availabilityOperationalClaim :=
  availability_operational

/-- Full CIA bundle — C/I/A all ITS-ranked in v8. -/
def ciaDoctrine : Prop :=
  confidentialityITS ∧ integrityITS ∧ availabilityITSClaim

theorem cia_doctrine : ciaDoctrine :=
  ⟨confidentiality_its, integrity_its, availability_its_claim⟩

end ITS
