# ITS-routing: Pipe (stdin / stdout)

## License: GNU GPLv3 Only

Selected subcommands accept `-` for **stdin** or **stdout**. Same syntax in **bash**, **zsh**, and **fish**.

Human status lines go to **stderr** when binary data is written to stdout.

---

## Time-lock

```bash
# Lock plaintext from stdin → puzzle on stdout
echo -n "secret payload" | its-routing time-lock -f - -o - -e 50 > locked.its

# Unlock puzzle from stdin → plaintext on stdout
its-routing time-unlock -p - -o - < locked.its

# Round-trip pipe (small epoch count for demo only)
echo -n "timelock pipe" | its-routing time-lock -f - -o - -e 30 \
  | its-routing time-unlock -p - -o -
```

File form (unchanged):

```bash
its-routing time-lock -f document.zip -o document.its -e 1000
its-routing time-unlock -p document.its -o document.zip
```

---

## Fingerprint erasure (Γ)

When OTP side files are **not** used, normalized output can stream:

```bash
its-routing fingerprint-erasure --in - --out - < input.bin > normalized.bin
```

With `--out-otp` / `--pad`, paths must be real files (pad consumption is stateful).

---

## Compose with ITS-ASYMMETRIC (manual shell chain)

No single binary merges these layers — pipe via files or stdout:

```bash
# 1) ITS encrypt to wire on stdout
its_asymmetric encrypt --pk bob.key --in msg.txt --out - > msg.wire

# 2) Optional: time-lock the wire file bytes
its-routing time-lock -f msg.wire -o msg.wire.its -e 100

# 3) Optional: ITS-OTM sign (if built)
its_otm sign --state alice.state --in msg.wire --out msg.otm
```

Reverse after delay: `time-unlock` → `decrypt`.

Demo: `scripts/pipe_timelock.sh` · ITS E2E: `scripts/pipe_its_e2e.sh`

---

## Fish

```fish
echo -n "hello" | its-routing time-lock -f - -o - -e 30 > locked.its
```

---

## Cross-links

- [ITS-routing manual](ITS-routing_manual.md)
- [ITS-routing KEEP boundary](ITS-routing_KEEP_BOUNDARY.md)
- [ITS-KeyManagement pipe](https://github.com/0x1F980/ITS-KeyManagement/blob/main/ITS-KeyManagement_PIPE.md) — orchestrated send/receive + ratchet seed export
- [ITS-ASYMMETRIC pipe](https://github.com/0x1F464/ITS-asymmetric/blob/master/ITS-asymmetric_PIPE.md)

---

## Ratchet seed (AEH send/receive)

ITS-routing accepts **32 raw bytes** via `--ratchet-seed-file`. Passwords and duress policy live in **ITS-KeyManagement**:

```bash
its-km export-ratchet-seed --contact bob --out /tmp/seed.bin --password '...'
its-routing client-send -f payload.bin -d 3 --aeh --ratchet-seed-file /tmp/seed.bin -c config.toml
rm -f /tmp/seed.bin
```

User-facing orchestration (encrypt → send → receive → decrypt):

```bash
its-km send --contact bob --file msg.txt
its-km receive --contact bob
```
