# ITS-net: Pipe (stdin / stdout)

## License: GNU GPLv3 Only

Selected subcommands accept `-` for **stdin** or **stdout**. Same syntax in **bash**, **zsh**, and **fish**.

Human status lines go to **stderr** when binary data is written to stdout.

---

## Time-lock

```bash
# Lock plaintext from stdin → puzzle on stdout
echo -n "secret payload" | its-net time-lock -f - -o - -e 50 > locked.its

# Unlock puzzle from stdin → plaintext on stdout
its-net time-unlock -p - -o - < locked.its

# Round-trip pipe (small epoch count for demo only)
echo -n "timelock pipe" | its-net time-lock -f - -o - -e 30 \
  | its-net time-unlock -p - -o -
```

File form (unchanged):

```bash
its-net time-lock -f document.zip -o document.its -e 1000
its-net time-unlock -p document.its -o document.zip
```

---

## Fingerprint erasure (Γ)

When OTP side files are **not** used, normalized output can stream:

```bash
its-net fingerprint-erasure --in - --out - < input.bin > normalized.bin
```

With `--out-otp` / `--pad`, paths must be real files (pad consumption is stateful).

---

## Compose with ITS-ASSYMETRIC (manual shell chain)

No single binary merges these layers — pipe via files or stdout:

```bash
# 1) ITS encrypt to wire on stdout
its_assymetric encrypt --pk bob.key --in msg.txt --out - > msg.wire

# 2) Optional: time-lock the wire file bytes
its-net time-lock -f msg.wire -o msg.wire.its -e 100

# 3) Optional: ITS-OTM sign (if built)
its_otm sign --state alice.state --in msg.wire --out msg.otm
```

Reverse after delay: `time-unlock` → `decrypt`.

Demo: `scripts/pipe_timelock.sh`

---

## Fish

```fish
echo -n "hello" | its-net time-lock -f - -o - -e 30 > locked.its
```

---

## Cross-links

- [ITS-net manual](ITS-net_manual.md)
- [ITS-ASSYMETRIC pipe](https://github.com/0x1F464/ITS-assymetric/blob/master/ITS-assymetric_PIPE.md)
