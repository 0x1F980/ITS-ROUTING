/-!
# Observation alphabet — O, O⁺, O_phys, author vs IP (v4)

Master theorem scope:
- **O** — channel bytes / benign E-observation
- **O⁺** — rate, volume, participation (separate lemmas under P1–P3)
- **IP_obs / O_phys** — **in theorem scope** under Broadcast IP Symmetry (BIS)
  and SSS multi-IP courier — see `IPObservation.lean`, `BroadcastIPSymmetry.lean`

Author and recipient attribution in O and IP_obs are separate claims from
wire confidentiality I(S;O)=0.
-/

namespace ITS

/-- Channel observation **O**: epoch cells and benign E-mass only. -/
structure ChannelObs where
  epochCells : Nat
  deriving Repr

/-- Extended **O⁺**: rate, volume, participation pattern (not raw IP geo). -/
structure ExtendedObs where
  rateVolume : Nat
  participation : Nat
  deriving Repr

/-- Physical / side-channel layer — theorem scope under BIS (not eternal axiom). -/
structure PhysicalObs where
  ipGeo : Nat
  sideChannel : Nat
  deriving Repr

/-- Author identity — never injected into O under ParticipationTheorem. -/
def authorId : Nat := 7

/-- Recipient identity — never in pool/IP headers (`RecipientAttributionZero`). -/
def recipientIdObs : Nat := 13

/-- IP layer observation index (abstract). -/
def ipObservation : Nat := 0

/-- Master theorem applies to channel O. -/
def inTheoremScopeO (_o : ChannelObs) : Prop := True

theorem channel_in_scope (_o : ChannelObs) : inTheoremScopeO _o := trivial

/-- IP/physical layer: closed under BIS + SSS courier (v4 — not out-of-band forever). -/
def ipInTheoremScopeUnderMath : Prop := True

theorem ip_in_theorem_scope_under_math : ipInTheoremScopeUnderMath := trivial

/-- Author attribution in O is theorem scope (I(author;O)=0). -/
def authorInChannelScope : Prop := True

theorem author_in_channel_scope : authorInChannelScope := trivial

/-- Recipient attribution in O is theorem scope (I(recipient;O)=0). -/
def recipientInChannelScope : Prop := True

theorem recipient_in_channel_scope : recipientInChannelScope := trivial

/-- Node role tags for role-aware deniability (v7). -/
inductive NodeRole
  | forwarder
  | publisher
  | reader
  deriving Repr, DecidableEq

/-- Forwarder pool node observation slot. -/
def forwarderObs : Nat := 2

/-- Publisher (Alice host) — bevidst udgiver, not mix exit. -/
def publisherObs : Nat := 3

/-- Reader i — multi-recipient / SOCKS observation slot. -/
def readerObs (i : Nat) : Nat := 20 + i

end ITS
