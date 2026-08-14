#!/usr/bin/env python3
"""verify_objectives.py — L7-S7 objective coverage gate (honest scope).

# CLAIM: FLOOR-RAISE

What this gates (product-true today):

1. registries/objectives.toml
   - non-empty [[objective]] set (vacuous empty = ERROR)
   - each objective has non-empty claim_ids
   - every claim_id resolves in registries/claims.toml

2. Domain bank coverage over the DECLARED module set
   - the module set is DERIVED from knowledge/domains.toml — the same registry
     build_learn.py turns into web/data/modules_index.json (the Learn index),
     the same one verify_coverage.py derives its floors from, and the same one
     bank_policy.toml's [[domain_min]] rows are keyed against. It is not a
     range literal.
   - every required module has ≥1 bank item (module field); empty bank = ERROR
   - a module may be held out ONLY by a recorded
     `[[coverage_exempt]] module = N, reason = "…"` row in
     knowledge/bank_policy.toml — the same one ledger verify_coverage.py reads.
     An exemption without a reason, for an undeclared module, or contradicting
     a [[domain_min]] floor, is an ERROR, not an exemption.

3. Topic coverage via bank topic_ids (practical / soft by default)
   - Count how many topics.toml rows in a REQUIRED domain have ≥ min items via
     bank topic_ids; report shortfalls always.
   - Default: shortfalls are WARNINGS (not RED) — full topic×item matrix is
     incomplete / not the same as product objectives. Use --strict-topics to
     hard-fail on primary-topic shortfalls when that floor is intentional.
   - Topics in a RECORDED-EXEMPT domain are reported only, never required.
   - A topic whose domain the registry never declared is cross-source DRIFT and
     an ERROR: the two sources disagreeing about which modules exist is exactly
     how module 15 came to be assessed without being taught.
   - Vacuous: topics.toml with zero primary topics = ERROR when file present.

## Why the derivation, and not `range(1, 15)` (bd-lt7)

Until 2026-08-14 this gate read `PRIMARY_MODULES = range(1, 15)` and skipped
module 15 as "ops-adjacent, reported only, never required". Module 15 was, at
that time, assessed but never taught — so this gate had written a KNOWN DEFECT
down as a rule. It did not go red when the defect was fixed; it stayed green,
because an exemption cannot fail. That is the worse failure: a gate that cannot
notice the thing it exists to check.

The defect was never "someone hardcoded 14". It was that the bound came from
OBSERVED STATE rather than from a stated contract. domains.toml is the contract.

The old `domains_listed < 14` soft warning went with it. It was a FLOOR, so it
never held a module out — but its comparand was the same observed count, and
once the module set is derived from domains.toml the check is tautological:
it compares the registry against itself. It is replaced by drift checks that
the registry can actually correct — an undeclared [[domain_min]] row, a topic in
an undeclared domain — and by the anti-vacuous floor below.

## Anti-vacuous

Zero declared modules, zero required modules after exemptions, zero bank items,
zero objectives, or a missing/unparseable domain registry are each an ERROR. An
empty scan set must never report like a scan that ran and came back clean.

## Verdict discipline

Every check is COLLECTED first; the report — verdict line included — is composed
and printed once, after the optional --write-json write. No PASS is ever emitted
before a path that can still raise.

What this does NOT claim (documented gap):

- registries/objectives.toml holds product-level outcomes (honesty, fluency,
  domain map, grade integrity, clean bank) — NOT per-module learning objectives.
- Bank items rarely populate objective_ids (usually []). There is no full
  LO × item matrix. When objective_ids is non-empty, ids must resolve.
- Soft topic coverage is intentional honesty: domain floor is hard; per-topic
  LO completeness is aspirational until bank tags catch up.
- A bank item in a module the registry never declared is REPORTED here, not
  failed: verify_coverage.py reports it the same way, and the hard gate on
  "assessed but untaught" is smoke_feedback_links.py, which fails any item on a
  real form whose module has no Learn surface.
- Objective coverage ≠ exam pass probability; study signal only.

Known-bad selftests (scripts/selftest_l7_objectives.sh):
  missing claim ref · empty objectives · empty claim_ids · empty bank · live GREEN

GAP (bd-lt7, 2026-08-14): that suite predates the derivation and asserts nothing
about it. The rules added here — a DECLARED module with no items goes RED naming
the module, an exemption without a reason is an ERROR, a recorded exemption with
a reason is honoured, a topic in an undeclared domain is drift, an empty registry
is anti-vacuous — are covered only by legs run by hand. They belong in the suite
(whose INJECTIONS= count is drift-guarded by verify_injection_count.py, so the
advertised total moves with them) or alongside the verify_coverage legs in
crates/cdcp_gate/tests/rebase_module_bounds.rs.

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
DEFAULT_POLICY = ROOT / "knowledge" / "bank_policy.toml"


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_declared_modules(domains_path: Path) -> tuple[dict[int, str], list[str]]:
    """The module set, derived from the domain registry. Never a range literal.

    Returns ({order: domain_id}, errors). A registry that is missing, malformed
    or empty yields zero modules AND an error — never a silent empty set that
    would make every floor below vacuously satisfied.

    Kept deliberately identical in shape to verify_coverage.load_declared_modules:
    two gates that disagree about which modules exist are two gates that can be
    played off against each other.
    """
    errors: list[str] = []
    declared: dict[int, str] = {}
    if not domains_path.is_file():
        return declared, [f"domain registry missing: {domains_path}"]
    try:
        data = load_toml(domains_path)
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
        if order in declared:
            errors.append(
                f"domains.toml: duplicate order {order} ({declared[order]} and {did})"
            )
            continue
        declared[order] = did or f"module-{order}"

    if not declared:
        errors.append(
            "domain registry declares zero modules (vacuous coverage is ERROR)"
        )
    return declared, errors


def load_exemptions(
    policy_path: Path, declared: dict[int, str]
) -> tuple[dict[int, str], list[str]]:
    """Recorded coverage exemptions: `[[coverage_exempt]] module, reason`.

    The ONE ledger, shared with verify_coverage.py. An exemption is the only
    sanctioned way to hold a declared module out of a floor, and it must say why.
    A row without a non-empty reason, for an undeclared module, or contradicting
    an explicit [[domain_min]] floor, is an ERROR — the escape hatch may not be
    quieter than the rule it escapes.
    """
    errors: list[str] = []
    exempt: dict[int, str] = {}
    if not policy_path.is_file():
        return exempt, errors
    try:
        bp = load_toml(policy_path)
    except Exception as e:  # noqa: BLE001 — fail-closed on a bad policy
        return exempt, [f"bank_policy.toml parse error: {e}"]
    floors = {
        int(r["module"])
        for r in (bp.get("domain_min") or [])
        if isinstance(r, dict)
        and str(r.get("module", "")).strip().lstrip("-").isdigit()
    }
    for row in bp.get("coverage_exempt") or []:
        if not isinstance(row, dict):
            errors.append(
                f"bank_policy.toml: coverage_exempt row is not a table: {row!r}"
            )
            continue
        try:
            mod = int(row["module"])
        except (KeyError, TypeError, ValueError):
            errors.append(
                f"bank_policy.toml: coverage_exempt row has no usable module: {row!r}"
            )
            continue
        reason = str(row.get("reason") or "").strip()
        if not reason:
            errors.append(
                f"bank_policy.toml: coverage_exempt module {mod} has no reason "
                f"(an exemption without a reason is a schema error)"
            )
            continue
        if mod not in declared:
            errors.append(
                f"bank_policy.toml: coverage_exempt module {mod} is not in the "
                f"domain registry"
            )
            continue
        if mod in floors:
            errors.append(
                f"bank_policy.toml: module {mod} is both coverage_exempt and has a "
                f"[[domain_min]] floor — pick one"
            )
            continue
        exempt[mod] = reason
    return exempt, errors


def domain_min_drift(policy_path: Path, declared: dict[int, str]) -> list[str]:
    """A [[domain_min]] row for a module the registry never declared is DRIFT.

    This gate applies its own floor of ≥1 item per required module — the sized
    floors are verify_coverage.py's job — but it reads the same policy file for
    exemptions, so it says so when the two sources disagree about which modules
    exist. That disagreement is how module 15 came to be assessed but untaught.
    """
    errors: list[str] = []
    if not policy_path.is_file():
        return errors
    try:
        bp = load_toml(policy_path)
    except Exception as e:  # noqa: BLE001
        return [f"bank_policy.toml parse error: {e}"]
    for row in bp.get("domain_min") or []:
        if not isinstance(row, dict):
            errors.append(f"bank_policy.toml: unusable [[domain_min]] row {row!r}")
            continue
        try:
            mod = int(row["module"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"bank_policy.toml: unusable [[domain_min]] row {row!r}")
            continue
        if mod not in declared:
            errors.append(
                f"bank_policy.toml: [[domain_min]] module {mod} is not declared in "
                f"the domain registry"
            )
    return errors


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
        "--policy",
        type=Path,
        default=DEFAULT_POLICY,
        help=f"bank_policy.toml path — the exemption ledger (default: {DEFAULT_POLICY})",
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
    policy_path = resolve(args.policy)
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

    # --- 2) The module set, derived from the domain registry ---
    declared, declared_errors = load_declared_modules(domains_path)
    errors.extend(declared_errors)
    exempt, exempt_errors = load_exemptions(policy_path, declared)
    errors.extend(exempt_errors)
    errors.extend(domain_min_drift(policy_path, declared))

    required = sorted(m for m in declared if m not in exempt)
    # …and a run with nothing left to require reports exactly like one that
    # checked everything and found it sound, which is the failure this rebase
    # exists to remove.
    if declared and not required:
        errors.append(
            "zero required modules after exemptions (vacuous coverage is ERROR)"
        )
    required_domain_ids = {declared[m] for m in required}
    exempt_domain_ids = {declared[m] for m in exempt if m in declared}

    def is_primary_domain(domain_id: str) -> bool:
        """A domain the registry declares and no recorded row exempts."""
        return str(domain_id or "").strip() in required_domain_ids

    # --- 3) Bank load + domain coverage over the required set ---
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
    for mod in required:
        have = module_counts.get(mod, 0)
        if have < 1:
            msg = f"domain module {mod}: {have} items < min 1"
            errors.append(msg)
            domain_shortfalls.append({"module": mod, "have": have, "min": 1})

    # Modules the bank carries that the registry never declared: reported, not
    # failed — same as verify_coverage.py. The hard gate on "assessed but
    # untaught" is smoke_feedback_links.py, which fails any item on a real form
    # whose module has no Learn surface.
    extra_modules = sorted(m for m in module_counts if m not in declared)

    # --- 4) Topic coverage (required domains) ---
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

    undeclared_topic_domains: list[str] = []
    for t in topics:
        tid = t.get("id")
        dom = t.get("domain") or ""
        if not isinstance(tid, str) or not tid.strip():
            errors.append("topic with empty/missing id")
            continue
        tid = tid.strip()
        dom = str(dom).strip() if isinstance(dom, str) else ""
        if is_primary_domain(dom):
            primary_topics.append(t)
        elif dom in exempt_domain_ids:
            optional_topics.append(t)
        else:
            # Cross-source drift: topics.toml and domains.toml disagree about
            # which modules exist. That disagreement is how module 15 came to be
            # assessed without being taught, so it is an ERROR and it names the
            # topic and the domain.
            undeclared_topic_domains.append(f"{tid} (domain={dom!r})")

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
    elif not primary_topics and topics_path.is_file() and declared:
        # topics file exists but zero topics in a required domain = ERROR
        # (anti-vacuous: an empty topic set must not pass like a covered one)
        errors.append("topics.toml has zero topics in a required domain")

    # Topics in a RECORDED-EXEMPT domain: report only, never required.
    optional_uncovered = 0
    for t in optional_topics:
        tid = str(t.get("id") or "").strip()
        if not tid:
            continue
        if topic_item_counts.get(tid, 0) < 1:
            optional_uncovered += 1

    # Drift is only meaningful against a registry that loaded; a missing or
    # empty domains.toml is already an ERROR above and must not also bury the
    # report under one line per topic.
    if undeclared_topic_domains and declared:
        for msg in undeclared_topic_domains[:20]:
            errors.append(f"topics.toml: topic in an undeclared domain: {msg}")
        if len(undeclared_topic_domains) > 20:
            errors.append(
                f"… and {len(undeclared_topic_domains) - 20} more topics in "
                f"undeclared domains"
            )

    # --- Report (composed once; the verdict is decided last) ---
    topic_mode = (
        "skipped"
        if args.skip_topic_coverage
        else ("strict" if args.strict_topics else "soft-warn")
    )
    body: list[str] = [
        "  gate=l7-objective-coverage",
        f"  objectives={objectives_path}",
        f"  claims={claims_path}",
        f"  registry={domains_path} declares={len(declared)}",
        f"  policy={'present' if policy_path.is_file() else 'absent'}",
        f"  bank={bank_dir}",
        f"  items={n_items}",
        f"  registry_objectives={len(obj_ids)} claim_resolve_ok={obj_claim_ok}",
        f"  known_claims={len(known_claims)}",
        f"  modules ({len(required)} required, derived from {domains_path.name}; "
        f"min 1 item each):",
    ]
    for mod in required:
        have = module_counts.get(mod, 0)
        flag = "ok" if have >= 1 and n_items > 0 else "SHORT"
        body.append(f"    m{mod:02d}: {have} [{flag}]")
    if exempt:
        body.append("  recorded exemptions (bank_policy.toml [[coverage_exempt]]):")
        for mod in sorted(exempt):
            body.append(f"    m{mod:02d}: {module_counts.get(mod, 0)} — exempt: {exempt[mod]}")
    if extra_modules:
        body.append("  undeclared modules present in the bank (not required for green):")
        for mod in extra_modules:
            body.append(f"    m{mod:02d}: {module_counts[mod]} (not in the domain registry)")
    body.extend(
        [
            f"  primary_topics={len(primary_topics)} "
            f"covered={len(primary_topics) - uncovered_primary} "
            f"shortfalls={uncovered_primary} "
            f"min_per_topic={min_topic} mode={topic_mode}",
            f"  exempt_domain_topics={len(optional_topics)} "
            f"uncovered={optional_uncovered} (not required)",
            f"  bank_items_with_objective_ids={items_with_objective_ids} "
            f"(of {n_items}; product-level objectives, not per-module LOs)",
            "  gap: no full LO×item matrix — objectives.toml is product outcomes + claim_ids",
            "  note: coverage ≠ exam pass probability; study signal only",
        ]
    )

    if warnings:
        body.append("  warnings:")
        for w in warnings[:20]:
            body.append(f"    - {w}")
        if len(warnings) > 20:
            body.append(f"    ... +{len(warnings) - 20} more")

    # JSON summary
    try:
        bank_rel = str(bank_dir.resolve().relative_to(ROOT.resolve()))
    except ValueError:
        bank_rel = str(bank_dir)

    def summary_for(status_word: str) -> dict:
        return {
            "schema_version": 2,
            "gate": "l7-objective-coverage",
            "status": status_word.lower(),
            "bank": bank_rel,
            "item_count": n_items,
            "module_source": domains_path.name,
            "declared_modules": sorted(declared),
            "required_modules": required,
            "exemptions": {str(k): v for k, v in sorted(exempt.items())},
            "registry_objectives": {
                "count": len(obj_ids),
                "ids": obj_ids,
                "claim_resolve_ok": obj_claim_ok,
            },
            "known_claims": len(known_claims),
            "domain_counts": {str(m): module_counts.get(m, 0) for m in required},
            "extra_counts": {str(m): module_counts[m] for m in extra_modules},
            "domain_shortfalls": domain_shortfalls,
            "primary_topics": len(primary_topics),
            "primary_topic_shortfalls": topic_shortfalls[:100],
            "primary_topic_shortfall_count": uncovered_primary,
            "exempt_domain_topics_uncovered": optional_uncovered,
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

    # The write happens BEFORE any verdict is printed, and a failed write is a
    # failure of this gate — not a traceback under a PASS someone already read.
    if args.write_json is not None:
        out = args.write_json
        if not out.is_absolute():
            out = (ROOT / out).resolve()
        try:
            out.parent.mkdir(parents=True, exist_ok=True)
            provisional = "PASS" if not errors else "FAIL"
            out.write_text(
                json.dumps(summary_for(provisional), indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
            body.append(f"  wrote {out}")
        except OSError as e:
            errors.append(f"could not write summary to {out}: {e}")

    # The verdict is the LAST thing decided and the first thing on a report that
    # is printed exactly once, after every path that could still raise.
    status = "PASS" if not errors else "FAIL"
    report = [status]
    report.extend(body)
    if errors:
        report.append("  failures:")
        for e in errors[:50]:
            report.append(f"    - {e}")
        if len(errors) > 50:
            report.append(f"    ... +{len(errors) - 50} more")
    else:
        span = " ".join(f"m{m:02d}" for m in required)
        if uncovered_primary and not args.strict_topics and not args.skip_topic_coverage:
            report.append(
                f"  objective coverage GREEN "
                f"(registry claims + {len(required)} required modules: {span}; "
                f"{uncovered_primary} topic shortfalls soft-warn)"
            )
        else:
            report.append(
                f"  objective coverage GREEN "
                f"(registry claims + {len(required)} required modules: {span} "
                f"+ primary topics)"
            )
    print("\n".join(report))
    return 1 if errors else 0


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
