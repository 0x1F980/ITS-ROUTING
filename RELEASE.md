# ITS ROUTING — release process (constitution trio + proxy)

## License: GNU GPLv3 Only

**Scope:** Signed operator binaries for I2P/Nym switch path — `its-routing`, `its-km` (sibling repo), `its_asymmetric`, `its-pool-proxy`.

---

## Artifacts per release

| Binary | Crate / repo | Notes |
|--------|--------------|-------|
| `its-routing` | `its_routing` / ROUTING | transport + pool |
| `its-km` | ITS-KeyManagement | constitution CLI |
| `its_asymmetric` | ITS-asymmetric | wire encrypt/decrypt |
| `its-pool-proxy` | `its_pool_proxy` / ROUTING | SOCKS egress (M19 v2) |

Build:

```bash
./scripts/its-operator-install.sh /path/to/ecosystem
cargo build --release -p its_pool_proxy --manifest-path ROUTING/Cargo.toml
```

---

## Version tags (target)

| Repo | Tag pattern | Example |
|------|-------------|---------|
| ROUTING | `routing-v*` | `routing-v2.0.0` |
| ITS-KeyManagement | `km-v*` | `km-v2.0.0` |
| ITS-asymmetric | `v*` | `v0.10.0` |
| Meta | `ecosystem-v*` | `ecosystem-v2.0.0-complete` |

Align sibling tags before meta-tag. See [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md).

---

## Checksums + GitHub release (skeleton)

1. Tag each repo at matching release SHA.
2. Build release binaries on clean checkout (or CI matrix).
3. Publish SHA256 sums:

```bash
sha256sum its-routing its-km its_asymmetric its-pool-proxy > SHA256SUMS
gpg --detach-sign --armor SHA256SUMS
```

4. Create GitHub release with assets + `SHA256SUMS` + `SHA256SUMS.asc`.
5. Link release from [QUICKSTART.md](QUICKSTART.md) and [ITS_INDEPENDENT_REVIEW_CHECKLIST.md](ITS_INDEPENDENT_REVIEW_CHECKLIST.md).

---

## Pre-ship gates

```bash
./scripts/verify_math.sh
./scripts/verify_ecosystem.sh /path/to/ecosystem
./scripts/verify_cli_completions.sh
./scripts/pipe_its_socks_pool_e2e.sh   # M19 v2
```

**Z10 fresh-clone:** isolated bootstrap + full verify — operator checklist in independent review doc.

**Human sign-off:** independent reviewer confirms [PROOF_MANIFEST.md](PROOF_MANIFEST.md) v10 + [REFINEMENT_MANIFEST.md](REFINEMENT_MANIFEST.md).

---

## Operator install path

Preferred: `./scripts/its-operator-install.sh` → `source ~/.its/env.sh` → `./scripts/its-onboard.sh`.

Docker prebuilds: `./deploy/docker/docker_build_all.sh` + `docker compose -f deploy/docker/docker-compose.yml up -d`.
