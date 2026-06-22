# ITS UES — 5-minute QUICKSTART

**Constitution flow:** seven essentials only — see [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md).

Send and receive with `its-km`. Switch online ↔ offline by changing `routing.toml` `[pool]` (or `--pool-dir`), not by learning new commands.

## Why Eve can't win (math, not trust)

Eve may own 99.999%+ of pool mirrors — that is axiom A0, not a failure mode. Under ITS v9:

| Pillar | What Eve gets | Your mitigation (config) |
|--------|---------------|--------------------------|
| **C** | 0 bits about message content in \(O\) | Shannon wire + L3 pool — no config needed |
| **I** | \(\leq 1\) false accept per \(2.147\times10^9\) forgery tries | OTM verify on **your** endpoint keys only |
| **A** | Cannot stay on \(\mathcal{M}_{\text{valid}}\) if she omits | `multi_pool_urls` + `witness_pool_urls` (online §2) |

**Numeric walkthrough:** [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) §Va.

---

## 1. Bootstrap

**One-command operator bundle (recommended):**

```bash
cd /path/to/ecosystem
./ROUTING/scripts/its-operator-install.sh
source ~/.its/env.sh
./ROUTING/scripts/its-onboard.sh
```

Manual build (same result):

```bash
cd /path/to/ecosystem
./ROUTING/scripts/bootstrap.sh
cargo build --release -p its_routing -p its_keymgmt --manifest-path ROUTING/Cargo.toml
cargo build --release --manifest-path ITS-asymmetric/Cargo.toml --bin its_asymmetric --features "bundle,parallel,std,compact-wire"
cargo build --release --manifest-path ITS-KeyManagement/Cargo.toml --bin its-km
```

Ensure `its-km`, `its-routing`, and `its_asymmetric` are on `PATH` (or `source ~/.its/env.sh` after operator install).

**Docker prebuilds v2:** `./ROUTING/deploy/docker/docker_build_all.sh` then `docker compose -f ROUTING/deploy/docker/docker-compose.yml up -d`

---

## Constitution steps (all profiles)

### Vault + contact (both peers)

```bash
its-km vault init --vault-key-dir ~/.its/km-vault-keys
its-km --true-secret ~/.its/km-vault-keys/true/secret.key entry add \
  --alias bob --public /path/to/bob.public.key --routing-config ~/.its/routing.toml
```

`entry add` auto-generates a **per-contact transport_ratchet** (32 bytes in vault). Sync OOB:

```bash
its-km export-qr --contact bob --layer transport-ratchet
# peer:
its-km import-qr --alias bob --layer transport-ratchet --payload 'its-km:qr:...'
```

If `~/.its/routing.toml` is missing, `entry add` copies `ROUTING/config.prod.toml` automatically.

### Send / receive

**Alice:**

```bash
its-km --true-secret ~/.its/km-vault-keys/true/secret.key send --contact bob --file doc.pdf
```

