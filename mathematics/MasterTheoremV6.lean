import MasterTheorem
import BroadcastIPDerivation
import CensorshipDisclosure
import RoleAwareDeniability
import OplusClosure
import EndpointSplit
import PublicPoolMulticast
import ForwardProof

/-!
# Master theorem v6/v8 — network ecosystem certificate (v7 absolutisme + v8 ITS-A)

\[
U_6 = U_5 \land A_{\text{abs}} \land \text{BIS}_{\text{derived}} \land \text{roleAwareDeniability}
\]
-/

namespace ITS

def bisFullyDerivedClosed : Prop :=
  b2DerivesFromL3Cell ∧
    b1DerivesFromL3PublicPool ∧
    b3DerivesFromZeroHopForward ∧
    broadcastIpSymmetryClosed bisFullyDerived

theorem bis_fully_derived_closed : bisFullyDerivedClosed :=
  ⟨b2_derives_from_l3_cell,
   b1_derives_from_l3_public_pool,
   b3_derives_from_zero_hop_forward,
   broadcast_ip_symmetry_closed bisFullyDerived⟩

def networkEcosystemCertificateV6 : Prop :=
  networkEcosystemCertificateV5 ∧
    aAbsolute ∧
    bisFullyDerivedClosed ∧
    roleAwareDeniability bisFullyDerived

theorem network_ecosystem_certificate_v6 : networkEcosystemCertificateV6 :=
  ⟨network_ecosystem_certificate_v5,
   a_absolute,
   bis_fully_derived_closed,
   role_aware_deniability bisFullyDerived⟩

/-- v7 — zero-stub participation + endpoint + BIS closure bundled. -/
def networkEcosystemCertificateV7 : Prop :=
  networkEcosystemCertificateV6 ∧
    participationPostulatesDerived ∧
    secureEndpointAxiom ∧
    publicPoolMulticastClosed

theorem network_ecosystem_certificate_v7 : networkEcosystemCertificateV7 :=
  ⟨network_ecosystem_certificate_v6,
   participation_postulates_derived,
   secure_endpoint_axiom,
   public_pool_multicast_closed⟩

/-- v8 — ITS availability via forward proof (ProofFwd + alternate mirror route). -/
def aItsForwardProofClosed : Prop :=
  availabilityITSForward ∧ publicPoolMulticastClosed

theorem a_its_forward_proof_closed : aItsForwardProofClosed :=
  ⟨availability_its_forward, public_pool_multicast_closed⟩

def networkEcosystemCertificateV8 : Prop :=
  networkEcosystemCertificateV7 ∧ aItsForwardProofClosed

theorem network_ecosystem_certificate_v8 : networkEcosystemCertificateV8 :=
  ⟨network_ecosystem_certificate_v7, a_its_forward_proof_closed⟩

end ITS
