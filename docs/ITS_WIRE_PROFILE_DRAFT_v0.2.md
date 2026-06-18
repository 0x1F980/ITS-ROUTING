# ITS Wire Profile — draft v0.2 (Eco D)

## License: GNU GPLv3 Only

**Status:** draft v0.2 — supersedes [v0.1](ITS_WIRE_PROFILE_DRAFT_v0.1.md)  
**Normative implementation:** ITS-asymmetric v0.10.1+

---

## 1. Scope

Static broadcast Shannon ITS on a hostile channel. One 32-byte `public.key` published; arbitrary senders post `.wire` blobs; one holder of `secret.key` decrypts.

---

## 2. ALPN / Content-Type tokens

| Token | Profile | Description |
|-------|---------|-------------|
| `its-wire/1` | standard | 4096² mask search/byte (Lean `Asymmetric`) |
| `its-wire/1-compact` | **compact (production default)** | 256² mask search/byte (Lean `Compact`) |
| `application/its-wire+1` | standard | HTTP Content-Type |
| `application/its-wire+1-compact` | compact | HTTP Content-Type |

Transport design: [ITS-asymmetric/docs/ITS_TLS_ALPN_DESIGN.md](../../ITS-asymmetric/docs/ITS_TLS_ALPN_DESIGN.md)  
nginx POC: [contrib/nginx-its-wire.conf](../../ITS-asymmetric/contrib/nginx-its-wire.conf)

---

## 3. public.key (32 bytes)

Unchanged from v0.1 — see v0.1 §2.

Sidecar: `public.epoch` (4 bytes BE u32).

---

## 4. Wire message (`.wire`)

Unchanged from v0.1 — per-byte expansion ≈ 13× body (both profiles; profile affects decrypt CPU only).

---

## 5. Profiles

| Profile | `KEY_DRAW_MASK` | Search/byte | Build | Default |
|---------|-----------------|-------------|-------|---------|
| **compact** | `0xFF` | 256² | `--features compact-wire,parallel` | **production** |
| standard | `0xFFF` | 4096² | default | max-audit |

KeyManagement: `ITS_WIRE_PROFILE=compact` (default) — requires `its_asymmetric` built with `compact-wire`.

Benchmarks: `benches/BENCH_SLA.txt`, `benches/BENCH_VS_PQC.md`.

---

## 6. Security properties (Lean)

| Profile | Lean module | Adversary floor |
|---------|-------------|-----------------|
| standard | `Asymmetric` K1–K8 | ≥256 candidates/byte |
| compact | `Compact` | ≥256 candidates/byte |

Composition (K8 ⊥ OTM attestation): `Asymmetric.Composition`.

---

## 7. Pipe stream

Magic `ITSC` + `(u32 len || wire bytes)` for plaintext > 65536 B.

---

## 8. Tools

| Tool | Role |
|------|------|
| `ROUTING/scripts/its-curl.sh` | POST wire with ALPN headers |
| `ROUTING/tools/its_wire_proxy.py` | Minimal HTTP receiver |
| `ROUTING/scripts/pipe_its_e2e.sh` | Encrypt/decrypt roundtrip |

---

## Cross-links

- [ITS-asymmetric DOMINANCE](../../ITS-asymmetric/ITS-asymmetric_DOMINANCE.md)
- [ITS_MIGRATION_GUIDES.md](../ITS_MIGRATION_GUIDES.md)
