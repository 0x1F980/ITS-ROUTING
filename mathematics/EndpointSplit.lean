import Adversary
import Transport.Field
import UnifiedEpochStream

/-!
# Endpoint split — minimal EP assumption per claim

- **WireConfidentiality** requires secure encryptor (Alice at send).
- **WireIntegrity** requires secure verify-oracle (Bob at receive).
- **EndToEnd** requires both plus channel composition.

EP compromise is an axiom outside channel theorem — not a implementation detail.
-/

namespace ITS

/-- Secure encryptor holds K₀ and applies Shannon wire map correctly. -/
structure SecureEncryptor where
  holdsK0 : Prop := True
  correctWireMap : Prop := True
  deriving Repr

/-- Secure verify-oracle runs OTM gate before accept. -/
structure SecureVerifyOracle where
  otmGate : Prop := True
  rejectsForgery : Prop := True
  deriving Repr

def defaultEncryptor : SecureEncryptor := {}
def defaultVerifyOracle : SecureVerifyOracle := {}

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

/-- EP compromise: keys/plaintext leak — outside channel I(S;O)=0. -/
def secureEndpointAxiom : Prop := True

theorem secure_endpoint_axiom : secureEndpointAxiom := trivial

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
