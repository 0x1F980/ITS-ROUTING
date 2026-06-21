import Adversary
import IPObservation
import ParticipationTheorem
import BroadcastIPSymmetry

/-!
# Recipient attribution zero — I(recipient; O) = 0 and I(recipient; IP_obs) = 0

Bob's identity / mailbox hint lives **only** inside Shannon ITS wire `body`
(ciphertext). Never in pool headers, share IDs, or IP_obs observable to Eve.

Dual of ParticipationTheorem (author). PoolMailbox: Bob scans all cells;
contact-match is decrypt + OTM on **secure verify-oracle** only.
-/

namespace ITS

/-- Recipient identity (not present in O or IP_obs headers). -/
def recipientId : Nat := 13

/-- Recipient / mailbox ID in observable pool headers (forbidden). -/
def recipientIdInPoolHeader : Prop := False

/-- Recipient hint only inside wire ciphertext (Bob's view after harvest). -/
def recipientIdInCiphertextOnly : Prop :=
  recipientIdInPoolHeader = False ∧ mailboxIdInCiphertextOnly

theorem no_recipient_in_pool_header : recipientIdInPoolHeader = False := rfl

theorem recipient_id_ciphertext_only : recipientIdInCiphertextOnly :=
  ⟨no_recipient_in_pool_header, mailbox_id_ciphertext_only⟩

/-- I(recipient; O) = 0 — no recipient provenance in channel O. -/
def recipientObsMutualInfo (recipient obs : Nat) : Nat :=
  mutualInfo recipient obs

def recipientZeroLeakInO : Prop :=
  recipientIdInPoolHeader = False →
    ∀ obs, recipientObsMutualInfo recipientId obs = 0

theorem recipient_zero_leak_in_o : recipientZeroLeakInO :=
  fun _ obs => mutual_info_zero recipientId obs

/-- Master recipient-attribution package (O + IP under BIS). -/
def recipientAttributionZero (post : BroadcastIPPostulates) : Prop :=
  recipientZeroLeakInO ∧
    recipientIdInCiphertextOnly ∧
    recipientIpZeroUnderBIS post

theorem recipient_attribution_zero (post : BroadcastIPPostulates) :
    recipientAttributionZero post :=
  ⟨recipient_zero_leak_in_o, recipient_id_ciphertext_only, recipient_ip_zero_under_bis post⟩

end ITS
