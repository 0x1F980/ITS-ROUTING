# ITS-net — formal verification (W6)

## License: GNU GPLv3 Only

---

## Build

```bash
cd its_net_cli && cargo test
```

Lean composition (upstream ITS repo): [ItsNet/Composition.lean](https://github.com/0x1F464/ITS/tree/master/mathematics/ItsMath/ItsNet/Composition.lean)

Master tracker: [VERIFICATION_STATUS.md](https://github.com/0x1F464/ITS/tree/master/mathematics/VERIFICATION_STATUS.md)

---

## Subcommand → proof map

| `its-net` path | Upstream kernel | Status |
|----------------|-----------------|--------|
| strict stack send (Γ + OTP + chaff) | `composed_send_roundtrip` | **Proved** |
| `--fingerprint-erasure` | `its_fingerprint_erasure` strict stack | **Proved (Rust)** |
| OTM verify | W3 `rust_otm_refines_ideal` | **Proved** |
| time-lock / unlock / deny | timelock `stl/` W4.3 | **Proved** |
| time-deny OTP layer | `time_deny_otp_layer` | **Proved** |

---

## Related ITS crates (this workspace)

| Crate | Role | Verification doc |
|-------|------|------------------|
| [ITS-assymetric](https://github.com/0x1F464/ITS-assymetric) | Wire v6 static broadcast encrypt | `ITS-assymetric_FORMAL_VERIFICATION.md` |
| [ITS-fingerprint_erasure](https://github.com/0x1F464/ITS-FINGERPRINT_ERASURE) | Γ extended / two-domain NF | `ITS-fingerprint_erasure_FORMAL_VERIFICATION.md` |
| [ITS-self_enclosed_timelock](https://github.com/0x1F464/ITS-self_enclosed_timelock) | Time-lock puzzles | upstream W4 |

**W6 status:** ITS-net orchestration refines verified upstream kernels; no `sorry` in local CLI tests.

---

## Rust tests (this repo)

| Claim | Test / command |
|-------|----------------|
| CLI builds | `cargo build --release` in `its_net_cli/` |
| Unit tests | `cargo test` in `its_net_cli/` |
| FE strict stack integration | `--fingerprint-erasure` path (see `ITS-net_SECURITY_LAYERS.md`) |

See also: [ITS-net_SECURITY_LAYERS.md](ITS-net_SECURITY_LAYERS.md), [ITS-net_mathematics.md](ITS-net_mathematics.md).
