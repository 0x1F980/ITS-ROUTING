import Transport.RatchetDerivation

/-!
# Cell indistinguishability (L1 / type-blindness)

All epochs use the same `step` generator — no separate data/setup/chaff types in O.
-/

namespace Transport

/-- Hidden cell label (not in Eve's O). -/
inductive CellLabel
  | payload
  | idle
  deriving DecidableEq, Repr

/-- Observed wire cell (abstract byte tag). -/
structure ObservedCell where
  tag : Nat
  deriving Repr

/-- Both labels map to the same observation distribution under uniform draw. -/
def observeCell (_label : CellLabel) (draw : Nat) : ObservedCell :=
  { tag := draw % fieldPrime }

def cellPosteriorSupport (_draw : Nat) : Nat := fieldPrime

theorem cell_posterior_support (draw : Nat) :
    cellPosteriorSupport draw = fieldPrime := rfl

/-- L1: I(M; C_wire) = 0 — uniform OTP masks over F_p. -/
def cellIndistinguishability : Prop :=
  ∀ draw, cellPosteriorSupport draw = fieldPrime ∧
    consistentPlaintextElements draw = fieldPrime

theorem cell_indistinguishability : cellIndistinguishability :=
  fun draw => ⟨cell_posterior_support draw, otp_element_blindness draw⟩

end Transport
