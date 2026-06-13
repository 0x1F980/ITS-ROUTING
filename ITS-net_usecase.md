# ITS-net: Transport Use-Cases, Network Integration & Transport Fork Guide (ITS-net_usecase)

## License: GNU GPLv3 Only
## Target: Network Security Engineers, Protocol Forkers & Tactical Node Operators

> **Scope:** [ITS-net_SECURITY_LAYERS.md](ITS-net_SECURITY_LAYERS.md).


---

## 1. Real-World & Tactical Use-Cases

The `its_net_cli` daemon (executable name: `its-net`) implements the active-transport network logic of our Information-Theoretic Secrecy routing. Core network deployment use-cases include:

### Tactical Scenario 1: Metadata-Invariant Anonymity Tunnel
* **Objective:** Establish an unbreakable communication link between Alice and Bob that completely hides *when* they are communicating and *how much* data they are sending.
* **Mechanism:** Constant-Rate Chaffing + Lorenz Chaotic Jitter.
* **Deployment:** When active, the `its-net` daemon sends packets over the wire at a strictly uniform rate. If Alice has no real SSS-shares to transmit, the daemon automatically generates and sends dummy "chaff" packets. The transmission timing intervals are randomized using a finite field chaotic Lorenz Attractor. Eve's global passive ISP recorders observe only flat, non-correlated statistical white noise, completely neutralizing timing-correlation and volume analysis.

### Tactical Scenario 2: Morphic Blinding Packet-Mixing Node
* **Objective:** Operate an intermediate routing node that mixes traffic from multiple active paths without having access to decrypted payloads or keys, and without letting Eve track packets entering vs. leaving the node.
* **Mechanism:** Morphic Onion Packet blind linear combinations over $\mathbb{F}_p$.
* **Deployment:** Intermediate mixing nodes receive inbound onion packets. Instead of forwarding them sequentially (which would allow Eve to track them), the node performs a blind linear combination of incoming packets using public coefficients. Since the equation systems are mathematically underdetermined by $3L$ dimensions, Eve cannot correlate outgoing packets to incoming ones, while the recipient Bob decapsulates the mixed algebraic sum.

### Tactical Scenario 3: Offline Steganographic Visual Sneakernets
* **Objective:** Route shares in extreme censured zones (WWIII) where standard IP traffic is blocked or mandated to carry a tracking Citizen ID.
* **Mechanism:** High-density structured QR codes and visual VSSS sheets.
* **Deployment:** The offline client exports shares as high-density visual QR codes. These QR codes are printed or displayed on optical terminals and captured using offline cameras. This optical visual link completely bypasses standard IP routing and ISP hardware.

### Tactical Scenario 4: Air-Gapped Document Time-Lock (Dead-Man Custody)
* **Objective:** Encrypt a sensitive local document so it cannot be read until a fixed amount of sequential CPU work has elapsed, with perfect deniability if the operator is coerced.
* **Mechanism:** `ITS-self_enclosed_timelock` via `its-net time-lock`, `time-unlock`, and `time-deny`.
* **Deployment:** On an air-gapped terminal, run `its-net time-lock --file secret.pdf --epochs 1000000 --out secret.its`, then securely erase the plaintext. After the delay, `its-net time-unlock --puzzle secret.its --out secret.pdf` recovers the document. Under duress, `its-net time-deny` produces an alternative `.its` file that decrypts to a harmless cover story.

### Tactical Scenario 5: Public OTM Integrity Audit
* **Objective:** Allow any third party to verify AEH/sneakernet share integrity without access to the signer's ratchet or trapdoor.
* **Mechanism:** `ITS-OTM_public_attestation` public bundles + `its_otm verify`.
* **Deployment:** Signer publishes attestation bundles alongside stego payloads. Auditors run standalone verification without forging capability for future messages.

---

## 2. Network Integration Guide

`its_net_cli` depends on **`ITS-self_enclosed_timelock`** for time-lock operations and **`ITS-OTM_public_attestation`** for Wegman-Carter OTM verification in AEH receive paths. Routing, onion mixing, and SSS fragmentation continue to use `ITS` (`core_logic`, which re-exports `core_logic::otm::*`).

The CLI client (`its-net`) can be integrated alongside other local desktop and mobile applications (such as a local chat app, a secure email client, or an administrative dashboard) to serve as their secure transport hub.

### Topology: Integrating `its-net` as a Local Transport Hub

```
+--------------------+        +--------------------+
| Local Chat App /   |        |  its-net Daemon    |             Unsecured Wire
| Security Frontend  | -----> | (Local Loopback    | --------> (Constant-Rate Chaff)
|  (User Input)      |        |   on Port 127.0.0.1|             (Lorenz Jitter)
+--------------------+        +--------------------+
```

### Configuration Syntax (`config.toml`)
Configure the `its-net` daemon to launch a local listener and establish tunnels with peer nodes:

```toml
[node]
alias = "Morphic_Mixing_Node_1"
listen_addr = "127.0.0.1:4000"
private_key = "0x123456789abcdef..."

[traffic]
# Constant-Rate Chaffing configuration
chaff_rate_pps = 10                  # Maintain 10 packets per second
chaffing_enabled = true
lorenz_jitter_enabled = true         # Enable chaotic interval randomization

[aeh]
# Ambient Entropy Harvesting sources
entropy_sources = ["http://api.blockcypher.com/v1/btc/main"]
clue_offset = 12
```

---

## 3. How to Fork & Extend the Network Engine

Downstream forkers can extend `its-net` to support alternative transport protocols, mesh topologies, or custom steganographic media pipelines.

### Implementing Alternative Transports (WebSockets, LoRa, Bluetooth Mesh)
Currently, `its-net` routes over raw UDP. If your application targets low-power mesh radios (e.g., **LoRa** or **Bluetooth LE**), you can implement the `PacketCourier` trait:

1. **Implement `PacketCourier` for LoRa:**
   ```rust
   use core_logic::routing::MorphicOnionPacket;

   pub struct LoraCourier {
       pub device_address: u16,
   }

   impl LoraCourier {
       pub fn transmit_payload(&self, target_addr: u16, packet: &MorphicOnionPacket) -> Result<(), std::io::Error> {
           let serialized = packet.serialize_to_bytes();
           // Invoke raw SPI/GPIO serial bindings to write to the LoRa transmitter module
           unsafe { lora_spi_write(target_addr, &serialized) };
           Ok(())
       }
   }
   ```

### Custom Steganographic Image LSB Encoder
For extreme WWIII scenarios, you can fork the client to automatically embed entropic-flattened SSS-shares into the least significant bits (LSB) of standard, citizen-ID-signed public JPEG or PNG files:

```rust
pub struct LsbStegEncoder;

impl LsbStegEncoder {
    pub fn embed_share_in_image(image_bytes: &mut [u8], share_bytes: &[u8]) -> Result<(), &'static str> {
        // Embed share bits directly into the lowest bits of image pixels
        let mut share_bit_iter = share_bytes.iter().flat_map(|&byte| (0..8).map(move |i| (byte >> i) & 1));
        
        for pixel in image_bytes.iter_mut() {
            if let Some(bit) = share_bit_iter.next() {
                // Modify the least significant bit (LSB)
                *pixel = (*pixel & 0xFE) | bit;
            } else {
                break;
            }
        }
        Ok(())
    }
}
```
This converts outgoing visual posts into steganographically disguised SSS carriers that easily bypass Eve's automated packet-signature scanners.
