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
# Pattern from pipe_its_sneakernet_e2e.sh — export epoch cells to removable media
its-routing -c ~/.its/routing.toml client-send --pool --file msg.wire --ratchet-seed-file ratchet.seed
# Copy pool export / analog shares to USB; physically deliver to Bob
its-routing -c ~/.its/routing.toml client-receive --pool --file recv.wire --ratchet-seed-file ratchet.seed
its_asymmetric decrypt --sk bob.secret.key --pk bob.public.key --in recv.wire --out received.txt
```

**Why ITS beats I2P here:** I2P requires live overlay — **no delivery when net is down**. ITS keeps **same Shannon wire + OTM** on offline medium; C/I unchanged after secure-endpoint verify.

**Gate:** `pipe_its_sneakernet_e2e.sh`

## Claims

- Censorship affects **A (availability)** — not C/I in O when OTM verify runs on secure endpoint.
- Eve deleting cells ≠ deanonymization.
