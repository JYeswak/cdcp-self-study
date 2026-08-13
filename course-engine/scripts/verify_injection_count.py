#!/usr/bin/env python3
"""verify_injection_count.py — L4 drift guard for the advertised known-bad count.

WHY THIS EXISTS
---------------
README.md advertises a number of known-bad injections in four places (a badge,
the TL;DR, the "gates proven to trip" section, the rigor table). That number is
hand-typed, so it rots the moment a suite gains or loses an injection — measured
twice in one session. A hand-typed count that nobody checks is a self-signed
certificate about the very machinery that exists to stop self-signed
certificates.

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

Usage:
  python3 scripts/verify_injection_count.py --log /tmp/injections.txt
  python3 scripts/verify_injection_count.py --log L --readme R --require A,B
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
)

_LINE = re.compile(r"^INJECTIONS=(\d+)\s+SUITE=(\S+)\s*$")

# Numbers README advertises about injections. Badge markup writes them as
# "20_injections"; prose writes "20 known-bad injections" / "20 known-bad
# faults" / "6 suites, 20 injections".
_ADVERTISED = re.compile(
    r"(\d+)[\s_]+(?:known-bad[\s_]+)?(?:injections?|faults)", re.IGNORECASE
)

_WORD_NUM = {
    "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6,
    "seven": 7, "eight": 8, "nine": 9, "ten": 10, "eleven": 11, "twelve": 12,
}
_SUITE_COUNT = re.compile(
    r"\b(\d+|" + "|".join(_WORD_NUM) + r")\s+(?:selftest\s+)?suites?\b",
    re.IGNORECASE,
)


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


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="known-bad injection count drift guard")
    ap.add_argument("--log", type=Path, required=True)
    ap.add_argument("--readme", type=Path, default=DEFAULT_README)
    ap.add_argument(
        "--require",
        default=",".join(REGISTERED_SUITES),
        help="comma-separated suite names that MUST self-report",
    )
    args = ap.parse_args(argv)

    required = tuple(s.strip() for s in args.require.split(",") if s.strip())
    if not required:
        print("FAIL")
        print("  - no suites required (a gate over an empty registry is vacuous)")
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

    total = sum(counts.get(s, 0) for s in required)

    advertised: list[tuple[int, int]] = []
    suite_claims: list[tuple[int, int]] = []
    if not args.readme.is_file():
        errors.append(f"README missing: {args.readme}")
    else:
        for lineno, line in enumerate(
            args.readme.read_text(encoding="utf-8").splitlines(), 1
        ):
            for m in _ADVERTISED.finditer(line):
                advertised.append((lineno, int(m.group(1))))
            for m in _SUITE_COUNT.finditer(line):
                tok = m.group(1).lower()
                suite_claims.append(
                    (lineno, int(tok) if tok.isdigit() else _WORD_NUM[tok])
                )
        if not advertised:
            errors.append(
                "README advertises no known-bad injection count at all "
                "(nothing to check is an ERROR, not a pass)"
            )
        for lineno, n in advertised:
            if n != total:
                errors.append(
                    f"README.md:{lineno} advertises {n} known-bad injections; "
                    f"the suites self-reported {total}"
                )
        for lineno, n in suite_claims:
            if n != len(required):
                errors.append(
                    f"README.md:{lineno} advertises {n} selftest suites; "
                    f"{len(required)} are registered"
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
