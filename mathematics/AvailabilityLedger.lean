import CensorshipDisclosure
import PublicPoolMulticast

/-!
# Availability ledger — strike / slash enforcement (v7 A extension)

Ledger-backed punishment for repeated availability attacks on the public pool.
Attack observables align with `CensorshipDisclosure` (selective omit, mirror
mismatch, L3 rate delta, SSS deletion bound) — not ad-hoc operator flags.

\[
\text{strikes}(a) \geq N \Rightarrow \neg\text{poolPublishAllowed}(a)
\]

Links to `aAbsolute` via shared censorship-disclosure observables.
-/

namespace ITS

/-- Availability attack tags tied to CensorshipDisclosure / PublicPoolMulticast observables. -/
inductive AvailabilityAttackKind
  | selectiveOmit
  | mirrorMismatch
  | rateDeltaGap
  | sssDeletionExceedsBound
  deriving Repr

/-- Map attack tag to the corresponding disclosure observable (CensorshipDisclosure §). -/
def attackKindDisclosed (k : AvailabilityAttackKind) (epoch subscriber witness f : Nat) : Prop :=
  match k with
  | AvailabilityAttackKind.selectiveOmit =>
      mirrorMismatchOnSelectiveOmit subscriber witness
  | AvailabilityAttackKind.mirrorMismatch =>
      mirrorMismatchOnSelectiveOmit subscriber witness
  | AvailabilityAttackKind.rateDeltaGap =>
      l3GapRateDelta epoch
  | AvailabilityAttackKind.sssDeletionExceedsBound =>
      (f + thresholdK ≤ totalSharesN → canReconstruct f) ∨ l3GapRateDelta epoch

theorem attack_selective_omit_disclosed (subscriber witness : Nat) :
    attackKindDisclosed AvailabilityAttackKind.selectiveOmit 0 subscriber witness 0 :=
  mirror_mismatch_on_selective_omit subscriber witness

theorem attack_mirror_mismatch_disclosed (subscriber witness : Nat) :
    attackKindDisclosed AvailabilityAttackKind.mirrorMismatch 0 subscriber witness 0 :=
  mirror_mismatch_on_selective_omit subscriber witness

theorem attack_rate_delta_disclosed (epoch : Nat) :
    attackKindDisclosed AvailabilityAttackKind.rateDeltaGap epoch 0 0 0 :=
  l3_gap_rate_delta epoch

theorem attack_sss_deletion_disclosed (epoch f : Nat) :
    attackKindDisclosed AvailabilityAttackKind.sssDeletionExceedsBound epoch 0 0 f := by
  right
  exact l3_gap_rate_delta epoch

/-- Strike threshold N before send-rights revocation (operational default). -/
def defaultAvailabilityStrikeThreshold : Nat := 3

/-- Per-actor strike counter in ledger state. -/
structure AvailabilityLedgerState where
  strikeThreshold : Nat := defaultAvailabilityStrikeThreshold
  strikeCount : Nat → Nat

def defaultAvailabilityLedgerState : AvailabilityLedgerState where
  strikeCount := fun _ => 0

def ledgerStrikeCount (L : AvailabilityLedgerState) (actor : Nat) : Nat :=
  L.strikeCount actor

/-- Record one ledger strike for `actor` (slash event). -/
def recordLedgerStrike (L : AvailabilityLedgerState) (actor : Nat) : AvailabilityLedgerState :=
  { L with
    strikeCount := fun a =>
      if a = actor then L.strikeCount a + 1 else L.strikeCount a }

/-- Slash on disclosed availability attack — observable must match CensorshipDisclosure. -/
def slashOnDisclosedAttack (L : AvailabilityLedgerState) (actor : Nat)
    (epoch subscriber witness f : Nat) (k : AvailabilityAttackKind)
    (_h : attackKindDisclosed k epoch subscriber witness f) : AvailabilityLedgerState :=
  recordLedgerStrike L actor

/-- Send rights revoked when strike count reaches threshold N. -/
def sendRightsRevoked (L : AvailabilityLedgerState) (actor : Nat) : Prop :=
  ledgerStrikeCount L actor ≥ L.strikeThreshold

