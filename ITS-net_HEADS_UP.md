# ITS-net: Tactical Heads-Up, Threat Profile & Worst-Case Survival Guide (ITS-net_HEADS_UP)

## License: GNU GPLv3 Only
## Target: Network Security Researchers, Cryptographic Auditors & Tactical Operations Teams

> **Scope:** [ITS-net_SECURITY_LAYERS.md](ITS-net_SECURITY_LAYERS.md) — evaluate upstream crates per layer.


---

## Sektion A: Prerequisite for Validity (Forudsætningen)

### The Absolute Endpoint Constraint:
All network-level anonymization, constant-rate chaffing loops, and chaotic de-correlation mechanisms in `ITS-net` are strictly predicated on the **host security of the local execution endpoint**.

If an adversary has compromised Bob's operating system with a resident network tap or a software-level Trojan that reads packets directly from the loopback interface or memory queue before they are passed to the physical network layer, our traffic-obfuscation boundaries can be bypassed. While our `its_net_cli` daemon hides timing signatures over the physical network wire, it cannot protect Bob if his active queue structures are read directly from RAM. Absolute control and auditing of the local execution host are the non-negotiable prerequisites for all network-level security properties.

---

## Sektion B: Eve's Physical Domain Control (Angrebet)

We operate under the ultimate trilateral threat scenario, where the adversary, **Eve, possesses absolute physical control over the entire communications, storage, and manufacturing domain**:

1. **Global Passive Surveillance (Traffic Volume Analysis):** Eve passive-records all IP packet transits on the planet. She attempts to perform timing and packet-count correlation to match Alice's outbound flows to Bob's incoming coordinates.
2. **Silicon & Firmware Exploitation:** Eve has backdoored the physical motherboards, central processing units, and network interface cards (NICs) inside Bob's terminal, actively attempting to inject microsecond timing finger-prints into outgoing streams to identify routing participants across physical checkpoints.
3. **Physical Seizure & Torture:** Eve can physically detain Alice and Bob and subject them to physical coercion to extract vault passwords and private keys.

---

## Sektion C: Defensive Impregnability (Forsvaret)

Even under this absolute physical domain control, `ITS-net` establishes absolute, unbreakable communications security by implementing our defensive protocols:

### 1. Constant-Rate Chaffing (Dummy Injection):
To defeat traffic volume analysis, our network courier maintains a perfectly constant, invariant packet transmission rate. If there are no real SSS-shares in the queue, the daemon automatically generates and transmits cryptographically indistinguishable dummy packets ("chaff"). This converts the network stream into a flat, constant-rate profile, making traffic volume analysis completely blind.

### 2. Lorenz Chaotic Jitter:
To prevent Eve from filtering dummy packets based on periodic timing analysis, the packet transmission intervals are randomized using a Lorenz chaotic system calculated over the finite field. Since chaotic trajectories are non-periodic and highly sensitive to initial conditions, Eve cannot perform statistical de-jittering.

### 3. Faraday Cages & Physical Shielding:
To defeat physical SDR-based TEMPEST surveillance near the terminal, Bob must house his terminal inside a Faraday cage. This blocks all electromagnetic emissions from escaping the physical zone, preventing physical timing correlation.

### 4. Air-Gapping, QR Codes & Offline Operation:
Under extreme requirements, Bob must completely decouple his terminal from any physical network cable. By operating in a 100% air-gapped terminal room, SSS shares are generated offline and manually transported. The system operates fully offline, generating, packing, and parsing shares as high-density, structured QR codes. These QR codes are read directly from physical screens or printed paper using air-gapped camera sensors, establishing an optical visual link that completely bypasses any copper or wireless connection to online infrastructure.

### 5. Hermetic Prebuilt Binaries:
To guarantee absolute, untampered runtime execution in offline zones, the system provides precompiled, cryptographically hashed, static prebuilt binaries. Generated through our hermetic Nix and Docker reproducible build pipelines, these binaries can be audited and signed on verified master devices, distributed via write-once physical media (e.g., optical CD-Rs), and executed on offline nodes without requiring external compilers, internet links, or dynamically linked package caches. This eliminates any possibility of compiler-inserted backdoors or dynamic-link interception.

### 6. Duress Ratchets:
To survive physical coercion, the system utilizes a Dual-Seed Duress Ratchet. If Bob is forced to input his password, entering a decoy password derives a decoy seed that unlocks completely benign files, while immediately purging the master key registers.

### 6. Heartbeat Self-Wipe:
The physical terminal requires a continuous, scheduled heartbeat signal from the operator. If the operator is seized, the absence of the heartbeat immediately triggers an automated memory sanitization routine, writing zeros to all sensitive registers using volatile writes and sequentially consistent compiler fences (`SeqCst`).

