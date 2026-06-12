# ITS-net: Network-Level Threat Model & Transition Strategy (ITS-net_vision)

## License: GNU GPLv3 Only
## Target: Network Security Researchers, Cryptographic Auditors & Tactical Operations Teams

This document details the network-level threat landscape and operational transitions managed by `ITS-net`.

---

## 1. The Network Threat Model

We assume that the network transport layer is inherently hostile. The network daemon (`its_net_cli`) is designed specifically to operate under a complete infrastructure compromise.

### 1. Global Passive Surveillance (Traffic Volume Analysis):
Even if packets are perfectly encrypted, an omnipotent adversary (Eve) who monitors all internet backbones can observe traffic volumes. If Alice sends a burst of data, and Bob receives a corresponding burst of data on the other side of the world, Eve can easily correlate their connection using volume/packet-count matching, destroying their anonymity.

### 2. Global Timing Correlation:
If packets transit through multiple onion routers, Eve can record the exact microsecond arrival and departure times at each hop. By computing cross-correlation functions over the packet streams, Eve can match the statistical signatures of the flow, tracing the entire route from Alice to Bob.

### 3. Active Packet Injection and Interception:
Eve owns the routing infrastructure. She can intercept, inject, modify, or delay packets to perform active routing attacks, trying to isolate specific nodes or force them to route through her compromised servers.

---

## 2. Structural Transport Mitigations

`ITS-net` implements a multi-layered defense to neutralize these network-level vectors:

### Constant-Rate Chaffing (Dummy Injection):
Our network courier maintains a perfectly constant, invariant packet transmission rate. If there are no real SSS-shares in the queue, the daemon automatically generates and transmits cryptographically indistinguishable dummy packets ("chaff"). This converts the network stream into a flat, constant-rate profile, making traffic volume analysis completely blind.

### Lorenz Chaotic Jitter:
To prevent Eve from filtering dummy packets based on periodic timing analysis, the packet transmission intervals are randomized using a Lorenz chaotic system calculated over the finite field. Since chaotic trajectories are non-periodic and highly sensitive to initial conditions, Eve cannot perform statistical de-jittering.

### Active Anomaly Detection:
The daemon continuously monitors incoming traffic patterns, round-trip times, and packet drop rates. If it detects any statistical anomaly (e.g., Eve attempting to perform timing correlation or selective packet dropping), the self-healing router immediately terminates the compromised tunnel and dynamically establishes a fresh, asymmetric path.

---

## 3. The Active vs. Passive Operational Transition

`ITS-net` offers a tactical choice between two operational modes, depending on the required level of anonymity:

### Option A: Active Onion Routing (Concentrated Culpability)
*   **Tactical Profile:** Perfect cryptographic deniability. The user runs an active node, participating in the multi-hop onion routing mesh. If physically coerced, the **Dual-Seed Duress Ratchet** allows Bob to safely reveal a decoy password, unlocking harmless recipes.
*   **Operational Risk:** High behavioral visibility. Eve can easily detect that you are running an active routing daemon. While she cannot read the data, the mere act of participating is visible on your network profile.

### Option B: Ambient Entropy Harvesting (AEH) (Parasitic Diffused Culpability)
*   **Tactical Profile:** Complete behavioral invisibility. Alice and Bob close all incoming ports and shut down active routing nodes. Alice steganographically camouflages her SSS-shares inside massive, high-volume public web channels (Wikipedia, NASA, DNS TXT logs). Bob passively harvests these channels for algebraic echoes.
*   **Operational Risk:** Zero. Alice and Bob do not connect to each other. Their network traffic consists entirely of normal, outbound HTTPS/DNS requests to world-class public servers. Their culpability is completely diffused and absorbed by millions of ordinary web users.

---

## 4. The Absolute Necessity of Asynchronous Manual Transition

To prevent metadata leakage, **the transition between Option A and Option B must remain strictly asynchronous and manual.**

If the software attempted to automate this transition (e.g., via an "auto-switch" protocol), it would require an online coordination signal between Alice and Bob. In a network fully monitored by Eve, this signal would create an instant timing correlation: Eve would observe Alice and Bob's active nodes shutting down precisely as steganographic traffic emerges in public pools, completely destroying their anonymity.

By enforcing a manual, offline, or pre-scheduled transition (e.g., Alice and Bob agree beforehand to switch to stealth AEH-mode at midnight), **the physical hardware state and network connections are completely decoupled.** Eve is left with a dead UDP port on one side and millions of ordinary Wikipedia readers on the other, mathematically incapable of linking the two.
