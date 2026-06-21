import Stl.Security.Deniability
import Stl.TimeLock
import ItsMath.Field.M31
import ItsMath.AxiomRegistry

/-!
# Coercion model (C4) — ITS-timelock bridge (P5.1 / M15)

Eve may coerce an alternative starting share or decoy puzzle; Stl L2 deniability
remains algebraically consistent under `ItsMath.coercion_model`.
-/

namespace ITS

open ItsMath

/-- Coercion walk yields consistent plaintext (Stl L2 deny roundtrip). -/
def coercionModelAx : Prop :=
  ∀ (c sT m : Nat), sT < M31 → m < M31 →
    Stl.Security.denyDecrypt (Stl.encryptPayload m sT) sT = m

theorem coercion_model_ax : coercionModelAx :=
  fun c sT m hT hm => Stl.Security.deny_produces_consistent_plaintext c sT m hT hm

/-- ROUTING C4 coercion model claim (cross-import). -/
def coercionModel : Prop := coercionModelAx

theorem coercion_model : coercionModel :=
  coercion_model_ax

/-- Deny decrypt produces consistent plaintext for any coerced share walk. -/
theorem timelock_deny_consistent (c sT m : Nat) (hT : sT < M31) (hm : m < M31) :
    Stl.Security.denyDecrypt (Stl.encryptPayload m sT) sT = m :=
  Stl.Security.deny_produces_consistent_plaintext c sT m hT hm

end ITS
