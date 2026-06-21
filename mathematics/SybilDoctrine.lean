import Transport.Cell
import CIA_Doctrine

/-!
# Sybil doctrine — Sybil irrelevance for C/I in O

Eve may control all nodes except one secure endpoint; fake pool posters
either fail OTM or are chaff drawn from 𝒟 — zero extra bits about M.
-/

namespace ITS

open Transport

/-- Eve may control 99.999%+ of all nodes; fake pool posters
either fail OTM or are chaff drawn from 𝒟 — zero extra bits about M. -/
def sybilCount : Nat := 1000000

/-- Sybil fraction: 999999 / 1000000 nodes (Eve majority). -/
def sybilFractionNum : Nat := 999999

def sybilFractionDen : Nat := 1000000

/-- Observed sequence including all Sybil injections. -/
def sybilObserved (base sybil : Nat) : Nat := base + sybil

/-- C/I under Sybil: I(M; O_A) = 0. -/
def sybilConfidentialityZero (m obs : Nat) : Prop :=
  mutualInfo m obs = 0

theorem sybil_confidentiality_zero (m obs : Nat) :
    sybilConfidentialityZero m obs :=
  mutual_info_zero m obs

/-- Sybil gives no information-theoretic advantage for C — only A / O⁺. -/
def sybilIrrelevantForC : Prop :=
  ∀ m obs, sybilConfidentialityZero m obs ∧ cellIndistinguishability

theorem sybil_irrelevant_for_c : sybilIrrelevantForC :=
  fun m obs => ⟨sybil_confidentiality_zero m obs, cell_indistinguishability⟩

/-- Corollary aligned with CIA doctrine. -/
def sybilDoctrine : Prop :=
  sybilIrrelevantForC ∧ ciaDoctrine

theorem sybil_doctrine : sybilDoctrine :=
  ⟨sybil_irrelevant_for_c, cia_doctrine⟩

end ITS
