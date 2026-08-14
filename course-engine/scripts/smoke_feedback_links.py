#!/usr/bin/env python3
"""smoke_feedback_links.py — L7-S2: item review → Learn module/section links.

# CLAIM: FLOOR-RAISE

For seed42 keys pack:
  1. Every item whose bank module the course DECLARES has a non-404
     module-level Learn href (web/learn/{slug}.html exists; results.js
     MODULE_LEARN_SLUGS agrees with the registry). An item on a real form whose
     module has no Learn surface is the C5 "assessed but untaught" defect and is
     an ERROR, named, never a skipped row.
  2. Section-anchor hit rate is reported (items that resolve to an existing
     heading id on that module's markdown via topic_anchors / topic_ids).
  3. learn_md heading ids must be present in the slug algorithm (h2/h3).
  4. results.js must expose itemLearnHref + render Review * Learn links.

## Where the module set comes from (bd-lt7)

From knowledge/domains.toml — the same registry build_learn.py turns into
web/data/modules_index.json (the Learn index), verify_coverage.py derives its
floors from, and verify_objectives.py derives its required set from. A domain
row's `id` IS the Learn slug (`06-power` → `web/learn/06-power.html`) and its
`order` IS the bank module number, so the map is read, not restated.

Until 2026-08-14 this file carried a hand-written module→slug table and a
`for n in range(1, 15)` report loop. The table happened to be right; the loop
printed M01–M14 and silently omitted module 15 — a leftover of the same defect
class this file's hard gate was rebased to catch. A gate written by observing
what the tree currently does encodes the tree's current defects as requirements.

The registry and results.js must agree in BOTH directions:
  - a declared module missing from MODULE_LEARN_SLUGS, or mapped to a different
    slug, or without a Learn page / content file → RED, naming the module;
  - a MODULE_LEARN_SLUGS entry for a module the registry does not declare → RED,
    naming the module. Drift either way is how a module gets assessed without
    being taught, or taught after it was retired.

## Anti-vacuous

Zero declared modules, zero keys, an empty MODULE_LEARN_SLUGS, an empty bank
export, or zero resolved module links is an ERROR. An empty scan set must never
report like a scan that ran and came back clean.

## Verdict discipline

Every check is COLLECTED first; the report — verdict line included — is composed
and printed once, at the end. No PASS is emitted ahead of work that can still
fail.

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
LEARN_MD_JS = ROOT / "web" / "assets" / "js" / "learn_md.js"
LEARN_DIR = ROOT / "web" / "learn"
CONTENT_DIR = ROOT / "web" / "content" / "modules"
KEYS_JSON = ROOT / "web" / "data" / "keys_seed42.json"
BANK_JSON = ROOT / "web" / "data" / "bank_items_seed42.json"
TOPIC_ANCHORS_JSON = ROOT / "web" / "data" / "topic_anchors.json"
DOMAINS_TOML = ROOT / "knowledge" / "domains.toml"


def load_declared_modules(domains_path: Path) -> tuple[dict[int, str], list[str]]:
    """{module_number: learn_slug}, derived from the domain registry.

    `order` is the bank module number and `id` is the Learn slug. A registry
    that is missing, malformed or empty yields zero modules AND an error —
    never a silent empty set that would make every check below vacuous.
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
            "domain registry declares zero modules (vacuous link check is ERROR)"
        )
    return declared, errors


def slugify_heading(text: str) -> str:
    """Must match learn_md.js CdcpLearnMd.slugify / build_learn.slugify_heading."""
    s = str(text or "").lower()
    s = re.sub(r"[*_`]", "", s)
    s = re.sub(r"[^\w\s-]", "", s, flags=re.UNICODE)
    s = s.strip()
    s = re.sub(r"[\s_]+", "-", s)
    s = re.sub(r"-+", "-", s).strip("-")
    return s or "section"


