# Censorship Recovery Playbook

Manual steps — **no auto-switch P↔AEH** (theorem requirement).

## Step 1: Multi-mirror + fountain

1. Add mirrors to `multi_pool_urls` in `routing.toml`.
2. Enable `fountain_enabled = true`.
3. Re-send; receiver uses `--continuous` with longer `--timeout-secs`.

**Gate:** `pipe_its_censorship_recovery_e2e.sh`

## Step 2: AEH last-resort (manual)

When all pool mirrors are blocked:

```bash
its-km send --contact bob --file doc.pdf --aeh
its-km receive --contact alice --aeh --out received.pdf
```

## Step 3: Sneakernet (total blackout)

When **no network path** exists (grid down, air-gap, physical exfil only):

```bash
cp ROUTING/config.offline.toml ~/.its/routing.toml
# Alice — write epoch cells to USB (KM orchestrates encrypt + pool publish)
its-km --true-secret ~/.its/km-vault-keys/true/secret.key send \
  --contact bob --file doc.pdf --pool-dir /media/usb/its-pool
# Physically deliver USB to Bob
its-km --true-secret ~/.its/km-vault-keys/true/secret.key receive \
  --contact alice --out received.pdf --pool-dir /media/usb/its-pool
```

Same routing logic as online — only `pool_file` / `--pool-dir` changes the carrier. See [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md).

**Why ITS beats I2P here:** I2P requires live overlay — **no delivery when net is down**. ITS keeps **same Shannon wire + OTM** on offline medium; C/I unchanged after secure-endpoint verify.

**Gate:** `pipe_its_km_sneakernet_e2e.sh` (M28) · routing unit: `pipe_its_sneakernet_e2e.sh`

## Claims

- Censorship affects **A (availability)** — not C/I in O when OTM verify runs on secure endpoint.
- Eve deleting cells ≠ deanonymization.
