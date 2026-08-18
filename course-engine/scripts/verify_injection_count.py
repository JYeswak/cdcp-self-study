#!/usr/bin/env python3
# RUST MIGRATION: differential oracle for cdcp_gate verify-injection-count (bd-substrate-python-gates-viu)
# Retire when Rust gate passes all differential tests and L4 selftest coverage is proven.
"""verify_injection_count.py — L4 drift guard for the advertised known-bad count.

WHY THIS EXISTS
---------------
README.md advertises a number of known-bad injections in four places (a badge,
the TL;DR, the "gates proven to trip" section, the rigor table). That number is
hand-typed, so it rots the moment a suite gains or loses an injection — measured
twice in one session. A hand-typed count that nobody checks is a self-signed
certificate about the very machinery that exists to stop self-signed
certificates.

WHAT THE ADVERTISED NUMBER COUNTS (decided 2026-08-14, bd-wf2;
permanent 2026-08-15, bd-n7uk)
--------------------------------------------------------------
It counts EXACTLY ONE population: the `INJECTIONS=<n> SUITE=<name>` receipts
emitted by the registered SHELL selftest suites (`scripts/selftest_*.sh`) during
a real `check.sh` run — one increment per injection that suite observed go RED.

The Rust known-bad legs (the `#[cfg(test)]` cases inside `crates/cdcp_gate`, and
the per-gate ports that follow) are deliberately NOT in this total. The reason,
stated because an exclusion without a reason is a schema error:

  * they emit no receipt — `cargo test` reports pass/fail, not a count, so there
    is nothing for `check.sh` to tee into the log and nothing for this gate to
    sum. Their number would have to be hand-typed somewhere, which reintroduces
    the exact defect this gate exists to remove;
  * they are internal to a test binary's own accounting, which this gate cannot
    observe, so a Rust leg that silently stopped asserting would be invisible
    here while inflating the advertised number.

Consequence, and it is the honest reading: the advertised number is a floor on
the repo's known-bad population, not its total. README must therefore say what
it counts ("shell selftest suites"), so a reader cannot mistake it for "every
known-bad in the repo". Folding the Rust legs in is a real option later, but it
is a mechanism change, not a number change: those legs must first emit receipts
that `check.sh` aggregates and be registered in REGISTERED_SUITES below. Until
they do, counting them would be a claim with no receipt behind it.

The number is REGENERATED, never hand-maintained: `--write-readme` rewrites every
advertised site from the receipts that were actually collected. It refuses to
write when the receipts themselves are unsound, so a bogus log cannot launder a
wrong number into README.

The reachable caller is `check.sh` with `CDCP_INJECTION_COUNT_WRITE_README=1`
(bd-injection-count-regen-unreachable-lu45). Without that flag the same
invocation is still a drift check (RED on disagreement). The flag cannot
launder an unsound total.

PER-SUITE COLUMN (bd-per-suite-injection-column-unguarded-aop9)
--------------------------------------------------------------
The "Gates proven to trip" table carries a per-suite `n` cell
(`| `selftest_orphan` | 6 | ... |`). The `_ADVERTISED` regex cannot see it —
the number is not followed by `injections|faults` — so the column was folklore
until this floor-raise. Each `INJECTIONS=<n> SUITE=<name>` receipt is now
compared to that cell. A disagreeing cell is RED naming file:line, suite, and
both numbers. A row whose name is not in REGISTERED_SUITES, and a registered
suite with a receipt but no row, are RED. Zero parsed suite rows is an ERROR
(anti-vacuous). `--write-readme` rewrites the cells from the receipts.

WHY NOT THE OBVIOUS IMPLEMENTATIONS (measured, do not re-derive)
---------------------------------------------------------------
* Counting "# a)" style header comments does not work: three suites declare
  zero cases in their headers yet demonstrably inject.
* Grepping for a shared assert helper does not work either: seven of the suites
  hand-roll their own assert idiom, so a grep-based counter under-counts and
  produces a GREEN gate certifying a WRONG number — strictly worse than no gate.

So each suite SELF-REPORTS at runtime. Its assert helper increments a counter
only on a genuinely observed RED, and the suite prints one machine-readable
line on its success path:

    INJECTIONS=<n> SUITE=<name>

check.sh captures those lines while the suites run for real and hands the log
here. This gate sums them and compares against every number README advertises.

ANTI-VACUOUS (mandatory)
------------------------
A registered suite that emits NO line is an ERROR, not a zero — otherwise a
suite that stopped reporting reads exactly like a suite with nothing to report.
An empty log, a suite reporting zero injections, an unregistered suite name, and
a README advertising no number at all are all errors.

POPULATION QUALIFIER (bd-n7uk)
------------------------------
An advertisement site that says "known-bad" (or the shields.io spelling
"known--bad") must also say "shell" or "selftest" on the same line. The badge
is two of the five sites and is included. Without that qualifier a reader
takes the number for every known-bad in the repo. Zero advertisement sites
is still an ERROR (anti-vacuous). The Rust `#[cfg(test)]` legs stay
uncounted: they emit no receipt, and inventing a cargo count would be the
defect this gate exists to remove.

PARTIAL COVERAGE IS ALSO AN ERROR (bd-wf2)
------------------------------------------
The subtler failure is not "no sites parse" — that is already caught. It is ONE
site quietly falling out of the scanner while the others still parse: coverage
drops and the gate reports identically to full coverage. Two defences, because
either alone leaves a gap:

 1. Counts spelled in English words parse (zero..ninety-nine), so rewriting a
    site as "thirty-six known-bad injections" keeps it under the gate instead of
    removing it from the gate.
 2. MIN_ADVERTISEMENT_SITES is a floor on how many sites must parse at all. A
    site that becomes unreadable in any OTHER way — "three dozen", a number
    above the word vocabulary, a deleted line — drops the count and trips it.

Usage:
  python3 scripts/verify_injection_count.py --log /tmp/injections.txt
  python3 scripts/verify_injection_count.py --log L --readme R --require A,B
  python3 scripts/verify_injection_count.py --log L --write-readme
  CDCP_INJECTION_COUNT_WRITE_README=1 sh scripts/check.sh
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ENGINE = Path(__file__).resolve().parents[1]
DEFAULT_README = ENGINE.parent / "README.md"

# Every suite that asserts known-bad injections MUST be registered here. A new
# suite that is not registered fails this gate by design: adding injections
# without updating the advertised count is the exact drift being guarded.
#
# Deliberately NOT registered, with the reason (an exclusion without a reason is
# a schema error):
#   tests/publishability-bar.sh — verifies that the publishability audit's
#     claims are true of the repo. It asserts facts; it plants no known-bad and
#     asserts no RED, so counting it as an "injection" suite would inflate the
#     advertised number with checks that never proved they can trip.
#   crates/*/src/**/#[cfg(test)] known-bad legs — see the module header. They
#     emit no receipt, so they cannot be summed; registering them without a
#     receipt mechanism would put a hand-typed number back in the badge.
REGISTERED_SUITES = (
    "selftest_known_bad",
    "selftest_l5",
    "selftest_l5_honesty",
    "selftest_l6_coverage",
    "selftest_l7_objectives",
    "selftest_reconstructed",
    "selftest_orphan",
    "selftest_doc_consistency",
    "selftest_injection_count",
    "wasm-freshness",
)

# How many advertisement sites must parse before the comparison is worth
# anything. The shipped README advertises the count at five sites (the badge
# markup contributes two), and the selftest's specimen README also writes five.
# This is a FLOOR, not an equality: adding an advertisement is free, removing or
# obscuring one is a deliberate decision that has to edit this constant. Without
# it, a README where one site stopped parsing reports exactly like a README
# where all of them still do.
MIN_ADVERTISEMENT_SITES = 5

_LINE = re.compile(r"^INJECTIONS=(\d+)\s+SUITE=(\S+)\s*$")


def _cardinals() -> dict[str, int]:
    """English cardinals zero..ninety-nine, both hyphen and space compounds.

    Bounded on purpose. A count above ninety-nine spelled in words is not
    recognised — it drops the site out of `advertised`, which trips the
    MIN_ADVERTISEMENT_SITES floor rather than passing silently.
    """
    ones = (
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight",
        "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen",
        "sixteen", "seventeen", "eighteen", "nineteen",
    )
    tens = (
        "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy",
        "eighty", "ninety",
    )
    words = {w: i for i, w in enumerate(ones)}
    for t in range(2, 10):
        words[tens[t]] = t * 10
        for u in range(1, 10):
            for sep in ("-", " "):
                words[f"{tens[t]}{sep}{ones[u]}"] = t * 10 + u
    return words


WORD_NUM = _cardinals()

# Alternation order is part of the pattern: `re` is leftmost-first, not
# longest-match, so "eighteen" must be offered before "eight" and "twenty-one"
# before "twenty". Longest first, ties broken lexicographically so the Rust port
# can rebuild the identical ordering.
_WORD_ALT = "|".join(sorted(WORD_NUM, key=lambda w: (-len(w), w)))

# Numbers README advertises about injections. Badge markup writes them as
# "20_injections"; prose writes "20 known-bad injections" / "20 known-bad
# faults" / "6 suites, 20 injections" / "twenty known-bad injections".
#
# The `\b` guards the WORD branch only: a digit run after a dot ("v1.7
# injections") is still a number, but "eight" inside "freighter" is not.
_ADVERTISED = re.compile(
    r"(\d+|\b(?:" + _WORD_ALT + r"))"
    r"[\s_]+(?:known-bad[\s_]+)?(?:injections?|faults)",
    re.IGNORECASE,
)

_SUITE_COUNT = re.compile(
    r"\b(\d+|" + _WORD_ALT + r")\s+(?:selftest\s+)?suites?\b",
    re.IGNORECASE,
)

# Parse scope (so scanners do not eat their own docs): a suite row is a
# markdown table row whose first cell is a backticked name matching
# `selftest_[a-z0-9_]+` and whose second cell is an integer. Mentions of the
# table in this crate's comments, in selftest_injection_count.sh headers, or
# in CHARTER are NOT rows — only `--readme` is scanned. Zero parsed rows is
# an ERROR (anti-vacuous) when `--require` names any `selftest_*` suite.
_SUITE_ROW = re.compile(
    r"^(\s*\|\s*`)(selftest_[a-z0-9_]+|wasm-freshness)(`\s*\|\s*)(\d+)(\s*\|)"
)
_SELFTEST_NAME = re.compile(r"^selftest_[a-z0-9_]+$")


def count_value(tok: str) -> int:
    """The integer a matched count token denotes, digits or words."""
    t = tok.lower()
    return int(t) if t.isdigit() else WORD_NUM[t]


def parse_log(path: Path) -> tuple[dict[str, int], list[str]]:
    errors: list[str] = []
    counts: dict[str, int] = {}
    if not path.is_file():
        return counts, [f"injection log missing: {path}"]
    lines = [ln for ln in path.read_text(encoding="utf-8").splitlines() if ln.strip()]
    if not lines:
        return counts, [
            "injection log is empty — zero suites self-reported "
            "(empty scan set is an ERROR, not a pass)"
        ]
    for raw in lines:
        m = _LINE.match(raw.strip())
        if not m:
            errors.append(f"unparseable receipt line: {raw.strip()!r}")
            continue
        n, suite = int(m.group(1)), m.group(2)
        if suite in counts and counts[suite] != n:
            errors.append(
                f"suite {suite} reported two different counts "
                f"({counts[suite]} then {n}) in one run"
            )
        counts[suite] = n
    return counts, errors


def _split_eol(line: str) -> tuple[str, str]:
    if line.endswith("\r\n"):
        return line[:-2], "\r\n"
    if line.endswith("\n") or line.endswith("\r"):
        return line[:-1], line[-1]
    return line, ""


def parse_suite_row(line: str) -> tuple[str, int, int, int] | None:
    """Return (suite, n, digit_start, digit_end) on a per-suite table row."""
    body, _eol = _split_eol(line)
    m = _SUITE_ROW.match(body)
    if not m:
        return None
    return m.group(2), int(m.group(4)), m.start(4), m.end(4)


def is_selftest_suite(name: str) -> bool:
    return _SELFTEST_NAME.fullmatch(name) is not None


def regenerate(text: str, total: int, counts: dict[str, int] | None = None) -> tuple[str, int]:
    """Rewrite every advertised injection count to `total`.

    Returns the new text and the number of sites rewritten. Line terminators are
    preserved exactly (`keepends=True`), and only the count token itself is
    replaced, so surrounding markup and prose are untouched. A word-spelled site
    is normalised to digits — regeneration produces the checkable form.

    Suite counts are NOT rewritten: the suite roster changes only when a suite is
    added or removed, which is already a deliberate edit to REGISTERED_SUITES,
    and rewriting them would rewrite prose ("Nine selftest suites") that no
    caller asked this gate to author.

    When `counts` is given, each per-suite `n` cell whose suite is in that map
    is rewritten to the receipt. Missing or unregistered rows are left in place
    so the comparison can still name them; this function does not invent rows.
    """
    lines = text.splitlines(keepends=True)
    rewritten = 0
    for i, line in enumerate(lines):
        body, eol = _split_eol(line)
        parts: list[str] = []
        last = 0
        for m in _ADVERTISED.finditer(body):
            parts.append(body[last : m.start(1)])
            parts.append(str(total))
            last = m.end(1)
            rewritten += 1
        if parts:
            parts.append(body[last:])
            body = "".join(parts)
        if counts:
            row = _SUITE_ROW.match(body)
            if row is not None:
                suite = row.group(2)
                if suite in counts:
                    new = str(counts[suite])
                    if new != row.group(4):
                        body = body[: row.start(4)] + new + body[row.end(4) :]
        lines[i] = body + eol
    return "".join(lines), rewritten


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="known-bad injection count drift guard")
    ap.add_argument("--log", type=Path, required=True)
    ap.add_argument("--readme", type=Path, default=DEFAULT_README)
    ap.add_argument(
        "--require",
        default=",".join(REGISTERED_SUITES),
        help="comma-separated suite names that MUST self-report",
    )
    ap.add_argument(
        "--write-readme",
        action="store_true",
        help="regenerate the advertised counts in README from the receipts",
    )
    args = ap.parse_args(argv)

    required = tuple(s.strip() for s in args.require.split(",") if s.strip())
    if not required:
        print("FAIL")
        print("  - no suites required (a gate over an empty registry is vacuous)")
        return 1

    # A suite named twice would be summed twice, inflating measured_total — the
    # one direction that turns real drift GREEN. Silently de-duplicating would
    # accept a caller that does not know its own roster, so this is an ERROR.
    duplicated = sorted({s for s in required if required.count(s) > 1})
    if duplicated:
        print("FAIL")
        print(
            "  - --require names "
            + ", ".join(repr(s) for s in duplicated)
            + " more than once; a repeated suite is summed twice, which inflates "
            "measured_total and is the direction that turns real drift GREEN"
        )
        return 1

    counts, errors = parse_log(args.log)

    for suite in required:
        if suite not in counts:
            errors.append(
                f"registered suite {suite!r} emitted no INJECTIONS= line — "
                f"that is an ERROR, never a silent zero"
            )
        elif counts[suite] <= 0:
            errors.append(
                f"suite {suite!r} self-reported {counts[suite]} injections — "
                f"a known-bad suite that asserts no RED is not a gate"
            )
    for suite in sorted(counts):
        if suite not in required:
            errors.append(
                f"suite {suite!r} self-reported but is not registered in "
                f"REGISTERED_SUITES — register it (and update the advertised "
                f"count) rather than letting the total drift"
            )

    # `required` is duplicate-free by the check above, so this is a plain sum.
    total = sum(counts.get(s, 0) for s in required)

    # Regeneration runs BEFORE the comparison, and only when the receipts
    # themselves are sound. A missing suite, a zero suite, an unparseable line or
    # an unregistered suite means the total is not trustworthy, and writing an
    # untrustworthy number into README would launder it into a certificate.
    receipts_sound = not errors
    regen_note: str | None = None

    if args.write_readme:
        if not receipts_sound:
            regen_note = (
                "regeneration SKIPPED: the receipts are not sound, so the total "
                "is not a number worth writing"
            )
        # ABSENT-OK: REGEN, NOT A VERDICT. The comparison below still
        # records "README missing" on the same absence.
        elif not args.readme.is_file():
            regen_note = "regeneration SKIPPED: README is not readable"
        else:
            before = args.readme.read_text(encoding="utf-8")
            after, sites = regenerate(before, total, counts)
            if after == before:
                if sites == 0:
                    regen_note = (
                        f"regeneration wrote nothing: {args.readme} advertises no "
                        f"parseable count to rewrite"
                    )
                else:
                    regen_note = (
                        f"regenerated {args.readme}: {sites} site(s) already "
                        f"advertise {total}"
                    )
            else:
                args.readme.write_text(after, encoding="utf-8")
                if sites == 0:
                    regen_note = (
                        f"regenerated {args.readme}: per-suite cells now match "
                        f"receipts"
                    )
                else:
                    regen_note = (
                        f"regenerated {args.readme}: {sites} site(s) now advertise "
                        f"{total}"
                    )

    advertised: list[tuple[int, int]] = []
    suite_claims: list[tuple[int, int]] = []
    col_rows: list[tuple[int, str, int]] = []
    col_seen: dict[str, int] = {}
    if not args.readme.is_file():
        errors.append(f"README missing: {args.readme}")
    else:
        for lineno, line in enumerate(
            args.readme.read_text(encoding="utf-8").splitlines(), 1
        ):
            adv_hits = list(_ADVERTISED.finditer(line))
            for m in adv_hits:
                advertised.append((lineno, count_value(m.group(1))))
            if adv_hits:
                low = line.lower()
                if ("known-bad" in low or "known--bad" in low) and not (
                    "shell" in low or "selftest" in low
                ):
                    errors.append(
                        f"{args.readme}:{lineno} advertises known-bad injections "
                        f"without a shell/selftest qualifier — the counted "
                        f"population is shell selftest suites, not every "
                        f"known-bad in the repo"
                    )
            for m in _SUITE_COUNT.finditer(line):
                suite_claims.append((lineno, count_value(m.group(1))))
            parsed = parse_suite_row(line)
            if parsed is not None:
                suite, n, _ds, _de = parsed
                if suite in col_seen:
                    errors.append(
                        f"{args.readme}:{lineno} suite {suite} appears more than "
                        f"once in the per-suite table"
                    )
                else:
                    col_seen[suite] = lineno
                    col_rows.append((lineno, suite, n))
        if not advertised:
            errors.append(
                "README advertises no known-bad injection count at all "
                "(nothing to check is an ERROR, not a pass)"
            )
        elif len(advertised) < MIN_ADVERTISEMENT_SITES:
            errors.append(
                f"only {len(advertised)} advertisement site(s) parsed in "
                f"{args.readme}; at least {MIN_ADVERTISEMENT_SITES} are expected "
                f"— a site that stopped parsing loses coverage while reporting "
                f"exactly like full coverage"
            )
        # Findings name the file that was actually scanned. Hardcoding
        # "README.md" sent the next reader to an innocent file whenever
        # --readme pointed elsewhere.
        for lineno, n in advertised:
            if n != total:
                errors.append(
                    f"{args.readme}:{lineno} advertises {n} known-bad "
                    f"injections; the suites self-reported {total}"
                )
        for lineno, n in suite_claims:
            if n != len(required):
                errors.append(
                    f"{args.readme}:{lineno} advertises {n} selftest suites; "
                    f"{len(required)} are registered"
                )

        # Per-suite n column. Applied when --require names any selftest_*
        # suite (the live roster) or when the file already has such a row.
        # Specimens that use synthetic names (spec_alpha) and carry no table
        # are unchanged — their contract is still the total and the suite count.
        expect_col = any(is_selftest_suite(s) for s in required) or bool(col_rows)
        if expect_col:
            if not col_rows:
                errors.append(
                    "README per-suite injection table parsed to zero suite rows "
                    "(empty scan set is an ERROR, not a pass)"
                )
            else:
                for lineno, suite, n in col_rows:
                    if suite not in required:
                        errors.append(
                            f"{args.readme}:{lineno} table row {suite!r} is not "
                            f"in REGISTERED_SUITES"
                        )
                        continue
                    got = counts.get(suite)
                    if got is not None and got != n:
                        errors.append(
                            f"{args.readme}:{lineno} suite {suite} advertises "
                            f"{n} injections; the suite self-reported {got}"
                        )
                for suite in required:
                    if suite not in col_seen:
                        errors.append(
                            f"registered suite {suite!r} has no per-suite "
                            f"table row — that is an ERROR, never a silent skip"
                        )

    status = "PASS" if not errors else "FAIL"
    print(status)
    print(f"  log={args.log}")
    print(f"  registered_suites={len(required)}")
    print(f"  measured_total={total}")
    print(f"  readme_claims={sorted({n for _, n in advertised})}")
    for suite in required:
        got = counts.get(suite)
        print(f"    {suite}: {'MISSING' if got is None else got}")
    if regen_note is not None:
        print(f"  {regen_note}")

    if errors:
        print("  failures:")
        for e in errors[:40]:
            print(f"    - {e}")
        if len(errors) > 40:
            print(f"    ... +{len(errors) - 40} more")
        return 1

    print(f"  injection count GREEN (README and the suites both say {total})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
