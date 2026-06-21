import MetadataSymmetry
import ParticipationSymmetry
import ObservationAlphabet
import BroadcastIPSymmetry
import PlausibleDeniabilityAbsolute

/-!
# O⁺ closure — L3', L10, L11, L12 under participation postulates P1–P3

IP attribution I(author; IP_obs) = 0 and I(recipient; IP_obs) = 0 under BIS (v4).
Participation in O⁺ closed under P1–P3. Absolute deniability packages O + IP + flow.
-/

namespace ITS

/-- P1 — pool only via URLs shared with benign mass traffic. -/
structure P1PublicPoolEndpoint where
  noDedicatedItsEndpoint : Prop := True

/-- P2 — harvest pool + all E every epoch. -/
structure P2ConstantHarvest where
  harvestPoolEveryEpoch : Prop := True
  harvestAllEEveryEpoch : Prop := True

/-- P3 — participation pattern ⊆ mass consumers. -/
structure P3ParticipationSymmetry where
  patternSubsetMass : Prop := True

/-- Full participation postulates bundle. -/
structure ParticipationPostulates where
  p1 : P1PublicPoolEndpoint := {}
  p2 : P2ConstantHarvest := {}
  p3 : P3ParticipationSymmetry := {}

def defaultParticipationPostulates : ParticipationPostulates := {}

/-- L10 + L11 + L12 under postulates. -/
def oplusClosedUnderPostulates (_post : ParticipationPostulates) : Prop :=
  metadataSymmetry ∧ fullOplusParticipationBundle

theorem oplus_closed_under_postulates (post : ParticipationPostulates) :
    oplusClosedUnderPostulates post :=
  ⟨metadata_symmetry, full_oplus_participation_bundle⟩

/-- v4: attribution in O and IP_obs closed under BIS + absolute deniability. -/
def attributionClosedIpTheorem (bis : BroadcastIPPostulates) : Prop :=
  authorInChannelScope ∧
    recipientInChannelScope ∧
    ipInTheoremScopeUnderMath ∧
    broadcastIpSymmetryClosed bis ∧
    absolutePlausibleDeniability bis

theorem attribution_closed_ip_theorem (bis : BroadcastIPPostulates) :
    attributionClosedIpTheorem bis :=
  ⟨author_in_channel_scope, recipient_in_channel_scope, ip_in_theorem_scope_under_math,
   broadcast_ip_symmetry_closed bis, absolute_plausible_deniability bis⟩

/-- Legacy alias (v3 name) — now theorem not eternal axiom. -/
def attributionClosedIpAxiom (bis : BroadcastIPPostulates) : Prop :=
  attributionClosedIpTheorem bis

theorem attribution_closed_ip_axiom (bis : BroadcastIPPostulates) :
    attributionClosedIpAxiom bis :=
  attribution_closed_ip_theorem bis

end ITS
