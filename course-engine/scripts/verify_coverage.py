#!/usr/bin/env python3
"""verify_coverage.py — L6 domain coverage oracle.

# CLAIM: FLOOR-RAISE

Every module the course DECLARES must carry at least N bank items:
  - N from knowledge/bank_policy.toml [[domain_min]] min_items when present
  - else N=1 (OQ-05 ASSUMED floor)

The module set is DERIVED from knowledge/domains.toml — the same registry
build_learn.py turns into web/data/modules_index.json (the Learn index) and the
same one bank_policy.toml's [[domain_min]] rows are keyed against. It is not a
range literal.

## Why the derivation, and not `range(1, 15)` (bd-lt7)

Until 2026-08-14 this gate read `PRIMARY_MODULES = range(1, 15)` and module 15
was exempted from the floor with the comment "may appear in counts but is not
required for green". Module 15 was, at that time, assessed but never taught —
so the gate had written a KNOWN DEFECT down as a rule. When module 15 was
taught, three sibling gates went RED for the fix being correct. This one did not
go red; it stayed green by LUCK, because an exemption cannot fail. That is worse:
a gate that cannot notice the thing it was supposed to check.

The defect was never "someone hardcoded 14". It was that the bound was derived
from OBSERVED STATE rather than from a stated contract. domains.toml is the
contract. If a module must be exempt, that is a RECORDED row —
`[[coverage_exempt]] module = N, reason = "..."` in bank_policy.toml, with a
reason string — and an exemption without a reason is an ERROR, not a default.

## Anti-vacuous

Zero modules discovered is an ERROR. Zero items loaded is an ERROR. An empty
scan set must never report like a scan that ran and came back clean. That rule
holds at FILE granularity too: a single bank file whose `items[]` yields zero
items is named and is RED, because the aggregate count would otherwise stay
healthy on the strength of the files around it (bd-0czh).

## What this gate cannot decide

It counts items, not coverage: twenty near-identical items satisfy a floor of
twenty. It says nothing about whether an item is correct, well written, or
mapped to the right topic, and nothing about exam pass probability. A module
above its floor is a module that is not STARVED, which is all that is claimed.

Exit 0 with per-module counts; non-zero if the bank is empty, the registry is
empty, an exemption is malformed, or any required module is below N.

Optional: --write-json PATH writes a machine-readable summary (e.g. web/data/coverage.json).
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
DEFAULT_BANK = ROOT / "bank" / "items"
DEFAULT_POLICY = ROOT / "knowledge" / "bank_policy.toml"
DEFAULT_DOMAINS = ROOT / "knowledge" / "domains.toml"

DEFAULT_N = 1  # OQ-05 ASSUMED


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_declared_modules(domains_path: Path) -> tuple[dict[int, str], list[str]]:
    """The module set, derived from the domain registry. Never a range literal.

    Returns ({order: domain_id}, errors). A registry that is missing, malformed,
    or empty yields zero modules AND an error — never a silent empty set that
    would make every floor below vacuously satisfied.
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

    An exemption is the ONLY sanctioned way to hold a declared module out of the
    floor, and it must say why. A row without a non-empty reason, for an
    undeclared module, or contradicting an explicit [[domain_min]] floor, is an
    ERROR — the escape hatch may not be quieter than the rule it escapes.
    """
    errors: list[str] = []
    exempt: dict[int, str] = {}
    if not policy_path.is_file():
        return exempt, errors
    bp = load_toml(policy_path)
    floors = {
        int(r["module"])
        for r in (bp.get("domain_min") or [])
        if isinstance(r, dict) and str(r.get("module", "")).strip().lstrip("-").isdigit()
    }
    for row in bp.get("coverage_exempt") or []:
        if not isinstance(row, dict):
            errors.append(f"bank_policy.toml: coverage_exempt row is not a table: {row!r}")
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


def load_domain_mins(
    policy_path: Path, required: list[int]
) -> tuple[dict[int, int], list[str]]:
    """Per-module floors from [[domain_min]]; default N=1 when absent.

    A [[domain_min]] row for a module the registry does not declare is an ERROR:
    the two sources of truth for "which modules exist" have drifted, and that
    drift is exactly how module 15 came to be assessed without being taught.
    """
    errors: list[str] = []
    mins: dict[int, int] = {m: DEFAULT_N for m in required}
    if not policy_path.is_file():
        return mins, errors
    bp = load_toml(policy_path)
    rows = bp.get("domain_min") or []
    for row in rows:
        try:
            mod = int(row["module"])
            need = int(row["min_items"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"bank_policy.toml: unusable [[domain_min]] row {row!r}")
            continue
        if mod in mins:
            mins[mod] = max(1, need)  # never below OQ-05 floor of 1
        else:
            errors.append(
                f"bank_policy.toml: [[domain_min]] module {mod} is not a required "
                f"module in the domain registry"
            )
    return mins, errors


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
            before = len(loaded)
            for it in data["items"]:
                if isinstance(it, dict):
                    loaded.append((path.name, it))
            if len(loaded) == before:
                # Anti-vacuous at FILE granularity (bd-0czh, the class sweep of
                # bd-2kr). `items = []` — or an items[] holding nothing this loop
                # can read as an item — takes the list branch, adds nothing, and
                # never reaches the `no id or items[]` leg below, because `elif`
                # cannot run once `if` has. Without this line a file that was
                # never really checked reports exactly like one that passed, and
                # the aggregate `empty bank` check below stays satisfied because
                # the other files carry the count.
                errors.append(
                    f"{path.name}: items[] yielded zero items "
                    "(vacuous file scan is ERROR)"
                )
        elif "id" in data:
            loaded.append((path.name, data))
        else:
            errors.append(f"{path.name}: no id or items[]")
    return loaded, errors


def count_modules(loaded: list[tuple[str, dict]]) -> tuple[Counter[int], list[str]]:
    counts: Counter[int] = Counter()
    errors: list[str] = []
    for fname, it in loaded:
        mod = it.get("module")
        try:
            mi = int(mod)
        except (TypeError, ValueError):
            iid = it.get("id") or fname
            errors.append(f"{iid}: bad module {mod!r}")
            continue
        counts[mi] += 1
    return counts, errors


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="L6 domain coverage oracle (module set derived from domains.toml)"
    )
    ap.add_argument(
        "--bank",
        type=Path,
        default=DEFAULT_BANK,
        help=f"bank items directory (default: {DEFAULT_BANK})",
    )
    ap.add_argument(
        "--policy",
        type=Path,
        default=DEFAULT_POLICY,
        help=f"bank_policy.toml path (default: {DEFAULT_POLICY})",
    )
    ap.add_argument(
        "--domains",
        type=Path,
        default=DEFAULT_DOMAINS,
        help=f"domains.toml registry path (default: {DEFAULT_DOMAINS})",
    )
    ap.add_argument(
        "--write-json",
        type=Path,
        default=None,
        help="optional path to write coverage.json summary",
    )
    args = ap.parse_args(argv)

    bank_dir = args.bank
    if not bank_dir.is_absolute():
        bank_dir = (ROOT / bank_dir).resolve()
    policy_path = args.policy
    if not policy_path.is_absolute():
        policy_path = (ROOT / policy_path).resolve()
    domains_path = args.domains
    if not domains_path.is_absolute():
        domains_path = (ROOT / domains_path).resolve()

    errors: list[str] = []

    declared, declared_errors = load_declared_modules(domains_path)
    errors.extend(declared_errors)
    exempt, exempt_errors = load_exemptions(policy_path, declared)
    errors.extend(exempt_errors)

    required = sorted(m for m in declared if m not in exempt)
    domain_mins, min_errors = load_domain_mins(policy_path, required)
    errors.extend(min_errors)

    loaded, load_errors = load_items(bank_dir)
    module_counts, mod_errors = count_modules(loaded)
    errors.extend(load_errors)
    errors.extend(mod_errors)

    n = len(loaded)
    # Vacuous empty = ERROR (anti-vacuous: empty scan set must not pass)
    if n == 0:
        errors.append("empty bank: zero items loaded (vacuous coverage is ERROR)")
    # …and so is a run with nothing left to require. A gate whose required set
    # emptied out reports exactly like one that checked everything and found it
    # sound, which is the failure this whole rebase exists to remove.
    if not required:
        errors.append(
            "zero required modules after exemptions (vacuous coverage is ERROR)"
        )

    shortfalls: list[dict] = []
    for mod in required:
        need = domain_mins[mod]
        have = module_counts.get(mod, 0)
        if have < need:
            msg = f"module {mod}: {have} items < min {need}"
            errors.append(msg)
            shortfalls.append({"module": mod, "have": have, "min": need})

    # Report: every required module, then recorded exemptions, then anything the
    # bank carries that the registry never declared.
    status = "PASS" if not errors else "FAIL"
    print(status)
    print(f"  bank={bank_dir}")
    print(f"  items={n}")
    print(f"  policy={'present' if policy_path.is_file() else 'absent (N=1 OQ-05)'}")
    print(f"  registry={domains_path.name} declares={len(declared)}")
    print(f"  modules ({len(required)} required, derived from the domain registry):")
    for mod in required:
        have = module_counts.get(mod, 0)
        need = domain_mins[mod]
        flag = "ok" if have >= need and n > 0 else "SHORT"
        print(f"    m{mod:02d}: {have} (min {need}) [{flag}]")
    if exempt:
        print("  recorded exemptions (bank_policy.toml [[coverage_exempt]]):")
        for mod in sorted(exempt):
            have = module_counts.get(mod, 0)
            print(f"    m{mod:02d}: {have} — exempt: {exempt[mod]}")
    extras = sorted(m for m in module_counts if m not in declared)
    if extras:
        print("  undeclared modules present in the bank (not required for green):")
        for mod in extras:
            print(f"    m{mod:02d}: {module_counts[mod]} (not in the domain registry)")

    # Prefer repo-relative bank path in JSON for portable commits
    try:
        bank_rel = str(bank_dir.resolve().relative_to(ROOT.resolve()))
    except ValueError:
        bank_rel = str(bank_dir)

    summary = {
        "schema_version": 2,
        "gate": "l6-domain-coverage",
        "status": status.lower(),
        "bank": bank_rel,
        "item_count": n,
        "module_source": domains_path.name,
        "declared_modules": sorted(declared),
        "primary_modules": required,
        "exemptions": {str(k): v for k, v in sorted(exempt.items())},
        "domain_min": {str(k): v for k, v in sorted(domain_mins.items())},
        "counts": {str(k): module_counts.get(k, 0) for k in required},
        "extra_counts": {str(k): module_counts[k] for k in extras},
        "shortfalls": shortfalls,
        "oq05_default_n": DEFAULT_N,
        # The regeneration command lives IN the artifact. web/data/coverage.json
        # was hand-authored before 2026-08-14 and had drifted from the bank on
        # three numbers; a machine ledger that only a human can refresh is a
        # ledger that will be wrong.
        "note": (
            "Coverage ≠ exam pass probability; study signal only. "
            "Regenerate via: python3 scripts/verify_coverage.py "
            "--write-json web/data/coverage.json"
        ),
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
        for e in errors[:40]:
            print(f"    - {e}")
        if len(errors) > 40:
            print(f"    ... +{len(errors) - 40} more")
        return 1

    # Enumerated, not spanned: an exemption can leave a gap, and "1–15" would
    # read as covering a module that was held out.
    span = " ".join(f"m{m:02d}" for m in required)
    print(f"  coverage GREEN ({len(required)} required modules ≥ domain_min: {span})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
