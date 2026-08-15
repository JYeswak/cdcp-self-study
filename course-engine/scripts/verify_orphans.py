#!/usr/bin/env python3
"""verify_orphans.py — topic <-> bank referential integrity ("orphan item").

ORACLE-GAUNTLET.md lists "orphan item" among the known-bads that MUST trip.
This is the gate that makes that claim true. It is bidirectional:

  1. orphan topic  — a topic id declared in knowledge/topics.toml that NO
                     APPROVED bank item references. The syllabus asserts a
                     thing no learner can be drawn on: coverage prose outran
                     the drawable pool.
  2. orphan ref    — a bank item whose topic_ids names an id that does not
                     exist in topics.toml. The item is anchored to nothing;
                     weak-links / micro-checks / Learn routing silently drop it.
  3. unanchored    — a bank item with missing or empty topic_ids. Same defect
                     as (2) with the dangling pointer left implicit.

WHICH POOL THE ORPHAN PREDICATE MEASURES (bd-orphans-counts-retired-items-farl)
------------------------------------------------------------------------------
Reachability is measured against `status == "approved"` and never against the
file set. C1 restricts assembly to approved items
(`cdcp_assemble::sample_item_ids` filters `is_approved()`), so a topic whose
only referencing items are retired is exactly as unreachable as a topic nobody
references — and a file-set floor reports GREEN on it by construction.

Until this fix neither side of the twin read `status`. The report printed
`items=804` and `topics_referenced=106` about a pool the sampler never draws
from. A topic referenced only by retired items was not an orphan. The error
can only ever run one way: the file set is a superset of the drawable pool.

The report names BOTH populations on every line that carries a count
(`items=N scanned, M approved`; `topics_referenced=X approved of Y
referencing`). A topic that is an orphan because its refs are all retired
says so: `referenced by 0 approved items of N referencing`.

A status outside `approved`/`draft`/`retired` is an ERROR naming the item,
never a silent drop into "not approved". An absent `status` is `draft`,
matching `cdcp_bank`'s fail-closed default: silence is never approval.

Anti-vacuous discipline (L4): an empty input set is an ERROR, not a pass.
Zero topics, zero items, or a missing directory all exit non-zero. The
empty-bank leg (`zero items loaded`) is a scan-set property and stays.
Zero approved items in a non-empty bank is a different failure with the
same verdict: the predicate would otherwise measure a pool no learner can
be assessed from. That rule holds at FILE granularity too: a single bank
file that yields zero items is named and is RED (bd-2kr).

Exit 0 only when both directions are clean over a non-empty approved pool.

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

# C1 lifecycle. `APPROVED` is the ONLY status `cdcp_assemble` may draw, so it is
# the only population the orphan predicate may be measured against. A missing
# status is `draft` — fail-closed, matching `cdcp_bank::ItemStatus`'s serde
# default — because silence must never read as approval.
APPROVED = "approved"
KNOWN_STATUSES = ("approved", "draft", "retired")


def is_approved(it: dict) -> bool:
    """Is this item in the pool `cdcp_assemble` may draw from?"""
    return it.get("status", "draft") == APPROVED


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
            before = len(loaded)
            for it in data["items"]:
                if isinstance(it, dict):
                    loaded.append((path.name, it))
            if len(loaded) == before:
                # Anti-vacuous at FILE granularity. `items = []` — or an items[]
                # holding nothing this loop can read as an item — takes the list
                # branch, adds nothing, and never reaches the `no id or items[]`
                # leg below, because `elif` cannot run once `if` has. Without
                # this line a file that was never really checked reports exactly
                # like one that passed, and the aggregate item count stays
                # healthy because the other files carry it.
                errors.append(
                    f"{path.name}: items[] yielded zero items "
                    "(vacuous file scan is ERROR)"
                )
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

    approved_n = 0
    for fname, it in loaded:
        status = it.get("status", "draft")
        if status == APPROVED:
            approved_n += 1
        elif status not in KNOWN_STATUSES:
            # Fail-closed AND loud. Dropping an unmodelled status silently into
            # "not approved" would be a bucket decided by guess rather than by
            # the recorded lifecycle.
            iid = it.get("id") or fname
            errors.append(f"{iid}: unknown status {status!r}")

    # ── anti-vacuous: an empty scan set is an ERROR, never a pass ────────────
    if not known:
        errors.append(
            "empty topic registry: zero topic ids "
            "(vacuous referential integrity is ERROR)"
        )
    if not loaded:
        errors.append("empty bank: zero items loaded (vacuous orphan scan is ERROR)")
    elif approved_n == 0:
        # A bank FULL of files and empty of drawable items is the exact state
        # a file-set orphan scan reported green on. Named separately from the
        # empty-bank leg because it is a different failure with the same verdict.
        errors.append(
            f"zero approved items ({len(loaded)} scanned): the orphan predicate "
            "measures a pool no learner can be assessed from "
            "(vacuous orphan scan is ERROR)"
        )

    referenced: set[str] = set()
    approved_referenced: set[str] = set()
    ref_count: dict[str, int] = {}
    orphan_refs: list[str] = []
    unanchored: list[str] = []

    for fname, it in loaded:
        iid = it.get("id") or fname
        approved = is_approved(it)
        tids = it.get("topic_ids")
        if not tids or not isinstance(tids, list):
            unanchored.append(f"{iid}: missing/empty topic_ids (orphan item)")
            continue
        for t in tids:
            if not isinstance(t, str) or not t.strip():
                unanchored.append(f"{iid}: blank topic_id entry (orphan item)")
                continue
            referenced.add(t)
            ref_count[t] = ref_count.get(t, 0) + 1
            if approved:
                approved_referenced.add(t)
            if t not in known:
                orphan_refs.append(f"{iid}: unknown topic_id {t!r} (orphan item)")

    orphan_topics = [t for t in declared if t not in approved_referenced]

    errors.extend(unanchored)
    errors.extend(orphan_refs)
    errors.extend(
        f"orphan topic {t!r}: declared in topics.toml, "
        f"referenced by 0 approved items of {ref_count.get(t, 0)} referencing"
        for t in orphan_topics
    )

    status = "PASS" if not errors else "FAIL"
    print(status)
    print(f"  topics={topics_path}")
    print(f"  bank={bank_dir}")
    print(f"  topics_declared={len(known)}")
    print(
        f"  items={len(loaded)} scanned, {approved_n} approved "
        "(orphan predicate counts the approved pool only)"
    )
    print(
        f"  topics_referenced={len(approved_referenced & known)} approved "
        f"of {len(referenced & known)} referencing"
    )
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

    print(
        "  orphan integrity GREEN "
        "(every topic assessed by an approved item; every ref resolves)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
