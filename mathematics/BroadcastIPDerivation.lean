import UnifiedEpochStream
import Transport.Cell
import AEH.StegoIndistinguishability
import BroadcastIPSymmetry

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

end ITS
