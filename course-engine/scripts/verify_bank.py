#!/usr/bin/env python3
"""verify_bank.py — fail-closed checks for course-engine bank items.

Library mode: pool must be ≥ pool_min_items (default 10× exam size).
Validates schema, topics, source_class, domain floors, correct-letter diversity.

Two output-shape contracts this gate keeps, both fixed 2026-08-14 (bd-hw3):

  - `pool_min_items` and `exam_n_items` are POSITIVE INTEGERS or the key is
    absent. 0, "0", a negative, and non-numeric junk are all the same recorded
    finding — never a silent fallback and never a crash. See
    `policy_positive_int`.
  - The verdict line is composed with the rest of the report and written last.
    Nothing that can raise runs between "PASS" and the end of stdout.
"""
from __future__ import annotations

import re
import sys
from collections import Counter
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
ITEMS_DIR = ROOT / "bank" / "items"
TOPICS_PATH = ROOT / "knowledge" / "topics.toml"
MANIFEST_PATH = ROOT / "bank" / "MANIFEST.toml"
FACT_POLICY_PATH = ROOT / "knowledge" / "fact_policy.toml"
BANK_POLICY_PATH = ROOT / "knowledge" / "bank_policy.toml"

# Emission order is part of this gate's output contract.
#
# `ALLOWED_CORRECT` was a bare frozenset until 2026-08-14 and the diversity loop
# below iterated it directly, so the order of any "correct=X is N% of pool" lines
# was PYTHONHASHSEED-dependent. That was harmless only by arithmetic accident:
# two letters cannot both exceed 70% without summing past 140%, so at most one
# such line is ever emitted. Lower that threshold below 50% and this gate's
# stdout becomes nondeterministic run to run — which would silently break every
# byte-exact differential test built against it, and would look like a port bug
# rather than an oracle bug.
#
# CORRECT_LETTERS pins the emission order. ALLOWED_CORRECT stays a frozenset
# because membership (`correct in ALLOWED_CORRECT`) must keep Python set
# semantics exactly, including raising TypeError on an unhashable value; a tuple
# would quietly answer False there and change the report.
CORRECT_LETTERS = ("A", "B", "C", "D")
ALLOWED_CORRECT = frozenset(CORRECT_LETTERS)
ALLOWED_BLOOM = frozenset(
    {"remember", "understand", "apply", "analyze", "evaluate", "create"}
)


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def policy_positive_int(
    bp: dict, key: str, default: int, errors: list[str]
) -> int | None:
    """A policy value that must be a positive integer. Absent → `default`.

    Present → coerced with `int()` and required to be > 0. Zero, negatives and
    non-numeric junk are all recorded findings and return None; the caller must
    then skip whatever check consumed the value, because the config error IS the
    finding.

    This was `int(bp.get(key) or default)` until 2026-08-14. `or` treats 0 as
    absent, so `exam_n_items = 0` silently became 40 while `exam_n_items = "0"`
    is truthy, survived to `n / exam_n`, and raised ZeroDivisionError AFTER three
    lines of a PASS report had already been written — two spellings of the same
    value, two different wrong behaviours, neither an error. A negative floor was
    worse still: it disabled the pool check outright and reported PASS. All of
    those are now the same fail-closed finding.
    """
    if key not in bp:
        return default
    raw = bp[key]
    try:
        val = int(raw)
    except (TypeError, ValueError):
        errors.append(f"bank_policy.toml: {key} must be an integer, got {raw!r}")
        return None
    if val <= 0:
        errors.append(f"bank_policy.toml: {key} must be > 0, got {val}")
        return None
    return val


def topic_ids_from_registry() -> set[str]:
    text = TOPICS_PATH.read_text(encoding="utf-8")
    return set(re.findall(r'(?m)^\s*id\s*=\s*"([^"]+)"', text))


