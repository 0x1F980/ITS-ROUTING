# ITS UES — 5-minute QUICKSTART

Send and receive over the UES Monocell Pool with one command per side.

## Why Eve can't win (math, not trust)

Eve may own 99.999%+ of pool mirrors — that is axiom A0, not a failure mode. Under ITS v9:

| Pillar | What Eve gets | Your mitigation (config) |
|--------|---------------|--------------------------|
| **C** | 0 bits about message content in \(O\) | Shannon wire + L3 pool — no config needed |
| **I** | \(\leq 1\) false accept per \(2.147\times10^9\) forgery tries | OTM verify on **your** endpoint keys only |
| **A** | Cannot stay on \(\mathcal{M}_{\text{valid}}\) if she omits | `multi_pool_urls` + `witness_pool_urls` below |

**Numeric walkthrough** (epochs 0–5, evil mirror omit, k-of-n witness): [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) §Va.

## 1. Bootstrap

```bash
cd /path/to/ecosystem
./ROUTING/scripts/bootstrap.sh
cargo build --release -p its_routing -p its_keymgmt --manifest-path ROUTING/Cargo.toml
cargo build --release --manifest-path ITS-asymmetric/Cargo.toml --bin its_asymmetric --features "bundle,parallel,std,compact-wire"
```

Ensure `its-routing`, `its-km`, and `its_asymmetric` are on `PATH`.

## 2. Routing config

```bash
mkdir -p ~/.its
cp ROUTING/config.prod.toml ~/.its/routing.toml
```

Edit `~/.its/routing.toml` — set `pool_url` or `multi_pool_urls` to your public mirror (see `ROUTING/deploy/pool-mirror/`).

**ITS-A (availability):** list mirrors + independent witnesses. Eve-only pools cannot satisfy ValidFwd if they omit — they are de-whitelisted automatically.

```toml
# Primary mirrors (at least one must be honest forwarder)
multi_pool_urls = [
  "http://mirror1:8787",
  "http://mirror2:8787",
]

# A2′ witness mirrors — k-of-n consensus (Charlie role)
witness_pool_urls = [
  "http://witness-charlie:8787",
  "http://witness2:8787",
  "http://witness3:8787",
]
consensus_k = 2          # 2-of-3 witnesses must agree on cell c at epoch e
valid_fwd_window = 64    # ValidFwd history window W (epochs)
```

With \(k=2, n=3\): two witnesses must harvest the same \(c_e\) for `consensusAtEpoch`. Example: Eve-A omits epoch 3, but Charlie + W3 (both honest) return \(c_3\) → `ProofFwd(3,c_3)`. You need **one** mirror in \(\mathcal{M}_{\text{valid}}\) for harvest — not a majority of \(10^9\) nodes.

## 3. Vault + contacts (both peers)

```bash
its-km vault init --vault-key-dir ~/.its/km-vault-keys
its-km --true-secret ~/.its/km-vault-keys/true/secret.key entry add \
  --alias bob --public /path/to/bob.public.key --routing-config ~/.its/routing.toml
```

`entry add` auto-generates a **per-contact transport_ratchet** (32 bytes in vault). A QR payload is printed on add; peer imports with:

```bash
its-km import-qr --alias bob --layer transport-ratchet --payload 'its-km:qr:...'
# or: its-km export-qr --contact bob --layer transport-ratchet
```

If `~/.its/routing.toml` is missing, `entry add` copies `ROUTING/config.prod.toml` automatically.

## 4. Send / receive

**Alice:**

```bash
its-km --true-secret ~/.its/km-vault-keys/true/secret.key send --contact bob --file doc.pdf
```

**Bob** (receive — message was sealed to bob's public key; use bob's keypair on the alice contact entry):

```bash
its-km --true-secret ~/.its/km-vault-keys/true/secret.key entry add \
  --alias alice --public /path/to/bob.public.key --secret /path/to/bob.secret.key \
  --routing-config ~/.its/routing.toml --transport-ratchet-file /path/to/shared-ratchet.seed
its-km --true-secret ~/.its/km-vault-keys/true/secret.key receive --contact alice --out received.pdf
```

## 5. Verify

```bash
ROUTING/scripts/verify_ecosystem.sh /home/user
```

## Optional: SOCKS proxy (v1.8)

```bash
python3 ROUTING/tools/its_pool_proxy.py --listen 127.0.0.1:1080 --config ~/.its/routing.toml
```

Point apps at `SOCKS5 127.0.0.1:1080` (requires Bob receiver running).

## Why better than I2P/Nym

See [ITS-routing_SUPERIORITY.md](ITS-routing_SUPERIORITY.md).
