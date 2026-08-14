#!/usr/bin/env python3
"""Extract glossary terms from GLOSSARY.md → web/data/glossary.json for popovers.

## Verdict shape (bd-builder-verdict-shape-qm65)

The verdict is the FIRST line of a report composed only once every check is
done. Measured 2026-08-14, before the fix, on an empty glossary — both this
script and its Rust port, byte for byte:

    PASS: glossary terms=0 → web/data/glossary.json
    FAIL: need ≥15 terms
    (exit 1)

A human skimming stdout saw PASS; CI saw non-zero; which one won depended on
whether anyone looked. This is the same defect bd-lt7 fixed in build_units.py,
which makes it a CLASS rather than a bug: a verdict printed before the checks
that decide it. No line claiming success may be emitted on a path that returns
non-zero.

## Write-after-verdict (same bead)

The artifact is written only on the GREEN path. This script used to write
glossary.json BEFORE evaluating the term floor, so a below-floor run left a
short glossary.json in web/data/ and a later reader could not tell a passing
artifact from the residue of a failed run. The side effect depends on the
verdict, never the reverse.

## Anti-vacuous

Zero terms is an ERROR, and so is a term count below MIN_TERMS. A run that
parsed nothing must not report like a run that parsed everything.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "web" / "content" / "reference" / "GLOSSARY.md"
if not SRC.is_file():
    SRC = ROOT.parent / "reference" / "GLOSSARY.md"
OUT = ROOT / "web" / "data" / "glossary.json"

MIN_TERMS = 15


def main() -> int:
    if not SRC.is_file():
        print(f"FAIL: missing glossary at {SRC}")
        return 1
    text = SRC.read_text(encoding="utf-8")
    terms: dict[str, str] = {}
    # | **Term** | Definition |
    for m in re.finditer(
        r"\|\s*\*\*([^*]+)\*\*\s*\|\s*([^|]+)\|",
        text,
    ):
        term = m.group(1).strip()
        defn = m.group(2).strip()
        if term.lower() in ("term", "---"):
            continue
        if not defn or defn.startswith("---"):
            continue
        terms[term] = defn
        # also bare key without parens for matching
        bare = re.sub(r"\s*\([^)]*\)\s*", " ", term).strip()
        if bare != term:
            terms.setdefault(bare, defn)

    body = (
        json.dumps(
            {
                "schema_version": 1,
                "generated_by": "scripts/build_glossary_json.py",
                "source": str(SRC.relative_to(ROOT)) if SRC.is_relative_to(ROOT) else str(SRC),
                "term_count": len(terms),
                "terms": dict(sorted(terms.items(), key=lambda x: x[0].casefold())),
            },
            indent=2,
            ensure_ascii=False,
        )
        + "\n"
    )

    # The verdict is decided BEFORE anything is printed and before anything is
    # written. See the header: this block used to write, then print PASS, then
    # print FAIL underneath it on the way to returning 1.
    failures: list[str] = []
    if len(terms) < MIN_TERMS:
        failures.append(f"need ≥{MIN_TERMS} terms, got {len(terms)}")

    if failures:
        report = [f"FAIL: glossary terms={len(terms)}"]
        report.extend(f"  - {f}" for f in failures)
        report.append(
            f"  out={OUT.relative_to(ROOT)} NOT WRITTEN (a failing build leaves no artifact)"
        )
        print("\n".join(report))
        return 1

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(body, encoding="utf-8", newline="\n")
    print(f"PASS: glossary terms={len(terms)} → {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
