# ITS-net: Command-Line Reference, Configurations & Operations (ITS-net_manual)

## License: GNU GPLv3 Only
## Target: Systems Developers, Incident Responders & Field Engineers

This document details the configuration formats, CLI commands, and deployment models managed by `ITS-net`.

---

## 1. Complete CLI Reference

`ITS-net` is executed via the unified binary `its-net` (formerly `morphic-its` / `hydra_cli`).

### Command 1: Start an Active Routing Node
Starts an active onion router node on a VPS or bare-metal host:
```bash
its-net start-node --config config.toml --port 8180 --chaff-rate 100
```

### Command 2: Single-Shot AEH Transmission
Dispatches a single authenticated, steganographically-camouflaged SSS share across public pools:
```bash
its-net client-send --msg "Secret Classified Message" --dest 3 --aeh --config config.toml
```

### Command 3: Continuous Decoy Chaffing Loop (Alice)
Starts a permanent background schedule loop, uploading mock blocks and substituting real blocks:
```bash
its-net client-send --msg "Secret Intelligence" --dest 3 --aeh --continuous --config config.toml
```

### Command 4: Continuous Winnowing Loop (Bob)
Runs Bob's receiver schedule, passively monitoring channels and verifying Wegman-Carter tags (via **`ITS-OTM_public_attestation`**):
```bash
its-net client-receive --aeh --continuous --config config.toml
```

Standalone public bundle verification (air-gapped audit):
```bash
its_otm verify --bundle share_attestation.otm
```
See [ITS-OTM_public_attestation_manual.md](https://github.com/0x1F464/ITS-OTM_public_attestation/blob/main/ITS-OTM_public_attestation_manual.md).

### Command 5: Duress / Password Protected Decryption
Unlocks the secure storage vault under coercion. If a duress password is used, the system decrypts harmless decoy records:
```bash
its-net client-vault-unlock --vault-path vault.bin --password duress_password
```

### Command 6: Analog Share Export & Import
*   **Export a share:**
    ```bash
    its-net client-export-share --msg "Secret" -k 3 -n 5
    ```
*   **Import a share:**
    ```bash
    its-net client-import-share --file shares.txt -k 3
    ```

### Command 7: Hybrid Time-Lock (Generate)
Wraps a local document in a hybrid SSS-Chained Time-Lock puzzle via `ITS-self_enclosed_timelock`:
```bash
its-net time-lock --file secret.pdf --epochs 1000 --out secret.its
```

### Command 8: Time-Unlock (Solve)
Sequentially solves the RSW96 squaring chain and decrypts the payload:
```bash
its-net time-unlock --puzzle secret.its --out secret.pdf
```

### Command 9: Time-Deny (Decoy Reconstruction)
Builds an alternative, mathematically consistent puzzle that decrypts to a decoy message:
```bash
its-net time-deny --puzzle secret.its --decoy "Cover story text" --out decoy.its
```

### Command 10: Provenance Erasure (Γ + optional OTP)
```bash
its-net fingerprint-erasure --file tainted.jpg --out clean.png --pad offline.pad --out-otp wire.bin
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

To prevent compiler-level Trojan injections (e.g., reflections or build-time code-modifications), `ITS-net` supports a fully hermetic build toolchain.

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
