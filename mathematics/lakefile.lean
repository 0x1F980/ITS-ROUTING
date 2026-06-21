import Lake
open Lake DSL

require «asymmetric-math» from "../../ITS-asymmetric/mathematics"
require «otm-math» from "../../ITS-OTM_public_attestation/mathematics"

package «routing-math» where
  version := v!"0.2.0"

@[default_target]
lean_lib «routing-math-cert» where
  roots := #[`UnattackableCertificate]

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
