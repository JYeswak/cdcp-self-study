#!/usr/bin/env python3
"""sample_mock.py — build a 40-item mock exam from the pool (seeded, stratified).

################################################################################
# NOT THE SAMPLER OF RECORD.  DO NOT REGENERATE ANY GOLDEN WITH THIS FILE.
#
# bd-golden-sampler-divergence-09q (resolved): `cdcp_assemble::assemble()` is the
# AUTHORITATIVE sampler. This script's CPython MT19937 stream disagrees with it —
# measured at seed 42, 37 of 40 item ids differed, and 0 of the 3 shared ids sat
# at the same index. Stratification quality was identical (15 modules, peak 3
# per module, both inside policy), so the divergence is purely the PRNG stream.
#
# Regenerate goldens/fixtures/mock40_seed42.json with the RUST path:
#     UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens fixture --seed 42
#     UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens generate
# Pointing this script's --out at the golden would silently reinstate the fooled
# certificate this bead removed; crates/cdcp_cli/tests/cli.rs
# ::golden_fixture_is_the_rust_sampler_output goes RED if you do.
#
# PINNED, NOT LIVE: this file is the historical reference implementation that
# explains what the pre-2026-08-13 fixture contained. It may not be deleted by
# the substrate migration until that provenance question is dead. It is
# registered in registries/substrate_allowlist.toml.
################################################################################

Usage (ad-hoc exploration only — never to write a golden):
  python3 scripts/sample_mock.py --seed 42
  python3 scripts/sample_mock.py --seed 42 --out /tmp/mock.json

Does not grade; assembly only. Deterministic for a given seed + bank snapshot —
but note the bank drifts: this script no longer reproduces the fixture it
originally wrote, because bank_fingerprint moved from 0557953e8a49a3cf to a
later snapshot while the fixture stayed frozen. That drift is the second half of
the fooled certificate.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import random
import re
import sys
from collections import defaultdict
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
ITEMS_DIR = ROOT / "bank" / "items"
BANK_POLICY = ROOT / "knowledge" / "bank_policy.toml"
EXAM_FORM = ROOT / "knowledge" / "exam_form.toml"


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_items() -> list[dict]:
    items: list[dict] = []
    for path in sorted(ITEMS_DIR.glob("*.toml")):
        data = load_toml(path)
        if "id" in data:
            items.append(data)
    return items


def bank_fingerprint(items: list[dict]) -> str:
    h = hashlib.sha256()
    for it in sorted(items, key=lambda x: x["id"]):
        h.update(it["id"].encode())
        h.update(b"\0")
        h.update(it.get("correct", "").encode())
    return h.hexdigest()[:16]


def sample(
    items: list[dict],
    n: int,
    seed: int,
    max_per_module: int,
    min_modules: int,
) -> list[dict]:
    by_mod: dict[int, list[dict]] = defaultdict(list)
    for it in items:
        by_mod[int(it["module"])].append(it)
    for m in by_mod:
        by_mod[m].sort(key=lambda x: x["id"])

    rng = random.Random(seed)
    chosen: list[dict] = []
    used_ids: set[str] = set()
    mod_counts: dict[int, int] = defaultdict(int)

    # Round-robin across modules until n filled (stratified)
    modules = sorted(by_mod.keys())
    rng.shuffle(modules)
    # Shuffle each module list with seed derivative
    for m in modules:
        rng.shuffle(by_mod[m])

    # First pass: at least one from as many modules as possible
    for m in modules:
        if len(chosen) >= n:
            break
        for it in by_mod[m]:
            if it["id"] in used_ids:
                continue
            if mod_counts[m] >= max_per_module:
                break
            chosen.append(it)
            used_ids.add(it["id"])
            mod_counts[m] += 1
            break

    # Second pass: fill remaining with round-robin
    while len(chosen) < n:
        progress = False
        for m in modules:
            if len(chosen) >= n:
                break
            if mod_counts[m] >= max_per_module:
                continue
            for it in by_mod[m]:
                if it["id"] in used_ids:
                    continue
                chosen.append(it)
                used_ids.add(it["id"])
                mod_counts[m] += 1
                progress = True
                break
        if not progress:
            # relax max_per_module
            for it in items:
                if it["id"] not in used_ids:
                    chosen.append(it)
                    used_ids.add(it["id"])
                    if len(chosen) >= n:
                        break
            break

    chosen = chosen[:n]
    # Final order shuffle for exam presentation
    order = list(range(len(chosen)))
    rng.shuffle(order)
    chosen = [chosen[i] for i in order]

    if len({int(c["module"]) for c in chosen}) < min_modules and len(by_mod) >= min_modules:
        # soft warning only — still return
        pass
    return chosen


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, required=True)
    ap.add_argument("--out", type=Path, default=None)
    args = ap.parse_args()

    n = 40
    max_per = 8
    min_mod = 8
    if BANK_POLICY.is_file():
        bp = load_toml(BANK_POLICY)
        n = int(bp.get("exam_n_items") or n)
        asm = bp.get("assembly") or {}
        max_per = int(asm.get("max_per_module_in_mock") or max_per)
        min_mod = int(asm.get("min_modules_represented") or min_mod)
    if EXAM_FORM.is_file():
        ef = load_toml(EXAM_FORM)
        n = int(ef.get("n_items") or n)

    items = load_items()
    if len(items) < n:
        print(f"FAIL: pool {len(items)} < exam {n}", file=sys.stderr)
        return 1

    picked = sample(items, n, args.seed, max_per, min_mod)
    payload = {
        "exam_id": f"mock{n}",
        "seed": args.seed,
        "n_items": len(picked),
        "bank_fingerprint": bank_fingerprint(items),
        "item_ids": [p["id"] for p in picked],
        "modules": sorted({int(p["module"]) for p in picked}),
        "items": [
            {
                "id": p["id"],
                "module": p["module"],
                "stem": p["stem"],
                "choices": p["choices"],
                # correct omitted from learner export; include for grading fixtures
                "correct": p["correct"],
                "topic_ids": p.get("topic_ids") or [],
            }
            for p in picked
        ],
    }
    text = json.dumps(payload, indent=2, sort_keys=True)
    if args.out:
        args.out.write_text(text + "\n", encoding="utf-8")
        print(f"wrote {args.out} n={len(picked)} modules={payload['modules']}")
    else:
        print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
