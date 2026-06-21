import Transport.Composition
import UnattackableCertificate

/-!
# L9 composition — mode bundles ∧ master unattackable certificate

Imports **downward** from mode bundles and cert; no inverse cert ← composition edge.
-/

namespace Transport

/-- L9: mode bundles + master unattackable certificate. -/
def modeCompositionZeroLeak : Prop :=
  modePoolZeroLeak ∧ modeAehZeroLeak ∧ ITS.unattackableCertificate

theorem mode_composition_zero_leak : modeCompositionZeroLeak :=
  ⟨mode_pool_zero_leak, mode_aeh_zero_leak, ITS.unattackable_certificate⟩

end Transport