def main() -> int:
    errors: list[str] = []

    if not ITEMS_DIR.is_dir():
        print("FAIL: bank/items/ missing")
        return 1
    if not TOPICS_PATH.is_file():
        print("FAIL: knowledge/topics.toml missing")
        return 1

    known_topics = topic_ids_from_registry()
    if not known_topics:
        errors.append("topics.toml has zero topic ids")

    allowed_qe: set[str] = {
        "free_url",
        "licensed_note",
        "qualitative_only",
        "exam_form_public",
    }
    if FACT_POLICY_PATH.is_file():
        pol = load_toml(FACT_POLICY_PATH)
        allowed_qe = set(pol.get("allowed_quantity_evidence") or allowed_qe)

    pool_min: int | None = 400
    exam_n: int | None = 40
    domain_mins: dict[int, int] = {}
    if BANK_POLICY_PATH.is_file():
        bp = load_toml(BANK_POLICY_PATH)
        pool_min = policy_positive_int(bp, "pool_min_items", pool_min, errors)
        exam_n = policy_positive_int(bp, "exam_n_items", exam_n, errors)
        for row in bp.get("domain_min") or []:
            domain_mins[int(row["module"])] = int(row["min_items"])

    item_files = sorted(ITEMS_DIR.glob("*.toml"))
    loaded: list[tuple[str, dict]] = []
    for path in item_files:
        data = load_toml(path)
        if "items" in data and isinstance(data["items"], list):
            for it in data["items"]:
                loaded.append((path.name, it))
        elif "id" in data:
            loaded.append((path.name, data))
        else:
            # multi [[item]] — tomllib may not parse bare arrays of tables only
            errors.append(f"{path.name}: no id or items[]")

    n = len(loaded)
    if n == 0:
        errors.append("zero items loaded")
    # Skipped only when the floor itself is unusable — that config error is
    # already recorded above, so this can never turn a bad policy into a pass.
    if pool_min is not None and exam_n is not None and n < pool_min:
        errors.append(
            f"pool too small: {n} < pool_min_items {pool_min} "
            f"(need ≥{pool_min // exam_n}× exam size {exam_n})"
        )

    ids: list[str] = []
    letter_counts: Counter[str] = Counter()
    module_counts: Counter[int] = Counter()

    for fname, it in loaded:
        iid = it.get("id")
        if not iid or not isinstance(iid, str):
            errors.append(f"{fname}: missing id")
            continue
        ids.append(iid)

        stem = (it.get("stem") or "").strip()
        if not stem:
            errors.append(f"{iid}: empty stem")

        choices = it.get("choices")
        if not isinstance(choices, list) or len(choices) != 4:
            errors.append(f"{iid}: choices must be length 4")
        elif any(not str(c).strip() for c in choices):
            errors.append(f"{iid}: empty choice text")

        correct = it.get("correct")
        if correct not in ALLOWED_CORRECT:
            errors.append(f"{iid}: correct must be A-D, got {correct!r}")
        else:
            letter_counts[str(correct)] += 1

        expl = (it.get("explanation") or "").strip()
        if len(expl) < 12:
            errors.append(f"{iid}: explanation too short")

        tids = it.get("topic_ids") or []
        if not tids:
            errors.append(f"{iid}: topic_ids required")
        for t in tids:
            if t not in known_topics:
                errors.append(f"{iid}: unknown topic_id {t!r}")

        sc = it.get("source_class")
        if sc != "original":
            errors.append(f"{iid}: source_class must be original, got {sc!r}")

        qe = it.get("quantity_evidence")
        if qe not in allowed_qe:
            errors.append(f"{iid}: bad quantity_evidence {qe!r}")

        bloom = it.get("bloom")
        if bloom not in ALLOWED_BLOOM:
            errors.append(f"{iid}: bad bloom {bloom!r}")

        mod = it.get("module")
        try:
            mi = int(mod)
            module_counts[mi] += 1
        except (TypeError, ValueError):
            errors.append(f"{iid}: bad module {mod!r}")

    if len(ids) != len(set(ids)):
        dup = [i for i, c in Counter(ids).items() if c > 1]
        errors.append(f"duplicate ids: {dup[:10]}")

    # Exact duplicate stems (normalized strip) are forbidden — report all ids per group
    stem_ids: dict[str, list[str]] = {}
    for _fname, it in loaded:
        iid = it.get("id")
        if not iid or not isinstance(iid, str):
            continue
        stem = (it.get("stem") or "").strip()
        if not stem:
            continue
        stem_ids.setdefault(stem, []).append(iid)
    for stem, group in sorted(stem_ids.items(), key=lambda kv: (-len(kv[1]), kv[1][0])):
        if len(group) > 1:
            errors.append(
                f"duplicate stem ({len(group)} items {group}): {stem[:100]!r}"
            )

    for mod, need in sorted(domain_mins.items()):
        have = module_counts.get(mod, 0)
        if have < need:
            errors.append(f"module {mod}: {have} items < domain_min {need}")

    # Correct-letter diversity: no letter > 70% of pool (avoid all-B libraries)
    if n >= 40:
        # CORRECT_LETTERS, not ALLOWED_CORRECT: see the note at the top. Any
        # line emitted from this loop reaches stdout, so its order is contract.
        for L in CORRECT_LETTERS:
            frac = letter_counts.get(L, 0) / n
            if frac > 0.70:
                errors.append(
                    f"correct={L} is {frac:.0%} of pool (max 70% for diversity)"
                )
        # At least 3 letters used
        if len([L for L in CORRECT_LETTERS if letter_counts.get(L, 0) > 0]) < 3:
            errors.append("need at least 3 distinct correct letters in the pool")

    # MANIFEST optional but if present should match count
    if MANIFEST_PATH.is_file():
        man = load_toml(MANIFEST_PATH)
        mc = man.get("item_count")
        if mc is not None and int(mc) != n:
            errors.append(f"MANIFEST item_count {mc} != loaded {n}")

    # The verdict is composed in full BEFORE a single byte of it is written. A
    # gate that prints PASS and then dies leaves stdout and CI disagreeing, and
    # which one wins depends on whether anyone looked; every line below is
    # therefore built first, so a raise here means no verdict at all rather than
    # a verdict that is wrong.
    if errors:
        report = ["FAIL"]
        report.extend(f"  - {e}" for e in errors[:80])
        if len(errors) > 80:
            report.append(f"  ... +{len(errors) - 80} more")
        print("\n".join(report))
        return 1

    # Unreachable with a bad policy: `pool_min`/`exam_n` are None only when
    # `policy_positive_int` recorded a finding, and `errors` returned above.
    assert pool_min is not None and exam_n is not None
    report = [
        "PASS",
        f"  items={n}",
        f"  unique_ids={len(set(ids))}",
        f"  pool_min={pool_min} exam_n={exam_n} multiplier≈{n / exam_n:.1f}x",
        f"  topics_registry={len(known_topics)}",
        f"  correct_dist={dict(sorted(letter_counts.items()))}",
        f"  modules={dict(sorted(module_counts.items()))}",
        "  source_class=original",
    ]
    print("\n".join(report))
    return 0


if __name__ == "__main__":
    sys.exit(main())
