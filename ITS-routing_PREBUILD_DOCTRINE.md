# ITS-routing: Pre-Build Doctrine v2.0

**Status:** Active — UES Monocell Pool v2.0 operator ship.  
**Purpose:** Build profiles, pipe decoupling, completion/man gates, and container prebuild matrix before ecosystem-wide release.

See also: [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md) · [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) · [ITS_ADVANCED_RIDGES.md](ITS_ADVANCED_RIDGES.md)

Archived UDP/onion narrative: [docs/archive/ITS-routing_PREBUILD_DOCTRINE.md](docs/archive/ITS-routing_PREBUILD_DOCTRINE.md)

---

## 1. Build profiles

Default `its_routing` build is **transport only** — ridges are opt-in.

| Profile | Command | What you get |
|---------|---------|--------------|
| **Transport** (default) | `cargo build -p its_routing --release` | Pool, SSS, AEH core, client-send/receive |
| **Full daemon** | `cargo build -p its_routing --release --features full` | Transport + all operational ridges |
| **Custom** | `cargo build -p its_routing --release --features timelock,ledger` | Transport + selected ridges |

Config templates:

| Template | Use |
|----------|-----|
| `config.prod.toml` | Online mirrors + witnesses (`consensus_k = 2`) |
| `config.offline.toml` | Air-gap / sneakernet (`entropy_sources = []`) |
| `config.fast.toml` | Lab latency (`epoch_interval_ms = 50`, `consensus_k = 1`) |
| `config.robust.toml` | Production ITS-A + fountain |

Ecosystem verify uses `--features full` when testing the complete routing binary.

---

## 2. Pipe decoupling

Transport **must not** Cargo-depend on `its_asymmetric`. Math repos run as **subprocess / pipe**.

```bash
cargo tree -p its_routing 2>/dev/null | grep -E 'its_asymmetric|core_logic'  # must be empty
./scripts/verify_ecosystem.sh /path/to/ecosystem-root
```

Operator identity: **ITS-KeyManagement only** — routing never accepts vault passwords.

Stdio policy: [ITS_PIPE_STDIO_POLICY.md](ITS_PIPE_STDIO_POLICY.md).

---

## 3. Completions + man sync (gate M27)

| Check | Command |
|-------|---------|
| Drift gate | `./scripts/verify_cli_completions.sh` |
| Install (constitution + optional ridges) | `./scripts/install_completions.sh --all` |
| Wired in ecosystem verify | M27 section of `verify_ecosystem.sh` |

**Forbidden in prod completions/man:** ghost subcommands (`status-audit`, `verify-path`), `--mailbox-fingerprint` on `client-send` (receive-only flag).

---

## 4. Carrier safety (constitution)

| Rule | Gate |
|------|------|
| `--pool-dir` clears HTTP mirrors + AEH URLs | `pipe_its_km_pooldir_prod_hazard.sh` (M28b) |
| Offline base for sneakernet | `config.offline.toml` + [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) WARNING |
| Positive sneakernet | `pipe_its_km_sneakernet_e2e.sh` (M28) |

---

## 5. Container prebuild matrix

| Artifact | Location | Role |
|----------|----------|------|
| Per-repo Dockerfile | each repo root | Static musl / scratch operator image |
| Ecosystem compose | `deploy/docker/docker-compose.yml` | Local multi-service smoke |
| Build all | `deploy/docker/docker_build_all.sh` | Batch image build |
| Nix | `flake.nix` (ROUTING minimum) | Reproducible dev shell + `its-routing` package |
| Firecracker | `deploy/firecracker/README.md` | MicroVM threat model skeleton |

---

## 6. Pre-build checklist (10 gates)

Run from ecosystem root:

| # | Gate | Command |
|---|------|---------|
| 1 | Ecosystem verify | `./ROUTING/scripts/verify_ecosystem.sh /home/user` |
| 2 | Math (optional v10) | `VERIFY_MATH_V10=1 ./ROUTING/scripts/verify_ecosystem.sh /home/user` |
| 3 | No forbidden deps | `cargo tree -p its_routing` |
| 4 | Default build | `cargo build -p its_routing --release` |
| 5 | Full test | `cargo test -p its_transport -p its_routing --features full` |
| 6 | M27 completions | `./ROUTING/scripts/verify_cli_completions.sh` |
| 7 | M28 + M28b pipes | sneakernet + pooldir hazard scripts |
| 8 | Doctrine doc | This file linked from `ITS_ECOSYSTEM.md` |
| 9 | Constitution docs | QUICKSTART + ITS_CONSTITUTION_CLI aligned |
| 10 | Independent review | [ITS_INDEPENDENT_REVIEW_CHECKLIST.md](ITS_INDEPENDENT_REVIEW_CHECKLIST.md) human sign-off |

When all automated gates are green: ready for parallel build (Docker/Nix/Firecracker) and tagged ecosystem release.

---

## 7. Forbid in prod claims

| Path | Prod? |
|------|-------|
| AEH without `--ratchet-seed-file` | **No** — lab/demo only |
| `start-node` onion default | **Dev-only** |
| Raw `its-routing client-send` in operator quick starts | **No** — use `its-km` |
| `config.prod.toml` + `--pool-dir` without offline awareness | **Unsafe** — M28b must pass |