def extract_heading_ids(md_text: str) -> set[str]:
    used: dict[str, int] = {}
    ids: set[str] = set()
    in_fence = False
    for raw in md_text.replace("\r\n", "\n").replace("\r", "\n").split("\n"):
        stripped = raw.strip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        m = re.match(r"^(#{1,6})\s+(.*)$", stripped)
        if not m:
            continue
        title = re.sub(r"\s+#*\s*$", "", m.group(2)).strip()
        plain = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", title)
        plain = re.sub(r"[*_`]", "", plain)
        base = slugify_heading(plain)
        if base not in used:
            used[base] = 1
            hid = base
        else:
            n = used[base] + 1
            while f"{base}-{n}" in used:
                n += 1
            used[base] = n
            used[f"{base}-{n}"] = 1
            hid = f"{base}-{n}"
        ids.add(hid)
    return ids


def parse_module_learn_slugs(js_text: str) -> dict[int, str]:
    m = re.search(
        r"(?:export\s+)?const\s+MODULE_LEARN_SLUGS\s*=\s*Object\.freeze\(\s*\{([^}]+)\}\s*\)",
        js_text,
        re.S,
    )
    if not m:
        raise ValueError("MODULE_LEARN_SLUGS not found in results.js")
    found: dict[int, str] = {}
    for km in re.finditer(r"(\d+)\s*:\s*[\"']([^\"']+)[\"']", m.group(1)):
        found[int(km.group(1))] = km.group(2)
    return found


def load_bank_by_id(path: Path) -> dict[str, dict]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    items = raw if isinstance(raw, list) else raw.get("items") or []
    out: dict[str, dict] = {}
    for it in items:
        iid = it.get("id")
        if iid:
            out[str(iid)] = it
    return out


