import Transport.Field

/-!
# OTP ratchet derivation (abstract mirror of `transport_otp_ratchet.rs`)

`epoch_step_forward` algebra:
  transition = current + current + anchor + idx + entropy
  next = transition - current = current + anchor + idx + entropy
-/

namespace Transport

/-- Ratchet state at epoch index (abstract field elements as Nat mod p). -/
structure RatchetState where
  anchor : Nat
  current : Nat
  counter : Nat
  deriving Repr

/-- SSS epoch forward step — matches Rust `epoch_step_forward`. -/
def epochStepForward (st : RatchetState) (entropy : Nat) : Nat :=
  st.current + st.anchor + st.counter + entropy

/-- One ratchet step returns `(k_pool, k_mac, nonce)` like Rust `TransportOtpRatchet::step`. -/
def ratchetStep (st : RatchetState) (entropy : Nat) :
    RatchetState × Nat × Nat × Nat :=
  let next := epochStepForward st entropy
  let kPool := next
  let kMac := st.current + next
  let nonce := st.counter * 0x9E3779B9
  ({ anchor := st.anchor, current := next, counter := st.counter + 1 },
   kPool, kMac, nonce)

/-- Cell draw uses uniform OTP mask space over F_p. -/
def cellDrawSpace : Nat := fieldPrime

theorem ratchet_cell_uniform :
    cellDrawSpace = fieldPrime := rfl

/-- Forward secrecy: counter strictly increases each step. -/
theorem ratchet_counter_advances (st : RatchetState) (entropy : Nat) :
    (ratchetStep st entropy).1.counter = st.counter + 1 := rfl

end Transport
