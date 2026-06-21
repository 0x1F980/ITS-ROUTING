import Lake
open Lake DSL

require «asymmetric-math» from "../../ITS-asymmetric/mathematics"
require «otm-math» from "../../ITS-OTM_public_attestation/mathematics"
require stl from "../../ITS-self_enclosed_timelock/mathematics/stl"

package «routing-math» where
  version := v!"0.2.0"

@[default_target]
lean_lib «routing-math-cert» where
  roots := #[
    `UnattackableCertificate,
    `BroadcastIPDerivation,
    `PublicPoolMulticast,
    `CensorshipDisclosure,
    `RoleAwareDeniability,
    `TimelessSecurity,
    `MediumIndependence,
    `CoercionModel,
    `Transport.TimelockComposition,
    `MasterTheorem,
    `MasterTheoremV6
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
