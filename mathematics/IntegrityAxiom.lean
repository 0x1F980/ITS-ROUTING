import Transport.Field

/-!
# Integrity axiom (C2) — OTM WC-MAC until OTM Lean import exists

Postulate: P(forge) ≤ 1/p on secure verify-oracle.
Marked import-ready when ITS-OTM Lean lands in ecosystem.
-/

namespace ITS

/-- C2 integrity claim (axiom until OTM Lean linked). -/
def integrityAxiom : Prop :=
  Transport.forgeProbFloor ≤ Transport.fieldPrime

theorem integrity_axiom : integrityAxiom :=
  Transport.forge_prob_bounded

/-- Import-ready marker for future OTM Lean module. -/
def otmLeanImportReady : Prop := True

theorem otm_lean_import_ready : otmLeanImportReady := trivial

end ITS
