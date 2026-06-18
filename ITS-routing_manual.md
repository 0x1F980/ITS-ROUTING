# ITS-routing: Command-Line Reference, Configurations & Operations (ITS-routing_manual)

## License: GNU GPLv3 Only
## Target: Systems Developers, Incident Responders & Field Engineers

> **Scope:** [ITS-routing_SECURITY_LAYERS.md](ITS-routing_SECURITY_LAYERS.md) — subcommand → scope table in §2.


This document details the configuration formats, CLI commands, and deployment models managed by `ITS-routing`.

---

## 0. Pipe (stdin / stdout)

Use `-` for **stdin** or **stdout** on selected subcommands. Same syntax in **bash**, **zsh**, and **fish**.

```bash
echo -n "payload" | its-routing time-lock -f - -o - -e 30 | its-routing time-unlock -p - -o -
its-routing fingerprint-erasure --in - --out - < input.bin > normalized.bin
```

Full guide: [ITS-routing_PIPE.md](ITS-routing_PIPE.md). Demo: `scripts/pipe_timelock.sh`.

---

## 1. Complete CLI Reference

`ITS-routing` is executed via the unified binary `its-routing` (formerly `morphic-its` / `hydra_cli`).

### Command 1: Start an Active Routing Node
Starts an active onion router node on a VPS or bare-metal host:
```bash
its-routing start-node --config config.toml --port 8180 --chaff-rate 100
```

### Command 2: Single-Shot AEH Transmission
Dispatches a single authenticated, steganographically-camouflaged SSS share across public pools.

**Production:** export a 32-byte ratchet seed from [ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement), then pass it to routing (routing never accepts passwords):

```bash
its-km export-ratchet-seed --contact bob --out /tmp/seed.bin --password '...'
its-routing client-send --msg "Secret Classified Message" --dest 3 --aeh \
  --ratchet-seed-file /tmp/seed.bin --config config.toml
rm -f /tmp/seed.bin
```

**Lab demo only** (no seed file): anchor + whitening from `[crypto]` in config — non-production.

```bash
its-routing client-send --msg "Secret Classified Message" --dest 3 --aeh --config config.toml
```

Optional **Γ v3 Church-Rosser universal normalform** before send (off by default; **max security** when enabled — text/image/audio/PDF/code, DCT, lexicon, Δ=32/Δ=256 audio):

```bash
its-routing client-send --file tainted.jpg --dest 3 --fingerprint-erasure --config config.toml
its-routing client-send --file song.wav --dest 3 --fingerprint-erasure --fe-format wav --config config.toml
its-routing client-send --file doc.pdf --dest 3 --fingerprint-erasure --fe-format txt --config config.toml
its-routing client-send --file main.rs --dest 3 --fingerprint-erasure --fe-format code --config config.toml
```

Balanced (v1-like) or custom spectral params:

```bash
its-routing client-send --file tainted.jpg --dest 3 --fingerprint-erasure --fe-mode balanced --config config.toml
its-routing client-send --file note.txt --dest 3 --fingerprint-erasure --fe-lexicon da-en --config config.toml
```

Or Γ + OTP wire on the network path (Bob uses `its_fe otp-unmask` with the same pad):
```bash
its-routing client-send --file tainted.jpg --dest 3 --fingerprint-erasure --fe-pad offline.pad --config config.toml
```
Flags: `--fingerprint-erasure` / `--gamma`, `--fe-strict` / `--strict`, `--fe-strict-stack` / `--strict-stack` (legacy: `--fe-uangribelig`), `--fe-permissive` / `--permissive` (v5 escape), `--fe-domain` / `--domain discrete|continuous`, `--fe-kind` / `--kind text|image|audio|pdf|code`, `--fe-mode` / `--mode standard|extended|minimal` (aliases: max, annihilator, balanced), `--fe-lexicon` / `--lexicon`, `--fe-delta` / `--delta`, `--fe-format` / `--format auto|sem1|png|txt|bin|wav|code`, `--fe-pad` / `--pad`, `--fe-dct-q`, `--fe-sigma-delta`, `--fe-lab-delta-ab`.

**v0.8 default:** `--fingerprint-erasure` alone enables strict stack + requires `--fe-pad` and chaff in config. Use `--fe-permissive` for v5 permissive Γ.

**Strict stack** (strict policy + extended mode + OTP + chaff):

```bash
its-routing client-send --file song.wav --dest 3 --fingerprint-erasure \
  --fe-strict-stack --fe-kind audio --fe-domain continuous \
  --fe-pad offline.pad --config config.toml
```

Post-save automatic Γ (v0.8):

```bash
its_fe watch --dir ~/Documents --in-place --strict-stack
```

