import Transport.Epoch
import Transport.Cell
import Transport.ChaffIndistinguishability
import Transport.MixAnonymity
import Transport.WireComposition
import AEH.StegoIndistinguishability
import AEH.EpochGate
import UnattackableCertificate

/-!
# Composition — Mode P ⊗ Mode AEH (L9)

Delegates end-to-end math certificate to `UnattackableCertificate.lean`.
Mode-specific zero-leak bundles remain for transport-mode reasoning.
-/

namespace Transport

/-- Transport mode tag. -/
inductive TransportMode
  | pool
  | aeh
  deriving DecidableEq, Repr

/-- Mode P: O = published cell sequence. -/
def modePoolZeroLeak : Prop :=
  cellIndistinguishability ∧ l3ConstantRate ∧ wireCellL1Chain ITS.defaultMessageLen ITS.default_message_len

theorem mode_pool_zero_leak : modePoolZeroLeak :=
  ⟨cell_indistinguishability, l3_constant_rate, wire_cell_l1_chain ITS.defaultMessageLen ITS.default_message_len⟩

/-- Mode AEH: O = benign E observation. -/
def modeAehZeroLeak : Prop :=
  AEH.stegoIndistinguishability ∧ AEH.epochGateZeroLeak

theorem mode_aeh_zero_leak : modeAehZeroLeak :=
  ⟨AEH.stego_indistinguishability, AEH.epoch_gate_zero_leak⟩

/-- L9: mode bundles + master unattackable certificate. -/
def modeCompositionZeroLeak : Prop :=
  modePoolZeroLeak ∧ modeAehZeroLeak ∧ ITS.unattackableCertificate

theorem mode_composition_zero_leak : modeCompositionZeroLeak :=
  ⟨mode_pool_zero_leak, mode_aeh_zero_leak, ITS.unattackable_certificate⟩

end Transport
