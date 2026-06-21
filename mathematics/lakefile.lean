import Lake
open Lake DSL

require «asymmetric-math» from "../../ITS-asymmetric/mathematics"

package «routing-math» where
  version := v!"0.2.0"

@[default_target]
lean_lib «routing-math» where
  roots := #[
    `Transport,
    `AEH,
    `Refinement,
    `ObservationAlphabet,
    `IPObservation,
    `BroadcastIPSymmetry,
    `RecipientAttributionZero,
    `FlowAttributionZero,
    `EndpointEitherOr,
    `SSSMultiIPCourier,
    `PlausibleDeniabilityAbsolute,
    `Adversary,
    `EndpointSplit,
    `IntegrityAxiom,
    `BroadcastForward,
    `OplusClosure,
    `OfflineChannel,
    `AuthorAttributionZero,
    `UnattackableCertificate,
    `FewUserDoctrine,
    `UnifiedEpochStream,
    `LinkParticipation,
    `ParticipationTheorem,
    `PlausibleDeniability,
    `AvailabilityResilience,
    `MetadataSymmetry,
    `ParticipationSymmetry,
    `ComparativeThreatDoctrine,
    `CIA_Doctrine,
    `SybilDoctrine,
    `MathSupremacyDoctrine
  ]
