import ParticipationTheorem
import PlausibleDeniability
import LinkParticipation
import SybilDoctrine
import BroadcastForward
import AEH.StegoIndistinguishability

/-!
# Author attribution zero — I(author; O) = 0 package

Pool: structural no-provenance in O.
AEH: φ ~ 𝒟_benign statistical indistinguishability.
Sybil: fake posters give zero extra bits about M.
Broadcast: forward hops preserve author-zero.
-/

namespace ITS

open AEH

/-- Pool mode: no author provenance in O. -/
def poolAuthorZero : Prop :=
  participationZeroLeak ∧ globalAnonymFeed

theorem pool_author_zero : poolAuthorZero :=
  ⟨participation_zero_leak, global_anonym_feed⟩

/-- AEH mode: Alice/Bob blend into benign E mass. -/
def aehAuthorZero : Prop :=
  plausibleDeniability ∧ stegoIndistinguishability

theorem aeh_author_zero : aehAuthorZero :=
  ⟨plausible_deniability, stego_indistinguishability⟩

/-- No link identifier in channel O. -/
def linkAuthorZero : Prop :=
  linkParticipationZeroLeak

theorem link_author_zero : linkAuthorZero :=
  link_participation_zero_leak

/-- Sybil irrelevant for author attribution in C/I. -/
def sybilAuthorZero : Prop :=
  sybilIrrelevantForC

theorem sybil_author_zero : sybilAuthorZero :=
  sybil_irrelevant_for_c

/-- Master author-attribution package (pool + AEH + link + Sybil + broadcast). -/
def authorAttributionZero : Prop :=
  poolAuthorZero ∧ aehAuthorZero ∧ linkAuthorZero ∧
    sybilAuthorZero ∧
    (∀ author obs cells, broadcastForwardZeroAuthor author obs cells)

theorem author_attribution_zero : authorAttributionZero :=
  ⟨pool_author_zero, aeh_author_zero, link_author_zero, sybil_author_zero,
   fun author obs cells => broadcast_forward_zero_author author obs cells⟩

end ITS
