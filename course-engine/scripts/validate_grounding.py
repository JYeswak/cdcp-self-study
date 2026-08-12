#!/usr/bin/env python3
"""validate_grounding.py — anti-hallucination heuristics + corpus overlap.

Fail-closed on high-severity patterns. Soft-fail (WARN) on low module overlap
when --strict-overlap is set (default: WARN only until library curated).

Exit 0 if no high-severity failures; 1 otherwise.
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

# Standard family names that are OK to mention without clause numbers
OK_FAMILY = re.compile(
    r"\b(?:TIA-?942|ISO/?IEC\s*22237|EN\s*50600|ASHRAE|NFPA|Uptime|AHJ|DCIM|BMS|UPS|ATS|STS|CRAC|CRAH|PUE|WUE)\b",
    re.I,
)


def load_item(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_corpus_text() -> str:
    chunks: list[str] = []
    for base in (MODULES, REFERENCE, KNOWLEDGE):
        if not base.exists():
            continue
        for p in base.rglob("*"):
            if p.suffix.lower() in {".md", ".toml", ".txt"} and p.is_file():
                if "corpus/public" in str(p):
                    continue  # added below
                try:
                    chunks.append(p.read_text(encoding="utf-8", errors="replace"))
                except OSError:
                    pass
    if CORPUS_PUBLIC.is_dir():
        for p in CORPUS_PUBLIC.glob("*.txt"):
            try:
                chunks.append(p.read_text(encoding="utf-8", errors="replace"))
            except OSError:
                pass
    return "\n".join(chunks).lower()


def topic_labels() -> dict[str, str]:
    text = TOPICS.read_text(encoding="utf-8") if TOPICS.is_file() else ""
    labels: dict[str, str] = {}
    blocks = re.split(r"\n\[\[topic\]\]\n", text)
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
        iid = it.get("id", path.name)
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

    print(f"scanned_items={n}")
    print(f"high_severity={len(high)}")
    print(f"low_overlap_warns={len(low_overlap)}")
    if low_overlap:
        low_overlap.sort(key=lambda x: x[1])
        print("lowest_overlap_samples:")
        for iid, sc in low_overlap[: args.sample_report]:
            print(f"  {sc:.3f}  {iid}")

    if high:
        print("FAIL")
        for e in high[:60]:
            print(f"  - {e}")
        if len(high) > 60:
            print(f"  ... +{len(high) - 60} more")
        return 1

    print("PASS")
    print("  no high-severity hallucination heuristics")
    if warns:
        print(f"  warns={len(warns)} (use --strict-overlap to fail)")
    print(f"  corpus_chars={len(corpus)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
