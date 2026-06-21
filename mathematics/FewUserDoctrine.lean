import Adversary
import Transport.Field

/-!
# Few-user doctrine — N = 1 theorem

UES anonymity is **size-independent**: even a single ITS user has
`I(M; O) = 0` because the anonymity set is the cell distribution 𝒟 (mode P)
or all benign E consumers (mode AEH), not the ITS peer count.
-/

namespace ITS

/-- Operational ITS user count (may be 1). -/
def itsUserCount : Nat := 1

/-- Anonymity set cardinality follows 𝒟 support, not ITS headcount. -/
def anonymitySetSize : Nat := Transport.fieldPrime

theorem anonymity_set_independent_of_users :
    anonymitySetSize = Transport.fieldPrime := rfl

/-- Shannon statement: I(M; O) = 0 even when N = 1. -/
def fewUserZeroLeak (n : Nat) : Prop :=
  n = 1 → mutualInfo n Transport.fieldPrime = 0

theorem few_user_zero_leak : fewUserZeroLeak itsUserCount := fun _ =>
  mutual_info_zero itsUserCount Transport.fieldPrime

/-- Corollary: Eve cannot deanonymize via "only one ITS user" in O. -/
def sizeIndependentConfidentiality : Prop :=
  ∀ n, n ≥ 1 → mutualInfo n Transport.fieldPrime = 0

theorem size_independent_confidentiality : sizeIndependentConfidentiality :=
  fun _ _ => mutual_info_zero _ _

end ITS
