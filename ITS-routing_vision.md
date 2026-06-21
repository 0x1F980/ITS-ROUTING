# ITS-routing: Network-Level Threat Model & Transition Strategy (ITS-routing_vision)

## License: GNU GPLv3 Only
## Target: Network Security Researchers, Cryptographic Auditors & Tactical Operations Teams

> **Scope:** [ITS-routing_SECURITY_LAYERS.md](ITS-routing_SECURITY_LAYERS.md).


This document details the network-level threat landscape and operational transitions managed by `ITS-routing`.

> **Production default (v2.0):** UES Monocell Pool + CoverTransport — see [QUICKSTART.md](QUICKSTART.md). Option A onion routing below is **dev-only** (`dev-onion-mix` feature).

## 1. The Network Threat Model

We assume that the network transport layer is inherently hostile. The network daemon (`its_routing`) is designed specifically to operate under a complete infrastructure compromise.

### 1. Global Passive Surveillance (Traffic Volume Analysis):
Even if packets are perfectly encrypted, an omnipotent adversary (Eve) who monitors all internet backbones can observe traffic volumes. If Alice sends a burst of data, and Bob receives a corresponding burst of data on the other side of the world, Eve can easily correlate their connection using volume/packet-count matching, destroying their anonymity.

### 2. Global Timing Correlation:
If packets transit through multiple onion routers, Eve can record the exact microsecond arrival and departure times at each hop. By computing cross-correlation functions over the packet streams, Eve can match the statistical signatures of the flow, tracing the entire route from Alice to Bob.

### 3. Active Packet Injection and Interception:
Eve owns the routing infrastructure. She can intercept, inject, modify, or delay packets to perform active routing attacks, trying to isolate specific nodes or force them to route through her compromised servers.

---

## 2. Structural Transport Mitigations

`ITS-routing` implements a multi-layered defense to neutralize these network-level vectors:

### Constant-Rate Chaffing (Dummy Injection):
Our network courier maintains a perfectly constant, invariant packet transmission rate. If there are no real SSS-shares in the queue, the daemon automatically generates and transmits cryptographically indistinguishable dummy packets ("chaff"). This converts the network stream into a flat, constant-rate profile, making traffic volume analysis completely blind.

### Lorenz Chaotic Jitter:
To prevent Eve from filtering dummy packets based on periodic timing analysis, the packet transmission intervals are randomized using a Lorenz chaotic system calculated over the finite field. Since chaotic trajectories are non-periodic and highly sensitive to initial conditions, Eve cannot perform statistical de-jittering.

---

## 3. Operational modes

**Production default (v2.0): UES Monocell Pool** — global fixed-size cells, CoverTransport harvest, optional `its-pool-proxy`. See [QUICKSTART.md](QUICKSTART.md). No active onion daemon required.

Legacy dev paths (require `dev-onion-mix` or `transport_mode = "dev"`):

### Option A: Active Onion Routing (dev-only — Concentrated Culpability)
*   **Tactical Profile:** Perfect cryptographic deniability. The user runs an active node, participating in the multi-hop onion routing mesh. Under physical coercion, **[ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement)** dual-password duress exports a decoy ratchet seed for `its-routing client-send --ratchet-seed-file` while showing a benign contact view.
*   **Operational Risk:** High behavioral visibility. Eve can easily detect that you are running an active routing daemon. While she cannot read the data, the mere act of participating is visible on your network profile.

### Option B: Ambient Entropy Harvesting (AEH) (Parasitic Diffused Culpability)
*   **Tactical Profile:** Complete behavioral invisibility. Alice and Bob close all incoming ports and shut down active routing nodes. Alice steganographically camouflages her SSS-shares inside massive, high-volume public web channels (Wikipedia, NASA, DNS TXT logs). Bob passively harvests these channels for algebraic echoes.
*   **Operational Risk:** Zero. Alice and Bob do not connect to each other. Their network traffic consists entirely of normal, outbound HTTPS/DNS requests to world-class public servers. Their culpability is completely diffused and absorbed by millions of ordinary web users.

---

## 4. The Absolute Necessity of Asynchronous Manual Transition

To prevent metadata leakage, **the transition between Option A and Option B must remain strictly asynchronous and manual.**

If the software attempted to automate this transition (e.g., via an "auto-switch" protocol), it would require an online coordination signal between Alice and Bob. In a network fully monitored by Eve, this signal would create an instant timing correlation: Eve would observe Alice and Bob's active nodes shutting down precisely as steganographic traffic emerges in public pools, completely destroying their anonymity.

By enforcing a manual, offline, or pre-scheduled transition (e.g., Alice and Bob agree beforehand to switch to stealth AEH-mode at midnight), **the physical hardware state and network connections are completely decoupled.** Eve is left with a dead UDP port on one side and millions of ordinary Wikipedia readers on the other, mathematically incapable of linking the two.