**Strict mode example** (explicit kind, Raw denied):

```bash
its-routing client-send --file song.wav --dest 3 --fingerprint-erasure --fe-strict --fe-kind audio --config config.toml
```

### Command 3: Continuous Decoy Chaffing Loop (Alice)
Starts a permanent background schedule loop, uploading mock blocks and substituting real blocks:
```bash
its-routing client-send --msg "Secret Intelligence" --dest 3 --aeh --continuous --config config.toml
```

### Command 4: Continuous Winnowing Loop (Bob)
Runs Bob's receiver schedule, passively monitoring channels and verifying Wegman-Carter tags (via **`ITS-OTM_public_attestation`**):

```bash
its-km export-ratchet-seed --contact bob --out /tmp/seed.bin --password '...'
its-routing client-receive --aeh --ratchet-seed-file /tmp/seed.bin --config config.toml
rm -f /tmp/seed.bin
```

Continuous mode:

```bash
its-routing client-receive --aeh --continuous --config config.toml
```

Standalone public bundle verification (air-gapped audit):
```bash
its_otm verify --bundle share_attestation.otm
```
See [ITS-OTM_public_attestation_manual.md](https://github.com/0x1F980/ITS-OTM_public_attestation/blob/main/ITS-OTM_public_attestation_manual.md).

### Command 5: Operator vault & duress (ITS-KeyManagement)

Vault passwords, contact registry, and dual-password duress views are **not** ITS-routing commands. Use **[ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement)**:

```bash
its-km vault init --vault ~/.its/km.vault
its-km vault unlock --password '...'          # full contact view
its-km vault unlock --password '...' --duress # decoy contact view
its-km contact add --alias bob --routing-dest 3 --routing-config ./config.toml
its-km export-ratchet-seed --contact bob --out /tmp/seed.bin
```

See [ITS-KeyManagement_manual.md](https://github.com/0x1F980/ITS-KeyManagement/blob/main/ITS-KeyManagement_manual.md).

### Command 6: Analog Share Export & Import
*   **Export a share:**
    ```bash
    its-routing client-export-share --msg "Secret" -k 3 -n 5
    ```
*   **Import a share:**
    ```bash
    its-routing client-import-share --file shares.txt -k 3
    ```

### Command 7: Hybrid Time-Lock (Generate)
Wraps a local document in a hybrid SSS-Chained Time-Lock puzzle via `ITS-self_enclosed_timelock`:
```bash
its-routing time-lock --file secret.pdf --epochs 1000 --out secret.its
```

### Command 8: Time-Unlock (Solve)
Sequentially solves the RSW96 squaring chain and decrypts the payload:
```bash
its-routing time-unlock --puzzle secret.its --out secret.pdf
```

### Command 9: Time-Deny (Decoy Reconstruction)
Builds an alternative, mathematically consistent puzzle that decrypts to a decoy message:
```bash
its-routing time-deny --puzzle secret.its --decoy "Cover story text" --out decoy.its
```

### Command 10: Provenance Erasure (Γ + optional OTP)
**Inline send (optional):** add `--fingerprint-erasure` to `client-send --file` (see Command 2).

**Standalone offline files:**
```bash
its-routing fingerprint-erasure --file tainted.jpg --out clean.png --pad offline.pad --out-otp wire.bin
its_fe otp-unmask --in wire.bin --pad offline.pad --out clean.png   # Bob
```
Flags: `--delta`, `--format`, `--pad`, `--out-otp`.

---

## 2. Configuration File Syntax (`config.toml`)

Configuration files are parsed using our high-assurance, macro-free, reflection-free custom TOML parser.

```toml
[traffic]
# Active routing and dummy traffic parameters
tick_rate_ms = 100
chaff_rate_pps = 10
payload_size_elements = 16

[aeh]
# Ambient Entropy Harvesting sources (simulated public telemetry and block headers)
entropy_sources = [
    "https://api.nasa.gov/planetary/apod",
    "https://blockchain.info/q/latesthash"
]
clue_offset = 12
```

---

## 3. Hermetic Toolchain and Reproducible Builds

To prevent compiler-level Trojan injections (e.g., reflections or build-time code-modifications), `ITS-routing` supports a fully hermetic build toolchain.

### Nix Environment (`shell.nix`):
The repository contains a declarative Nix expression pinning the exact version of the Rust compiler and system libraries, ensuring absolute reproducibility across build servers:
```nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = [
    pkgs.rustc
    pkgs.cargo
    pkgs.git
  ];
}
```

### Docker Compilation:
To execute a deterministic build within a sterile container:
```bash
docker build -t morphic-its-builder .
```
This guarantees that the generated binary's cryptographic hashes are 100% identical regardless of the host system's OS or localized environment variables.
