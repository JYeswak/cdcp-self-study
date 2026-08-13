#!/usr/bin/env python3
"""verify_objectives.py — L7-S7 objective coverage gate (honest scope).

What this gates (product-true today):

1. registries/objectives.toml
   - non-empty [[objective]] set (vacuous empty = ERROR)
   - each objective has non-empty claim_ids
   - every claim_id resolves in registries/claims.toml

2. Domain bank coverage (modules 1–14)
   - every primary domain has ≥1 bank item (module field)
   - empty bank = ERROR

3. Topic coverage via bank topic_ids (practical / soft by default)
   - Count how many primary-domain (01–14) topics.toml rows have ≥ min items
     via bank topic_ids; report shortfalls always.
   - Default: shortfalls are WARNINGS (not RED) — full topic×item matrix is
     incomplete / not the same as product objectives. Use --strict-topics to
     hard-fail on primary-topic shortfalls when that floor is intentional.
   - Domain-15 (ops-adjacent) topics are reported only, never required.
   - Vacuous: topics.toml with zero primary topics = ERROR when file present.

What this does NOT claim (documented gap):

- registries/objectives.toml holds product-level outcomes (honesty, fluency,
  domain map, grade integrity, clean bank) — NOT per-module learning objectives.
- Bank items rarely populate objective_ids (usually []). There is no full
  LO × item matrix. When objective_ids is non-empty, ids must resolve.
- Soft topic coverage is intentional honesty: domain floor is hard; per-topic
  LO completeness is aspirational until bank tags catch up.
- Objective coverage ≠ exam pass probability; study signal only.

Known-bad selftests (scripts/selftest_l7_objectives.sh):
  missing claim ref · empty objectives · empty bank · live GREEN

Exit 0 on PASS; non-zero on FAIL.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OBJECTIVES = ROOT / "registries" / "objectives.toml"
DEFAULT_CLAIMS = ROOT / "registries" / "claims.toml"
DEFAULT_TOPICS = ROOT / "knowledge" / "topics.toml"
DEFAULT_DOMAINS = ROOT / "knowledge" / "domains.toml"
DEFAULT_BANK = ROOT / "bank" / "items"

PRIMARY_MODULES = range(1, 15)  # 1–14 inclusive
PRIMARY_DOMAIN_PREFIXES = tuple(f"{m:02d}-" for m in PRIMARY_MODULES)


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_items(bank_dir: Path) -> tuple[list[tuple[str, dict]], list[str]]:
    """Load item dicts from *.toml under bank_dir. Returns (loaded, errors)."""
    errors: list[str] = []
    loaded: list[tuple[str, dict]] = []
    if not bank_dir.is_dir():
        return loaded, [f"bank dir missing: {bank_dir}"]

    for path in sorted(bank_dir.glob("*.toml")):
        try:
            data = load_toml(path)
        except Exception as e:  # noqa: BLE001 — fail-closed per file
            errors.append(f"{path.name}: parse error: {e}")
            continue
        if "items" in data and isinstance(data["items"], list):
            for it in data["items"]:
                if isinstance(it, dict):
                    loaded.append((path.name, it))
        elif "id" in data:
            loaded.append((path.name, data))
        else:
            errors.append(f"{path.name}: no id or items[]")
    return loaded, errors


def claim_ids_from_registry(claims: dict) -> set[str]:
    ids: set[str] = set()
    for row in claims.get("claim") or []:
        cid = row.get("id")
        if isinstance(cid, str) and cid.strip():
            ids.add(cid.strip())
    return ids


def is_primary_domain(domain_id: str) -> bool:
    return any(domain_id.startswith(p) for p in PRIMARY_DOMAIN_PREFIXES)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="L7-S7 objective coverage gate (registry objectives + domain/topic coverage)"
    )
    ap.add_argument(
        "--objectives",
        type=Path,
        default=DEFAULT_OBJECTIVES,
        help=f"objectives.toml path (default: {DEFAULT_OBJECTIVES})",
    )
    ap.add_argument(
        "--claims",
        type=Path,
        default=DEFAULT_CLAIMS,
        help=f"claims.toml path (default: {DEFAULT_CLAIMS})",
    )
    ap.add_argument(
        "--topics",
        type=Path,
        default=DEFAULT_TOPICS,
        help=f"topics.toml path (default: {DEFAULT_TOPICS})",
    )
    ap.add_argument(
        "--domains",
        type=Path,
        default=DEFAULT_DOMAINS,
        help=f"domains.toml path (default: {DEFAULT_DOMAINS})",
    )
    ap.add_argument(
        "--bank",
        type=Path,
        default=DEFAULT_BANK,
        help=f"bank items directory (default: {DEFAULT_BANK})",
    )
    ap.add_argument(
        "--min-items-per-topic",
        type=int,
        default=1,
        help="min bank items citing each primary-domain topic_id (default 1)",
    )
    ap.add_argument(
        "--strict-topics",
        action="store_true",
        help="hard-fail when a primary-domain topic has < min items (default: warn)",
    )
    ap.add_argument(
        "--skip-topic-coverage",
        action="store_true",
        help="only check registry objectives + domain modules (topic floor off)",
    )
    ap.add_argument(
        "--write-json",
        type=Path,
        default=None,
        help="optional path to write machine-readable summary",
    )
    args = ap.parse_args(argv)

    def resolve(p: Path) -> Path:
        return p if p.is_absolute() else (ROOT / p).resolve()

    objectives_path = resolve(args.objectives)
    claims_path = resolve(args.claims)
    topics_path = resolve(args.topics)
    domains_path = resolve(args.domains)
    bank_dir = resolve(args.bank)
    min_topic = max(0, int(args.min_items_per_topic))

    errors: list[str] = []
    warnings: list[str] = []

    # --- 1) Registry files present ---
    if not objectives_path.is_file():
        errors.append(f"missing objectives registry: {objectives_path}")
    if not claims_path.is_file():
        errors.append(f"missing claims registry: {claims_path}")
    if errors:
        print("FAIL")
        for e in errors:
            print(f"  - {e}")
        return 1

    try:
        objectives_doc = load_toml(objectives_path)
    except Exception as e:  # noqa: BLE001
        print("FAIL")
        print(f"  - parse objectives: {e}")
        return 1
    try:
        claims_doc = load_toml(claims_path)
    except Exception as e:  # noqa: BLE001
        print("FAIL")
        print(f"  - parse claims: {e}")
        return 1

    known_claims = claim_ids_from_registry(claims_doc)
    if not known_claims:
        errors.append("registries/claims.toml has zero [[claim]] rows (empty = ERROR)")

    objectives = objectives_doc.get("objective") or []
    if not objectives:
        errors.append(
            "registries/objectives.toml has zero [[objective]] rows (empty = ERROR)"
        )

    obj_ids: list[str] = []
    obj_claim_ok = 0
    for o in objectives:
        oid = o.get("id") if isinstance(o, dict) else None
        if not oid or not isinstance(oid, str) or not oid.strip():
            errors.append("objective with empty/missing id")
            continue
        oid = oid.strip()
        obj_ids.append(oid)
        cids = o.get("claim_ids") or []
        if not isinstance(cids, list) or not cids:
            errors.append(f"objective {oid}: claim_ids empty (must cite ≥1 claim)")
            continue
        all_ok = True
        for cid in cids:
            if not isinstance(cid, str) or not cid.strip():
                errors.append(f"objective {oid}: empty claim_id entry")
                all_ok = False
                continue
            cid = cid.strip()
            if cid not in known_claims:
                errors.append(
                    f"objective {oid}: unresolved claim_id {cid!r} "
                    f"(not in registries/claims.toml)"
                )
                all_ok = False
        if all_ok:
            obj_claim_ok += 1

    if len(obj_ids) != len(set(obj_ids)):
        dups = [i for i, c in Counter(obj_ids).items() if c > 1]
        errors.append(f"duplicate objective ids: {dups}")

    known_objectives = set(obj_ids)

    # --- 2) Bank load + domain coverage (modules 1–14) ---
    loaded, load_errors = load_items(bank_dir)
    errors.extend(load_errors)
    n_items = len(loaded)
    if n_items == 0:
        errors.append("empty bank: zero items loaded (vacuous coverage is ERROR)")

    module_counts: Counter[int] = Counter()
    topic_item_counts: Counter[str] = Counter()
    objective_item_counts: Counter[str] = Counter()
    items_with_objective_ids = 0

    for fname, it in loaded:
        iid = it.get("id") or fname
        mod = it.get("module")
        try:
            mi = int(mod)
            module_counts[mi] += 1
        except (TypeError, ValueError):
            errors.append(f"{iid}: bad module {mod!r}")

        tids = it.get("topic_ids") or []
        if isinstance(tids, list):
            for t in tids:
                if isinstance(t, str) and t.strip():
                    topic_item_counts[t.strip()] += 1

        oids = it.get("objective_ids") or []
        if isinstance(oids, list) and oids:
            items_with_objective_ids += 1
            for oid in oids:
                if not isinstance(oid, str) or not oid.strip():
                    errors.append(f"{iid}: empty objective_ids entry")
                    continue
                oid = oid.strip()
                objective_item_counts[oid] += 1
                if oid not in known_objectives:
                    errors.append(
                        f"{iid}: unknown objective_id {oid!r} "
                        f"(not in registries/objectives.toml)"
                    )

    domain_shortfalls: list[dict] = []
    for mod in PRIMARY_MODULES:
        have = module_counts.get(mod, 0)
        if have < 1:
            msg = f"domain module {mod}: {have} items < min 1"
            errors.append(msg)
            domain_shortfalls.append({"module": mod, "have": have, "min": 1})

    # --- 3) Topic coverage (primary domains) ---
    topics: list[dict] = []
    primary_topics: list[dict] = []
    optional_topics: list[dict] = []
    topic_shortfalls: list[dict] = []
    if topics_path.is_file():
        try:
            topics_doc = load_toml(topics_path)
            topics = [t for t in (topics_doc.get("topic") or []) if isinstance(t, dict)]
        except Exception as e:  # noqa: BLE001
            errors.append(f"parse topics: {e}")
    else:
        errors.append(f"missing topics registry: {topics_path}")

    for t in topics:
        tid = t.get("id")
        dom = t.get("domain") or ""
        if not isinstance(tid, str) or not tid.strip():
            errors.append("topic with empty/missing id")
            continue
        tid = tid.strip()
        if isinstance(dom, str) and is_primary_domain(dom):
            primary_topics.append(t)
        else:
            optional_topics.append(t)

    uncovered_primary = 0
    if not args.skip_topic_coverage and min_topic > 0 and primary_topics:
        for t in primary_topics:
            tid = str(t["id"]).strip()
            have = topic_item_counts.get(tid, 0)
            if have < min_topic:
                uncovered_primary += 1
                topic_shortfalls.append(
                    {
                        "topic_id": tid,
                        "domain": t.get("domain"),
                        "have": have,
                        "min": min_topic,
                    }
                )
                msg = (
                    f"topic {tid}: {have} items < min {min_topic} "
                    f"(domain={t.get('domain')})"
                )
                if args.strict_topics:
                    errors.append(msg)
                else:
                    warnings.append(msg)
    elif not primary_topics and topics_path.is_file():
        # topics file exists but zero primary topics = ERROR (anti-vacuous)
        errors.append("topics.toml has zero primary-domain (01–14) topics")

    # Optional domain-15 topics: report only
    optional_uncovered = 0
    for t in optional_topics:
        tid = str(t.get("id") or "").strip()
        if not tid:
            continue
        if topic_item_counts.get(tid, 0) < 1:
            optional_uncovered += 1

    # Domains file: soft consistency (primary domains listed)
    domains_listed = 0
    if domains_path.is_file():
        try:
            dom_doc = load_toml(domains_path)
            for d in dom_doc.get("domain") or []:
                did = d.get("id") if isinstance(d, dict) else None
                if isinstance(did, str) and is_primary_domain(did):
                    domains_listed += 1
        except Exception as e:  # noqa: BLE001
            warnings.append(f"domains.toml parse warning: {e}")
        if domains_listed < 14:
            warnings.append(
                f"domains.toml primary domains listed={domains_listed} (expected 14)"
            )
    else:
        warnings.append(f"domains.toml missing at {domains_path} (soft)")

    # --- Report ---
    status = "PASS" if not errors else "FAIL"
    print(status)
    print("  gate=l7-objective-coverage")
    print(f"  objectives={objectives_path}")
    print(f"  claims={claims_path}")
    print(f"  bank={bank_dir}")
    print(f"  items={n_items}")
    print(f"  registry_objectives={len(obj_ids)} claim_resolve_ok={obj_claim_ok}")
    print(f"  known_claims={len(known_claims)}")
    print("  domain modules (1–14, min 1 item each):")
    for mod in PRIMARY_MODULES:
        have = module_counts.get(mod, 0)
        flag = "ok" if have >= 1 and n_items > 0 else "SHORT"
        print(f"    m{mod:02d}: {have} [{flag}]")
    topic_mode = (
        "skipped"
        if args.skip_topic_coverage
        else ("strict" if args.strict_topics else "soft-warn")
    )
    print(
        f"  primary_topics={len(primary_topics)} "
        f"covered={len(primary_topics) - uncovered_primary} "
        f"shortfalls={uncovered_primary} "
        f"min_per_topic={min_topic} mode={topic_mode}"
    )
    print(
        f"  optional_topics(domain15+)={len(optional_topics)} "
        f"uncovered={optional_uncovered} (not required)"
    )
    print(
        f"  bank_items_with_objective_ids={items_with_objective_ids} "
        f"(of {n_items}; product-level objectives, not per-module LOs)"
    )
    print("  gap: no full LO×item matrix — objectives.toml is product outcomes + claim_ids")
    print("  note: coverage ≠ exam pass probability; study signal only")

    if warnings:
        print("  warnings:")
        for w in warnings[:20]:
            print(f"    - {w}")
        if len(warnings) > 20:
            print(f"    ... +{len(warnings) - 20} more")

    # JSON summary
    try:
        bank_rel = str(bank_dir.resolve().relative_to(ROOT.resolve()))
    except ValueError:
        bank_rel = str(bank_dir)

    summary = {
        "schema_version": 1,
        "gate": "l7-objective-coverage",
        "status": status.lower(),
        "bank": bank_rel,
        "item_count": n_items,
        "registry_objectives": {
            "count": len(obj_ids),
            "ids": obj_ids,
            "claim_resolve_ok": obj_claim_ok,
        },
        "known_claims": len(known_claims),
        "domain_counts": {str(m): module_counts.get(m, 0) for m in PRIMARY_MODULES},
        "domain_shortfalls": domain_shortfalls,
        "primary_topics": len(primary_topics),
        "primary_topic_shortfalls": topic_shortfalls[:100],
        "primary_topic_shortfall_count": uncovered_primary,
        "optional_topics_uncovered": optional_uncovered,
        "items_with_objective_ids": items_with_objective_ids,
        "min_items_per_topic": min_topic,
        "strict_topics": bool(args.strict_topics),
        "skip_topic_coverage": bool(args.skip_topic_coverage),
        "topic_mode": topic_mode,
        "gap": (
            "objectives.toml holds product-level outcomes with claim_ids, "
            "not per-module learning objectives; bank topic_ids are the LO proxy; "
            "primary topic shortfalls soft-warn unless --strict-topics"
        ),
        "note": "Objective coverage ≠ exam pass probability; study signal only.",
        "errors": errors[:80],
        "warnings": warnings,
    }

    if args.write_json is not None:
        out = args.write_json
        if not out.is_absolute():
            out = (ROOT / out).resolve()
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"  wrote {out}")

    if errors:
        print("  failures:")
        for e in errors[:50]:
            print(f"    - {e}")
        if len(errors) > 50:
            print(f"    ... +{len(errors) - 50} more")
        return 1

    if uncovered_primary and not args.strict_topics and not args.skip_topic_coverage:
        print(
            f"  objective coverage GREEN "
            f"(registry claims + domains 1–14; {uncovered_primary} topic shortfalls soft-warn)"
        )
    else:
        print(
            "  objective coverage GREEN (registry claims + domains 1–14 + primary topics)"
        )
    return 0


def _selftest_known_bad() -> int:
    """In-process L4-style known-bad (TEMP only). Env: CDCP_OBJECTIVES_SELFTEST=1."""
    import os
    import shutil
    import tempfile

    if os.environ.get("CDCP_OBJECTIVES_SELFTEST") != "1":
        return -1  # not requested

    tmp = Path(tempfile.mkdtemp(prefix="selftest_l7_obj_"))
    try:
        # (a) empty objectives
        empty_obj = tmp / "objectives_empty.toml"
        empty_obj.write_text(
            'schema_version = 1\n\n[registry]\nname = "objectives"\n',
            encoding="utf-8",
        )
        rc = main(
            [
                "--objectives",
                str(empty_obj),
                "--claims",
                str(DEFAULT_CLAIMS),
                "--bank",
                str(DEFAULT_BANK),
                "--skip-topic-coverage",
            ]
        )
        if rc == 0:
            print("SELFTEST FAIL: empty objectives stayed GREEN", file=sys.stderr)
            return 2
        print("selftest: empty objectives RED ok")

        # (b) missing claim ref
        bad_obj = tmp / "objectives_bad_claim.toml"
        bad_obj.write_text(
            "\n".join(
                [
                    "schema_version = 1",
                    "",
                    "[[objective]]",
                    'id = "obj-selftest-unresolved"',
                    'text = "planted"',
                    'claim_ids = ["claim-does-not-exist-selftest-only"]',
                    "",
                ]
            ),
            encoding="utf-8",
        )
        rc = main(
            [
                "--objectives",
                str(bad_obj),
                "--claims",
                str(DEFAULT_CLAIMS),
                "--bank",
                str(DEFAULT_BANK),
                "--skip-topic-coverage",
            ]
        )
        if rc == 0:
            print("SELFTEST FAIL: missing claim ref stayed GREEN", file=sys.stderr)
            return 2
        print("selftest: missing claim ref RED ok")

        # (c) empty bank
        empty_bank = tmp / "empty_bank"
        empty_bank.mkdir()
        rc = main(
            [
                "--objectives",
                str(DEFAULT_OBJECTIVES),
                "--claims",
                str(DEFAULT_CLAIMS),
                "--bank",
                str(empty_bank),
                "--skip-topic-coverage",
            ]
        )
        if rc == 0:
            print("SELFTEST FAIL: empty bank stayed GREEN", file=sys.stderr)
            return 2
        print("selftest: empty bank RED ok")

        # (d) live GREEN
        rc = main([])
        if rc != 0:
            print("SELFTEST FAIL: live tree not GREEN", file=sys.stderr)
            return 2
        print("selftest: live GREEN ok")
        print("CDCP_OBJECTIVES_SELFTEST: PASSED")
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    st = _selftest_known_bad()
    if st >= 0:
        sys.exit(st)
    sys.exit(main())
