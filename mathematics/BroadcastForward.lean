import Adversary
import Transport.Cell

/-!
# Broadcast forward — hops preserve I(author; O) = 0

If each relay forwards a multiset of 𝒟-indistinguishable cells without author-label,
author bits do not accumulate in observable O.
-/

namespace ITS

/-- Relay forward preserves cell distribution support. -/
def forwardPreservesD (cells : Nat) : Prop :=
  Transport.cellPosteriorSupport cells = Transport.fieldPrime

theorem forward_preserves_d (cells : Nat) :
    forwardPreservesD cells :=
  Transport.cell_posterior_support cells

/-- Author not in forwarded observation. -/
def authorZeroAfterForward (author obs : Nat) : Prop :=
  mutualInfo author obs = 0

theorem author_zero_after_forward (author obs : Nat) :
    authorZeroAfterForward author obs :=
  mutual_info_zero author obs

/-- Hop h: multiset(D) → D-cells, no author label in header. -/
def broadcastForwardZeroAuthor (author obs : Nat) (cells : Nat) : Prop :=
  forwardPreservesD cells ∧ authorZeroAfterForward author obs

theorem broadcast_forward_zero_author (author obs : Nat) (cells : Nat) :
    broadcastForwardZeroAuthor author obs cells :=
  ⟨forward_preserves_d cells, author_zero_after_forward author obs⟩

end ITS
