# ITS-net: Command-Line Reference, Configurations & Operations (ITS-net_manual)

## License: GNU GPLv3 Only
## Target: Systems Developers, Incident Responders & Field Engineers

This document details the configuration formats, CLI commands, and deployment models managed by `ITS-net`.

---

## 1. Complete CLI Reference

`ITS-net` is executed via the unified binary `morphic-its` (formerly `hydra_cli`).

### Command 1: Start an Active Routing Node
Starts an active onion router node on a VPS or bare-metal host:
```bash
morphic-its start-node --config config.toml --port 8180 --chaff-rate 100
```

### Command 2: Single-Shot AEH Transmission
Dispatches a single authenticated, steganographically-camouflaged SSS share across public pools:
```bash
morphic-its client-send --msg "Secret Classified Message" --dest 3 --aeh --config config.toml
```

### Command 3: Continuous Decoy Chaffing Loop (Alice)
Starts a permanent background schedule loop, uploading mock blocks and substituting real blocks:
```bash
morphic-its client-send --msg "Secret Intelligence" --dest 3 --aeh --continuous --config config.toml
```

### Command 4: Continuous Winnowing Loop (Bob)
Runs Bob's receiver schedule, passively monitoring channels and verifying Wegman-Carter tags:
```bash
morphic-its client-receive --aeh --continuous --config config.toml
```

### Command 5: Duress / Password Protected Decryption
Unlocks the secure storage vault under coercion. If a duress password is used, the system decrypts harmless decoy records:
```bash
morphic-its client-vault-unlock --vault-path vault.bin --password duress_password
```

### Command 6: Analog Share Export & Import
*   **Export a share:**
    ```bash
    morphic-its client-export-share --id 1 --share-path share1.txt
    ```
*   **Import a share:**
    ```bash
    morphic-its client-import-share --share-path share1.txt
    ```

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
