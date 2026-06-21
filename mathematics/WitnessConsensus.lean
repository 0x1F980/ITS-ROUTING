import ForwardProof
import EndpointSplit

/-!
# Witness consensus — k-of-n A2′ harvest agreement (v9 ITS-A)

Witness set \(\mathcal{W}_{A2'}\) tied to `SecureVerifyOracle` (Charlie).
\[
\text{consensusAtEpoch}(e,c,k) \Leftrightarrow
  \geq k \text{ witnesses in } \mathcal{W}_{A2'} \text{ harvest } c \text{ at } e
\]

Selective omit to `s` + witness consensus ⇒ `forwardProof` + `alternateRoute`.
-/

namespace ITS

/-- A2′ witness set — members run math-trusted verify-oracle (e.g. Charlie). -/
structure WitnessSetA2 where
  members : List Nat
  verifyOracle : SecureVerifyOracle := defaultVerifyOracle
  deriving Repr

def defaultWitnessSetA2 : WitnessSetA2 := { members := [] }

def witnessHarvests (V : PoolView) (w e c : Nat) : Bool :=
  match V.harvest w e with
  | some c' => c' == c
  | none => false

theorem witness_harvests_iff (V : PoolView) (w e c : Nat) :
    witnessHarvests V w e c = true ↔ V.harvest w e = some c := by
  simp [witnessHarvests]
  cases V.harvest w e <;> simp [beq_iff_eq]

/-- k-of-n witness consensus at epoch `e` for cell `c`. -/
def consensusAtEpoch (V : PoolView) (W : WitnessSetA2) (e c k : Nat) : Prop :=
  ∃ ws : List Nat, ws.length ≥ k ∧
    ∀ w, w ∈ ws → w ∈ W.members ∧ V.harvest w e = some c

/-- Count witnesses in `W` that harvest cell `c` at epoch `e` (refinement link). -/
def witnessHarvestCount (V : PoolView) (W : WitnessSetA2) (e c : Nat) : Nat :=
  (W.members.filter (witnessHarvests V · e c)).length

theorem consensus_gives_witness
    (V : PoolView) (W : WitnessSetA2) (e c k : Nat)
    (hcons : consensusAtEpoch V W e c k) (hk : 0 < k) :
    ∃ w, w ∈ W.members ∧ V.harvest w e = some c := by
  obtain ⟨ws, hlen, hmem⟩ := hcons
  cases ws with
  | nil =>
      rw [List.length_nil] at hlen
      have hk0 : k = 0 := Nat.le_zero.mp hlen
      subst hk0
      exact absurd hk (Nat.not_lt_zero 0)
  | cons w ws' =>
      exact ⟨w, (hmem w (List.mem_cons_self w ws')).1, (hmem w (List.mem_cons_self w ws')).2⟩

theorem consensus_witness_gives_forward_proof
    (V : PoolView) (L : CanonicalLog) (W : WitnessSetA2) (w e c : Nat)
    (hpub : published L e c)
    (hharvest : V.harvest w e = some c)
    (_hw : w ∈ W.members) :
    forwardProof V L e c :=
  ⟨hpub, w, hharvest⟩

/-- Selective omit + k-of-n consensus ⇒ alternate route + forward proof. -/
theorem selective_omit_consensus_gives_alternate_route
    (V : PoolView) (L : CanonicalLog) (W : WitnessSetA2) (s e c k : Nat)
    (hpub : published L e c)
    (hmiss : V.harvest s e ≠ some c)
    (hcons : consensusAtEpoch V W e c k)
    (hk : 0 < k) :
    ∃ w, alternateRoute V L s w e c ∧ forwardProof V L e c := by
  obtain ⟨w, hwmem, hwitness⟩ := consensus_gives_witness V W e c k hcons hk
  exact ⟨w, selective_omit_witness_gives_alternate_route V L s w e c hpub hmiss hwitness⟩

theorem selective_omit_consensus_implies_forward_proof
    (V : PoolView) (L : CanonicalLog) (W : WitnessSetA2) (s e c k : Nat)
    (hpub : published L e c)
    (hmiss : V.harvest s e ≠ some c)
    (hcons : consensusAtEpoch V W e c k)
    (hk : 0 < k) :
    forwardProof V L e c := by
  obtain ⟨_, h⟩ := selective_omit_consensus_gives_alternate_route V L W s e c k hpub hmiss hcons hk
  exact h.2

/-- Witness on A2′ verify-oracle certifies consensus harvest. -/
def witnessConsensusCert (V : PoolView) (L : CanonicalLog) (W : WitnessSetA2)
    (e c k : Nat) (_ver : SecureVerifyOracle)
    (hpub : published L e c)
    (hcons : consensusAtEpoch V W e c k) (hk : 0 < k) : Prop :=
  forwardProof V L e c

theorem witness_consensus_cert
    (V : PoolView) (L : CanonicalLog) (W : WitnessSetA2) (e c k : Nat)
    (_ver : SecureVerifyOracle) (hpub : published L e c)
    (hcons : consensusAtEpoch V W e c k) (hk : 0 < k) :
    witnessConsensusCert V L W e c k _ver hpub hcons hk := by
  obtain ⟨w, _, hwitness⟩ := consensus_gives_witness V W e c k hcons hk
  exact ⟨hpub, w, hwitness⟩

/-- Closed bundle: consensus ⇒ ∃ witness harvest + forward proof. -/
def witnessConsensusClosed : Prop :=
  ∀ (V : PoolView) (L : CanonicalLog) (W : WitnessSetA2) (s e c k : Nat),
    published L e c →
      V.harvest s e ≠ some c →
        consensusAtEpoch V W e c k →
          0 < k →
            (∃ w, alternateRoute V L s w e c) ∧ forwardProof V L e c

theorem witness_consensus_closed : witnessConsensusClosed :=
  fun V L W s e c k hpub hmiss hcons hk => by
    obtain ⟨w, h⟩ := selective_omit_consensus_gives_alternate_route V L W s e c k hpub hmiss hcons hk
    exact And.intro ⟨w, h.1⟩ h.2

end ITS
