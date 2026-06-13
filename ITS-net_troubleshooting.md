# ITS-net: Anomaly Detection, Anonymity Drifts & Recovery (ITS-net_troubleshooting)

## License: GNU GPLv3 Only
## Target: Systems Developers, Incident Responders & Field Engineers

> **Scope:** [ITS-net_SECURITY_LAYERS.md](ITS-net_SECURITY_LAYERS.md).


This document details the safeguards, validation procedures, and recovery steps for the `ITS-net` layer.

---

## 1. Active Traffic Anomaly Detection & Rerouting

In a multi-hop Morphic Routing Network, Eve may selectively drop packets, inject timing delays, or alter routes to perform correlation attacks (Traffic Analysis).

### The Detection Mechanism:
*   The `AnomalyDetector` (implemented in `hydra_cli/src/anomaly_detection.rs`) continuously tracks:
    1.  **Packet Delivery Ratio (PDR):** $PDR = \frac{\text{Received Packets}}{\text{Sent Packets}}$. If the PDR drops below 95%, Eve might be dropping packets to isolate a node.
    2.  **Delay Variance (Jitter):** If packet latency exhibits non-chaotic, predictable patterns, an adversary might be injecting artificial delays to match flows.

### Safe Fallbacks & Recovery:
*   When the anomaly threshold is crossed, the `SelfHealingRouter` immediately:
    1.  Tears down the active tunnel.
    2.  Logs a warning to the console:
        ```
        [WARNING] Anomaly Detected: Latency variance out of bounds. Route compromised.
        ```
    3.  Blacklists the compromised node indices in the registry.
    4.  Establishes a fresh, asymmetric, multi-hop path bypass route through uncompromised nodes.

---

## 2. Packet Loss Recovery in UDP Transport

Because `ITS-net` utilizes raw UDP sockets to avoid the heavy state machines and connection handshakes of TCP (which leak metadata), packets can be lost in transit.

### SSS-Based Reconstruction Fallback:
*   Instead of relying on packet retransmissions (which create timing spikes that Eve can correlate), `ITS-net` relies on **Shamir's Secret Sharing (SSS) Redundancy**.
*   When Alice fragments her message into $n$ shares with a threshold $t$ (e.g., $k=3, n=5$), Bob only needs to receive any $t$ shares to reconstruct the complete plaintext.
*   **The Safe Recovery:** If up to $n-t$ UDP packets are lost or dropped by the network, Bob recovers the message seamlessly with zero retransmissions. This mathematically defeats the latency leaks of TCP resends.

---

## 3. Configuration Drift & Client Resync

If the client's configuration or local key registry drifts out of synchronization with Bob, communication will fail.

### Symptom:
*   Sending SSS-shares results in the receiver failing to decrypt the payload:
    ```
    [ERROR] Decapsulation Failed: Tag validation failed.
    ```

### Step-by-Step Resolution:
1.  **Verify Shared Nonce State:** Ensure that both Alice and Bob are running on the same ratchet sequence block. If their local counters have drifted, execute the resync command:
    ```bash
    morphic-its client-resync-registry --peer 2 --token-offset 100
    ```
2.  **Check Configuration Section Headers:** Ensure your `config.toml` contains the updated `[aeh]` section instead of the deprecated biological `[pep]` headers:
    ```toml
    [aeh]
    entropy_sources = [...]
    ```

---

## 4. Time-Lock CLI Recovery (`time-lock`, `time-unlock`, `time-deny`)

Offline time-lock operations use the external crate **`ITS-self_enclosed_timelock`**. See upstream [ITS-self_enclosed_timelock_troubleshooting.md](https://github.com/0x1F464/ITS-self_enclosed_timelock/blob/master/ITS-self_enclosed_timelock_troubleshooting.md) for crate-level errors.

### Symptom: Generation fails immediately
```
Fejl: Ugyldige parametre (tom fil eller epochs=0).
```
**Recovery:** Use a non-empty `--file` and `--epochs` ≥ 1.

### Symptom: Invalid puzzle file
```
Fejl: Ugyldigt tidslås-filformat
```
**Recovery:** Verify `.its` text format (see [ITS-net_manual.md](ITS-net_manual.md) commands 7–9). Each epoch needs `transitions_1_block_N` and `transitions_2_block_N` lines.

### Symptom: Unlock fails after long CPU run
```
Fejl: Kunne ikke dekryptere tidslåsen (muligvis korrupt data).
```
**Recovery:** Do not hand-edit puzzle files. Regenerate with `time-lock`. Confirm `t` matches transition block count.

### Symptom: Decoy length warning on `time-deny`
**Recovery:** Pass `--decoy` with the same byte length as the original plaintext for predictable padding.

### Symptom: `cargo build` cannot fetch git dependencies
**Recovery:** Ensure [`.cargo/config.toml`](.cargo/config.toml) contains `git-fetch-with-cli = true` and SSH access to GitHub deploy keys.

---

## 5. OTM Public Attestation (`ITS-OTM_public_attestation`)

AEH and sneakernet OTM verification uses the external crate **`ITS-OTM_public_attestation`**. See upstream [ITS-OTM_public_attestation_troubleshooting.md](https://github.com/0x1F464/ITS-OTM_public_attestation/blob/main/ITS-OTM_public_attestation_troubleshooting.md).

### Symptom: Tag validation failed on receive
**Recovery:** Confirm ratchet counter matches `share_id`. For **public audit**, verify published `.otm` bundles with `its_otm verify --bundle FILE` (one-time keys must be included in the bundle).

### Symptom: Modulus mismatch between signer and verifier
**Recovery:** Build `its-net` and `ITS-OTM_public_attestation` with the same `m61` feature flag.
