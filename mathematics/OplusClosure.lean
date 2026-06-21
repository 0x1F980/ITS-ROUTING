import MetadataSymmetry
import ParticipationSymmetry
import ParticipationTheorem
import RecipientAttributionZero
import BroadcastIPSymmetry
import PlausibleDeniabilityAbsolute

/-!
# O⁺ closure — L3', L10, L11, L12 under participation postulates P1–P3

IP attribution I(author; IP_obs) = 0 and I(recipient; IP_obs) = 0 under BIS (v4).
Participation in O⁺ closed under P1–P3. Absolute deniability packages O + IP + flow.
-/

namespace ITS

/-- P1 — pool only via URLs shared with benign mass traffic. -/
def p1PublicPoolEndpointProp : Prop := coverNoDedicatedItsEndpoint

theorem p1_public_pool_endpoint : p1PublicPoolEndpointProp :=
  cover_no_dedicated_its_endpoint

structure P1PublicPoolEndpoint where
  noDedicatedItsEndpoint : Prop := p1PublicPoolEndpointProp

/-- P2 — harvest pool + all E every epoch. -/
def p2HarvestPoolProp : Prop := coverHarvestPoolEveryEpoch

theorem p2_harvest_pool : p2HarvestPoolProp :=
  cover_harvest_pool_every_epoch

def p2HarvestAllEProp : Prop := coverHarvestAllEEveryEpoch

theorem p2_harvest_all_e : p2HarvestAllEProp :=
  cover_harvest_all_e_every_epoch

structure P2ConstantHarvest where
  harvestPoolEveryEpoch : Prop := p2HarvestPoolProp
  harvestAllEEveryEpoch : Prop := p2HarvestAllEProp

/-- P3 — participation pattern ⊆ mass consumers. -/
def p3PatternSubsetMassProp : Prop := participationSymmetryZero

theorem p3_pattern_subset_mass_prop : p3PatternSubsetMassProp :=
  participation_symmetry_zero

structure P3ParticipationSymmetry where
  patternSubsetMass : Prop := p3PatternSubsetMassProp

/-- Full participation postulates bundle. -/
structure ParticipationPostulates where
  p1 : P1PublicPoolEndpoint := {}
  p2 : P2ConstantHarvest := {}
  p3 : P3ParticipationSymmetry := {}

def defaultParticipationPostulates : ParticipationPostulates := {}

theorem default_p1_no_dedicated :
    defaultParticipationPostulates.p1.noDedicatedItsEndpoint :=
  p1_public_pool_endpoint

theorem default_p2_harvest_pool :
    defaultParticipationPostulates.p2.harvestPoolEveryEpoch :=
  p2_harvest_pool

theorem default_p2_harvest_all_e :
    defaultParticipationPostulates.p2.harvestAllEEveryEpoch :=
  p2_harvest_all_e

theorem default_p3_pattern_subset :
    defaultParticipationPostulates.p3.patternSubsetMass :=
  p3_pattern_subset_mass_prop

def participationPostulatesDerived : Prop :=
  defaultParticipationPostulates.p1.noDedicatedItsEndpoint ∧
    defaultParticipationPostulates.p2.harvestPoolEveryEpoch ∧
    defaultParticipationPostulates.p2.harvestAllEEveryEpoch ∧
    defaultParticipationPostulates.p3.patternSubsetMass

theorem participation_postulates_derived : participationPostulatesDerived :=
  ⟨default_p1_no_dedicated, default_p2_harvest_pool,
   default_p2_harvest_all_e, default_p3_pattern_subset⟩

/-- CoverTransport derived from P1 (public pool) + P2 (constant harvest). -/
def coverTransportFromPostulates (post : ParticipationPostulates) : CoverTransport :=
  { harvestPoolEveryEpoch := post.p2.harvestPoolEveryEpoch
    harvestAllEEveryEpoch := post.p2.harvestAllEEveryEpoch
    noDedicatedItsEndpoint := post.p1.noDedicatedItsEndpoint }

theorem cover_transport_from_default_postulates :
    (coverTransportFromPostulates defaultParticipationPostulates).harvestPoolEveryEpoch ∧
      (coverTransportFromPostulates defaultParticipationPostulates).harvestAllEEveryEpoch ∧
      (coverTransportFromPostulates defaultParticipationPostulates).noDedicatedItsEndpoint :=
  ⟨default_p2_harvest_pool, default_p2_harvest_all_e, default_p1_no_dedicated⟩

/-- P2 postulates ⇒ L11 constant O⁺ participation. -/
theorem l11_from_participation_postulates (_post : ParticipationPostulates) :
    l11CoverConstantParticipation :=
  l11_cover_constant_participation

/-- P1–P3 postulates ⇒ L12 participation symmetry under L3'. -/
theorem l12_from_participation_postulates (_post : ParticipationPostulates) :
    l12ParticipationSymmetry :=
  l12_participation_symmetry

/-- L10 + L11 + L12 under postulates. -/
def oplusClosedUnderPostulates (_post : ParticipationPostulates) : Prop :=
  metadataSymmetry ∧ fullOplusParticipationBundle

theorem oplus_closed_under_postulates (post : ParticipationPostulates) :
    oplusClosedUnderPostulates post :=
  ⟨metadata_symmetry, full_oplus_participation_bundle⟩

/-- Master theorem applies to channel O under participation zero-leak. -/
def inTheoremScopeO (_o : ChannelObs) : Prop := participationZeroLeak

theorem channel_in_scope (_o : ChannelObs) : inTheoremScopeO _o :=
  participation_zero_leak

/-- IP/physical layer: Shannon MI zero for author attribution under math. -/
def ipInTheoremScopeUnderMath : Prop :=
  ∀ author ipObs, authorIpMutualInfo author ipObs = 0

theorem ip_in_theorem_scope_under_math : ipInTheoremScopeUnderMath :=
  fun author ipObs => mutual_info_zero author ipObs

/-- Author attribution in O is theorem scope (I(author;O)=0). -/
def authorInChannelScope : Prop := participationZeroLeak

theorem author_in_channel_scope : authorInChannelScope :=
  participation_zero_leak

/-- Recipient attribution in O is theorem scope (I(recipient;O)=0). -/
def recipientInChannelScope : Prop := recipientZeroLeakInO

theorem recipient_in_channel_scope : recipientInChannelScope :=
  recipient_zero_leak_in_o

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
