# ITS Independent Crypto Review Checklist (Eco F)

## License: GNU GPLv3 Only

Reproducible verification badge for adopters.

---

## Pre-review

- [ ] Clone all repos at tagged release (ITS-asymmetric **v0.10.0**)
- [ ] `./scripts/verify.sh` green (ITS-asymmetric)
- [ ] `lake build Asymmetric` + `lake build Compact`
- [ ] `cargo test --release -- --ignored` (adversary tests)

---

## Code review focus

- [ ] `otm_root` never derivable from `public.key` bytes
- [ ] `validate_for_public` rejects cross-keypair secrets
- [ ] Wire tags use public MAC only; secret path decrypt-only
- [ ] Epoch seal replay: old wires decrypt after advance
- [ ] Bundle mapping shares never on wire (OOB only)

---

## Proof review

- [ ] Lean `asymmetric_certificate` — no `sorry`
- [ ] K6 sender cannot decrypt matches Rust tests
- [ ] Compact profile documented separately from standard K8 cert

---

## Operational review

- [ ] RNG from `/dev/urandom` (CLI) or platform CSPRNG
- [ ] `public.epoch` published after rotation
- [ ] Coercion threat documented (secret.key holder)

---

## Ecosystem

- [ ] KeyManagement `--wire-only` path tested
- [ ] OTM verify-cert before `--require-cert` sends
- [ ] ROUTING pipe E2E script runs

---

Badge text (when all checked): **ITS v0.10 verify.sh + Lean green — reproducible build**