**Bob** (bob's keypair on the alice contact entry):

```bash
its-km --true-secret ~/.its/km-vault-keys/true/secret.key entry add \
  --alias alice --public /path/to/bob.public.key --secret /path/to/bob.secret.key \
  --routing-config ~/.its/routing.toml --transport-ratchet-file /path/to/shared-ratchet.seed
its-km --true-secret ~/.its/km-vault-keys/true/secret.key receive --contact alice --out received.pdf
```

---

## 2. Online profile (ITS-A)

```bash
mkdir -p ~/.its
cp ROUTING/config.prod.toml ~/.its/routing.toml
```

Edit `~/.its/routing.toml` — set `pool_url` or `multi_pool_urls` to your public mirror (see `ROUTING/deploy/pool-mirror/`).

**ITS-A (availability):** list mirrors + independent witnesses. Eve-only pools cannot satisfy ValidFwd if they omit — they are de-whitelisted automatically.

```toml
multi_pool_urls = [
  "http://mirror1:8787",
  "http://mirror2:8787",
]
witness_pool_urls = [
  "http://witness-charlie:8787",
  "http://witness2:8787",
  "http://witness3:8787",
]
consensus_k = 2
valid_fwd_window = 64
```

With \(k=2, n=3\): two witnesses must harvest the same \(c_e\) for `consensusAtEpoch`. You need **one** mirror in \(\mathcal{M}_{\text{valid}}\) for harvest — not a majority of \(10^9\) nodes.

Use the constitution send/receive commands above — no raw `its-routing` CLI in production.

---

## 3. Offline profile (sneakernet / air-gap)

```bash
cp ROUTING/config.offline.toml ~/.its/routing.toml
# or per-contact:
its-km entry add --alias bob --public bob.public.key \
  --routing-config ROUTING/config.offline.toml
```

`pool_file` points at a local directory where `epoch_*.bin` cells are written and read. SSS k-of-n tolerates one missing cell after physical handoff.

**USB / removable media:**

```bash
# Alice — write directly to USB pool path
its-km send --contact bob --file doc.pdf --pool-dir /media/usb/its-pool

# Copy USB to Bob; same routing params, same pool path on his mount
its-km receive --contact alice --out received.pdf --pool-dir /media/usb/its-pool
```

Or set `pool_file = "/media/usb/its-pool"` in `routing.toml` and omit `--pool-dir`.

**C/I unchanged** on offline medium; ITS-A witness requires network — offline A comes from redundant physical copies. See [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md) step 3.

---

## 4. Verify

```bash
ROUTING/scripts/verify_ecosystem.sh /home/user
ROUTING/scripts/pipe_its_socks_pool_e2e.sh   # M19 v2 SOCKS gate (optional)
```

**Proof manifests (green badges):**

| Doc | Role |
|-----|------|
| [PROOF_MANIFEST.md](PROOF_MANIFEST.md) | Lean + implementation certificate (M1–M26) |
| [REFINEMENT_MANIFEST.md](REFINEMENT_MANIFEST.md) | v10 refinement closure |

Signed releases: [RELEASE.md](RELEASE.md) · Independent review: [ITS_INDEPENDENT_REVIEW_CHECKLIST.md](ITS_INDEPENDENT_REVIEW_CHECKLIST.md)

---

## Optional: SOCKS proxy (production)

```bash
cargo build --release -p its_pool_proxy --manifest-path ROUTING/Cargo.toml
its-pool-proxy \
  --listen 127.0.0.1:1080 \
  --config ~/.its/routing.toml \
  --ratchet-seed-file ~/.its/shared-ratchet.seed \
  --pk ~/.its/contacts/bob/public.key \
  --sk ~/.its/keys/alice/secret.key \
  --own-pk ~/.its/keys/alice/public.key
```

Point apps at `SOCKS5 127.0.0.1:1080` (requires Bob receiver / ingress bridge). See [ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md).

---

## 5. Operator playbook — mirrors, witnesses, offline `--pool-dir`

| Step | Online (ITS-A) | Offline (sneakernet) |
|------|----------------|----------------------|
| Base config | `cp ROUTING/config.robust.toml ~/.its/routing.toml` (or `config.prod.toml`) | `cp ROUTING/config.offline.toml` |
| Pool carrier | Edit `multi_pool_urls` + `witness_pool_urls`; deploy [deploy/pool-mirror/](deploy/pool-mirror/) | `pool_file` or `--pool-dir /media/usb/its-pool` |
| Witness quorum | `consensus_k = 2` with 3 witnesses (2-of-3) | N/A — redundant USB copies |
| Latency tuning | See **When to use which profile** below | Same offline template |
| Safety | Never prod mirrors + `--pool-dir` alone — see [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) WARNING | M28b gate: `pipe_its_km_pooldir_prod_hazard.sh` |

### When to use which profile

| Template | When | Key settings |
|----------|------|--------------|
| [`config.prod.toml`](config.prod.toml) | Default online onboarding; empty URLs until fleet is configured | `consensus_k = 2`, commented mirror/witness placeholders |
| [`config.robust.toml`](config.robust.toml) | Production ITS-A + censorship recovery | Mirrors + witnesses + `fountain_enabled = true`, `valid_fwd_window = 64` |
| [`config.fast.toml`](config.fast.toml) | Lab / LAN latency tests only | `epoch_interval_ms = 50`, `consensus_k = 1`, `fountain_enabled = false` — **not** for prod witness quorum |
| [`config.offline.toml`](config.offline.toml) | Air-gap / USB sneakernet | File pool only; no HTTP mirrors |

Full ridge scope: [ITS_ADVANCED_RIDGES.md](ITS_ADVANCED_RIDGES.md) · Pipe policy: [ITS_PIPE_STDIO_POLICY.md](ITS_PIPE_STDIO_POLICY.md)

## Why better than I2P/Nym

| Doc | Use |
|-----|-----|
| [ITS_OVERLAY_SWITCH.md](ITS_OVERLAY_SWITCH.md) | Feature → ITS path → gate (one page) |
| [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md) | Why ITS over I2P/Nym + honest boundaries |
| [docs/ITS_DOMINANCE_PITCH.md](docs/ITS_DOMINANCE_PITCH.md) | 5-min pitch, Sybil FAQ, offline demo (M28) |
| [ITS_MIGRATION_GUIDES.md](ITS_MIGRATION_GUIDES.md) | Switch from I2P/Nym in one evening |
| [ITS_HIDDEN_SERVICE.md](ITS_HIDDEN_SERVICE.md) | Pairwise hidden service (Bob nginx + Alice SOCKS) |
| [ITS-routing_SUPERIORITY.md](ITS-routing_SUPERIORITY.md) | W1–W13 win-conditions table |
