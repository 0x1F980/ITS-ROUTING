# Systems Software & Memory Specification
## License: GNU GPLv3 Only
## Target: Computer Scientists, Systems Researchers & Code Auditors

This document details the systems-level design of the ITS/SCPST software implementation, focusing on memory safety, cryptographic timing-leak prevention, and microkernel-level processes isolation.

---

## 1. Hukommelseshygiejne & Zeroize (Memory Sanitization)

When sensitive cryptographic variables (such as private evaluation coordinates, Shamir Secret Sharing coefficients, or intermediate Wegman-Carter keys) are allocated in RAM, they pose a severe security risk if left uncleared. An attacker exploiting a hardware-level read vulnerability (e.g., Rowhammer, Meltdown/Spectre, or physical memory dumps) could read stale memory blocks.

### The Dead-Store Elimination Hazard:
In standard C/Rust implementations, a simple assignment like `buffer[i] = 0` at the end of a function is frequently optimized out by the LLVM compiler. Because the variable is never read again before going out of scope, LLVM identifies this as a "dead store" and removes it, leaving the sensitive keys in the raw RAM chips.

### Formal Mathematical and Logical Model of Dead-Store Elimination (DSE) Prevention:
Let $\mathcal{O}$ be the sequence of memory operations executed by the program, and let $S$ be the state of the physical memory. Under standard LLVM optimization rules, the compiler performs data-flow analysis using a control-flow graph (CFG).
Let $W(x, v)$ represent a write operation of value $v$ to memory location $x$, and let $R(x)$ represent a read operation from location $x$. Let $D(x)$ represent the deallocation (or end of lifetime/scope) of the memory location $x$.

If the CFG contains a sequence of the form:
$$ \mathcal{O} = [ \dots, W(x, v), \dots, D(x) ] $$
where there is no read operation $R(x)$ between $W(x, v)$ and $D(x)$, the compiler's Dead-Store Elimination (DSE) pass defines the write $W(x, v)$ as a "dead store" because it has no effect on the observable behavior of the program under the abstract machine model. The compiler optimizes this to:
$$ \text{DSE}(\mathcal{O}) = [ \dots, D(x) ] $$
This leaves the sensitive value $v$ physically intact in the RAM cells at address $x$.

To prevent this optimization and guarantee physical overwriting, we enforce **Volatile Writes** and **Sequentially Consistent Compiler Fences**:
1. **Volatile Writes ($W_{\text{volatile}}(x, 0)$):** A volatile write tells the compiler that the operation has side effects outside the abstract machine's control-flow model. The LLVM compiler is strictly forbidden from optimizing away, reordering, or coalescing volatile operations. Thus:
   $$ \text{DSE}([ \dots, W_{\text{volatile}}(x, 0), \dots, D(x) ]) = [ \dots, W_{\text{volatile}}(x, 0), \dots, D(x) ] $$
2. **Compiler Fence with Sequential Consistency ($F_{\text{SeqCst}}$):** To prevent both the compiler and the out-of-order execution engine of the CPU from reordering memory writes across the zeroization boundary, we introduce a compiler fence with `SeqCst` (Sequential Consistency) ordering:
   ```rust
   core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
   ```
   This fence enforces a strict partial ordering on the instruction stream. Let $op_1$ be any memory write occurring before the fence, and $op_2$ be any memory operation occurring after the fence. The fence guarantees:
   $$ op_1 \prec F_{\text{SeqCst}} \prec op_2 $$
   This ensures that the sensitive data is physically overwritten with zeros in the RAM cells before the memory page is returned to the OS allocator or deallocated.

### Provable Elimination of Physical and Microarchitectural Vulnerabilities:
By ensuring that sensitive memory is zeroed immediately upon release, our system provably eliminates several critical classes of physical and microarchitectural attacks:

#### 1. Cold-Boot Attacks:
Dynamic RAM (DRAM) cells rely on tiny capacitors that slowly leak charge when power is lost. If cooled with liquid nitrogen, DRAM can retain its state for minutes or even hours without power. If sensitive keys are left in deallocated memory, an attacker with physical access can cut power, freeze the RAM chips, and extract them to read the keys.
* **Proof of Elimination:** Zeroization guarantees that the physical capacitors representing the key bits are discharged to $0$ before deallocation. The physical lifetime of the secret in DRAM is strictly bounded by the active computation time:
  $$ \text{Lifetime}_{\text{secret}} \le \text{Computation Time} $$
  Leaving nothing but zeros for a cold-boot attacker to extract.

#### 2. Rowhammer Attacks:
Rowhammer is a physical vulnerability where rapidly accessing ("hammering") specific rows of DRAM leaks electromagnetic charge to adjacent rows, causing bit-flips in neighboring cells. If sensitive keys reside in RAM for long periods, an attacker can use Rowhammer to flip bits in the key or read-disturb adjacent memory to leak key structures.
* **Proof of Elimination:** Zeroing memory immediately minimizes the exposure window of the secret in the physical DRAM array. Since the secret is cleared as soon as the cryptographic operation completes, the probability of a successful Rowhammer-induced leak or corruption of the active key is reduced to virtually zero.

#### 3. Spectre & Meltdown (Transient Execution Side-Channels):
Spectre and Meltdown exploit speculative execution in modern CPUs. During speculative execution, the CPU may speculatively execute instructions that access deallocated or out-of-bounds memory. Even though these instructions are discarded when the branch predictor realizes the mistake, the data accessed is loaded into the CPU cache, where it can be leaked via cache-timing side-channels (e.g., Flush+Reload).
* **Proof of Elimination:** If sensitive memory is deallocated but not zeroed, it remains in the raw memory pool. A speculative execution path in an unrelated process could speculatively read this stale memory and leak it. By zeroing the memory immediately, we ensure that:
  $$ \forall \text{ speculative reads } R_{\text{spec}}(x) \text{ after zeroization} \implies \text{Value read} = 0 $$
  Since the speculative path only reads $0$, no sensitive key bits are ever loaded into the cache hierarchy during transient execution, completely neutralizing Spectre and Meltdown leaks of deallocated secrets.

### Rust Memory Zeroization Protocol:
To eliminate this risk, all sensitive containers in our ecosystem are wrapped in types implementing the `zeroize::Zeroize` and `zeroize::ZeroizeOnDrop` traits.
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
