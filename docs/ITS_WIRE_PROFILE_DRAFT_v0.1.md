# ITS Wire Profile — draft v0.1 (Eco D)

## License: GNU GPLv3 Only

**Status:** draft — not IETF submitted  
**Normative implementation:** [ITS-asymmetric](https://github.com/0x1F980/ITS-ASSYMETRIC) v0.10+

---

## 1. Scope

Static broadcast Shannon ITS on a hostile channel. One 32-byte `public.key` published; arbitrary senders post `.wire` blobs; one holder of `secret.key` decrypts.

---

## 2. public.key (32 bytes)

| Offset | Size | Field |
|--------|------|-------|
| 0 | 4 | `bind_x` (u32 BE, field element) |
| 4 | 4 | `completion_x` (u32 BE) |
| 8 | 4 | `morphic_c1` (u32 BE, production 3) |
| 12 | 4 | `morphic_c2` (u32 BE, production 5) |
| 16 | 16 | `identity` (fingerprint, no secrets) |

Sidecar: `public.epoch` (4 bytes BE u32) — published seal for senders.

---

## 3. Wire message (`.wire`)

| Field | Size | Description |
|-------|------|-------------|
| `msg_id` | 8 | BE u64: lower 32 = user msg id; upper 32 = `seal_epoch` |
| `body_len` | 4 | BE u32 |
| `body` | n | OTP body bytes |
| `sigma_count` | 4 | = body_len |
| `sigma[]` | 4×n | Shamir share A per byte |
| `morphic_blend[]` | 4×n | Morphic blend per byte |
| `otm_tags[]` | 4×n | Public integrity MAC per byte |

Per-byte expansion ≈ 13× body (standard profile).

---

## 4. Profiles

| Profile | `KEY_DRAW_MASK` | Search/byte | Lean module |
|---------|-----------------|-------------|-------------|
| standard | `0xFFF` | 4096² | `Asymmetric` |
| compact | `0xFF` | 256² | `Compact` |

---

## 5. Security properties (Lean)

Standard: K1–K8 + K6 sender cannot decrypt (`lake build Asymmetric`).  
Compact: separate `lake build Compact` — same Shannon adversary floor (256 candidates/byte).

---

## 6. Pipe stream (optional)

Magic `ITSC` + repeated `(u32 len || wire bytes)` for messages &gt; 65536 B plaintext chunking.

---

## Cross-links

- [ITS-asymmetric DOMINANCE](../ITS-asymmetric/ITS-asymmetric_DOMINANCE.md)
- [ITS-routing PIPE](ITS-routing_PIPE.md)
- [ITS-KeyManagement PIPE](https://github.com/0x1F980/ITS-KeyManagement/blob/main/ITS-KeyManagement_PIPE.md)
