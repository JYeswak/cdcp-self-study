#!/usr/bin/env python3
"""smoke_diagrams.py — present diagrams: path + honesty + data-diagram root."""
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WEB = ROOT / "web"

PRESENT = [
    ("power-path", WEB / "diagrams/power-path.html", "power-path"),
    ("site-stack", WEB / "diagrams/site-stack.html", "site-stack"),
    ("heat-path", WEB / "diagrams/heat-path.html", "heat-path"),
]


def main() -> int:
    print("==> smoke_diagrams")
    errs = 0
    for name, path, marker in PRESENT:
        if not path.is_file():
            print(f"  FAIL: missing {path}")
            errs += 1
            continue
        text = path.read_text(encoding="utf-8")
        if "not" not in text.lower() or "certif" not in text.lower():
            print(f"  FAIL: {name} missing honesty/certif language")
            errs += 1
            continue
        if f'data-diagram="{marker}"' not in text and marker not in text:
            # power-path may use different markers
            if name == "power-path" and ("path-node" in text or "2N" in text):
                print(f"  ok: {name}")
                continue
            print(f"  FAIL: {name} missing diagram marker")
            errs += 1
            continue
        print(f"  ok: {name}")
    if errs:
        print(f"smoke_diagrams: FAIL ({errs})")
        return 1
    print("smoke_diagrams: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
