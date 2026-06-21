import Lake
open Lake DSL

require «asymmetric-math» from "../../ITS-asymmetric/mathematics"
require «otm-math» from "../../ITS-OTM_public_attestation/mathematics"
require stl from "../../ITS-self_enclosed_timelock/mathematics/stl"

package «routing-math» where
  version := v!"0.2.0"

/-- All cert-path modules listed as roots so `lake build` works from clean. -/
@[default_target]
lean_lib «routing-math-cert» where
  roots := #[
    `Adversary,
    `AEH,
    `AEH.CovertChannel,
    `AEH.EpochGate,
    `AEH.StegoIndistinguishability,
    `AuthorAttributionZero,
    `AvailabilityLedger,
    `AvailabilityResilience,
    `BroadcastForward,
    `BroadcastIPDerivation,
    `BroadcastIPSymmetry,
    `CensorshipDisclosure,
    `CIA_Doctrine,
    `CoercionModel,
    `ComparativeThreatDoctrine,
    `EndpointEitherOr,
    `EndpointSplit,
    `FewUserDoctrine,
    `FlowAttributionZero,
    `ForwardProof,
    `ForwardReceiveGate,
    `IntegrityAxiom,
    `IPObservation,
    `LinkParticipation,
    `MasterTheorem,
    `MasterTheoremV6,
    `MathSupremacyDoctrine,
    `MediumIndependence,
    `MetadataSymmetry,
    `ObservationAlphabet,
    `OfflineChannel,
    `OplusClosure,
    `ParticipationSymmetry,
    `ParticipationTheorem,
    `PlausibleDeniability,
    `PlausibleDeniabilityAbsolute,
    `PublicPoolMulticast,
    `RecipientAttributionZero,
    `RoleAwareDeniability,
    `SSSMultiIPCourier,
    `SybilDoctrine,
    `TimelessSecurity,
    `Transport,
    `Transport.Basic,
    `Transport.Cell,
    `Transport.Epoch,
    `Transport.Field,
    `Transport.FiniteMutualInfo,
    `Transport.L9Composition,
    `Transport.RatchetDerivation,
    `Transport.TimelockComposition,
    `Transport.WireComposition,
    `UnattackableCertificate,
    `UnifiedEpochStream,
    `ValidForwardParty,
    `WitnessConsensus
  ]

lean_lib «routing-math-dev» where
  roots := #[
    `Transport.ChaffIndistinguishability,
    `Transport.MixAnonymity,
    `Transport.Composition
  ]

lean_lib «routing-math-refinement» where
  roots := #[
    `Refinement,
    `Refinement.EpochCellCorrectness
  ]
