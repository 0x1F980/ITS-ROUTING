# Participation Symmetry Doctrine (L11–L12) + Broadcast IP Symmetry (BIS)

## Postulates

**P1 — No ITS-only endpoint:** Pool traffic uses public mirrors/CDN/APIs shared with benign mass traffic.

**P2 — Constant participation:** Every `epoch_interval_ms`, harvest pool **and** all `entropy_sources` — even when pool is empty.

**P3 — Symmetry:** Bob's O⁺ participation ⊆ {DNS/NASA/news readers + mirror clients}.

**B1 — IP symmetric emit:** Every epoch all IPs in 𝒩 emit traffic ~ 𝒟_IP (size/timing/dst).

**B2 — Indistinguishable payload:** ITS cells are draws from 𝒟_IP (AEH: φ ~ 𝒟_benign).

**B3 — Multicast forward:** Relays forward multiset; no author-label in IP header.

## Lean (v4)

- `ParticipationSymmetry.lean` — L11 CoverTransport, L12 I(link; O⁺_participation)=0
- `BroadcastIPSymmetry.lean` — I(author; IP_obs)=0, I(recipient; IP_obs)=0 under B1–B3
- `RecipientAttributionZero.lean` — I(recipient; O)=0
- `FlowAttributionZero.lean` — I(flow; O)=0, I(flow; IP)=0
- `PlausibleDeniabilityAbsolute.lean` — master absolute deniability package
- `ComparativeThreatDoctrine.lean` — L13 passiv ISP ⊆ aktiv Eve
- Listed in [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) v4

## Operator checklist

1. Set `multi_pool_urls` to public mirrors (not a private 2-user pool).
2. Set non-empty `entropy_sources` in `[aeh]`.
3. Use `epoch_interval_ms` polling — never selective `--timeout-secs` as sole receive strategy in prod.
4. Require 32-byte `--ratchet-seed-file` (or vault `transport_ratchet`).
5. SSS multi-IP courier: distribute shares across courier paths; all emit chaff each epoch.

## Math law (not software trust)

Eve's backdoored pool/relay stack is **transcript only**. IP attribution closes under **BIS + SSS courier**
in Lean — see `./scripts/verify_math.sh`. Side-channels on **compromised EP** remain outside channel theorem.