/-- Pool epoch publish allowed only below strike threshold. -/
def poolPublishAllowed (L : AvailabilityLedgerState) (actor : Nat) : Prop :=
  ledgerStrikeCount L actor < L.strikeThreshold

theorem ledger_slash_implies_send_rights_revoked
    (L : AvailabilityLedgerState) (actor : Nat)
    (h : ledgerStrikeCount L actor ≥ L.strikeThreshold) :
    sendRightsRevoked L actor := h

theorem send_rights_revoked_forbids_pool_publish
    (L : AvailabilityLedgerState) (actor : Nat)
    (h : sendRightsRevoked L actor) :
    ¬ poolPublishAllowed L actor := by
  intro hp
  exact Nat.not_lt_of_ge h hp

theorem pool_publish_allowed_iff_not_revoked
    (L : AvailabilityLedgerState) (actor : Nat) :
    poolPublishAllowed L actor ↔ ¬ sendRightsRevoked L actor := by
  constructor
  · intro hp hrev
    exact Nat.not_lt_of_ge hrev hp
  · intro hnc
    exact Nat.lt_of_not_ge hnc

/-- Slash strictly increases strike count for the slashed actor. -/
theorem slash_on_attack_increments_strikes
    (L : AvailabilityLedgerState) (actor epoch subscriber witness f : Nat)
    (k : AvailabilityAttackKind)
    (h : attackKindDisclosed k epoch subscriber witness f) :
    ledgerStrikeCount (slashOnDisclosedAttack L actor epoch subscriber witness f k h) actor =
      ledgerStrikeCount L actor + 1 := by
  simp [slashOnDisclosedAttack, recordLedgerStrike, ledgerStrikeCount]

/-- After N disclosed attacks, send rights are revoked (fratagelse af afsendelsesret). -/
theorem slash_threshold_revokes_send_rights
    (L : AvailabilityLedgerState) (actor : Nat)
    (h : ledgerStrikeCount L actor ≥ L.strikeThreshold) :
    sendRightsRevoked L actor ∧ ¬ poolPublishAllowed L actor :=
  ⟨ledger_slash_implies_send_rights_revoked L actor h,
   send_rights_revoked_forbids_pool_publish L actor
     (ledger_slash_implies_send_rights_revoked L actor h)⟩

/-- Ledger enforcement bundle: slash ⇒ revoked ⇒ no pool publish. -/
def availabilityLedgerEnforcement : Prop :=
  ∀ L actor,
    sendRightsRevoked L actor → ¬ poolPublishAllowed L actor

theorem availability_ledger_enforcement : availabilityLedgerEnforcement :=
  fun L actor h => send_rights_revoked_forbids_pool_publish L actor h

/-- Operational ledger closed under default threshold. -/
def availabilityLedgerOperational : Prop :=
  availabilityLedgerEnforcement ∧
    defaultAvailabilityLedgerState.strikeThreshold = defaultAvailabilityStrikeThreshold

theorem availability_ledger_operational : availabilityLedgerOperational :=
  ⟨availability_ledger_enforcement, rfl⟩

/-- Availability attack observable under L3 stream (links to silent omit impossible). -/
def availabilityAttackObservable (k : AvailabilityAttackKind)
    (epoch subscriber witness f : Nat) : Prop :=
  attackKindDisclosed k epoch subscriber witness f ∧ l3StreamZeroLeak

theorem availability_attack_observable_from_l3
    (k : AvailabilityAttackKind) (epoch subscriber witness f : Nat)
    (hdis : attackKindDisclosed k epoch subscriber witness f)
    (hl3 : l3StreamZeroLeak) :
    availabilityAttackObservable k epoch subscriber witness f :=
  ⟨hdis, hl3⟩

/-- Full A bundle: censorship disclosure + ledger slash enforcement. -/
def aAbsoluteWithLedgerEnforcement : Prop :=
  aAbsolute ∧ availabilityLedgerOperational

theorem a_absolute_with_ledger_enforcement : aAbsoluteWithLedgerEnforcement :=
  ⟨a_absolute, availability_ledger_operational⟩

end ITS
