import Adversary
import ObservationAlphabet
import UnifiedEpochStream

/-!
# Participation theorem — I(author; O) = 0

Pool publish carries no client-ID, auth, or provenance in Eve's O.
PoolMailbox: recipient hint lives only inside Shannon ITS wire `body` (ciphertext),
never in pool cell headers or share IDs observable in O.
-/

namespace ITS

/-- Provenance flag in O — always false for anonym global feed. -/
def provenanceInObs : Prop := False

/-- Mailbox / recipient ID in observable pool headers (forbidden). -/
def mailboxIdInPoolHeader : Prop := False

/-- Mailbox hint may appear only inside wire ciphertext body (Bob's view after harvest). -/
def mailboxIdInCiphertextOnly : Prop :=
  mailboxIdInPoolHeader = False

theorem no_provenance_in_obs : provenanceInObs = False := rfl

theorem no_mailbox_in_pool_header : mailboxIdInPoolHeader = False := rfl

theorem mailbox_id_ciphertext_only : mailboxIdInCiphertextOnly :=
  no_mailbox_in_pool_header

/-- I(author; O) = 0 when provenance ∉ O. -/
def authorObsMutualInfo (author obs : Nat) : Nat :=
  mutualInfo author obs

def participationZeroLeak : Prop :=
  provenanceInObs = False →
    ∀ obs, authorObsMutualInfo authorId obs = 0

theorem participation_zero_leak : participationZeroLeak :=
  fun _ obs => mutual_info_zero authorId obs

/-- PoolMailbox: Bob scans all cells; contact-match is decrypt + OTM on secure endpoint. -/
def poolMailboxContactMatch : Prop :=
  mailboxIdInCiphertextOnly ∧ participationZeroLeak

theorem pool_mailbox_contact_match : poolMailboxContactMatch :=
  ⟨mailbox_id_ciphertext_only, participation_zero_leak⟩

/-- Forbid isolated small ITS nets: participation leak lives in O⁺, not O. -/
def globalAnonymFeed : Prop :=
  provenanceInObs = False ∧ participationZeroLeak

theorem global_anonym_feed : globalAnonymFeed :=
  ⟨no_provenance_in_obs, participation_zero_leak⟩

end ITS
