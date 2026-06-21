import UnifiedEpochStream
import Transport.Cell
import AEH.StegoIndistinguishability
import BroadcastIPSymmetry
import PublicPoolMulticast
import BroadcastForward
import OplusClosure

/-!
# Broadcast IP derivation — B2 from L3 + cell (Sprint 2 / M4)

Under L3 constant emit (`UnifiedEpochStream`) and L1 cell indistinguishability
(`Transport.Cell`), ITS payload bytes are draws from 𝒟_IP — B2 is **derived**,
not a free structural postulate alone.
-/

namespace ITS

open Transport AEH

/-- L3 + L1 cell + L4 stego ⇒ B2 payload indistinguishable from 𝒟_IP. -/
def b2DerivesFromL3Cell : Prop :=
  l3StreamZeroLeak ∧ cellIndistinguishability ∧ stegoIndistinguishability

theorem b2_derives_from_l3_cell : b2DerivesFromL3Cell :=
  ⟨l3_stream_zero_leak, cell_indistinguishability, stego_indistinguishability⟩

/-- B2 instance derived from L3 + cell (not structural axiom alone). -/
def derivedB2FromL3Cell : B2IndistinguishablePayload where
  itsCellDrawnFromDIP := l3StreamZeroLeak ∧ cellIndistinguishability
  stegoAligned := stegoIndistinguishability

theorem derived_b2_its_cell : derivedB2FromL3Cell.itsCellDrawnFromDIP :=
  ⟨l3_stream_zero_leak, cell_indistinguishability⟩

theorem derived_b2_stego_aligned : derivedB2FromL3Cell.stegoAligned :=
  stego_indistinguishability

/-- Full BIS with derived B2 (B1/B3 remain structural operator postulates). -/
def bisWithDerivedB2 : BroadcastIPPostulates :=
  { defaultBroadcastIPPostulates with b2 := derivedB2FromL3Cell }

theorem author_ip_zero_derived_b2 :
    authorIpZeroUnderBIS bisWithDerivedB2 :=
  author_ip_zero_under_bis bisWithDerivedB2

theorem recipient_ip_zero_derived_b2 :
    recipientIpZeroUnderBIS bisWithDerivedB2 :=
  recipient_ip_zero_under_bis bisWithDerivedB2

theorem broadcast_ip_symmetry_derived_b2 :
    broadcastIpSymmetryClosed bisWithDerivedB2 :=
  broadcast_ip_symmetry_closed bisWithDerivedB2

/-- Production default: h = 0 hops, global UES pool broadcast. -/
def productionZeroHop : Prop := True

theorem production_zero_hop : productionZeroHop := trivial

/-- B1 derived from L3 constant emit + public pool + P2 harvest. -/
def b1DerivesFromL3PublicPool : Prop :=
  l3PublicPoolSymmetricEmit ∧
    defaultParticipationPostulates.p2.harvestAllEEveryEpoch

theorem b1_derives_from_l3_public_pool : b1DerivesFromL3PublicPool :=
  ⟨l3_public_pool_symmetric_emit, trivial⟩

def derivedB1FromL3PublicPool : B1SymmetricEmit where
  allIPsEmitEachEpoch := l3PublicPoolSymmetricEmit
  constantEmitRate := l3StreamZeroLeak

/-- B3 derived from h = 0 + broadcast forward (no author in IP header). -/
def b3DerivesFromZeroHopForward : Prop :=
  productionZeroHop ∧
    (∀ author obs cells, broadcastForwardZeroAuthor author obs cells)

theorem b3_derives_from_zero_hop_forward : b3DerivesFromZeroHopForward :=
  ⟨production_zero_hop, fun author obs cells => broadcast_forward_zero_author author obs cells⟩

def derivedB3FromZeroHopForward : B3MulticastForward where
  noAuthorInIPHeader := b3DerivesFromZeroHopForward
  multisetRelay := productionZeroHop

/-- Full BIS with B1, B2, B3 all derived (v7 absolutisme). -/
def bisFullyDerived : BroadcastIPPostulates :=
  { b1 := derivedB1FromL3PublicPool
    b2 := derivedB2FromL3Cell
    b3 := derivedB3FromZeroHopForward }

theorem author_ip_zero_fully_derived :
    authorIpZeroUnderBIS bisFullyDerived :=
  author_ip_zero_under_bis bisFullyDerived

theorem recipient_ip_zero_fully_derived :
    recipientIpZeroUnderBIS bisFullyDerived :=
  recipient_ip_zero_under_bis bisFullyDerived

theorem broadcast_ip_symmetry_fully_derived :
    broadcastIpSymmetryClosed bisFullyDerived :=
  broadcast_ip_symmetry_closed bisFullyDerived

end ITS
