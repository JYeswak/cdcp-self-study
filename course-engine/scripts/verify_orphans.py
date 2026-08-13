#!/usr/bin/env python3
"""verify_orphans.py — topic <-> bank referential integrity ("orphan item").

ORACLE-GAUNTLET.md lists "orphan item" among the known-bads that MUST trip.
This is the gate that makes that claim true. It is bidirectional:

  1. orphan topic  — a topic id declared in knowledge/topics.toml that NO bank
                     item references. The syllabus asserts a thing we never
                     assess: coverage prose outruns the bank.
  2. orphan ref    — a bank item whose topic_ids names an id that does not
                     exist in topics.toml. The item is anchored to nothing;
                     weak-links / micro-checks / Learn routing silently drop it.
  3. unanchored    — a bank item with missing or empty topic_ids. Same defect
                     as (2) with the dangling pointer left implicit.

Anti-vacuous discipline (L4): an empty input set is an ERROR, not a pass.
Zero topics, zero items, or a missing directory all exit non-zero. A registry
that was never scanned must never report like one that passed.

Exit 0 only when both directions are clean over a non-empty input set.

Usage:
  python3 scripts/verify_orphans.py
  python3 scripts/verify_orphans.py --bank /tmp/planted --topics /tmp/topics.toml
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BANK = ROOT / "bank" / "items"
DEFAULT_TOPICS = ROOT / "knowledge" / "topics.toml"

# topics.toml is a flat list of [[topic]] tables; ids are the only bare `id =`
# keys in the file. Parse by regex (same contract verify_bank.py uses) so a
# schema tweak elsewhere cannot silently empty the registry.
_ID_RE = re.compile(r'(?m)^\s*id\s*=\s*"([^"]+)"')

MAX_REPORT = 40


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def topic_ids(topics_path: Path) -> tuple[list[str], list[str]]:
    """Return (ids in declaration order, errors)."""
    if not topics_path.is_file():
        return [], [f"topics registry missing: {topics_path}"]
    text = topics_path.read_text(encoding="utf-8")
    ids = _ID_RE.findall(text)
    errors: list[str] = []
    seen: set[str] = set()
    dupes: list[str] = []
    for t in ids:
        if t in seen:
            dupes.append(t)
        seen.add(t)
    if dupes:
        errors.append(f"duplicate topic ids in registry: {sorted(set(dupes))[:10]}")
    return ids, errors


def load_items(bank_dir: Path) -> tuple[list[tuple[str, dict]], list[str]]:
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


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="topic<->bank referential integrity (orphan item gate)"
    )
    ap.add_argument("--bank", type=Path, default=DEFAULT_BANK)
    ap.add_argument("--topics", type=Path, default=DEFAULT_TOPICS)
    args = ap.parse_args(argv)

    bank_dir = args.bank if args.bank.is_absolute() else (ROOT / args.bank)
    topics_path = args.topics if args.topics.is_absolute() else (ROOT / args.topics)

    errors: list[str] = []

    declared, topic_errors = topic_ids(topics_path)
    errors.extend(topic_errors)
    known = set(declared)

    loaded, load_errors = load_items(bank_dir)
    errors.extend(load_errors)

    # ── anti-vacuous: an empty scan set is an ERROR, never a pass ────────────
    if not known:
        errors.append(
            "empty topic registry: zero topic ids "
            "(vacuous referential integrity is ERROR)"
        )
    if not loaded:
        errors.append("empty bank: zero items loaded (vacuous orphan scan is ERROR)")

    referenced: set[str] = set()
    orphan_refs: list[str] = []
    unanchored: list[str] = []

    for fname, it in loaded:
        iid = it.get("id") or fname
        tids = it.get("topic_ids")
        if not tids or not isinstance(tids, list):
            unanchored.append(f"{iid}: missing/empty topic_ids (orphan item)")
            continue
        for t in tids:
            if not isinstance(t, str) or not t.strip():
                unanchored.append(f"{iid}: blank topic_id entry (orphan item)")
                continue
            referenced.add(t)
            if t not in known:
                orphan_refs.append(f"{iid}: unknown topic_id {t!r} (orphan item)")

    orphan_topics = [t for t in declared if t not in referenced]

    errors.extend(unanchored)
    errors.extend(orphan_refs)
    errors.extend(
        f"orphan topic {t!r}: declared in topics.toml, referenced by zero bank items"
        for t in orphan_topics
    )

    status = "PASS" if not errors else "FAIL"
    print(status)
    print(f"  topics={topics_path}")
    print(f"  bank={bank_dir}")
    print(f"  topics_declared={len(known)}")
    print(f"  items={len(loaded)}")
    print(f"  topics_referenced={len(referenced & known)}")
    print(f"  orphan_topics={len(orphan_topics)}")
    print(f"  orphan_item_refs={len(orphan_refs)}")
    print(f"  unanchored_items={len(unanchored)}")

    if errors:
        print("  failures:")
        for e in errors[:MAX_REPORT]:
            print(f"    - {e}")
        if len(errors) > MAX_REPORT:
            print(f"    ... +{len(errors) - MAX_REPORT} more")
        return 1

    print("  orphan integrity GREEN (every topic assessed; every ref resolves)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
