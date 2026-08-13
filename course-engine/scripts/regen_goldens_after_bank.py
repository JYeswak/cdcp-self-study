#!/usr/bin/env python3
"""Regenerate goldens/bank_hash + grade digests after bank growth (matches cdcp_core law).

Pure-Python twin of:
  UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens generate

Also patches web/data/*_seed42.json bank_hash and appends new items into
bank_items_seed42.json so browser packs stay aligned without full export-web
when only additive items landed.

Does NOT reshuffle mock40 item_ids (fixture stays golden-pinned).
"""
from __future__ import annotations

import hashlib
import json
import sys
from collections import defaultdict
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
ITEMS_DIR = ROOT / "bank" / "items"
GOLDENS = ROOT / "goldens"
FIXTURE = GOLDENS / "fixtures" / "mock40_seed42.json"
WEB_DATA = ROOT / "web" / "data"
BANK_DOMAIN = b"cdcp-bank-v1\0"
WRONG = {"A": "B", "B": "C", "C": "D", "D": "A"}


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_bank() -> dict[str, dict]:
    out: dict[str, dict] = {}
    for path in sorted(ITEMS_DIR.glob("*.toml")):
        it = load_toml(path)
        if "id" not in it:
            continue
        out[it["id"]] = it
    return out


def canonical_dumps(obj) -> bytes:
    """Match serde_json compact + BTreeMap key sort at every object level."""
    return json.dumps(
        obj,
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
        allow_nan=False,
    ).encode("utf-8")


def hash_payload(it: dict) -> dict:
    topics = sorted(str(t) for t in (it.get("topic_ids") or []))
    return {
        "bloom": str(it.get("bloom") or ""),
        "choices": list(it.get("choices") or []),
        "correct": str(it.get("correct") or ""),
        "explanation": str(it.get("explanation") or ""),
        "id": str(it["id"]),
        "module": int(it["module"]),
        "quantity_evidence": str(it.get("quantity_evidence") or ""),
        "source_class": str(it.get("source_class") or ""),
        "stem": str(it.get("stem") or ""),
        "topic_ids": topics,
    }


def compute_bank_hash(bank: dict[str, dict]) -> str:
    h = hashlib.sha256()
    h.update(BANK_DOMAIN)
    for iid in sorted(bank.keys()):
        h.update(canonical_dumps(hash_payload(bank[iid])))
        h.update(b"\0")
    return h.hexdigest()


def is_weak(correct: int, total: int) -> bool:
    if total == 0:
        return False
    return 5 * correct < 3 * total


def grade_report(
    bank: dict[str, dict],
    bank_hash: str,
    exam_id: str,
    seed: int,
    item_ids: list[str],
    mode: str,
) -> dict:
    item_results = []
    mod: dict[int, list[int]] = defaultdict(lambda: [0, 0])  # correct, total
    score_correct = 0
    for iid in item_ids:
        it = bank[iid]
        correct = str(it["correct"]).upper()
        if mode == "all-correct":
            chosen = correct
        elif mode == "all-wrong":
            chosen = WRONG[correct]
        else:
            raise ValueError(mode)
        ok = chosen == correct
        if ok:
            score_correct += 1
        module = int(it["module"])
        mod[module][1] += 1
        if ok:
            mod[module][0] += 1
        item_results.append(
            {
                "chosen": chosen,
                "correct": correct,
                "is_correct": ok,
                "item_id": iid,
            }
        )
    by_module = []
    weak = []
    for m in sorted(mod.keys()):
        c, t = mod[m]
        by_module.append({"correct": c, "module": m, "total": t})
        if is_weak(c, t):
            weak.append(m)
    return {
        "bank_hash": bank_hash,
        "by_module": by_module,
        "exam_id": exam_id,
        "item_results": item_results,
        "passed_study_signal": score_correct >= 27,
        "schema_version": 1,
        "score_correct": score_correct,
        "score_total": len(item_ids),
        "seed": seed,
        "weak_modules": weak,
    }


def digest_report(report: dict) -> str:
    return hashlib.sha256(canonical_dumps(report)).hexdigest()


def bank_item_export(it: dict) -> dict:
    """BankItem serde shape used by web bank_items JSON (no tags)."""
    return {
        "bloom": str(it.get("bloom") or ""),
        "choices": list(it.get("choices") or []),
        "correct": str(it.get("correct") or ""),
        "explanation": str(it.get("explanation") or ""),
        "id": str(it["id"]),
        "module": int(it["module"]),
        "quantity_evidence": str(it.get("quantity_evidence") or ""),
        "source_class": str(it.get("source_class") or ""),
        "stem": str(it.get("stem") or ""),
        "topic_ids": list(it.get("topic_ids") or []),
    }


def write_pretty_sorted(path: Path, value) -> None:
    def sort_val(v):
        if isinstance(v, dict):
            return {k: sort_val(v[k]) for k in sorted(v.keys())}
        if isinstance(v, list):
            return [sort_val(x) for x in v]
        return v

    text = json.dumps(sort_val(value), indent=2, ensure_ascii=False) + "\n"
    path.write_text(text, encoding="utf-8")


def main() -> int:
    bank = load_bank()
    if not bank:
        print("FAIL: empty bank", file=sys.stderr)
        return 1
    bh = compute_bank_hash(bank)
    fix = json.loads(FIXTURE.read_text(encoding="utf-8"))
    item_ids = list(fix["item_ids"])
    exam_id = fix.get("exam_id") or "mock40"
    seed = int(fix.get("seed") or 42)
    for iid in item_ids:
        if iid not in bank:
            print(f"FAIL: fixture id missing from bank: {iid}", file=sys.stderr)
            return 1

    ac = digest_report(grade_report(bank, bh, exam_id, seed, item_ids, "all-correct"))
    aw = digest_report(grade_report(bank, bh, exam_id, seed, item_ids, "all-wrong"))

    (GOLDENS / "bank_hash.txt").write_text(bh + "\n", encoding="utf-8")
    (GOLDENS / "mock40_seed42_all_correct.sha256").write_text(ac + "\n", encoding="utf-8")
    (GOLDENS / "mock40_seed42_all_wrong.sha256").write_text(aw + "\n", encoding="utf-8")
    print(f"goldens bank_hash={bh}")
    print(f"goldens all-correct={ac}")
    print(f"goldens all-wrong={aw}")

    # Patch web packs if present
    for name in ("mock40_seed42.json", "keys_seed42.json"):
        path = WEB_DATA / name
        if not path.is_file():
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        data["bank_hash"] = bh
        write_pretty_sorted(path, data)
        print(f"patched {path}")

    bi_path = WEB_DATA / "bank_items_seed42.json"
    if bi_path.is_file():
        # Rebuild full bank array sorted by id (stable)
        items = [bank_item_export(bank[iid]) for iid in sorted(bank.keys())]
        write_pretty_sorted(bi_path, items)
        print(f"rewrote {bi_path} n={len(items)}")

    # Sanity: recompute bank hash from export shape
    rebuilt = {it["id"]: it for it in items} if bi_path.is_file() else bank
    bh2 = compute_bank_hash(rebuilt)
    if bh2 != bh:
        print(f"FAIL: export shape hash drift {bh2} != {bh}", file=sys.stderr)
        return 1
    print("regen_goldens_after_bank ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
