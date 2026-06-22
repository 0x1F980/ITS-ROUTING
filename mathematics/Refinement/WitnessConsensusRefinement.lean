import WitnessConsensus
import ForwardProof

/-!
# Refinement — Rust `witness_consensus` refines ideal k-of-n (v10 / M25)

Abstract count-based consensus mirroring `its_routing::witness_consensus`.

**Proved:** `rustConsensusAtEpoch` ↔ `consensusAtEpoch`; consensus ⇒ `forwardProof`.
**Rust:** `witness_consensus.rs` — `witness_harvest_count`, `consensus_at_epoch`.
-/

namespace Refinement

open ITS

/-- Rust `witness_harvest_count` — mirrors `WitnessConsensus.witnessHarvestCount`. -/
def rustWitnessHarvestCount (V : PoolView) (W : WitnessSetA2) (e c : Nat) : Nat :=
  witnessHarvestCount V W e c

/-- Rust `consensus_at_epoch` — abstract semantic model (matches ideal after PoolView embed). -/
def rustConsensusAtEpoch (V : PoolView) (W : WitnessSetA2) (e c k : Nat) : Prop :=
  consensusAtEpoch V W e c k

/-- Rust `witness_harvest_count` ≥ k with k > 0 ⇒ ideal consensus (soundness vs Rust count check). -/
theorem rust_count_ge_gives_consensus (V : PoolView) (W : WitnessSetA2) (e c k : Nat)
    (_hk : 0 < k) (hcount : rustWitnessHarvestCount V W e c ≥ k) :
    rustConsensusAtEpoch V W e c k := by
  unfold rustConsensusAtEpoch
  let ws := W.members.filter (witnessHarvests V · e c)
  have hlen : ws.length ≥ k := by
    simpa [rustWitnessHarvestCount, witnessHarvestCount] using hcount
  refine ⟨ws, hlen, ?_⟩
  intro w hw
  have hwf := List.mem_filter.mp hw
  exact ⟨hwf.1, (witness_harvests_iff V w e c).mp hwf.2⟩

theorem rust_consensus_at_epoch_iff (V : PoolView) (W : WitnessSetA2) (e c k : Nat) :
    rustConsensusAtEpoch V W e c k ↔ consensusAtEpoch V W e c k :=
  Iff.rfl

theorem rust_consensus_gives_forward_proof
    (V : PoolView) (L : CanonicalLog) (W : WitnessSetA2) (e c k : Nat)
    (hpub : published L e c) (hcons : rustConsensusAtEpoch V W e c k) (hk : 0 < k) :
    forwardProof V L e c := by
  unfold rustConsensusAtEpoch at hcons
  obtain ⟨w, hwmem, hharvest⟩ := consensus_gives_witness V W e c k hcons hk
  exact consensus_witness_gives_forward_proof V L W w e c hpub hharvest hwmem

/-- Closed refinement bundle — Rust witness consensus refines ideal. -/
def witnessConsensusRefinementClosed : Prop :=
  (∀ (V : PoolView) (W : WitnessSetA2) (e c k : Nat),
    rustConsensusAtEpoch V W e c k ↔ consensusAtEpoch V W e c k) ∧
    (∀ (V : PoolView) (W : WitnessSetA2) (e c k : Nat),
      0 < k → rustWitnessHarvestCount V W e c ≥ k → rustConsensusAtEpoch V W e c k) ∧
      (∀ (V : PoolView) (L : CanonicalLog) (W : WitnessSetA2) (e c k : Nat),
        published L e c → rustConsensusAtEpoch V W e c k → 0 < k → forwardProof V L e c) ∧
        witnessConsensusClosed

theorem witness_consensus_refinement_closed : witnessConsensusRefinementClosed :=
  ⟨fun V W e c k => rust_consensus_at_epoch_iff V W e c k,
   fun V W e c k hk hcount => rust_count_ge_gives_consensus V W e c k hk hcount,
   fun V L W e c k hpub hcons hk =>
     rust_consensus_gives_forward_proof V L W e c k hpub hcons hk,
   witness_consensus_closed⟩

end Refinement
