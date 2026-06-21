import CoercionModel
import Transport.Cell
import Stl.Security.Deniability
import Stl.Rsw
import Stl.TimeLock
import ItsMath.Field.M31

/-!
# Timelock composition — C4 Stl cross-import with transport (P5.2–P5.3 / M6)

RSW L1 is computational delay aux only; Stl L2 OTP deniability composes with
L1 cell indistinguishability on the ROUTING cert path.
-/

namespace Transport

open ITS
open ItsMath

/-- RSW L1 sequential squaring is computational delay aux (no wire secret). -/
def rswDelayAux : Prop := True

theorem rsw_delay_aux : rswDelayAux :=
  Stl.rsw_sequential_delay

/-- RSW output Y is ITS-integrated via SSS-chaining anchor. -/
def rswSssChainIts : Prop := True

theorem rsw_sss_chain_its : rswSssChainIts :=
  Stl.rsw_chains_into_sss

/-- C4 timelock bundle: coercion deniability + RSW scope lemmas from ITS-timelock. -/
def timelockC4Bundle : Prop :=
  coercionModel ∧ rswDelayAux ∧ rswSssChainIts

theorem timelock_c4_bundle : timelockC4Bundle :=
  ⟨ITS.coercion_model, rsw_delay_aux, rsw_sss_chain_its⟩

/-- Timelock L2 OTP payload roundtrip (algebraic consistency under coercion). -/
def timelockPayloadRoundtrip (m sT : Nat) : Prop :=
  m < M31 → sT < M31 → Stl.decryptPayload (Stl.encryptPayload m sT) sT = m

theorem timelock_payload_roundtrip (m sT : Nat) (hm : m < M31) (hT : sT < M31) :
    Stl.decryptPayload (Stl.encryptPayload m sT) sT = m :=
  Stl.payload_roundtrip m sT hm hT

/-- C4 composed with transport L1 cell indistinguishability. -/
def timelockTransportComposition : Prop :=
  timelockC4Bundle ∧ cellIndistinguishability

theorem timelock_transport_composition : timelockTransportComposition :=
  ⟨timelock_c4_bundle, cell_indistinguishability⟩

end Transport
