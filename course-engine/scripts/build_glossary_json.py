#!/usr/bin/env python3
"""Extract glossary terms from GLOSSARY.md → web/data/glossary.json for popovers."""
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

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(
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
        + "\n",
        encoding="utf-8",
        newline="\n",
    )
    print(f"PASS: glossary terms={len(terms)} → {OUT.relative_to(ROOT)}")
    if len(terms) < 15:
        print("FAIL: need ≥15 terms")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
