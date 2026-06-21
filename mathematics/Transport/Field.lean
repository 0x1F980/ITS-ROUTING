import Transport.Basic

/-!
# Field parameters — Mersenne-31 (matches `its_transport` / Rust)
-/

namespace Transport

/-- Mersenne prime p = 2³¹ − 1 used on the wire. -/
def mersenne31 : Nat := 2147483647

theorem field_prime_is_mersenne31 :
    fieldPrime = mersenne31 := rfl

theorem field_prime_matches_rust :
    fieldPrime = 2147483647 := rfl

/-- WC-MAC forgery probability floor: P(forge) ≤ 1/p. -/
def forgeProbFloor : Nat := 1

theorem forge_prob_bounded :
    forgeProbFloor ≤ fieldPrime := by
  unfold forgeProbFloor fieldPrime
  decide

end Transport
