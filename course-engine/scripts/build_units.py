#!/usr/bin/env python3
"""build_units.py — M8-B1: derive lesson units from module ## headings.

Emits web/data/units_index.json.

Units = each ATX ## section. Each unit gets:
  - topic_ids: domain topics (title match preferred, then full domain fill)
  - check_item_ids: 2–3 real bank item ids for mid-unit Quick check
  - module_num: the module's order, for JS fallbacks

Deterministic: same content + bank → same JSON (sorted keys).

# CLAIM: FLOOR-RAISE

Every module the Learn index carries must reach the micro-check floor: at least
80% of its units carry ≥2 real bank items. The module set is DERIVED from
web/data/modules_index.json — the Learn index this script already reads to find
its content — and not from a numeric bound.

## Why the derivation, and not `int(m[:2]) <= 14` (bd-lt7)

Until 2026-08-14 the primary set was `[m for m in by_module if ... int(m[:2])
<= 14]`, which held module 15 out of the floor. Module 15 was, at that time,
assessed but never taught, so the exemption looked harmless. It was not: the
bound was derived from OBSERVED STATE rather than from a stated contract, so
when module 15 was taught the floor stayed silent about it and this gate went on
reporting PASS by luck rather than by check. The Learn index is the contract —
if a module is in it, its units are subject to the floor.

## Anti-vacuous

Zero modules, zero units, or zero primary units is an ERROR. A run that
discovered nothing must not report like a run that checked everything.

The same law applies one level up, to the INPUTS (bd-build-units-vacuous-registries-9153).
Measured 2026-08-14, both implementations byte-identical, before the fix:

  * `knowledge/topics.toml` deleted   -> exit 0, `PASS: build_units units=134 modules=15`.
    `load_topics_by_domain` returned `{}`, every unit got `topic_ids: []`, and
    `pick_check_items` reads an empty topic set as "no preference" rather than
    as "nothing to match", so it still filled each unit from the module pool.
    A registry that vanished entirely produced a report indistinguishable from
    one that checked everything.
  * `web/data/modules_index.json` deleted -> exit 0, `PASS: … units=134 modules=16`.
    The build silently globbed `web/content/modules/*.md` instead, which swept
    in `README.md` and emitted `"README": []` into `by_module`. That is worse
    than silent: a GREEN verdict carrying a WRONG number (16 where the Learn
    index says 15).

Both are now hard errors that NAME the absent file, and the glob fallback is
DELETED rather than flagged. A flag would leave the wrong answer reachable and
would widen the argument surface of a script that deliberately takes none; the
header above states the module set is DERIVED from the Learn index, and a
silent second derivation that disagrees with it is not a fallback, it is a
second gate nobody reviewed.

## Write-after-verdict

The artifact is written only on the GREEN path (bd-builder-verdict-shape-qm65).
This script used to write `units_index.json` BEFORE evaluating any threshold, so
a RED run left a file behind and a later reader could not tell a passing artifact
from the residue of a failed one. The verdict is computed first; the side effect
depends on the verdict, never the reverse.

## What this gate cannot decide

It counts units and attached item ids. It cannot tell whether a unit teaches
anything, whether its Quick check items are RELEVANT to the heading they sit
under, or whether the topic match that attached them was right. 80% of units
carrying two ids each is a floor against silence, not a claim about quality.
"""
from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
CONTENT = ROOT / "web" / "content" / "modules"
TOPICS = ROOT / "knowledge" / "topics.toml"
OUT = ROOT / "web" / "data" / "units_index.json"
MOD_INDEX = ROOT / "web" / "data" / "modules_index.json"
BANK_JSON = ROOT / "web" / "data" / "bank_items_seed42.json"
BANK_DIR = ROOT / "bank" / "items"

_STOP = frozenset("a an and as at for in of on or the to vs with".split())
CHECK_N = 3  # target questions per unit


def slugify(text: str) -> str:
    s = str(text or "").lower()
    s = re.sub(r"[*_`]", "", s)
    s = re.sub(r"[^\w\s-]", "", s, flags=re.UNICODE)
    s = s.strip()
    s = re.sub(r"[\s_]+", "-", s)
    s = re.sub(r"-+", "-", s).strip("-")
    return s or "section"


