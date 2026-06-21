import Adversary
import Transport.Cell
import Asymmetric.FiniteWireEnc

/-!
# Wire composition — C1 Shannon wire → transport cell (L1 chain)

Channel zero-leak is **conditional** on `Asymmetric.fullWireEncShannonIts`,
not an axiom. Payload inside C_e inherits wire blindness + cell indistinguishability.
-/

namespace Transport

open ITS

/-- Wire Shannon certificate implies payload confidentiality in channel observation. -/
def wirePayloadConfidentiality (n : Nat) (hn : n ≥ 1) : Prop :=
  Asymmetric.fullWireEncShannonIts n hn

theorem wire_payload_confidentiality (n : Nat) (hn : n ≥ 1) :
    wirePayloadConfidentiality n hn :=
  Asymmetric.full_wire_enc_shannon_its n hn

/-- Full L1 chain: wire Shannon + cell indistinguishability. -/
def wireCellL1Chain (n : Nat) (hn : n ≥ 1) : Prop :=
  wirePayloadConfidentiality n hn ∧ cellIndistinguishability

theorem wire_cell_l1_chain (n : Nat) (hn : n ≥ 1) : wireCellL1Chain n hn :=
  ⟨wire_payload_confidentiality n hn, cell_indistinguishability⟩

/-- Corollary: I(secret; O) = 0 given wire certificate (not by definition alone). -/
def channelZeroFromWire (secret observed : Nat) (n : Nat) (hn : n ≥ 1) : Prop :=
  wirePayloadConfidentiality n hn → mutualInfo secret observed = 0

theorem channel_zero_from_wire (secret observed : Nat) (n : Nat) (hn : n ≥ 1)
    (hw : wirePayloadConfidentiality n hn) :
    mutualInfo secret observed = 0 :=
  channel_zero_given_wire secret observed (wirePayloadConfidentiality n hn) hw

end Transport
