# ITS-routing: Transport Recovery & Operational Procedures (ITS-routing_troubleshooting)

## License: GNU GPLv3 Only
## Target: Systems Developers, Incident Responders & Field Engineers

> **Scope:** [ITS-routing_SECURITY_LAYERS.md](ITS-routing_SECURITY_LAYERS.md).

This document details validation procedures and recovery steps for the `ITS-routing` transport layer.

---

## 1. Packet Loss Recovery in UDP Transport

Because `ITS-routing` utilizes raw UDP sockets to avoid the heavy state machines and connection handshakes of TCP (which leak metadata), packets can be lost in transit.

### SSS-Based Reconstruction Fallback:
*   Instead of relying on packet retransmissions (which create timing spikes that Eve can correlate), `ITS-routing` relies on **Shamir's Secret Sharing (SSS) Redundancy**.
*   When Alice fragments her message into $n$ shares with a threshold $t$ (e.g., $k=3, n=5$), Bob only needs to receive any $t$ shares to reconstruct the complete plaintext.
*   **The Safe Recovery:** If up to $n-t$ UDP packets are lost or dropped by the network, Bob recovers the message seamlessly with zero retransmissions. This mathematically defeats the latency leaks of TCP resends.

---

## 2. Configuration Drift & Client Resync

If the client's configuration or local key registry drifts out of synchronization with Bob, communication will fail.

### Symptom:
*   Sending SSS-shares results in the receiver failing to decrypt the payload:
    ```
    [ERROR] Decapsulation Failed: Tag validation failed.
    ```

### Step-by-Step Resolution:
1.  **Verify Shared Nonce State:** Ensure that both Alice and Bob are running on the same ratchet sequence block. Export matching seeds from **[ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement)** (`its-km export-ratchet-seed --contact <alias>`) and pass the same seed file to both sides via `--ratchet-seed-file`. Registry resync is an ITS-KeyManagement concern (future); routing does not expose `client-resync-registry`.
2.  **Check Configuration Section Headers:** Ensure your `config.toml` contains the `[aeh]` section (not legacy `[pep]` headers):
    ```toml
    [aeh]
    entropy_sources = [...]
    ```

---

## 3. Time-Lock CLI Recovery (`time-lock`, `time-unlock`, `time-deny`)

Offline time-lock operations use the external crate **`ITS-self_enclosed_timelock`**. See upstream [ITS-self_enclosed_timelock_troubleshooting.md](https://github.com/0x1F980/ITS-self_enclosed_timelock/blob/master/ITS-self_enclosed_timelock_troubleshooting.md) for crate-level errors.

### Symptom: Generation fails immediately
```
Error: Invalid parameters (empty file or epochs=0).
```
**Recovery:** Use a non-empty `--file` and `--epochs` ≥ 1.

### Symptom: Invalid puzzle file
```
Error: Invalid time-lock file format
```
**Recovery:** Verify `.its` text format (see [ITS-routing_manual.md](ITS-routing_manual.md) commands 7–9). Each epoch needs `transitions_1_block_N` and `transitions_2_block_N` lines.

### Symptom: Unlock returns garbage
**Recovery:** Confirm `--epochs` matches generation. Re-run `time-lock` with correct parameters.

### Duress: `time-deny` vs standalone `deny`
*   **`its-routing time-deny`** — builds a **decoy `.its` puzzle file** (coercion model; see timelock SECURITY_LAYERS §4).
*   **`its_timelock deny`** (standalone crate) — returns **decoy plaintext** from an alternative share offset; not a full decoy puzzle file.

---

## 4. Fingerprint-Erasure Strict Stack

If `client-send --file` fails with strict-stack errors, ensure `--fingerprint-erasure` is set and either `--fe-pad` or `[fingerprint_erasure].default_pad` in config points to a valid OTP pad file. See [ITS-routing_manual.md](ITS-routing_manual.md) and upstream ITS-FINGERPRINT_ERASURE docs.