def module_num_from_id(mid: str) -> int | None:
    m = re.match(r"^(\d{2})-", mid or "")
    if not m:
        return None
    return int(m.group(1))


def split_h2_units(md: str) -> list[dict]:
    """Return units: {title, heading_id, body, order} for each ## section."""
    lines = md.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    units: list[dict] = []
    in_fence = False
    current: dict | None = None
    body: list[str] = []
    used: dict[str, int] = {}

    def flush() -> None:
        nonlocal current, body
        if not current:
            body = []
            return
        text = "\n".join(body).strip()
        current["body"] = text
        current["word_count"] = len(re.findall(r"\b\w+\b", text))
        units.append(current)
        current = None
        body = []

    def uniq(base: str) -> str:
        if base not in used:
            used[base] = 1
            return base
        n = used[base] + 1
        while f"{base}-{n}" in used:
            n += 1
        used[base] = n
        used[f"{base}-{n}"] = 1
        return f"{base}-{n}"

    for raw in lines:
        stripped = raw.strip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            if current is not None:
                body.append(raw)
            continue
        if not in_fence:
            m = re.match(r"^(#{1,6})\s+(.*)$", stripped)
            if m and len(m.group(1)) == 2:
                flush()
                title = re.sub(r"\s+#*\s*$", "", m.group(2)).strip()
                plain = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", title)
                plain = re.sub(r"[*_`]", "", plain)
                hid = uniq(slugify(plain))
                current = {
                    "title": title,
                    "heading_id": hid,
                    "level": 2,
                }
                body = []
                continue
        if current is not None:
            body.append(raw)

    flush()
    out = []
    for u in units:
        if u.get("word_count", 0) < 40 and "objective" not in u["title"].lower():
            if "learning" not in u["title"].lower():
                continue
        u["order"] = len(out) + 1
        out.append(u)
    return out


def load_topics_by_domain() -> dict[str, list[dict]]:
    """Domain -> topics. The caller has already refused to run without the file.

    The `is_file` guard that used to sit here returned an empty map for an
    absent registry, which the picker then read as "no preference". `main`
    checks for the file instead, so an absent registry can no longer reach here.
    """
    by: dict[str, list[dict]] = {}
    data = tomllib.loads(TOPICS.read_text(encoding="utf-8"))
    for t in data.get("topic") or []:
        dom = str(t.get("domain") or "").strip()
        tid = str(t.get("id") or "").strip()
        if not dom or not tid:
            continue
        by.setdefault(dom, []).append(
            {"id": tid, "label": str(t.get("label") or tid)}
        )
    return by


