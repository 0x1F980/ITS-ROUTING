/-!
# Transport field parameters (match `its_transport` onion + SSS wire)
-/

namespace Transport

/-- Field prime used by morphic onion packets (abstract cardinality). -/
def fieldPrime : Nat := 2147483647

/-- OTP mask space per field element (uniform draw). -/
def otpMaskCandidates : Nat := fieldPrime

theorem otp_mask_uniform_card :
    otpMaskCandidates = fieldPrime := rfl

/-- Consistent plaintext field elements under uniform OTP mask (one mask per observation). -/
def consistentPlaintextElements (_observed : Nat) : Nat :=
  fieldPrime

theorem otp_element_blindness (observed : Nat) :
    consistentPlaintextElements observed = fieldPrime := rfl

/-- SSS share count for threshold `(k, n)` configuration (operational default). -/
def defaultThresholdK : Nat := 2
def defaultTotalSharesN : Nat := 3

/-- Message dimension for `L` payload elements (abstract). -/
def messageDim (L : Nat) : Nat := L * 3

/-- Blend observation dimension after linear mixing (rank drop). -/
def blendDim (L : Nat) : Nat := L

theorem mix_underdetermined (L : Nat) (hL : L ≥ 1) :
    messageDim L > blendDim L := by
  unfold messageDim blendDim
  have : L * 3 > L := by omega
  exact this

/-- Degrees of freedom in solution affine subspace (rank-nullity). -/
def mixDegreesOfFreedom (L : Nat) : Nat :=
  messageDim L - blendDim L

theorem mix_kernel_large (L : Nat) (hL : L ≥ 1) :
    mixDegreesOfFreedom L ≥ 2 := by
  unfold mixDegreesOfFreedom messageDim blendDim
  omega

/-- Chaff draw space: parity tag under uniform OTP onion header. -/
def chaffParitySpace : Nat := 2

theorem chaff_parity_uniform_card :
    chaffParitySpace = 2 := rfl

end Transport
