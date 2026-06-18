# ITS-routing — formal verification (W6)

## License: GNU GPLv3 Only

---

## Build

```bash
cd its_routing && cargo test
```

Lean composition (archived upstream): [ItsNet/Composition.lean](https://github.com/0x1F464/ITS/tree/master/mathematics/ItsMath/ItsNet/Composition.lean) — see [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md)

Master tracker (archived): [VERIFICATION_STATUS.md](https://github.com/0x1F464/ITS/blob/master/mathematics/VERIFICATION_STATUS.md)

---

## Subcommand → proof map

| `its-routing` path | Upstream kernel | Status |
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
| [ITS-asymmetric](https://github.com/0x1F980/ITS-asymmetric) | Wire v6 static broadcast encrypt | `ITS-asymmetric_FORMAL_VERIFICATION.md` |
| [ITS-fingerprint_erasure](https://github.com/0x1F980/ITS-FINGERPRINT_ERASURE) | Γ extended / two-domain NF | `ITS-fingerprint_erasure_FORMAL_VERIFICATION.md` |
| [ITS-self_enclosed_timelock](https://github.com/0x1F980/ITS-self_enclosed_timelock) | Time-lock puzzles | upstream W4 |

**W6 status:** ITS-routing orchestration refines verified upstream kernels; no `sorry` in local CLI tests.

---

## Rust tests (this repo)

| Claim | Test / command |
|-------|----------------|
| CLI builds | `cargo build --release` in `its_routing/` |
| Unit tests | `cargo test` in `its_routing/` — analog SSS roundtrip, stdio paths |
| Timelock / OTM glue | `tests/timelock_integration.rs`, `tests/otm_verify_integration.rs` |
| FE strict stack | Proved in upstream `its_fingerprint_erasure`; invoked by `its-routing fingerprint-erasure` / send stack |

See also: [ITS-routing_SECURITY_LAYERS.md](ITS-routing_SECURITY_LAYERS.md), [ITS-routing_mathematics.md](ITS-routing_mathematics.md).
