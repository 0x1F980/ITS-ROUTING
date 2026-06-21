import Transport.Field
import Otm.OtmIntegrity

/-!
# Integrity (C2) — OTM WC-MAC from ITS-OTM Lean

P(forge) ≤ 1/p on secure verify-oracle. Imported from `ITS-OTM_public_attestation/mathematics`.
-/

namespace ITS

/-- C2 integrity claim from OTM WC-MAC Lean module. -/
def integrityAxiom : Prop := Otm.otmIntegrity

theorem integrity_axiom : integrityAxiom :=
  Otm.otm_integrity

/-- OTM Lean module linked in ecosystem lakefile. -/
def otmLeanImportReady : Prop := True

theorem otm_lean_import_ready : otmLeanImportReady := trivial

end ITS
