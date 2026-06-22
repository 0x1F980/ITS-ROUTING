# ITS UES — 5-minute QUICKSTART

Send and receive over the UES Monocell Pool with one command per side.

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

Optional ITS-A witness pool (A2′ Charlie mirrors):

```toml
witness_pool_urls = ["http://witness1:8787"]
consensus_k = 2          # k-of-n threshold
valid_fwd_window = 64    # ValidFwd history window W
```

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
