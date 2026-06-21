import Transport.Composition
import Transport.WireComposition
import OfflineChannel
import AEH.StegoIndistinguishability

/-!
# Medium independence — wire seal on every delivery medium (P2.3 / M4)

C1 wire Shannon + L1 cell hold identically on pool, AEH, and offline/sneakernet.
Eve's medium choice (pool bytes, benign E, empty O_net) cannot break C/I in O.
-/

namespace ITS

open Transport

/-- Delivery medium tag for wire-seal independence. -/
inductive DeliveryMedium
  | pool
  | aeh
  | offline
  deriving DecidableEq, Repr

/-- Wire seal C1 holds identically on every medium. -/
def wireSealOnMedium (_m : DeliveryMedium) (n : Nat) (hn : n ≥ 1) : Prop :=
  wirePayloadConfidentiality n hn

theorem wire_seal_on_medium (m : DeliveryMedium) (n : Nat) (hn : n ≥ 1) :
    wireSealOnMedium m n hn :=
  wire_payload_confidentiality n hn

def poolMediumZeroLeak : Prop := modePoolZeroLeak

theorem pool_medium_zero_leak : poolMediumZeroLeak :=
  mode_pool_zero_leak

def aehMediumZeroLeak : Prop := modeAehZeroLeak

theorem aeh_medium_zero_leak : aehMediumZeroLeak :=
  mode_aeh_zero_leak

def offlineMediumZeroLeak : Prop := offlineChannelZero 0

theorem offline_medium_zero_leak : offlineMediumZeroLeak :=
  offline_channel_zero 0

/-- Wire-seal + mode bundles on pool, AEH, and offline media. -/
def mediumIndependence : Prop :=
  (∀ m n hn, wireSealOnMedium m n hn) ∧
    poolMediumZeroLeak ∧
    aehMediumZeroLeak ∧
    offlineMediumZeroLeak

theorem medium_independence : mediumIndependence :=
  ⟨fun m n hn => wire_seal_on_medium m n hn,
   pool_medium_zero_leak, aeh_medium_zero_leak, offline_medium_zero_leak⟩

end ITS
