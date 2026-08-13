#!/usr/bin/env python3
"""verify_coverage.py — L6 domain coverage oracle (modules 1–14).

For each primary syllabus module 1–14, require item count ≥ N:
  - N from knowledge/bank_policy.toml [[domain_min]] min_items when present
  - else N=1 (OQ-05 ASSUMED floor)

Module 15 (ops-adjacent) may appear in counts but is not required for green.
Vacuous empty bank = ERROR (never green on zero items / empty scan set).

Exit 0 with per-module counts; non-zero if bank empty or any of 1–14 below N.

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

PRIMARY_MODULES = range(1, 15)  # 1–14 inclusive
DEFAULT_N = 1  # OQ-05 ASSUMED


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_domain_mins(policy_path: Path) -> dict[int, int]:
    """Per-module floors from [[domain_min]]; default N=1 when absent."""
    mins: dict[int, int] = {m: DEFAULT_N for m in PRIMARY_MODULES}
    if not policy_path.is_file():
        return mins
    bp = load_toml(policy_path)
    rows = bp.get("domain_min") or []
    if not rows:
        return mins
    for row in rows:
        try:
            mod = int(row["module"])
            need = int(row["min_items"])
        except (KeyError, TypeError, ValueError):
            continue
        if mod in mins:
            mins[mod] = max(1, need)  # never below OQ-05 floor of 1
    return mins


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
    ap = argparse.ArgumentParser(description="L6 domain coverage oracle (modules 1–14)")
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

    domain_mins = load_domain_mins(policy_path)
    loaded, load_errors = load_items(bank_dir)
    module_counts, mod_errors = count_modules(loaded)

    errors: list[str] = []
    errors.extend(load_errors)
    errors.extend(mod_errors)

    n = len(loaded)
    # Vacuous empty = ERROR (anti-vacuous: empty scan set must not pass)
    if n == 0:
        errors.append("empty bank: zero items loaded (vacuous coverage is ERROR)")

    shortfalls: list[dict] = []
    for mod in PRIMARY_MODULES:
        need = domain_mins[mod]
        have = module_counts.get(mod, 0)
        if have < need:
            msg = f"module {mod}: {have} items < min {need}"
            errors.append(msg)
            shortfalls.append({"module": mod, "have": have, "min": need})

    # Report (always print per-module counts for modules 1–14 + any extras)
    status = "PASS" if not errors else "FAIL"
    print(status)
    print(f"  bank={bank_dir}")
    print(f"  items={n}")
    print(f"  policy={'present' if policy_path.is_file() else 'absent (N=1 OQ-05)'}")
    print("  modules (1–14 required):")
    for mod in PRIMARY_MODULES:
        have = module_counts.get(mod, 0)
        need = domain_mins[mod]
        flag = "ok" if have >= need and n > 0 else "SHORT"
        print(f"    m{mod:02d}: {have} (min {need}) [{flag}]")
    extras = sorted(m for m in module_counts if m not in PRIMARY_MODULES)
    if extras:
        print("  extras (not required for green):")
        for mod in extras:
            print(f"    m{mod:02d}: {module_counts[mod]} (optional)")

    # Prefer repo-relative bank path in JSON for portable commits
    try:
        bank_rel = str(bank_dir.resolve().relative_to(ROOT.resolve()))
    except ValueError:
        bank_rel = str(bank_dir)

    summary = {
        "schema_version": 1,
        "gate": "l6-domain-coverage",
        "status": status.lower(),
        "bank": bank_rel,
        "item_count": n,
        "primary_modules": list(PRIMARY_MODULES),
        "domain_min": {str(k): v for k, v in sorted(domain_mins.items())},
        "counts": {str(k): module_counts.get(k, 0) for k in PRIMARY_MODULES},
        "extra_counts": {str(k): module_counts[k] for k in extras},
        "shortfalls": shortfalls,
        "oq05_default_n": DEFAULT_N,
        "note": "Coverage ≠ exam pass probability; study signal only.",
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

    print("  coverage GREEN (modules 1–14 ≥ domain_min)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