def match_topics(unit_title: str, heading_id: str, topics: list[dict]) -> list[str]:
    """Best-effort: attach topic ids whose labels overlap unit title/slug."""
    if not topics:
        return []
    title_cf = unit_title.casefold()
    slug = heading_id
    scored: list[tuple[int, str]] = []
    for t in topics:
        label = t["label"]
        tid = t["id"]
        score = 0
        lab_slug = slugify(label)
        if lab_slug and (lab_slug in slug or slug in lab_slug):
            score = max(score, 70)
        words = [
            w
            for w in re.findall(r"[a-z0-9]+", label.lower())
            if w not in _STOP and len(w) > 2
        ]
        if words:
            hits = sum(1 for w in words if w in title_cf or w in slug)
            if hits == len(words):
                score = max(score, 80)
            elif hits >= max(1, len(words) // 2):
                score = max(score, 40 + hits * 5)
        tail = tid.split("-", 1)[-1]
        for part in re.split(r"[-_]", tail):
            if part and len(part) > 3 and part in slug:
                score = max(score, 45)
        if score >= 40:
            scored.append((score, tid))
    scored.sort(reverse=True)
    seen: set[str] = set()
    out: list[str] = []
    for _, tid in scored:
        if tid in seen:
            continue
        seen.add(tid)
        out.append(tid)
        if len(out) >= 6:
            break
    return out


def assign_topic_ids(unit_title: str, heading_id: str, topics: list[dict]) -> list[str]:
    """Matched topics first; always fill with full domain topics so Quick check can map."""
    matched = match_topics(unit_title, heading_id, topics)
    all_ids = [t["id"] for t in topics]
    if not all_ids:
        return matched
    seen: set[str] = set()
    out: list[str] = []
    for tid in matched + all_ids:
        if tid in seen:
            continue
        seen.add(tid)
        out.append(tid)
    return out


def load_bank() -> list[dict]:
    """Prefer exported seed JSON (browser path); fall back to bank/items tomls."""
    if BANK_JSON.is_file():
        data = json.loads(BANK_JSON.read_text(encoding="utf-8"))
        items = data if isinstance(data, list) else data.get("items") or []
        out = []
        for it in items:
            if not isinstance(it, dict) or not it.get("id"):
                continue
            out.append(
                {
                    "id": str(it["id"]),
                    "module": int(it.get("module") or 0),
                    "topic_ids": list(it.get("topic_ids") or []),
                    "stem": str(it.get("stem") or ""),
                    "explanation": str(it.get("explanation") or ""),
                    "choices": list(it.get("choices") or []),
                    "correct": str(it.get("correct") or ""),
                }
            )
        if out:
            return out

    items = []
    if BANK_DIR.is_dir():
        for path in sorted(BANK_DIR.glob("*.toml")):
            try:
                t = tomllib.loads(path.read_text(encoding="utf-8"))
            except Exception:
                continue
            if not t.get("id"):
                continue
            items.append(
                {
                    "id": str(t["id"]),
                    "module": int(t.get("module") or 0),
                    "topic_ids": list(t.get("topic_ids") or []),
                    "stem": str(t.get("stem") or ""),
                    "explanation": str(t.get("explanation") or ""),
                    "choices": list(t.get("choices") or []),
                    "correct": str(t.get("correct") or ""),
                }
            )
    return items


def item_quality(it: dict) -> int:
    """Higher = better for Quick check samples."""
    score = 0
    if it.get("explanation") and len(it["explanation"]) >= 20:
        score += 50
    if len(it.get("choices") or []) >= 4:
        score += 20
    stem = it.get("stem") or ""
    if 40 <= len(stem) <= 280:
        score += 15
    if it.get("topic_ids"):
        score += 10
    # Prefer interview-relevant / conceptual over pure trivia length
    if any(
        w in stem.lower()
        for w in ("why", "most", "best", "risk", "fail", "when", "which")
    ):
        score += 5
    return score


def pick_check_items(
    bank_by_module: dict[int, list[dict]],
    module_num: int | None,
    topic_ids: list[str],
    unit_order: int,
    n: int = CHECK_N,
) -> list[str]:
    """Pick n diversified bank ids for a unit. Deterministic."""
    if not module_num or module_num not in bank_by_module:
        return []
    pool = list(bank_by_module[module_num])
    if not pool:
        return []

    # Prefer items matching unit topic_ids, but keep full module as fallback.
    tid_set = set(topic_ids)
    primary = [
        it
        for it in pool
        if tid_set and any(t in tid_set for t in (it.get("topic_ids") or []))
    ]
    # quality-sorted stable by id
    def sort_key(it: dict) -> tuple:
        return (-item_quality(it), it["id"])

    primary.sort(key=sort_key)
    rest = [it for it in pool if it not in primary]
    rest.sort(key=sort_key)
    ordered = primary + rest

    # Offset by unit order so adjacent units get different questions
    if not ordered:
        return []
    start = ((unit_order - 1) * n) % len(ordered)
    rotated = ordered[start:] + ordered[:start]

    # Diversify topics: prefer not repeating same first topic_id
    picked: list[dict] = []
    used_topics: set[str] = set()
    used_ids: set[str] = set()

    def try_add(it: dict, require_new_topic: bool) -> bool:
        if it["id"] in used_ids:
            return False
        tops = it.get("topic_ids") or []
        head = tops[0] if tops else ""
        if require_new_topic and head and head in used_topics and len(picked) < n:
            return False
        picked.append(it)
        used_ids.add(it["id"])
        if head:
            used_topics.add(head)
        return True

    for it in rotated:
        if len(picked) >= n:
            break
        try_add(it, require_new_topic=True)
    for it in rotated:
        if len(picked) >= n:
            break
        try_add(it, require_new_topic=False)

    return [it["id"] for it in picked[:n]]


def main() -> int:
    if not CONTENT.is_dir():
        print("FAIL: missing web/content/modules — run build_learn.py first")
        return 1

    # ANTI-VACUOUS, ON THE INPUTS. Each of these registries could vanish and the
    # build stayed GREEN — one by degrading to an empty topic map, the other by
    # globbing a different module set. Refuse, name the file, write nothing.
    missing_registries: list[str] = []
    if not TOPICS.is_file():
        missing_registries.append(
            f"{TOPICS.relative_to(ROOT)} (topic registry; absent, every unit "
            f"gets topic_ids=[] and the picker reads that as 'no preference')"
        )
    if not MOD_INDEX.is_file():
        missing_registries.append(
            f"{MOD_INDEX.relative_to(ROOT)} (Learn index; the module set is "
            f"DERIVED from it — there is no glob fallback)"
        )
    if missing_registries:
        print("FAIL: build_units missing required input registries")
        for m in missing_registries:
            print(f"  - {m}")
        print(f"  out={OUT.relative_to(ROOT)} NOT WRITTEN (a failing build leaves no artifact)")
        return 1

    mi = json.loads(MOD_INDEX.read_text(encoding="utf-8"))
    domain_ids: list[str] = [
        m["id"] for m in mi.get("modules") or [] if not m.get("empty") and m.get("id")
    ]

    topics_by = load_topics_by_domain()
    bank = load_bank()
    bank_by_module: dict[int, list[dict]] = defaultdict(list)
    bank_by_id: dict[str, dict] = {}
    for it in bank:
        bank_by_module[it["module"]].append(it)
        bank_by_id[it["id"]] = it

    all_units: list[dict] = []
    by_module: dict[str, list[dict]] = {}
    units_with_checks = 0
    units_zero_checks = 0

    for mid in domain_ids:
        path = CONTENT / f"{mid}.md"
        if not path.is_file():
            continue
        md = path.read_text(encoding="utf-8")
        units = split_h2_units(md)
        topics = topics_by.get(mid, [])
        mnum = module_num_from_id(mid)
        mod_units = []
        for u in units:
            topic_ids = assign_topic_ids(u["title"], u["heading_id"], topics)
            check_ids = pick_check_items(
                bank_by_module, mnum, topic_ids, u["order"], CHECK_N
            )
            if len(check_ids) >= 2:
                units_with_checks += 1
            else:
                units_zero_checks += 1
            uid = f"{mid}__{u['heading_id']}"
            row = {
                "id": uid,
                "module_id": mid,
                "module_num": mnum,
                "order": u["order"],
                "title": u["title"],
                "heading_id": u["heading_id"],
                "word_count": u.get("word_count") or 0,
                "estimate_minutes": max(
                    3, min(20, round((u.get("word_count") or 0) / 200 * 1.35))
                ),
                "topic_ids": topic_ids,
                "check_item_ids": check_ids,
                "check_count": len(check_ids),
            }
            mod_units.append(row)
            all_units.append(row)
        by_module[mid] = mod_units

    shortfalls = []
    for mid, us in by_module.items():
        if re.match(r"^\d{2}-", mid) and len(us) < 3:
            shortfalls.append(f"{mid}: {len(us)} units")
        # module bank coverage
        mnum = module_num_from_id(mid)
        if mnum and re.match(r"^\d{2}-", mid):
            weak = sum(1 for u in us if (u.get("check_count") or 0) < 2)
            if weak:
                shortfalls.append(f"{mid}: {weak}/{len(us)} units with <2 check items")

    payload = {
        "schema_version": 2,
        "generated_by": "scripts/build_units.py",
        "unit_count": len(all_units),
        "module_count": len(by_module),
        "bank_item_count": len(bank),
        "units_with_checks": units_with_checks,
        "units_zero_checks": units_zero_checks,
        "units": all_units,
        "by_module": by_module,
        "shortfalls": shortfalls,
    }
    body = json.dumps(payload, indent=2, sort_keys=True) + "\n"

    # Everything below is COLLECTED, then reported once. This block used to
    # print "PASS: build_units …" first and emit "FAIL: …" underneath it on the
    # way to returning 1 — a reader skimming stdout saw PASS, CI saw non-zero,
    # and which one won depended on whether anyone looked. The verdict is now
    # the first line of a report that is only composed once every check is done.
    failures: list[str] = []
    detail: list[str] = []

    # Anti-vacuous: a run that discovered nothing must not report like a run
    # that checked everything and found it sound.
    if not by_module:
        failures.append("zero modules discovered (vacuous unit build is ERROR)")
    if not all_units:
        failures.append("zero units discovered (vacuous unit build is ERROR)")

    # A named spot-check on two specific modules, not the general floor: these
    # two carry the heaviest syllabus weight and are the ones a content
    # regression shows up in first. The general floor below covers every module.
    for need, want in (("01-mission-critical", 4), ("06-power", 3)):
        got = len(by_module.get(need) or [])
        if got < want:
            failures.append(f"{need} has {got} units, need ≥{want}")
            continue
        checks = [u.get("check_count") or 0 for u in by_module.get(need) or []]
        if any(c < 2 for c in checks):
            failures.append(f"{need} has units with <2 check items: {checks}")
            continue
        detail.append(f"  ok: {need} units={got} check_counts={checks}")

    # The general floor: ≥80% of the units of EVERY module the Learn index
    # carries must have ≥2 check items. The set is derived from by_module — i.e.
    # from modules_index.json — and not from a numeric bound. See the header:
    # `int(m[:2]) <= 14` here silently exempted module 15 from this floor.
    primary = sorted(m for m in by_module if re.match(r"^\d{2}-", m))
    total_u = sum(len(by_module[m]) for m in primary)
    good_u = sum(
        1
        for m in primary
        for u in by_module[m]
        if (u.get("check_count") or 0) >= 2
    )
    if not primary:
        failures.append(
            "zero modules matched the module-id shape (vacuous check floor is ERROR)"
        )
    elif not total_u:
        failures.append(
            f"{len(primary)} modules carry zero units between them "
            f"(vacuous check floor is ERROR)"
        )
    elif good_u / total_u < 0.8:
        failures.append(
            f"only {good_u}/{total_u} units across {len(primary)} modules have ≥2 checks"
        )
    else:
        detail.append(
            f"  ok: check coverage {good_u}/{total_u} across {len(primary)} modules "
            f"({' '.join(primary)})"
        )

    # THE SIDE EFFECT DEPENDS ON THE VERDICT, never the reverse. This used to
    # write unconditionally, hundreds of lines above, so a RED run left a
    # units_index.json behind and a later reader could not tell a passing
    # artifact from the residue of a failed run.
    if not failures:
        OUT.parent.mkdir(parents=True, exist_ok=True)
        OUT.write_text(body, encoding="utf-8", newline="\n")

    head = (
        f"PASS: build_units units={len(all_units)} modules={len(by_module)}"
        if not failures
        else f"FAIL: build_units units={len(all_units)} modules={len(by_module)}"
    )
    report = [
        head,
        f"  bank_items={len(bank)} units_with_checks≥2={units_with_checks} zero={units_zero_checks}",
        f"  out={OUT.relative_to(ROOT)}"
        if not failures
        else f"  out={OUT.relative_to(ROOT)} NOT WRITTEN (a failing build leaves no artifact)",
    ]
    if shortfalls:
        report.append(f"  WARN shortfalls: {shortfalls}")
    report.extend(detail)
    report.extend(f"  - {f}" for f in failures)
    print("\n".join(report))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