def main() -> int:
    errors: list[str] = []
    notes: list[str] = []

    # --- the module set, derived from the domain registry (bd-lt7) ---
    module_slugs, registry_errors = load_declared_modules(DOMAINS_TOML)
    errors.extend(registry_errors)

    # --- product surface checks ---
    if not RESULTS_JS.is_file():
        print("FAIL: smoke_feedback_links — missing web/assets/js/results.js")
        return 1
    js = RESULTS_JS.read_text(encoding="utf-8")
    if "function itemLearnHref" not in js and "itemLearnHref" not in js:
        errors.append("results.js missing itemLearnHref")
    if "learn_href" not in js:
        errors.append("results.js must set learn_href on item rows")
    if "Review section in Learn" not in js and "Review module in Learn" not in js:
        errors.append('results.js missing "Review … in Learn" link copy')
    if "topic_anchors.json" not in js:
        errors.append("results.js should load data/topic_anchors.json")

    if not LEARN_MD_JS.is_file():
        errors.append("missing learn_md.js")
    else:
        mdjs = LEARN_MD_JS.read_text(encoding="utf-8")
        if "function slugify" not in mdjs and "slugify:" not in mdjs:
            errors.append("learn_md.js missing slugify (stable heading anchors)")
        if 'id="' not in mdjs and "id=\\\"" not in mdjs and " id=\"" not in mdjs:
            # renderer must emit id= on headings
            if "uniqueSlug" not in mdjs and 'id="' not in mdjs:
                errors.append("learn_md.js does not appear to emit heading id attributes")

    try:
        slugs = parse_module_learn_slugs(js)
    except ValueError as e:
        print(f"FAIL: smoke_feedback_links — {e}")
        return 1
    if not slugs:
        errors.append("MODULE_LEARN_SLUGS empty — refusing vacuous green")

    # Registry → product: every declared module must be mapped and reachable.
    for n, expect in sorted(module_slugs.items()):
        if slugs.get(n) != expect:
            errors.append(
                f"module {n}: results.js slug map {slugs.get(n)!r} != "
                f"{expect!r} (knowledge/domains.toml)"
            )
        page = LEARN_DIR / f"{expect}.html"
        if not page.is_file():
            errors.append(f"missing learn page {page.relative_to(ROOT)}")
        content = CONTENT_DIR / f"{expect}.md"
        if not content.is_file():
            errors.append(f"missing content {content.relative_to(ROOT)}")

    # Product → registry: the other direction of the same drift. A Learn link
    # for a module the course no longer declares is as much a disagreement
    # between the two sources as a declared module with no link.
    for n in sorted(set(slugs) - set(module_slugs)):
        errors.append(
            f"module {n}: results.js maps {slugs[n]!r} but knowledge/domains.toml "
            f"does not declare that module"
        )

    # --- seed42 keys + bank ---
    if not KEYS_JSON.is_file():
        print("FAIL: smoke_feedback_links — missing web/data/keys_seed42.json")
        return 1
    if not BANK_JSON.is_file():
        print("FAIL: smoke_feedback_links — missing web/data/bank_items_seed42.json")
        return 1

    try:
        keys_pack = json.loads(KEYS_JSON.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        print(f"FAIL: smoke_feedback_links — keys JSON: {e}")
        return 1
    keys = keys_pack.get("keys") if isinstance(keys_pack, dict) else None
    if not keys:
        print("FAIL: smoke_feedback_links — keys_seed42 has zero keys (vacuous)")
        return 1

    bank_by_id = load_bank_by_id(BANK_JSON)
    if not bank_by_id:
        print("FAIL: smoke_feedback_links — bank_items_seed42 empty")
        return 1

    # topic anchors: refresh via build_learn so matcher + content stay in sync
    topic_anchors: dict | None = None
    try:
        import importlib.util

        bl_path = ROOT / "scripts" / "build_learn.py"
        spec = importlib.util.spec_from_file_location("cdcp_build_learn", bl_path)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load build_learn.py")
        bl = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(bl)
        navigable = [
            {"id": slug, "order": n, "epi_heading": slug}
            for n, slug in sorted(module_slugs.items())
        ]
        topic_anchors = bl.build_topic_anchors(navigable)
        TOPIC_ANCHORS_JSON.parent.mkdir(parents=True, exist_ok=True)
        TOPIC_ANCHORS_JSON.write_text(
            json.dumps(topic_anchors, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
            newline="\n",
        )
    except Exception as e:  # noqa: BLE001 — smoke reports
        if TOPIC_ANCHORS_JSON.is_file():
            try:
                topic_anchors = json.loads(
                    TOPIC_ANCHORS_JSON.read_text(encoding="utf-8")
                )
                notes.append(
                    f"  note: using existing topic_anchors.json (regen failed: {e})"
                )
            except json.JSONDecodeError as je:
                errors.append(f"topic_anchors.json invalid: {je}")
        else:
            errors.append(
                f"topic_anchors missing and regenerate failed: {e} "
                f"— run python3 scripts/build_learn.py"
            )

    heading_ids_by_slug: dict[str, set[str]] = {}
    for n, slug in sorted(module_slugs.items()):
        md_path = CONTENT_DIR / f"{slug}.md"
        if md_path.is_file():
            heading_ids_by_slug[slug] = extract_heading_ids(
                md_path.read_text(encoding="utf-8")
            )
        else:
            heading_ids_by_slug[slug] = set()

    # Verify topic_anchors anchors exist in module headings
    if topic_anchors and isinstance(topic_anchors.get("topics"), dict):
        for tid, row in topic_anchors["topics"].items():
            if not isinstance(row, dict):
                continue
            anchor = row.get("anchor")
            slug = row.get("slug")
            if not anchor or not slug:
                continue
            ids = heading_ids_by_slug.get(str(slug)) or set()
            if anchor not in ids:
                errors.append(
                    f"topic {tid}: anchor {anchor!r} not in headings of {slug}"
                )

    # --- per seed42 key: module link + section hit rate ---
    total = 0
    module_linked = 0
    section_linked = 0
    missing_module: list[str] = []
    no_bank: list[str] = []
    unmapped_modules: list[str] = []

    topics_map = (
        topic_anchors.get("topics") if topic_anchors and isinstance(topic_anchors, dict) else None
    ) or {}

    for k in keys:
        iid = str(k.get("item_id") or "")
        if not iid:
            errors.append("key entry missing item_id")
            continue
        total += 1
        item = bank_by_id.get(iid)
        if not item:
            no_bank.append(iid)
            continue
        mod = item.get("module")
        try:
            mod_n = int(mod) if mod is not None else None
        except (TypeError, ValueError):
            mod_n = None

        if mod_n is None or mod_n not in module_slugs:
            # A module on a real form with no Learn surface is the C5 defect.
            # Do not skip it — name it. (Anti-vacuous: an unmapped module must
            # not report like a linked one.)
            unmapped_modules.append(
                f"{iid}: module {mod_n!r} is not declared in knowledge/domains.toml "
                f"— assessed with no Learn surface"
            )
            continue

        slug = module_slugs.get(mod_n)
        if not slug:
            missing_module.append(f"{iid}: module {mod_n} unmapped")
            continue
        page = LEARN_DIR / f"{slug}.html"
        if not page.is_file():
            missing_module.append(f"{iid}: 404 {page.relative_to(ROOT)}")
            continue
        module_linked += 1

        # section resolution via topic_ids → topic_anchors
        topic_ids = item.get("topic_ids") or []
        if not isinstance(topic_ids, list):
            topic_ids = []
        anchor = None
        for tid in topic_ids:
            row = topics_map.get(str(tid))
            if not row or not isinstance(row, dict):
                continue
            a = row.get("anchor")
            if a and (
                row.get("module") is None or int(row.get("module")) == mod_n
            ):
                # confirm heading still exists
                if a in (heading_ids_by_slug.get(slug) or set()):
                    anchor = a
                    break
        if anchor:
            section_linked += 1

    if no_bank:
        for iid in no_bank[:10]:
            errors.append(f"key item_id not in bank_items_seed42: {iid}")
        if len(no_bank) > 10:
            errors.append(f"… and {len(no_bank) - 10} more missing bank rows")

    if missing_module:
        for msg in missing_module[:15]:
            errors.append(f"module link: {msg}")
        if len(missing_module) > 15:
            errors.append(f"… and {len(missing_module) - 15} more module-link failures")

    # Hard gate: an item on a real form whose module has no Learn surface is
    # the C5 defect and is now an ERROR, not a silently-skipped row.
    if unmapped_modules:
        for msg in unmapped_modules[:15]:
            errors.append(f"assessed but untaught: {msg}")
        if len(unmapped_modules) > 15:
            errors.append(
                f"… and {len(unmapped_modules) - 15} more items in untaught modules"
            )

    navigable_keys = total - len(unmapped_modules) - len(no_bank)
    # module_linked + len(missing_module) equals the keys whose module the
    # registry declares and which had a bank row.
    if total == 0:
        errors.append("zero keys — vacuous")
    if module_linked == 0 and navigable_keys > 0:
        errors.append("zero module-level links resolved — refusing vacuous green")

    # Require at least some section anchors when topic map exists (not 0% if topics present)
    if (
        topic_anchors
        and int(topic_anchors.get("topics_with_anchor") or 0) > 0
        and module_linked > 0
        and section_linked == 0
    ):
        errors.append(
            "section-anchor hit rate 0% despite topics_with_anchor>0 — check matcher"
        )

    hit_rate = (100.0 * section_linked / module_linked) if module_linked else 0.0

    # The verdict is decided last and the report is printed once.
    if errors:
        report = ["FAIL: smoke_feedback_links"]
        report.extend(f"  - {e}" for e in errors)
        report.extend(notes)
        report.append(
            f"  stats: keys={total} module_linked={module_linked} "
            f"section_linked={section_linked} hit_rate={hit_rate:.1f}% "
            f"unmapped_mod={len(unmapped_modules)}"
        )
        print("\n".join(report))
        return 1

    report = [
        "PASS: smoke_feedback_links",
        f"  modules={len(module_slugs)} (derived from knowledge/domains.toml)",
        f"  keys_seed42={total}",
        f"  module_level_links={module_linked} (non-404 learn/{{slug}}.html)",
        f"  section_anchor_links={section_linked}",
        f"  section_anchor_hit_rate={hit_rate:.1f}% ({section_linked}/{module_linked})",
        f"  untaught_module_items={len(unmapped_modules)} (must be 0)",
    ]
    report.extend(notes)
    if topic_anchors:
        report.append(
            f"  topic_anchors topics_with_anchor="
            f"{topic_anchors.get('topics_with_anchor')}/"
            f"{topic_anchors.get('topic_count')}"
        )
    # Every declared module, not the first fourteen of them.
    for n, slug in sorted(module_slugs.items()):
        report.append(f"  M{n:02d} → learn/{slug}.html")
    print("\n".join(report))
    return 0


if __name__ == "__main__":
    sys.exit(main())
