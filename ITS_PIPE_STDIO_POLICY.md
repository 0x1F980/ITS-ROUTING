# ITS Pipe / Stdio Policy (`-` stdin and stdout)

## License: GNU GPLv3 Only

**Constitution default:** file paths on `its-km send/receive`. This document lists **which binaries** support POSIX `-` for stdin/stdout and deliberate exceptions.

---

## Summary table

| CLI | `-` stdin | `-` stdout | Notes |
|-----|-----------|------------|-------|
| `its_asymmetric` | Yes | Yes | Wire seal, encrypt/decrypt pipes |
| `its-routing time-lock` | Yes (`-f`) | Yes (`-o`) | Requires `--features timelock` |
| `its-routing time-unlock` | Yes (`-p`) | Yes (`-o`) | Puzzle path |
| `its-routing fingerprint-erasure` | Yes | Yes | Requires `--features fingerprint-erasure` |
| `its_otm` | Yes | Yes | Attestation pipe workflows |
| `sss_chain` | Yes | Yes | Share algebra CLI |
| `its_fe` | Yes | Yes | Standalone Γ |
| `its_timelock` | Yes | Yes | Standalone puzzle repo |
| `its_ledger` | **No** | **No** | File paths + registry only |
| **`its-km send/receive`** | **No** | **No** | **By design** — vault paths, work-dir bundles, `--pool-dir` |

---

## Rationale for `its-km` file-path only

KeyManagement orchestrates multi-step pipes (asymmetric seal → routing pool → decrypt). Work directories, vault unlock, and `--pool-dir` overrides assume **named files** for audit and recovery. Use `its_asymmetric` + `its-routing` directly if you need shell pipes without KM.

---

## Examples (constitution-adjacent)

```bash
# asymmetric wire pipe
echo -n payload | its_asymmetric encrypt --pk bob.public.key --in - --out msg.wire

# routing timelock pipe (feature build)
echo -n doc | its-routing time-lock -f - -o puzzle.its -e 1000
```

Full routing pipe guide: [ITS-routing_PIPE.md](ITS-routing_PIPE.md).

---

## Verification

Ecosystem gate M27 checks CLI/help drift; pipe behavior is documented here and in per-repo manuals — not every pipe is exercised in CI.

Cross-link: [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) · [ITS_ADVANCED_RIDGES.md](ITS_ADVANCED_RIDGES.md)
