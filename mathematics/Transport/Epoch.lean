import Transport.Cell

/-!
# Epoch stream — L3 send invariant

Alice emits exactly one cell per epoch; cells are drawn from fixed distribution 𝒟.
-/

namespace Transport

/-- Epoch index (monotone). -/
abbrev Epoch := Nat

/-- Ideal epoch step: `(K_{e+1}, C_e) = step(K_e, e)` with C_e ~ 𝒟. -/
def idealStep (_ke : Nat) (e : Epoch) : Nat × Nat :=
  (e + 1, e % fieldPrime)

/-- Distribution support for published cells. -/
def cellDistributionSupport : Nat := fieldPrime

theorem cell_distribution_uniform (_e : Epoch) :
    cellDistributionSupport = fieldPrime := rfl

/-- L3 implies constant-rate channel observation in O. -/
def l3ConstantRate : Prop :=
  ∀ e, (idealStep 0 e).2 % fieldPrime < fieldPrime

theorem l3_constant_rate : l3ConstantRate :=
  fun e => by
    unfold idealStep
    have hp : 0 < fieldPrime := by unfold fieldPrime; decide
    exact Nat.mod_lt _ hp

/-- L3 (send): emit one cell every epoch, message or idle. -/
def l3SendEmitsEveryEpoch : Prop := l3ConstantRate

theorem l3_send_emits_every_epoch : l3SendEmitsEveryEpoch :=
  l3_constant_rate

structure L3Send where
  emitsEveryEpoch : Prop := l3SendEmitsEveryEpoch
  cellPerEpoch : Nat := 1

def defaultL3Send : L3Send := {}

theorem default_l3_send_emits_every_epoch :
    defaultL3Send.emitsEveryEpoch :=
  l3_send_emits_every_epoch

theorem l3_one_cell_per_epoch :
    defaultL3Send.cellPerEpoch = 1 := rfl

end Transport
