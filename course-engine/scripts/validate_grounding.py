#!/usr/bin/env python3
# RUST MIGRATION: differential oracle for cdcp_gate validate-grounding (bd-substrate-python-gates-viu)
# Retire when Rust gate passes all differential tests and L4 selftest coverage is proven.
"""validate_grounding.py — anti-hallucination heuristics + corpus overlap.

Fail-closed on high-severity patterns. Soft-fail (WARN) on low module overlap
when --strict-overlap is set (default: WARN only until library curated).

Anti-vacuous (L4, bd-yje7): this gate checks bank claims against a grounding
corpus, so with zero items it checks nothing and with zero corpus there is
nothing that could contradict any claim — every item then scores clean and the
banner reads at its most reassuring exactly when the gate is blindest. Zero
items, a corpus below the recorded floor, and a missing or unlistable corpus
ROOT are therefore each an ERROR that NAMES ITSELF, never a pass. The floors are
deliberate minima with recorded reasons (see MIN_SCANNED_ITEMS /
MIN_CORPUS_CHARS below), not `> 0` — "non-empty" would move the hole rather than
close it.

Exit 0 if nothing vacuous and no high-severity failures; 1 otherwise.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
ITEMS = ROOT / "bank" / "items"
MODULES = ROOT.parent / "modules"
REFERENCE = ROOT.parent / "reference"
TOPICS = ROOT / "knowledge" / "topics.toml"
CORPUS_PUBLIC = ROOT / "knowledge" / "corpus" / "public"
KNOWLEDGE = ROOT / "knowledge"

# ── ANTI-VACUOUS FLOORS (bd-yje7) ────────────────────────────────────────────
# Recorded thresholds, each with its reason, because "greater than zero" moves
# the hole rather than closing it: a one-byte corpus and a one-item bank both
# clear a non-emptiness test while telling a reader nothing.
#
# MIN_SCANNED_ITEMS = 40 — one full exam form. knowledge/bank_policy.toml sets
#   exam_n_items = 40, so a bank that cannot fill a single form cannot produce
#   the artifact this course ships, and "no heuristic fired" over it says nothing
#   about the product. Deliberately ONE TENTH of verify_bank's
#   pool_min_items = 400: the pool floor belongs to that gate, and enforcing it
#   twice would turn a legitimately-still-growing bank RED here for a reason
#   another gate already owns. Live tree 2026-08-14: 804 FILES scanned
#   (this gate's population is the file set, not verify_bank's 779-item
#   approved pool). 20x this scan-set floor. The two populations differ
#   on purpose.
#
# MIN_CORPUS_CHARS = 20000 — one module's worth of prose. Measured 2026-08-14,
#   the 29 live modules run 749..47651 characters, median 23870, so this sits
#   just under one median module. Below it there is not enough text for
#   whole-word overlap to mean anything, and the only ways to get there are a
#   corpus that was never found, was emptied, or was truncated. Live tree:
#   659149 characters, 33x this floor — and 545885 of those come from OUTSIDE
#   knowledge/corpus/public, so the licensing remediation
#   (bd-corpus-public-captures-not-licensed-class-kej) can delete every capture
#   and this floor still clears at 27x, while an actual disappearance of the
#   corpus goes RED instead of green on the way down.
MIN_SCANNED_ITEMS = 40
MIN_CORPUS_CHARS = 20_000

# The directories load_corpus_text() reads. Each must EXIST and be LISTABLE: a
# missing root is not "no findings", it is "no evidence" — the walk silently
# contributes zero characters and the gate reports as though it had looked.
# Per-root emptiness is deliberately NOT checked; MIN_CORPUS_CHARS governs total
# volume, and the licensing remediation may legitimately leave
# knowledge/corpus/public empty while the rest of the tree still grounds the bank.
CORPUS_ROOTS = [
    ("../modules", MODULES),
    ("../reference", REFERENCE),
    ("knowledge", KNOWLEDGE),
    ("knowledge/corpus/public", CORPUS_PUBLIC),
]

# High severity — invented normative precision
CLAUSE_PATTERNS = [
    re.compile(
        r"\b(?:ISO|IEC|EN|ANSI|TIA|NFPA|IEEE)\s*[\d\-]+(?:-\d+)*\s*"
        r"(?:clause|section|§|part)\s*[\d\.]+",
        re.I,
    ),
    re.compile(r"\bclause\s+\d+\.\d+(?:\.\d+)*\b", re.I),
    re.compile(r"\b§\s*\d+\.\d+", re.I),
]

# Invented exact setpoints often hallucinated
NUMERIC_TRAP = re.compile(
    r"\b(?:exactly|precisely|must be)\s+\d+(?:\.\d+)?\s*°?\s*[CF]\b"
    r"|\b\d{2,3}\s*°\s*[CF]\s*(?:recommended|required|mandatory)\b",
    re.I,
)

DUMP_PHRASES = [
    re.compile(r"actual exam question", re.I),
    re.compile(r"brain\s*dump", re.I),
    re.compile(r"real EPI exam", re.I),
    re.compile(r"guaranteed pass", re.I),
]

def load_item(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_corpus_text() -> str:
    chunks: list[str] = []
    for base in (MODULES, REFERENCE, KNOWLEDGE):
        # ABSENT-OK: this walk COLLECTS text; missing roots are
        # corpus_root_errors()'s finding, not this function's.
        if not base.exists():
            continue
        for p in base.rglob("*"):
            # ABSENT-OK: walk type-filter; a non-file suffix match is not corpus text.
            if p.suffix.lower() in {".md", ".toml", ".txt"} and p.is_file():
                if "corpus/public" in str(p):
                    continue  # added below
                try:
                    chunks.append(p.read_text(encoding="utf-8", errors="replace"))
                except OSError:
                    pass
    # ABSENT-OK: collection walk; a missing public corpus is
    # corpus_root_errors()'s finding.
    if CORPUS_PUBLIC.is_dir():
        for p in CORPUS_PUBLIC.glob("*.txt"):
            try:
                chunks.append(p.read_text(encoding="utf-8", errors="replace"))
            except OSError:
                pass
    return "\n".join(chunks).lower()


def corpus_root_errors() -> list[str]:
    """Every declared corpus root that is missing or cannot be listed.

    `load_corpus_text` skips both cases in silence, which is precisely how a
    gate ends up reporting PASS over a corpus it never opened.
    """
    errs: list[str] = []
    for label, path in CORPUS_ROOTS:
        if not path.is_dir():
            errs.append(f"corpus root missing: {label}")
            continue
        try:
            next(iter(path.iterdir()), None)
        except OSError:
            errs.append(f"corpus root unreadable: {label}")
    return errs


def topic_labels() -> dict[str, str]:
    text = TOPICS.read_text(encoding="utf-8") if TOPICS.is_file() else ""
    labels: dict[str, str] = {}
    # The split pattern needs a NEWLINE before the block header, so a registry
    # whose very first byte opens a block used to lose that topic silently: the
    # raw id was tokenised in place of its label and every score built on it
    # degraded with no finding printed. Prepending "\n" makes the first block
    # reachable and changes nothing for a file that has a header line (bd-yje7).
    blocks = re.split(r"\n\[\[topic\]\]\n", "\n" + text)
    for b in blocks[1:]:
        mid = re.search(r'(?m)^id\s*=\s*"([^"]+)"', b)
        lab = re.search(r'(?m)^label\s*=\s*"([^"]+)"', b)
        if mid and lab:
            labels[mid.group(1)] = lab.group(1)
    return labels


def tokenize(s: str) -> set[str]:
    return {t for t in re.findall(r"[a-z0-9]{4,}", s.lower()) if t not in STOP}


STOP = frozenset(
    """
    that this with from they them then than have been were will when what which
    into over also only more most some such each both same other about after
    before under above while where their there these those being does done make
    used using very just like than because through during without within
    """.split()
)


def overlap_score(item_text: str, corpus: str, topic_words: set[str]) -> float:
    toks = tokenize(item_text)
    if not toks:
        return 0.0
    # Prefer topic label words + family hits
    hits = 0
    for t in toks:
        if t in topic_words or t in corpus:
            # cheap membership: count if appears
            if t in topic_words or re.search(rf"\b{re.escape(t)}\b", corpus):
                hits += 1
    return hits / max(len(toks), 1)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--strict-overlap",
        action="store_true",
        help="FAIL items with low corpus overlap (default: warn only)",
    )
    ap.add_argument("--min-overlap", type=float, default=0.08)
    ap.add_argument("--sample-report", type=int, default=15)
    args = ap.parse_args()

    if not ITEMS.is_dir():
        print("FAIL: bank/items missing")
        return 1

    corpus = load_corpus_text()
    labels = topic_labels()
    high: list[str] = []
    warns: list[str] = []
    n = 0
    low_overlap: list[tuple[str, float]] = []

    for path in sorted(ITEMS.glob("*.toml")):
        try:
            it = load_item(path)
        except Exception as e:
            high.append(f"{path.name}: parse error {e}")
            continue
        n += 1
        # `or`, not `get(..., default)`: `id = ""` is present-but-empty and used
        # to prefix every finding for this item with nothing at all. Same shape
        # as verify_orphans.py (bd-yje7).
        iid = it.get("id") or path.name
        text = " ".join(
            [
                str(it.get("stem", "")),
                " ".join(it.get("choices") or []),
                str(it.get("explanation", "")),
            ]
        )

        for pat in CLAUSE_PATTERNS:
            if pat.search(text):
                high.append(f"{iid}: hallucinated-clause pattern: {pat.pattern[:40]}...")
                break

        for pat in DUMP_PHRASES:
            if pat.search(text):
                high.append(f"{iid}: dump-language: {pat.pattern}")

        qe = it.get("quantity_evidence")
        # many °C/°F claims without free evidence
        if NUMERIC_TRAP.search(text) and qe not in {"free_url", "licensed_note", "exam_form_public"}:
            high.append(f"{iid}: numeric setpoint without free/licensed evidence")

        # bare "must per ISO 22237-3.5.2" style
        if re.search(r"\bISO/?IEC\s*22237[^\n]{0,40}\d+\.\d+\.\d+", text, re.I):
            high.append(f"{iid}: looks like fake multi-level clause cite")

        # grounding score
        tids = it.get("topic_ids") or []
        tw: set[str] = set()
        for t in tids:
            lab = labels.get(t, t)
            tw |= tokenize(lab.replace("-", " "))
        score = overlap_score(text, corpus, tw)
        if score < args.min_overlap:
            low_overlap.append((iid, score))
            msg = f"{iid}: low corpus overlap {score:.3f} < {args.min_overlap}"
            if args.strict_overlap:
                high.append(msg)
            else:
                warns.append(msg)

    # Anti-vacuous (bd-yje7). Each condition names ITSELF, because "PASS" over
    # an empty bank and "PASS" over a clean one are otherwise the same bytes.
    vacuous: list[str] = corpus_root_errors()
    if n < MIN_SCANNED_ITEMS:
        vacuous.append(
            f"scanned_items={n} < floor {MIN_SCANNED_ITEMS} "
            f"(fewer items than one exam form — nothing was meaningfully checked)"
        )
    corpus_chars = len(corpus)
    if corpus_chars < MIN_CORPUS_CHARS:
        vacuous.append(
            f"corpus_chars={corpus_chars} < floor {MIN_CORPUS_CHARS} "
            f"(no grounding text to contradict a claim with)"
        )

    print(f"scanned_items={n}")
    print(f"high_severity={len(high)}")
    print(f"low_overlap_warns={len(low_overlap)}")
    if low_overlap:
        low_overlap.sort(key=lambda x: x[1])
        print("lowest_overlap_samples:")
        for iid, sc in low_overlap[: args.sample_report]:
            print(f"  {sc:.3f}  {iid}")

    if vacuous:
        print("FAIL: vacuous grounding check")
        for e in vacuous:
            print(f"  - {e}")

    if high:
        print("FAIL")
        for e in high[:60]:
            print(f"  - {e}")
        if len(high) > 60:
            print(f"  ... +{len(high) - 60} more")

    if vacuous or high:
        return 1

    print("PASS")
    print("  no high-severity hallucination heuristics")
    if warns:
        print(f"  warns={len(warns)} (use --strict-overlap to fail)")
    print(f"  corpus_chars={corpus_chars}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