### 7. Offline Time-Lock Custody (`ITS-self_enclosed_timelock`):
For documents that must remain unreadable until sequential CPU work completes:
1. On an air-gapped host: `its-net time-lock --file secret.pdf --epochs N --out secret.its`
2. Securely erase the plaintext original.
3. After the delay: `its-net time-unlock --puzzle secret.its --out secret.pdf`
4. Under coercion: `its-net time-deny --puzzle secret.its --decoy "Cover story" --out decoy.its` — hand Eve the decoy puzzle file.

CPU thermal and TEMPEST guidance for long squaring runs: [ITS-self_enclosed_timelock_HEADS_UP.md](https://github.com/0x1F464/ITS-self_enclosed_timelock/blob/master/ITS-self_enclosed_timelock_HEADS_UP.md).

### 8. Public OTM Attestation (`ITS-OTM_public_attestation`):
AEH receive paths verify Wegman-Carter tags via the standalone OTM crate. For third-party audit without ratchet access, publish `.otm` bundle files and run `its_otm verify --bundle FILE`. Never publish `k_mac`/`nonce` before the attested message — one-time keys are revealed only with the bundle. See [ITS-OTM_public_attestation_HEADS_UP.md](https://github.com/0x1F464/ITS-OTM_public_attestation/blob/main/ITS-OTM_public_attestation_HEADS_UP.md).

---

## Sektion D: Worst-Case Scenario: World War III & Absolute Censorship

### The Doomsday Landscape:
In the event of World War III, a total collapse of the global internet, or the implementation of absolute sovereign firewalls, Eve mandates that **every single packet transmission must be cryptographically signed with a government-issued Citizen ID**.

### The Enemy is the Hardware:
Under this extreme scenario, the physical hardware itself acts as the enemy. Compromised motherboards and network interface cards (NICs) will actively attempt to inject subtle, microsecond timing jitters or packet spacing signatures (fingerprints) into outgoing streams, trying to track user identity across physical checkpoints even if the payloads are encrypted.

### Mathematics Overrules Software & Hardware:
Our strict mathematical boundaries completely overrule both backdoored software and compromised hardware:
1. **Underdetermined Systems:** Even if the hardware successfully fingerprints the transmission times, the data itself is represented as uniform coordinates over $\mathbb{F}_p$. Since the equation systems are strictly underdetermined, Eve's infinite quantum arrays face a flat probability distribution, rendering her incapable of proving the existence of a real payload.
2. **Entropic Footprintable Permissions (The Last Resort):**
   When absolute anonymity is barred, Alice and Bob must employ **entropic footprintable permissions** as their final option. Instead of attempting to bypass the Citizen ID requirement (which triggers instant blockade flags), they steganographically blend and hide their SSS-shares directly inside legitimate, signed citizen-ID public transactions or files. They sign their public telemetry posts or images with their state-mandated IDs to appease Eve's automated filters, but the underlying payload has been entropic-flattened (completely smoothed to remove any cryptographic patterns or signatures) and hidden in the least significant bits (LSBs).
3. **Provenance Erasure (`ITS-FINGERPRINT_ERASURE` v6 / v0.7.0):**
   Two layers: **Γ** (Church-Rosser universal NF — Bob accepts functional R, not Eve's bits) then **OTP** (wire, I(X;F)=0). **v0.8:** `--fingerprint-erasure` defaults to strict stack (extended mode) + requires OTP pad + chaff; escape `--fe-permissive`. Post-save: `its_fe watch --dir DIR --in-place`.
   ```bash
   # Strict-stack send (strict policy + extended mode + domain; pair with --fe-pad + chaff daemon)
   its-net client-send --file song.wav --dest 3 --fingerprint-erasure \
     --fe-strict-stack --fe-kind audio --fe-domain continuous \
     --fe-pad offline.pad --config config.toml

   # Permissive Γ (requires --fe-permissive feature)
   its-net client-send --file tainted.jpg --dest 3 --fingerprint-erasure --fe-permissive --config config.toml

   # Standalone offline
   its-net fingerprint-erasure --file tainted.jpg --out clean.png --pad offline.pad --out-otp wire.bin
   its_fe process --in tainted.jpg --out clean.png --strict-stack --kind image --domain continuous
   its_fe otp-mask --in clean.png --pad offline.pad --out wire.bin
   ```
   Bob: `its_fe otp-unmask --in wire.bin --pad offline.pad --out clean.png` — opens directly.
