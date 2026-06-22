# Firecracker microVM profile (skeleton)

## License: GNU GPLv3 Only

**Status:** Greenfield — threat model and export steps only. No automated rootfs pipeline in this repo yet.

---

## Threat model

| Asset | In VM | On host |
|-------|-------|---------|
| `its-routing` static binary | Yes | Build only |
| `routing.toml` with mirror URLs | Optional | Prefer secrets on host USB |
| KM vault / `secret.key` | **No** — mount read-only pool dir only | Host or HSM |
| Network | Single tap to pool mirror or isolated | Operator controlled |

Firecracker fits **pool mirror workers** and **receive-only harvest nodes** — not the operator KM vault.

---

## Planned artifacts

| File | Purpose |
|------|---------|
| `rootfs/` | Minimal musl rootfs with `its-routing` + `config.offline.toml` |
| `vm_config.json` | vCPU, memory, kernel, tap networking |
| `export_rootfs.sh` | Copy scratch image artifacts from Docker build |

---

## Build path (manual today)

```bash
# 1. Static binary
cd ROUTING && cargo build --release --target x86_64-unknown-linux-musl

# 2. Docker scratch reference
./deploy/docker/docker_build_all.sh

# 3. Future: pack rootfs + Firecracker JSON — see ITS-routing_PREBUILD_DOCTRINE.md §5
```

---

## Links

- [ITS-routing_PREBUILD_DOCTRINE.md](../../ITS-routing_PREBUILD_DOCTRINE.md)  
- [deploy/docker/docker-compose.yml](../docker/docker-compose.yml)  
- [ITS_CONSTITUTION_CLI.md](../../ITS_CONSTITUTION_CLI.md) — offline `--pool-dir` on host, not inside untrusted VM
