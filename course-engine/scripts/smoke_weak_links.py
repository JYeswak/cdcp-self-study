#!/usr/bin/env python3
"""smoke_weak_links.py — L6-S3: every navigable module maps to a Learn page.

Asserts:
  1. MODULE_LEARN_SLUGS in web/assets/js/results.js covers every navigable module
  2. Each mapped slug has web/learn/{slug}.html
  3. modules_index.json order→id agrees with the JS map (when index present)
  4. moduleLearnHref shape is learn/XX-slug.html for mapped modules; null outside

The range was 1–14 until 2026-08-15, when module 15 (ops-adjacent) was taught
rather than excluded (bd-hardening-c-status-hzs.4, CHARTER §11 row 8). A hard
1–14 bound here would have rejected the fix as "unexpected keys", which is what
it did on first run — so the bound now follows EXPECTED_SLUGS.

Exit 0 PASS · non-zero FAIL. Empty/missing map = ERROR (no vacuous green).
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RESULTS_JS = ROOT / "web" / "assets" / "js" / "results.js"
LEARN_DIR = ROOT / "web" / "learn"
INDEX_JSON = ROOT / "web" / "data" / "modules_index.json"

# Hardcoded expected slugs (must match web/learn/*.html + modules_index order).
EXPECTED_SLUGS: dict[int, str] = {
    1: "01-mission-critical",
    2: "02-standards",
    3: "03-site-building",
    4: "04-floor-ceiling",
    5: "05-lighting",
    6: "06-power",
    7: "07-emf",
    8: "08-racks",
    9: "09-cooling",
    10: "10-water",
    11: "11-network",
    12: "12-fire",
    13: "13-security",
    14: "14-auxiliary",
    15: "15-ops-adjacent",
}


def parse_module_learn_slugs(js_text: str) -> dict[int, str]:
    """Extract MODULE_LEARN_SLUGS object from results.js (simple numeric keys)."""
    m = re.search(
        r"export\s+const\s+MODULE_LEARN_SLUGS\s*=\s*Object\.freeze\(\s*\{([^}]+)\}\s*\)",
        js_text,
        re.S,
    )
    if not m:
        # allow non-export form
        m = re.search(
            r"(?:export\s+)?const\s+MODULE_LEARN_SLUGS\s*=\s*Object\.freeze\(\s*\{([^}]+)\}\s*\)",
            js_text,
            re.S,
        )
    if not m:
        raise ValueError("MODULE_LEARN_SLUGS Object.freeze({...}) not found in results.js")

    body = m.group(1)
    found: dict[int, str] = {}
    for km in re.finditer(r"(\d+)\s*:\s*[\"']([^\"']+)[\"']", body):
        found[int(km.group(1))] = km.group(2)
    return found


def main() -> int:
    errors: list[str] = []

    if not RESULTS_JS.is_file():
        print("FAIL: smoke_weak_links — missing web/assets/js/results.js")
        return 1

    js = RESULTS_JS.read_text(encoding="utf-8")
    try:
        slugs = parse_module_learn_slugs(js)
    except ValueError as e:
        print(f"FAIL: smoke_weak_links — {e}")
        return 1

    if not slugs:
        errors.append("MODULE_LEARN_SLUGS is empty — refusing vacuous green")

    # --- cover every navigable module exactly ---
    for n in EXPECTED_SLUGS:
        if n not in slugs:
            errors.append(f"module {n}: missing from MODULE_LEARN_SLUGS")
        elif slugs[n] != EXPECTED_SLUGS[n]:
            errors.append(
                f"module {n}: map slug {slugs[n]!r} != expected {EXPECTED_SLUGS[n]!r}"
            )

    extra = set(slugs) - set(EXPECTED_SLUGS)
    if extra:
        errors.append(f"unexpected MODULE_LEARN_SLUGS keys: {sorted(extra)}")

    # --- moduleLearnHref helper present ---
    if "function moduleLearnHref" not in js and "moduleLearnHref" not in js:
        errors.append("moduleLearnHref helper missing from results.js")
    if "Review weak modules in Learn" not in js:
        errors.append('CTA copy "Review weak modules in Learn" missing from results.js')
    # deep-link chip shape
    if "weak-chip--link" not in js and 'href="' not in js:
        errors.append("weak module chips do not appear to emit learn hrefs")
    if "moduleLearnHref" not in js or "learn/" not in js:
        errors.append("results.js must call moduleLearnHref / emit learn/… hrefs")

    # --- files exist under web/learn/ ---
    if not LEARN_DIR.is_dir():
        errors.append(f"missing learn dir {LEARN_DIR.relative_to(ROOT)}")
    else:
        for n, slug in EXPECTED_SLUGS.items():
            page = LEARN_DIR / f"{slug}.html"
            if not page.is_file():
                errors.append(
                    f"module {n}: mapped file missing {page.relative_to(ROOT)}"
                )

    # --- optional consistency with modules_index.json ---
    if INDEX_JSON.is_file():
        try:
            index = json.loads(INDEX_JSON.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            errors.append(f"modules_index.json invalid JSON: {e}")
            index = None
        if index is not None:
            for m in index.get("modules") or []:
                order = m.get("order")
                mid = m.get("id")
                empty = m.get("empty") is True
                if empty or order is None:
                    continue
                try:
                    n = int(order)
                except (TypeError, ValueError):
                    continue
                if n not in EXPECTED_SLUGS:
                    errors.append(
                        f"modules_index has navigable order={n} id={mid!r} with no "
                        f"MODULE_LEARN_SLUGS entry — a learner cannot reach it from results"
                    )
                    continue
                if EXPECTED_SLUGS.get(n) != mid:
                    errors.append(
                        f"modules_index order={n} id={mid!r} "
                        f"!= expected slug {EXPECTED_SLUGS.get(n)!r}"
                    )
                href = m.get("href") or ""
                want = f"learn/{EXPECTED_SLUGS[n]}.html"
                if href and href != want:
                    errors.append(
                        f"modules_index order={n} href={href!r} != {want!r}"
                    )
    else:
        # Index optional for this smoke; map+files are the hard gate.
        pass

    if errors:
        print("FAIL: smoke_weak_links")
        for e in errors:
            print(f"  - {e}")
        return 1

    print("PASS: smoke_weak_links")
    print(f"  modules_mapped={len(EXPECTED_SLUGS)}")
    print(f"  learn_dir={LEARN_DIR.relative_to(ROOT)}")
    for n in sorted(EXPECTED_SLUGS):
        print(f"  M{n:02d} → learn/{EXPECTED_SLUGS[n]}.html")
    return 0


if __name__ == "__main__":
    sys.exit(main())
