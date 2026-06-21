import PlausibleDeniabilityAbsolute
import ObservationAlphabet
import BroadcastForward

/-!
# Role-aware deniability — Forwarder / Publisher / Reader (v7)

Extends `noGuiltyNode` so Alice-as-host (publisher) is not confused with a
Tor-style last exit. Readers (Bob₁…Bobₙ) remain zero-leak in O under SOCKS/multi-reader.
-/

namespace ITS

/-- I(reader_i; O) = 0 for all readers. -/
def readerZeroInObs (reader obs : Nat) : Prop :=
  mutualInfo reader obs = 0

theorem reader_zero_in_obs (reader obs : Nat) :
    readerZeroInObs reader obs :=
  mutual_info_zero reader obs

/-- Publisher host ≠ Tor exit — role separation + zero publisher guilt in O. -/
def publisherNotExitNode : Prop :=
  publisherObs ≠ forwarderObs ∧
    ∀ obs, mutualInfo publisherObs obs = 0

theorem publisher_not_exit_node : publisherNotExitNode :=
  ⟨by native_decide, fun obs => mutual_info_zero publisherObs obs⟩

/-- Forwarder preserves author-zero under broadcast forward (h = 0 prod default). -/
def forwarderAuthorZero (author obs cells : Nat) : Prop :=
  broadcastForwardZeroAuthor author obs cells

theorem forwarder_author_zero (author obs cells : Nat) :
    forwarderAuthorZero author obs cells :=
  broadcast_forward_zero_author author obs cells

/-- Role-aware deniability: no guilty node + multi-reader zero-leak + publisher scope. -/
def roleAwareDeniability (post : BroadcastIPPostulates) : Prop :=
  noGuiltyNode post ∧
    (∀ reader obs, readerZeroInObs reader obs) ∧
    publisherNotExitNode ∧
    (∀ author obs cells, forwarderAuthorZero author obs cells)

theorem role_aware_deniability (post : BroadcastIPPostulates) :
    roleAwareDeniability post :=
  ⟨no_guilty_node post,
   fun reader obs => reader_zero_in_obs reader obs,
   publisher_not_exit_node,
   fun author obs cells => forwarder_author_zero author obs cells⟩

end ITS
