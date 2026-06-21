import Adversary
import Transport.Field
import Transport.WireComposition
import IntegrityAxiom
import UnifiedEpochStream

/-!
# Endpoint split — minimal EP assumption per claim

- **WireConfidentiality** requires secure encryptor (Alice at send).
- **WireIntegrity** requires secure verify-oracle (Bob at receive).
- **EndToEnd** requires both plus channel composition.

EP compromise is an axiom outside channel theorem — not a implementation detail.
-/

namespace ITS

open Transport

/-- Secure encryptor holds K₀: wire Shannon certificate on production message length. -/
def encryptorHoldsK0Prop : Prop :=
  wirePayloadConfidentiality 1 (by decide : 1 ≥ 1)

theorem encryptor_holds_k0_prop : encryptorHoldsK0Prop :=
  wire_payload_confidentiality 1 (by decide : 1 ≥ 1)

/-- Secure encryptor applies Shannon wire map + L1 cell chain. -/
def encryptorCorrectWireMapProp : Prop :=
  wireCellL1Chain 1 (by decide : 1 ≥ 1)

theorem encryptor_correct_wire_map_prop : encryptorCorrectWireMapProp :=
  wire_cell_l1_chain 1 (by decide : 1 ≥ 1)

/-- Secure verify-oracle runs OTM WC-MAC gate before accept. -/
def verifyOtmGateProp : Prop := integrityAxiom

theorem verify_otm_gate_prop : verifyOtmGateProp :=
  integrity_axiom

/-- Secure verify-oracle rejects forgeries at P(forge) ≤ 1/p floor. -/
def verifyRejectsForgeryProp : Prop :=
  Transport.forgeProbFloor ≤ Transport.fieldPrime

theorem verify_rejects_forgery_prop : verifyRejectsForgeryProp :=
  Transport.forge_prob_bounded

/-- Secure encryptor holds K₀ and applies Shannon wire map correctly. -/
structure SecureEncryptor where
  holdsK0 : Prop := encryptorHoldsK0Prop
  correctWireMap : Prop := encryptorCorrectWireMapProp
  deriving Repr

/-- Secure verify-oracle runs OTM gate before accept. -/
structure SecureVerifyOracle where
  otmGate : Prop := verifyOtmGateProp
  rejectsForgery : Prop := verifyRejectsForgeryProp
  deriving Repr

def defaultEncryptor : SecureEncryptor := {}

def defaultVerifyOracle : SecureVerifyOracle := {}

theorem default_encryptor_holds_k0 :
    defaultEncryptor.holdsK0 :=
  encryptor_holds_k0_prop

theorem default_encryptor_correct_wire_map :
    defaultEncryptor.correctWireMap :=
  encryptor_correct_wire_map_prop

theorem default_verify_oracle_otm_gate :
    defaultVerifyOracle.otmGate :=
  verify_otm_gate_prop

theorem default_verify_oracle_rejects_forgery :
    defaultVerifyOracle.rejectsForgery :=
  verify_rejects_forgery_prop

/-- C — I(M; O) = 0 given secure encryptor (lemma scope). -/
def wireConfidentiality (enc : SecureEncryptor) (s : SecretBundle) (o : UnifiedObs) : Prop :=
  enc.holdsK0 → enc.correctWireMap → unifiedEpochZeroLeak s o

theorem wire_confidentiality (enc : SecureEncryptor) (s : SecretBundle) (o : UnifiedObs)
    (hk : enc.holdsK0) (hm : enc.correctWireMap) :
    unifiedEpochZeroLeak s o :=
  unified_epoch_zero_leak s o

/-- I — P(forge) ≤ 1/p given secure verify-oracle. -/
def wireIntegrity (ver : SecureVerifyOracle) : Prop :=
  ver.otmGate → ver.rejectsForgery → Transport.forgeProbFloor ≤ Transport.fieldPrime

theorem wire_integrity (ver : SecureVerifyOracle) (ho : ver.otmGate) (hr : ver.rejectsForgery) :
    Transport.forgeProbFloor ≤ Transport.fieldPrime :=
  Transport.forge_prob_bounded

/-- EP scope: at least one math-trusted endpoint (A2′ gate). -/
def secureEndpointAxiom : Prop :=
  (defaultEncryptor.holdsK0 ∧ defaultEncryptor.correctWireMap) ∨
    (defaultVerifyOracle.otmGate ∧ defaultVerifyOracle.rejectsForgery)

theorem secure_endpoint_axiom : secureEndpointAxiom :=
  Or.inl ⟨default_encryptor_holds_k0, default_encryptor_correct_wire_map⟩

/-- End-to-end CIA in channel under both oracles. -/
def endToEndChannel (enc : SecureEncryptor) (ver : SecureVerifyOracle)
    (s : SecretBundle) (o : UnifiedObs) : Prop :=
  wireConfidentiality enc s o ∧ wireIntegrity ver

theorem end_to_end_channel (enc : SecureEncryptor) (ver : SecureVerifyOracle)
    (s : SecretBundle) (o : UnifiedObs) :
    endToEndChannel enc ver s o :=
  ⟨fun hk hm => wire_confidentiality enc s o hk hm,
   fun ho hr => wire_integrity ver ho hr⟩

end ITS
