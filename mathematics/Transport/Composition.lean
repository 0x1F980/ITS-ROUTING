import Transport.Epoch
import Transport.Cell
import Transport.WireComposition
import AEH.StegoIndistinguishability
import AEH.EpochGate

/-!
# Composition — Mode P ⊗ Mode AEH (mode bundles)

Mode-specific zero-leak bundles for transport-mode reasoning.
End-to-end L9 certificate (`modeCompositionZeroLeak`) lives in `Transport.L9Composition`.
-/

namespace Transport

/-- Transport mode tag. -/
inductive TransportMode
  | pool
  | aeh
  deriving DecidableEq, Repr

/-- Mode P: O = published cell sequence. -/
def modePoolZeroLeak : Prop :=
  cellIndistinguishability ∧ l3ConstantRate ∧ wireCellL1Chain 1 (by decide)

theorem mode_pool_zero_leak : modePoolZeroLeak :=
  ⟨cell_indistinguishability, l3_constant_rate, wire_cell_l1_chain 1 (by decide)⟩

/-- Mode AEH: O = benign E observation. -/
def modeAehZeroLeak : Prop :=
  AEH.stegoIndistinguishability ∧ AEH.epochGateZeroLeak

theorem mode_aeh_zero_leak : modeAehZeroLeak :=
  ⟨AEH.stego_indistinguishability, AEH.epoch_gate_zero_leak⟩

end Transport
