#!/usr/bin/env python3
"""D1–D9 doc sweep: 0x1F980 URLs, ecosystem master, ITS-ASSYMETRIC typo."""
from pathlib import Path

ECO = Path("/home/user")
REPOS = [
    "ITS-asymmetric",
    "ROUTING",
    "ITS-KeyManagement",
    "sidechannel_resistant_hardware",
    "ITS-ledger",
    "ITS-OTM_public_attestation",
    "ITS-self_enclosed_timelock",
    "ITS-fingerprint_erasure",
    "SSS_CHAIN",
]

REPLACEMENTS = [
    (
        "https://github.com/0x1F464/ITS/blob/master/ITS_SECURITY_LAYERS.md",
        "https://github.com/0x1F980/ITS-ROUTING/blob/master/ITS_ECOSYSTEM.md",
    ),
    ("https://github.com/0x1F464/ROUTING", "https://github.com/0x1F980/ITS-ROUTING"),
    ("https://github.com/0x1F464/ITS-routing", "https://github.com/0x1F980/ITS-ROUTING"),
    ("https://github.com/0x1F464/ITS-net", "https://github.com/0x1F980/ITS-ROUTING"),
    ("https://github.com/0x1F464/ITS-asymmetric", "https://github.com/0x1F980/ITS-asymmetric"),
    ("git@github.com:0x1F464/ITS-asymmetric", "git@github.com:0x1F980/ITS-asymmetric"),
    (
        "https://github.com/0x1F464/ITS-OTM_public_attestation",
        "https://github.com/0x1F980/ITS-OTM_public_attestation",
    ),
    (
        "git@github.com:0x1F464/ITS-OTM_public_attestation",
        "git@github.com:0x1F980/ITS-OTM_public_attestation",
    ),
    (
        "ssh://git@github.com/0x1F464/ITS-OTM_public_attestation.git",
        "ssh://git@github.com/0x1F980/ITS-OTM_public_attestation.git",
    ),
    (
        "https://github.com/0x1F464/ITS-self_enclosed_timelock",
        "https://github.com/0x1F980/ITS-self_enclosed_timelock",
    ),
    ("https://github.com/0x1F464/ITS-hardware", "https://github.com/0x1F980/sidechannel_resistant_hardware"),
    ("https://github.com/0x1F464/ITS-ledger", "https://github.com/0x1F980/ITS-ledger"),
    (
        "https://github.com/0x1F464/ITS-FINGERPRINT_ERASURE",
        "https://github.com/0x1F980/ITS-FINGERPRINT_ERASURE",
    ),
    (
        "https://github.com/0x1F464/ITS-fingerprint_erasure",
        "https://github.com/0x1F980/ITS-FINGERPRINT_ERASURE",
    ),
    ("https://github.com/0x1F980/ITS-ASSYMETRIC", "https://github.com/0x1F980/ITS-asymmetric"),
    ("Copyright (C) 2026 0x1F464.", "Copyright (C) 2026 0x1F980."),
]

SKIP = {"ITS_ECOSYSTEM.md", "its_klippe_v4_fundament.plan.md"}


def main() -> None:
    changed = 0
    for repo in REPOS:
        root = ECO / repo
        if not root.is_dir():
            continue
        for path in root.rglob("*.md"):
            if path.name in SKIP:
                continue
            if ".cursor" in path.parts or ".git" in path.parts:
                continue
            text = path.read_text(encoding="utf-8")
            orig = text
            for old, new in REPLACEMENTS:
                text = text.replace(old, new)
            if text != orig:
                path.write_text(text, encoding="utf-8")
                changed += 1
                print(f"updated {path}")
    print(f"done: {changed} files")


if __name__ == "__main__":
    main()
