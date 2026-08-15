#!/usr/bin/env python3
"""smoke_learn_v2.py — units index + unit shell + glossary + concept card assets."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WEB = ROOT / "web"
errs: list[str] = []


def fail(m: str) -> None:
    errs.append(m)
    print("  FAIL:", m)


def ok(m: str) -> None:
    print("  ok:", m)


def main() -> int:
    print("==> smoke_learn_v2 (M8 B/D assets)")
    units = WEB / "data/units_index.json"
    if not units.is_file():
        fail("missing units_index.json — run `cdcp build-units`")
    else:
        d = json.loads(units.read_text(encoding="utf-8"))
        by = d.get("by_module") or {}
        for mid, need in (("01-mission-critical", 4), ("06-power", 3)):
            n = len(by.get(mid) or [])
            if n < need:
                fail(f"{mid} units={n} need ≥{need}")
            else:
                ok(f"{mid} units={n}")
        if (d.get("unit_count") or 0) < 50:
            fail("unit_count too low")
        else:
            ok(f"unit_count={d.get('unit_count')}")
        # Quick-check coverage: units must carry real bank item ids
        units = d.get("units") or []
        with_checks = 0
        thin = []
        for u in units:
            n = len(u.get("check_item_ids") or [])
            if n >= 2:
                with_checks += 1
            else:
                thin.append(u.get("id"))
        if units and with_checks / len(units) < 0.8:
            fail(
                f"check_item_ids coverage {with_checks}/{len(units)} < 80% "
                f"(sample thin: {thin[:5]})"
            )
        else:
            ok(f"check_item_ids coverage {with_checks}/{len(units)}")
        m01 = (d.get("by_module") or {}).get("01-mission-critical") or []
        if m01 and min(len(u.get("check_item_ids") or []) for u in m01) < 2:
            fail("M01 unit missing ≥2 check_item_ids")
        else:
            ok("M01 every unit has ≥2 check items")

    gloss = WEB / "data/glossary.json"
    if not gloss.is_file():
        fail("missing glossary.json")
    else:
        g = json.loads(gloss.read_text(encoding="utf-8"))
        if (g.get("term_count") or 0) < 15:
            fail("glossary term_count < 15")
        else:
            ok(f"glossary terms={g.get('term_count')}")

    for rel in (
        "assets/js/learn_units.js",
        "assets/js/learn_glossary.js",
        "assets/js/concept_card.js",
    ):
        if not (WEB / rel).is_file():
            fail(f"missing {rel}")
        else:
            ok(rel)

    m01 = (WEB / "learn/01-mission-critical.html").read_text(encoding="utf-8")
    for needle in ("learn-unit-shell", "learn_units.js", "learn_glossary.js"):
        if needle not in m01:
            fail(f"M01 missing {needle}")
        else:
            ok(f"M01 {needle}")

    drill = (WEB / "drill.html").read_text(encoding="utf-8")
    if "concept_card.js" not in drill:
        fail("drill.html missing concept_card.js")
    else:
        ok("drill concept_card")

    if errs:
        print("smoke_learn_v2: FAIL")
        return 1
    print("smoke_learn_v2: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
