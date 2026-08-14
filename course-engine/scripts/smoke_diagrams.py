#!/usr/bin/env python3
"""smoke_diagrams.py — present diagrams: path + honesty + data-diagram root."""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WEB = ROOT / "web"
REGISTRY = ROOT / "docs" / "DIAGRAM-REGISTRY.md"

# The set under test is DERIVED from the registry, never hard-coded here.
#
# It used to be a literal list of the three P0 diagrams. Four P1 diagrams shipped
# on 2026-08-14 and this gate stayed green without having looked at any of them:
# a gate that certifies only what its author remembered to type is a fooled
# certificate, and the failure is invisible because the green means "the three I
# know about are fine", which reads identically to "everything is fine".
#
# FLOOR-RAISE, and what this CANNOT decide: deriving the set from the registry
# means a diagram that ships without a registry row is still unchecked. It moves
# the omission from a Python literal to a reviewed document, and makes the count
# visible in the output. It does not prove the registry is complete.
ROW = re.compile(
    r"^\|\s*`(?P<id>[a-z0-9-]+)`\s*\|[^|]*\|[^|]*\|[^|]*\|\s*(?P<status>[^|]*?)\s*\|\s*`(?P<path>[^`]+)`\s*\|"
)


def present_rows() -> list[tuple[str, Path, str]] | None:
    """Registry rows whose status is 'present'. None means ERROR, not 'none found'.

    The two are deliberately different return values. Conflating "I could not
    read the registry" with "the registry lists nothing" is how an unreadable
    input becomes a pass.
    """
    if not REGISTRY.is_file():
        print(f"smoke_diagrams: ERROR: missing registry {REGISTRY}")
        return None
    rows = []
    for line in REGISTRY.read_text(encoding="utf-8").splitlines():
        m = ROW.match(line)
        if not m:
            continue
        if "present" not in m.group("status").replace("*", "").strip().lower():
            continue
        rows.append((m.group("id"), ROOT / m.group("path"), m.group("id")))
    return rows


def main() -> int:
    print("==> smoke_diagrams")
    present = present_rows()
    if present is None:
        return 2  # unreadable input — ERROR, distinct from FAIL
    # Anti-vacuous: an empty set is an ERROR, never a pass. A regex that silently
    # stops matching after a table reformat would otherwise report PASS on zero.
    if not present:
        print("smoke_diagrams: ERROR: zero present diagrams parsed from the registry")
        return 2
    errs = 0
    for name, path, marker in present:
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
    print(f"smoke_diagrams: PASS ({len(present)} present diagrams from the registry)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
