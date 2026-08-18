#!/usr/bin/env python3
# RUST MIGRATION: differential oracle for cdcp_gate verify-doc-consistency (bd-substrate-python-gates-viu)
# Retire when Rust gate passes all differential tests and L4 selftest coverage is proven.
"""verify_doc_consistency.py — the roadmap docs must not contradict each other.

WHY THIS EXISTS
---------------
This repo gates code rigorously and gated prose not at all. A five-agent review
found eleven doc defects by hand — duplicated milestone rows with conflicting
status, a README roadmap copied from a stale CHARTER row, and a pile of pages
still describing the repository as unpublished after it went public. Nothing in
check.sh would have caught a single one, yet the prose is what a stranger reads
first. This is the gate that makes the roadmap machine-checkable.

WHAT IT CHECKS
--------------
(1) Milestone-status agreement. The milestone tables in CHARTER.md, README.md
    and course-engine/docs/PHASE-NEXT.md are parsed into (milestone id ->
    status) rows. It FAILS if:
      * one milestone id carries conflicting statuses across (or within) docs;
      * one milestone id appears twice in a single table;
      * a status cell uses vocabulary the gate does not recognise (fail-closed:
        an unreadable status is not a passing status);
      * a status cell asserts DONE and OPEN at once;
      * a row in a table that HAS a Status column is too short to reach it
        (see DECISION below).

DECISION (bd-hw3, 2026-08-14): A ROW SHORTER THAN ITS STATUS COLUMN IS RED
--------------------------------------------------------------------------
This function used to fall back to the section heading's status whenever a data
row had fewer cells than the Status column index. When the heading was not
itself a status word that fallback was ``None``, the row was recorded with
``status=None`` and NO error, and main() later died in
``",".join(sorted({r["status"] for r in rows}))`` with a TypeError — AFTER
printing the word PASS and most of the summary. A verdict followed by a crash is
the worst possible output shape: a reader skimming stdout sees PASS, CI sees
non-zero, and which one wins depends on whether anyone looked.

The fix is not to render the missing status as the string "None". It is to fail
closed, because:

  * Every OTHER unreadable status here already fails closed — an empty cell, an
    unrecognised word, a cell asserting DONE and OPEN at once. A status cell
    that is ABSENT ENTIRELY is strictly less readable than one that is present
    and unrecognised, so it cannot be the single case that passes. The old
    behaviour was fail-OPEN by accident (a None leaking into a join), not by
    design.
  * It corrupts the anti-vacuous counters. The row still counted toward
    ``milestone_rows`` and ``milestone_ids``, so the gate reported having read a
    row it could not read — exactly what "a doc that was never parsed must never
    report like one that agreed" forbids.
  * Rendering "None" would MINT A THIRD STATUS VALUE. A milestone that is DONE
    in one doc and ragged in another would then be reported as a cross-doc
    conflict "…=DONE · …=None", which names the wrong defect: the docs do not
    disagree, one row is malformed.

Kept deliberately: the heading-supplied status. A milestone table with NO Status
column at all, under a status-bearing heading (the PHASE-NEXT shape), still
takes its status from the heading. That is a table-level declaration and is
legitimate. The defect was conflating it with a row-level SHORTFALL inside a
table that does declare a Status column; separating those two is the whole fix.
Consequence worth stating: after this change ``row["status"]`` is always a str,
never None, so the summary join cannot raise.

STRUCTURAL RULE
---------------
The verdict is the LAST thing written, or it is not written. main() renders the
entire report into a buffer and prints it in one call, so no path that can still
raise runs after "PASS" has reached stdout.

(2) Publication truth. The repository is public (see REPO_PUBLIC below). It
    FAILS if any tracked markdown still asserts that publication is pending,
    blocked, deferred, or awaiting a human.

    DESCRIBING the detector is not asserting (bd-1sd.12). A line is skipped
    when it (a) carries ``<!-- doc-truth: describes-detector -->``, (b) names
    the detector (``scan_publication``, ``selftest_doc_consistency``, …), or
    (c) sits inside a CLOSED fenced code block. An unmarked "going public is
    pending" still fails. An unclosed fence is not an exemption (fail-closed).
    The ``_FLIP`` / ``_STUCK`` alternations are not narrowed.

ANTI-VACUOUS
------------
An empty input set is an ERROR, not a pass. Zero markdown files, a missing
roadmap doc, or a roadmap doc yielding zero milestone rows all exit non-zero.
A doc that was never parsed must never report like one that agreed.

Usage:
  python3 scripts/verify_doc_consistency.py
  python3 scripts/verify_doc_consistency.py --root /tmp/specimen
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ENGINE = Path(__file__).resolve().parents[1]
DEFAULT_ROOT = ENGINE.parent

# Roadmap docs whose milestone tables must agree. Paths are root-relative.
MILESTONE_DOCS = (
    "CHARTER.md",
    "README.md",
    "course-engine/docs/PHASE-NEXT.md",
)

# A table is a MILESTONE table when its first column header is one of these.
# Excluded on purpose, with the reason (an exclusion without a reason is a
# schema error): a table keyed by "Epic" records BEAD lifecycle, not milestone
# status — a tracking bead may legitimately stay open for a follow-up task
# after the milestone itself is green, so bead state is not a roadmap claim.
MILESTONE_KEY_HEADERS = frozenset({"id", "wave", "milestone", "phase"})

DONE_WORDS = (
    "done",
    "green",
    "closed",
    "complete",
    "completed",
    "shipped",
    "delivered",
    "landed",
)
OPEN_WORDS = (
    "open",
    "pending",
    "planned",
    "blocked",
    "ongoing",
    "todo",
    "deferred",
    "wip",
    "unstarted",
    "in progress",
    "in-progress",
    "not started",
    "not yet",
)

# ── Publication truth ───────────────────────────────────────────────────────
# Declared fact, not a guess. If the repository is ever made private again this
# constant must be flipped in the same commit — that is the point of pinning it
# here rather than inferring it from a git remote (private repos have remotes
# too) or from a doc (which is the thing under test).
REPO_PUBLIC = True
REPO_PUBLIC_SINCE = "2026-08-12"
REPO_PUBLIC_EVIDENCE = "github.com/JYeswak/cdcp-self-study"

_FLIP = r"(?:visibility flip|publication|publishing|going public|public release)"
_STUCK = r"(?:pending|blocked|not performed|deferred|awaiting|not yet done|remains open)"
PENDING_PUBLICATION_PATTERNS: tuple[tuple[str, str], ...] = (
    (rf"{_FLIP}[^.\n]{{0,60}}?\b{_STUCK}\b", "publication described as not done"),
    (rf"\b{_STUCK}\b[^.\n]{{0,60}}?{_FLIP}", "publication described as not done"),
    (r"\bpublic repo:\s*\**\s*no\b", "audit says the repo is not public"),
    (r"\bawaiting josh\b", "work parked on a human that already happened"),
    (r"\bflip is a human call\b", "visibility flip described as still to come"),
)

# Names of THIS scanner. A line that mentions one is documenting the detector,
# not claiming the repo is unpublished. Keep in lockstep with the Rust port.
_DETECTOR_NAMES = (
    "scan_publication",
    "selftest_doc_consistency",
    "verify_doc_consistency",
    "verify-doc-consistency",
    "_flip",
    "_stuck",
    "pending_publication_patterns",
)
_FENCE = re.compile(r"^[ \t]{0,3}(`{3,}|~{3,})")

MAX_REPORT = 40
_EMPH = re.compile(r"[*`_]+")
# Milestone ids are UPPERCASE by convention (M8, V11, M9-S1). Matching
# case-insensitively turns prose like "Learn v2" into a phantom milestone V2,
# so the prefix is deliberately case-sensitive.
_RANGE = re.compile(r"\b([MV])(\d+)\s*[-–—]\s*(?:[MV])?(\d+)\b")
_TOKEN = re.compile(r"\b([MV]\d+)((?:-S\d+)(?:/S\d+)*)?\b")


def strip_md(cell: str) -> str:
    return _EMPH.sub("", cell).strip()


def milestone_ids(cell: str) -> list[str]:
    """Extract milestone ids from a table's first cell.

    Handles ranges (``M0–M7`` -> M0..M7) and sub-milestone runs
    (``M9-S1/S2`` -> M9-S1, M9-S2). Returns [] when the cell names no
    milestone, which is how non-milestone rows are skipped.
    """
    text = strip_md(cell)
    out: list[str] = []
    consumed = text
    for m in _RANGE.finditer(text):
        prefix, lo, hi = m.group(1).upper(), int(m.group(2)), int(m.group(3))
        if lo <= hi and hi - lo <= 64:
            out.extend(f"{prefix}{i}" for i in range(lo, hi + 1))
            consumed = consumed.replace(m.group(0), " ")
    for m in _TOKEN.finditer(consumed):
        base = m.group(1).upper()
        suffix = m.group(2)
        if not suffix:
            out.append(base)
            continue
        for part in suffix.split("/"):
            num = part.replace("-S", "").replace("S", "")
            out.append(f"{base}-S{num}")
    seen: set[str] = set()
    uniq: list[str] = []
    for i in out:
        if i not in seen:
            seen.add(i)
            uniq.append(i)
    return uniq


def classify_status(cell: str) -> tuple[str | None, str | None]:
    """(status, error). status is 'DONE' or 'OPEN'."""
    text = strip_md(cell).lower()
    if not text:
        return None, "empty status cell"
    is_done = any(re.search(rf"\b{re.escape(w)}\b", text) for w in DONE_WORDS)
    is_open = any(re.search(rf"\b{re.escape(w)}\b", text) for w in OPEN_WORDS)
    if is_done and is_open:
        return None, f"status asserts DONE and OPEN at once: {cell.strip()!r}"
    if is_done:
        return "DONE", None
    if is_open:
        return "OPEN", None
    return None, f"unrecognised status vocabulary: {cell.strip()!r}"


def split_row(line: str) -> list[str]:
    body = line.strip()
    if body.startswith("|"):
        body = body[1:]
    if body.endswith("|"):
        body = body[:-1]
    return [c.strip() for c in body.split("|")]


def is_separator(line: str) -> bool:
    cells = split_row(line)
    return bool(cells) and all(re.fullmatch(r":?-{2,}:?", c) for c in cells)


def parse_doc(path: Path, rel: str) -> tuple[list[dict], list[str]]:
    """Return (rows, errors). A row is {id, status, doc, line, table}."""
    rows: list[dict] = []
    errors: list[str] = []
    if not path.is_file():
        return rows, [f"{rel}: roadmap doc missing (cannot verify agreement)"]

    lines = path.read_text(encoding="utf-8").splitlines()
    heading = ""
    i = 0
    n_tables = 0
    while i < len(lines):
        line = lines[i]
        if line.lstrip().startswith("#"):
            heading = line.lstrip("#").strip()
            i += 1
            continue
        if not line.lstrip().startswith("|"):
            i += 1
            continue

        start = i
        block: list[tuple[int, str]] = []
        while i < len(lines) and lines[i].lstrip().startswith("|"):
            block.append((i + 1, lines[i]))
            i += 1
        if len(block) < 3 or not is_separator(block[1][1]):
            continue

        header = [h.lower().strip() for h in split_row(block[0][1])]
        if not header or header[0] not in MILESTONE_KEY_HEADERS:
            continue

        status_col = next(
            (k for k, h in enumerate(header) if h == "status"),
            None,
        )
        heading_status, _ = classify_status(heading)
        if status_col is None and heading_status is None:
            # Milestone-keyed table with no status column and no status-bearing
            # heading: nothing to compare. Record it so a table that silently
            # stopped declaring status is visible in the report.
            errors.append(
                f"{rel}:{block[0][0]}: milestone table under heading "
                f"{heading!r} declares no status (no Status column, "
                f"heading is not a status)"
            )
            continue

        n_tables += 1
        table_id = f"{rel}:{start + 1}"
        seen_in_table: dict[str, int] = {}
        for lineno, raw in block[2:]:
            cells = split_row(raw)
            if not cells:
                continue
            ids = milestone_ids(cells[0])
            if not ids:
                continue
            # Four branches, all fail-closed: `status is None` implies
            # `err is not None`, so no row is ever recorded without a status.
            # See DECISION in the module docstring (bd-hw3).
            if status_col is not None and status_col < len(cells):
                status, err = classify_status(cells[status_col])
            elif status_col is not None:
                status, err = None, (
                    f"row is shorter than its Status column "
                    f"(has {len(cells)} cell(s), Status is column "
                    f"{status_col + 1}): {raw.strip()!r}"
                )
            elif heading_status is not None:
                # No Status column anywhere in the table; the status-bearing
                # heading declares it for every row. The PHASE-NEXT shape.
                status, err = heading_status, None
            else:  # pragma: no cover - the guard above already `continue`d
                status, err = None, "table declares no status"
            if err is not None:
                errors.append(f"{rel}:{lineno}: {err}")
                continue
            for mid in ids:
                if mid in seen_in_table:
                    errors.append(
                        f"{rel}:{lineno}: milestone {mid} appears twice in the "
                        f"same table (first at line {seen_in_table[mid]}) — a "
                        f"table cannot state two truths about one milestone"
                    )
                    continue
                seen_in_table[mid] = lineno
                rows.append(
                    {
                        "id": mid,
                        "status": status,
                        "doc": rel,
                        "line": lineno,
                        "table": table_id,
                    }
                )

    if n_tables == 0:
        errors.append(
            f"{rel}: zero milestone tables parsed "
            f"(empty scan set is an ERROR, not a pass)"
        )
    elif not rows:
        errors.append(f"{rel}: milestone tables yielded zero rows (vacuous)")
    return rows, errors


def markdown_files(root: Path) -> list[Path]:
    """Tracked + untracked-not-ignored *.md under root; filesystem fallback."""
    try:
        out = subprocess.run(
            [
                "git",
                "-C",
                str(root),
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "--",
                "*.md",
            ],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
        paths = [root / p for p in out.splitlines() if p.strip()]
        paths = [p for p in paths if p.is_file()]
        if paths:
            return sorted(set(paths))
    except (OSError, subprocess.CalledProcessError):
        pass
    return sorted(p for p in root.rglob("*.md") if p.is_file())


def _is_word_char(c: str) -> bool:
    return c.isalnum() or c == "_"


def _has_word(text: str, word: str) -> bool:
    """Same boundary rule as the Rust `has_word`: non-word on both sides."""
    start = 0
    n = len(word)
    while True:
        i = text.find(word, start)
        if i < 0:
            return False
        before = text[i - 1] if i else ""
        after_i = i + n
        after = text[after_i] if after_i < len(text) else ""
        if (not before or not _is_word_char(before)) and (
            not after or not _is_word_char(after)
        ):
            return True
        start = i + 1


def fence_mark(line: str) -> tuple[str, int] | None:
    """Opening/closing fence opener: 0–3 spaces/tabs then 3+ backticks or tildes."""
    m = _FENCE.match(line)
    if not m:
        return None
    mark = m.group(1)
    return mark[0], len(mark)


def closed_fence_mask(lines: list[str]) -> list[bool]:
    """True for lines inside a CLOSED fence, including the delimiters.

    An unclosed opener is NOT an exemption — fail-closed. A doc that starts a
    fence and never ends it cannot hide "going public is pending".
    """
    n = len(lines)
    mask = [False] * n
    i = 0
    while i < n:
        mark = fence_mark(lines[i])
        if mark is None:
            i += 1
            continue
        j = i + 1
        found = False
        while j < n:
            other = fence_mark(lines[j])
            if other is not None and other[0] == mark[0] and other[1] >= mark[1]:
                for k in range(i, j + 1):
                    mask[k] = True
                i = j + 1
                found = True
                break
            j += 1
        if not found:
            i += 1
    return mask


def has_describes_detector_marker(line: str) -> bool:
    """Honour ``<!-- doc-truth: describes-detector -->`` (optional whitespace)."""
    low = line.lower()
    pos = 0
    while True:
        i = low.find("<!--", pos)
        if i < 0:
            return False
        j = low.find("-->", i + 4)
        if j < 0:
            return False
        inner = low[i + 4 : j].strip()
        if inner.startswith("doc-truth:") and inner[len("doc-truth:") :].strip() == (
            "describes-detector"
        ):
            return True
        pos = j + 3


def describes_detector(line: str) -> bool:
    """True when the line is documenting this scanner, not asserting a stall."""
    if has_describes_detector_marker(line):
        return True
    low = line.lower()
    return any(_has_word(low, name) for name in _DETECTOR_NAMES)


def scan_publication(root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    files = markdown_files(root)
    if not files:
        return 0, [
            "zero markdown files scanned for publication truth "
            "(empty scan set is an ERROR, not a pass)"
        ]
    if not REPO_PUBLIC:
        return len(files), errors
    compiled = [(re.compile(p, re.IGNORECASE), why) for p, why in PENDING_PUBLICATION_PATTERNS]
    for path in files:
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as e:  # noqa: PERF203
            errors.append(f"{path}: unreadable ({e}) — refusing to pass unscanned")
            continue
        rel = path.relative_to(root)
        lines = text.splitlines()
        fenced = closed_fence_mask(lines)
        for lineno, line in enumerate(lines, 1):
            if fenced[lineno - 1] or describes_detector(line):
                continue
            for rx, why in compiled:
                if rx.search(line):
                    errors.append(
                        f"{rel}:{lineno}: {why} — repo has been public since "
                        f"{REPO_PUBLIC_SINCE} ({REPO_PUBLIC_EVIDENCE}): "
                        f"{line.strip()[:120]!r}"
                    )
                    break
    return len(files), errors


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="roadmap doc consistency gate")
    ap.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    args = ap.parse_args(argv)
    root = args.root.resolve()

    errors: list[str] = []
    all_rows: list[dict] = []
    for rel in MILESTONE_DOCS:
        rows, errs = parse_doc(root / rel, rel)
        all_rows.extend(rows)
        errors.extend(errs)

    if not all_rows:
        errors.append(
            "zero milestone rows parsed across all roadmap docs "
            "(empty scan set is an ERROR, not a pass)"
        )

    by_id: dict[str, list[dict]] = {}
    for row in all_rows:
        by_id.setdefault(row["id"], []).append(row)

    conflicts = 0
    for mid, rows in sorted(by_id.items()):
        statuses = {r["status"] for r in rows}
        if len(statuses) > 1:
            conflicts += 1
            where = " · ".join(f"{r['doc']}:{r['line']}={r['status']}" for r in rows)
            errors.append(
                f"milestone {mid} has conflicting status across the roadmap "
                f"docs: {where}"
            )

    n_md, pub_errors = scan_publication(root)
    errors.extend(pub_errors)

    # The verdict is the LAST thing written, or it is not written. Everything
    # below renders into `out` and reaches stdout in a single call at the end,
    # so a raise anywhere in report construction prints nothing at all rather
    # than a PASS followed by a traceback. (bd-hw3)
    out: list[str] = ["PASS" if not errors else "FAIL"]
    out.append(f"  root={root}")
    out.append(f"  roadmap_docs={len(MILESTONE_DOCS)}")
    out.append(f"  milestone_rows={len(all_rows)}")
    out.append(f"  milestone_ids={len(by_id)}")
    out.append(f"  conflicts={conflicts}")
    out.append(f"  markdown_scanned={n_md}")
    out.append(f"  repo_public={REPO_PUBLIC} since {REPO_PUBLIC_SINCE}")
    for mid, rows in sorted(by_id.items(), key=lambda kv: (kv[1][0]["id"])):
        seen = ",".join(sorted({r["status"] for r in rows}))
        out.append(f"    {mid}: {seen} ({len(rows)} row(s))")

    if errors:
        out.append("  failures:")
        for e in errors[:MAX_REPORT]:
            out.append(f"    - {e}")
        if len(errors) > MAX_REPORT:
            out.append(f"    ... +{len(errors) - MAX_REPORT} more")
        print("\n".join(out))
        return 1

    out.append("  roadmap GREEN (milestone status agrees; publication truth holds)")
    print("\n".join(out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
