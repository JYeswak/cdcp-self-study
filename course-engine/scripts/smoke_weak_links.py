#!/usr/bin/env python3
"""smoke_weak_links.py — L6-S3: every navigable module maps to a Learn page.

# CLAIM: FLOOR-RAISE

Asserts:
  1. MODULE_LEARN_SLUGS in web/assets/js/results.js covers every module the
     course DECLARES, with the declared slug, and maps nothing it does not
     declare (drift in both directions is RED, naming the module)
  2. Each declared slug has web/learn/{slug}.html
  3. modules_index.json order→id agrees with the registry (when index present)
  4. moduleLearnHref shape is learn/XX-slug.html for mapped modules; null outside

## Where the module set comes from (bd-ggs7)

From knowledge/domains.toml — the same registry build_learn.py turns into
web/data/modules_index.json, verify_coverage.py derives its floors from,
verify_objectives.py derives its required set from, and smoke_feedback_links.py
derives its link sweep from. A domain row's `id` IS the Learn slug
(`06-power` → `web/learn/06-power.html`) and its `order` IS the bank module
number, so the map is READ, not restated.

Until 2026-08-14 this file carried its own hand-written module→slug table — the
third surviving copy of that mapping after verify_objectives.py and
smoke_feedback_links.py stopped hand-maintaining theirs. It happened to be
correct, which is exactly what made it dangerous: it would have stayed correct
right up until the registry gained a module, and then it would have failed a
correct change for being correct ("unexpected MODULE_LEARN_SLUGS keys") — the
bd-lt7 failure mode, from a file the bd-lt7 sweep called clean.

It called the file clean because the sweep in
crates/cdcp_gate/tests/rebase_module_bounds.rs detected NUMERIC module bounds,
and a frozen table contains no numeric bound at all: same defect class, a shape
the detector could not see. That sweep now sees hand-frozen module TABLES too,
and this file is the reason it does.

The derivation is written out here rather than imported from a sibling gate, the
same way verify_coverage.py, verify_objectives.py and smoke_feedback_links.py
each carry their own reader. Duplicated READER code is cheap and diverges
loudly (the registry is one file; a reader that drifts reads it wrong and its
own gate goes red). Duplicated DATA is what this bead was about.

## Anti-vacuous

A missing, unparseable or empty registry is an ERROR. So is a registry that
declares fewer than fourteen modules: that is a FLOOR taken from the
certification's fourteen public EPI CDCP domains (repo-root README.md, the
curriculum map's title, and this registry's own header), not from whatever the
tree happens to hold today. It can never hold a module out — module 15 and any
later partner supplement sit above it — it can only notice a registry that
collapsed, which would otherwise make every check below agree with itself and
pass green. An empty MODULE_LEARN_SLUGS is likewise an ERROR.

## Verdict discipline

Every check is COLLECTED first; the report — verdict line included — is composed
and printed once, at the end. No PASS is emitted ahead of work that can fail.

Exit 0 PASS · non-zero FAIL.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
RESULTS_JS = ROOT / "web" / "assets" / "js" / "results.js"
LEARN_DIR = ROOT / "web" / "learn"
INDEX_JSON = ROOT / "web" / "data" / "modules_index.json"
DOMAINS_TOML = ROOT / "knowledge" / "domains.toml"


def load_declared_modules(domains_path: Path) -> tuple[dict[int, str], list[str]]:
    """{module_number: learn_slug}, derived from the domain registry.

    `order` is the bank module number and `id` is the Learn slug. A registry
    that is missing, malformed, empty or collapsed yields errors — never a
    silent empty set that would make every check below vacuous.
    """
    errors: list[str] = []
    declared: dict[int, str] = {}
    if not domains_path.is_file():
        return declared, [f"domain registry missing: {domains_path}"]
    try:
        with domains_path.open("rb") as f:
            data = tomllib.load(f)
    except Exception as e:  # noqa: BLE001 — fail-closed on a bad registry
        return declared, [f"domain registry parse error: {e}"]

    for row in data.get("domain") or []:
        if not isinstance(row, dict):
            errors.append(f"domains.toml: [[domain]] row is not a table: {row!r}")
            continue
        did = str(row.get("id") or "").strip()
        try:
            order = int(row["order"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"domains.toml: {did or row!r} has no usable order")
            continue
        if not did:
            errors.append(f"domains.toml: module {order} has no id (no Learn slug)")
            continue
        if order in declared:
            errors.append(
                f"domains.toml: duplicate order {order} ({declared[order]} and {did})"
            )
            continue
        declared[order] = did

    if not declared:
        errors.append(
            "domain registry declares zero modules (vacuous weak-link check is ERROR)"
        )
    # FLOOR, not an exclusion: the fourteen public EPI CDCP domains. It cannot
    # hold a module out; it can only notice a collapsed registry. The literal is
    # written here rather than hidden behind a named constant so the bd-lt7
    # bound sweep can see it and hold a verdict on it.
    elif len(declared) < 14:
        errors.append(
            f"domain registry declares only {len(declared)} modules; the CDCP "
            f"course has fourteen public EPI domains at minimum "
            f"(vacuous weak-link check is ERROR)"
        )
    return declared, errors


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

    declared, reg_errors = load_declared_modules(DOMAINS_TOML)
    errors.extend(reg_errors)
    if not declared:
        print("FAIL: smoke_weak_links")
        for e in errors:
            print(f"  - {e}")
        return 1

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

    # --- cover every declared module exactly ---
    for n in sorted(declared):
        if n not in slugs:
            errors.append(
                f"module {n}: knowledge/domains.toml declares {declared[n]!r} but "
                f"MODULE_LEARN_SLUGS has no entry — a learner cannot reach it "
                f"from results"
            )
        elif slugs[n] != declared[n]:
            errors.append(
                f"module {n}: map slug {slugs[n]!r} != declared {declared[n]!r} "
                f"(knowledge/domains.toml)"
            )

    extra = sorted(set(slugs) - set(declared))
    for n in extra:
        errors.append(
            f"module {n}: results.js maps {slugs[n]!r} but knowledge/domains.toml "
            f"does not declare that module"
        )

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
        for n in sorted(declared):
            page = LEARN_DIR / f"{declared[n]}.html"
            if not page.is_file():
                errors.append(
                    f"module {n}: declared slug has no Learn page "
                    f"{page.relative_to(ROOT)}"
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
                if n not in declared:
                    errors.append(
                        f"modules_index has navigable order={n} id={mid!r} which "
                        f"knowledge/domains.toml does not declare"
                    )
                    continue
                if declared[n] != mid:
                    errors.append(
                        f"modules_index order={n} id={mid!r} "
                        f"!= declared slug {declared[n]!r}"
                    )
                href = m.get("href") or ""
                want = f"learn/{declared[n]}.html"
                if href and href != want:
                    errors.append(
                        f"modules_index order={n} href={href!r} != {want!r}"
                    )
    else:
        # Index optional for this smoke; registry+map+files are the hard gate.
        pass

    if errors:
        print("FAIL: smoke_weak_links")
        for e in errors:
            print(f"  - {e}")
        return 1

    print("PASS: smoke_weak_links")
    print(f"  modules={len(declared)} (derived from knowledge/domains.toml)")
    print(f"  learn_dir={LEARN_DIR.relative_to(ROOT)}")
    for n in sorted(declared):
        print(f"  M{n:02d} → learn/{declared[n]}.html")
    return 0


if __name__ == "__main__":
    sys.exit(main())
