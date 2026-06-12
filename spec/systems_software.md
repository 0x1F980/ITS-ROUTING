# Systems Software & Memory Specification
## License: GNU GPLv3 Only
## Target: Computer Scientists, Systems Researchers & Code Auditors

This document details the systems-level design of the ITS/SCPST software implementation, focusing on memory safety, cryptographic timing-leak prevention, and microkernel-level processes isolation.

---

## 1. Hukommelseshygiejne & Zeroize (Memory Sanitization)

When sensitive cryptographic variables (such as private evaluation coordinates, Shamir Secret Sharing coefficients, or intermediate Wegman-Carter keys) are allocated in RAM, they pose a severe security risk if left uncleared. An attacker exploiting a hardware-level read vulnerability (e.g., Rowhammer, Meltdown/Spectre, or physical memory dumps) could read stale memory blocks.

### The Dead-Store Elimination Hazard:
In standard C/Rust implementations, a simple assignment like `buffer[i] = 0` at the end of a function is frequently optimized out by the LLVM compiler. Because the variable is never read again before going out of scope, LLVM identifies this as a "dead store" and removes it, leaving the sensitive keys in the raw RAM chips.

### Rust Memory Zeroization Protocol:
To eliminate this risk, all sensitive containers in our ecosystem are wrapped in types implementing the `zeroize::Zeroize` and `zeroize::ZeroizeOnDrop` traits.
* **Compiler Barrier:** Zeroization uses volatile writes and assembly compiler barriers (`core::sync::atomic::compiler_fence` with `SeqCst` ordering) to physically force the CPU to write zeros to the specified RAM coordinates before freeing the page.
* **Verification Targets:**
  Auditors should inspect `ZeroizedBuffer` in `hydra_cli/src/main.rs` and verify that any variable containing decrypted coordinates or key-shares is wrapped in zeroizing containers.

```rust
// Verify zeroization on drop behavior
pub struct ZeroizedBuffer {
    data: Vec<u8>,
}

impl Drop for ZeroizedBuffer {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}
```

---

## 2. Constant-Time Execution Paths (Timing Side-Channels)

Modern CPUs optimize execution speed using branch predictors and data caches. If a program branches or accesses memory based on secret data, an attacker on the same physical system or router can measure the execution time to extract the secret key (Timing Attack).

### Prevention of Branch Side-Channels:
We forbid the use of standard boolean comparison operators (`==`, `!=`, `<`, `>`) on cryptographic values. Instead, we enforce constant-time select and equality operations using the `subtle` crate:
* **Conditional Selection:** `subtle::ConditionallySelectable::conditional_select` is used to choose between two field elements based on a `subtle::Choice` mask.
* **Constant-Time Comparison:** Equality checks are executed by bitwise XORing all bytes and accumulating the results, ensuring that the execution path remains exactly identical whether the values match or differ.

```rust
// Constant-time selection model used in core_logic
let mask = Choice::from(1);
let result = FieldElement::conditional_select(&a, &b, mask);
// Guarantees zero branch instructions based on the value of mask
```

---

## 3. seL4 Microkernel Compartmentalization

For absolute security, the core cryptographic library (`core_logic`) is designed to execute inside an isolated **seL4 microkernel compartment** (or enclave) with zero network permissions. This guarantees that even if the transport layer (`hydra_cli`) is fully compromised, the master keys cannot be exfiltrated.

### 4KB Shared Page Memory Alignment:
Communication between the untrusted transport daemon and the isolated krypto-enclave is restricted to page-aligned, fixed-size shared memory regions.
*   **Struct Alignment:** The `Sel4SharedPage` structure in `core_logic/src/sel4_compat.rs` is explicitly aligned to 4096-byte boundaries (the standard page size of x86_64 and ARM processors):
    ```rust
    #[repr(C, align(4096))]
    pub struct Sel4SharedPage {
        pub data: [u8; 32], // Configurable to 64 bytes under m61
        pub padding: [u8; 4064],
    }
    ```
*   **Memory Isolation:** By padding the shared memory structure to exactly 4096 bytes, we guarantee that no other kernel data can occupy the same page. This prevents page-fault timing leaks and ensures that the seL4 microkernel can enforce page-level memory protection.

---

## 4. Total Dependency Elimination & Macro-Free Auditability

Standard modern software stacks suffer from immense "supply-chain inflation" due to nested transitive dependencies. Large frameworks (like `serde`, `tokio`, or `reqwest`) generate massive amounts of macro-expanded code that is practically impossible to review for backdoors.

### Macro-Free Design:
Our ecosystem completely replaces heavy transitive dependencies with primitive, hand-written standard library implementations:
*   **JSON/Serialization:** We completely eliminate `serde` and `serde_json`. Parsing of local system configuration files is executed using a hand-written, macro-free parser (`parse_config` in `hydra_cli/src/main.rs`) based on strict line splitting and string slicing.
*   **Network Transport:** We completely eliminate `tokio` and `reqwest` inside the CLI daemon. Network transmissions are decoupled using the synchronous `PacketCourier` trait and executed via standard `std::net::UdpSocket` threads.
*   **Entropy Gathering:** We completely eliminate external crates like `rand` inside the binary. Entropy is pulled directly from the OS-hardened `/dev/urandom` interface.

This allows any security researcher to audit the entire transport and cryptographic binaries in less than an hour, ensuring absolute supply-chain sterility.
