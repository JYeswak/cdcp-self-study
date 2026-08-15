#!/usr/bin/env python3
"""verify_paraphrase_pairs.py — ledger tripwire + stem-overlap REPORT (bd-e1yt).

C3 (`cdcp_gate near-duplicate-items`) catches COSMETIC clones (answer Jaccard).
Paraphrases of the same proposition score below every C3 threshold and pass
silently. CHARTER forbids an LLM as grader-of-record, so this script does not
grade meaning and does not retire items.

What it DOES decide (deterministic, offline):

  1. `registries/paraphrase_pairs.toml` is a non-empty ledger of the four
     measured pairs. Deleting a required pair row without leaving the row
     in place as status=adjudicated + a non-empty reason is RED.
  2. Empty [[pair]] is ERROR. A scan of zero item files is ERROR.
     Fewer than two approved items is ERROR (zero comparisons is not a pass).
  3. A cheap token/stem overlap REPORT prints candidates. It is NOT a
     verdict. The known-good pair m09-q206 / m09-q207 (hot-aisle vs
     cold-aisle) may appear in the report — that is the point.

What it does NOT do: delete, retire, or fail the build because two stems
look alike.

Usage:
  python3 scripts/verify_paraphrase_pairs.py
  python3 scripts/verify_paraphrase_pairs.py --selftest
  python3 scripts/verify_paraphrase_pairs.py --ledger /tmp/broken.toml   # RED demo

Exit 0 only when the live ledger holds and the in-process selftest reaches RED
on a planted missing pair / empty ledger / empty bank.
"""

from __future__ import annotations

import argparse
import copy
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_LEDGER = ROOT / "registries" / "paraphrase_pairs.toml"
DEFAULT_BANK = ROOT / "bank" / "items"

APPROVED = "approved"
KNOWN_STATUSES = ("approved", "draft", "retired")
STATUS_OPEN = "open"
STATUS_ADJUDICATED = "adjudicated"
ALLOWED_PAIR_STATUS = (STATUS_OPEN, STATUS_ADJUDICATED)
CORRECT_LETTERS = "ABCD"

# The four pairs measured by hand (bd-e1yt). Deleting one of these ids from
# the ledger is RED even if min_pairs is edited down: the id is the tripwire.
REQUIRED_PAIR_IDS = (
    "pp-m09-it-power-heat",
    "pp-m09-allowable-vs-recommended",
    "pp-m09-dehumidification-coil",
    "pp-m09-blanking-panels",
)

# Known-GOOD leg: genuinely distinct. Must remain listed so the report's
# false-positive shape cannot be silently dropped.
REQUIRED_DISTINCT_IDS = ("kd-hot-aisle-cold-aisle",)

# Report cut: high stem overlap AND answer below C3's 60% floor.
# Prints candidates. Never a fail condition on its own.
REPORT_STEM_PCT = 50
C3_ANSWER_PCT = 60
REPORT_CAP = 40


# ── C3-compatible token sets (integer Jaccard, no float on the decision path) ─


def normalize(s: str) -> str:
    """Lowercase; non-ASCII-alnum becomes a separator. Matches C3."""
    out: list[str] = []
    pending_space = False
    for c in s.lower():
        if c.isascii() and c.isalnum():
            if pending_space and out:
                out.append(" ")
            pending_space = False
            out.append(c)
        else:
            pending_space = True
    return "".join(out)


def tokens(s: str) -> frozenset[str]:
    return frozenset(t for t in normalize(s).split(" ") if t)


def sim_percent(a: frozenset[str], b: frozenset[str]) -> int:
    """Truncated Jaccard percent. Empty union is 0, never 100."""
    if not a and not b:
        return 0
    union = len(a | b)
    if union == 0:
        return 0
    return (len(a & b) * 100) // union


# ── loading ────────────────────────────────────────────────────────────────


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_items(bank_dir: Path) -> tuple[list[dict], list[str]]:
    """Return (items, errors). Zero files / zero parseable items is an error."""
    errors: list[str] = []
    loaded: list[dict] = []
    if not bank_dir.is_dir():
        return loaded, [f"bank dir missing: {bank_dir}"]
    files = sorted(bank_dir.glob("*.toml"))
    if not files:
        return loaded, [f"zero item files in {bank_dir} (vacuous scan is ERROR)"]
    seen: set[str] = set()
    for path in files:
        try:
            data = load_toml(path)
        except Exception as exc:  # noqa: BLE001 — fail-closed on any parse error
            errors.append(f"{path.name}: unparseable: {exc}")
            continue
        if "id" not in data:
            errors.append(f"{path.name}: no id")
            continue
        iid = data.get("id")
        if not isinstance(iid, str) or not iid.strip():
            errors.append(f"{path.name}: empty id")
            continue
        if iid in seen:
            errors.append(f"{path.name}: duplicate id {iid}")
            continue
        seen.add(iid)
        status = data.get("status", "draft")
        if status not in KNOWN_STATUSES:
            errors.append(f"{iid}: unknown status {status!r}")
            continue
        choices = data.get("choices")
        key = ""
        if isinstance(choices, list) and len(choices) == 4:
            correct = data.get("correct")
            if correct in CORRECT_LETTERS:
                key = str(choices[ord(correct) - ord("A")])
        loaded.append(
            {
                "id": iid,
                "file": path.name,
                "stem": str(data.get("stem") or ""),
                "key": key,
                "status": status,
            }
        )
    if not loaded:
        errors.append("zero items loaded (vacuous scan is ERROR)")
    return loaded, errors


def parse_ledger(raw: dict, source: str) -> tuple[dict, list[str]]:
    errors: list[str] = []
    schema = raw.get("schema_version")
    if schema != 1:
        errors.append(f"{source}: schema_version {schema!r} (expected 1)")
    reg = raw.get("registry") or {}
    if not isinstance(reg, dict):
        errors.append(f"{source}: [registry] missing")
        reg = {}
    min_pairs = reg.get("min_pairs")
    if not isinstance(min_pairs, int) or min_pairs < 1:
        errors.append(
            f"{source}: registry.min_pairs must be an integer >= 1 "
            f"(empty/zero ledger is ERROR); got {min_pairs!r}"
        )
        min_pairs = 0
    pairs = raw.get("pair")
    if pairs is None:
        pairs = []
        errors.append(f"{source}: zero [[pair]] rows (empty ledger is ERROR)")
    elif not isinstance(pairs, list):
        errors.append(f"{source}: [[pair]] is not a table array")
        pairs = []
    elif len(pairs) == 0:
        errors.append(f"{source}: zero [[pair]] rows (empty ledger is ERROR)")
    known = raw.get("known_distinct")
    if known is None:
        known = []
        errors.append(f"{source}: zero [[known_distinct]] rows")
    elif not isinstance(known, list):
        errors.append(f"{source}: [[known_distinct]] is not a table array")
        known = []
    ledger = {
        "source": source,
        "min_pairs": min_pairs,
        "pairs": pairs,
        "known_distinct": known,
    }
    return ledger, errors


def _canon_ids(a: str, b: str) -> tuple[str, str]:
    return (a, b) if a <= b else (b, a)


def check_ledger(ledger: dict, items: list[dict]) -> list[str]:
    """Decide the ledger against a loaded item list. Never grades meaning."""
    errors: list[str] = []
    by_id = {it["id"]: it for it in items}
    approved = {it["id"] for it in items if it["status"] == APPROVED}
    if len(items) == 0:
        errors.append("zero items loaded (vacuous scan is ERROR)")
    elif len(approved) < 2:
        errors.append(
            f"{len(approved)} approved item(s) — fewer than two means ZERO "
            "pair comparisons, which is not a pass"
        )

    pairs = ledger["pairs"]
    if len(pairs) < 1:
        errors.append("empty [[pair]] list (gate claims to track pairs)")
    if ledger["min_pairs"] < len(REQUIRED_PAIR_IDS):
        errors.append(
            f"registry.min_pairs={ledger['min_pairs']} is below the required "
            f"id floor ({len(REQUIRED_PAIR_IDS)}); lowering the floor to drop "
            "a measured pair is RED"
        )
    if len(pairs) < ledger["min_pairs"]:
        errors.append(
            f"{len(pairs)} [[pair]] row(s) < registry.min_pairs "
            f"{ledger['min_pairs']}"
        )

    seen_ids: set[str] = set()
    seen_ab: set[tuple[str, str]] = set()
    open_abs: set[tuple[str, str]] = set()
    for i, row in enumerate(pairs):
        if not isinstance(row, dict):
            errors.append(f"pair[{i}]: not a table")
            continue
        pid = row.get("id")
        a = row.get("a")
        b = row.get("b")
        status = row.get("status")
        if not isinstance(pid, str) or not pid.strip():
            errors.append(f"pair[{i}]: missing id")
            continue
        if pid in seen_ids:
            errors.append(f"pair {pid}: duplicate id")
            continue
        seen_ids.add(pid)
        if not isinstance(a, str) or not isinstance(b, str) or not a or not b:
            errors.append(f"pair {pid}: a and b are required item ids")
            continue
        if a == b:
            errors.append(f"pair {pid}: a and b are the same id ({a})")
            continue
        ab = _canon_ids(a, b)
        if ab in seen_ab:
            errors.append(f"pair {pid}: duplicate members {ab[0]} / {ab[1]}")
            continue
        seen_ab.add(ab)
        if status not in ALLOWED_PAIR_STATUS:
            errors.append(
                f"pair {pid}: status {status!r} (want open|adjudicated)"
            )
            continue
        if status == STATUS_ADJUDICATED:
            reason = row.get("adjudication_reason")
            if not isinstance(reason, str) or not reason.strip():
                errors.append(
                    f"pair {pid}: status=adjudicated requires a non-empty "
                    "adjudication_reason (a disappeared pair without a reason "
                    "is RED)"
                )
            continue
        # open: both members must still exist as approved items
        open_abs.add(ab)
        for mid in (a, b):
            if mid not in by_id:
                errors.append(
                    f"pair {pid}: item {mid} is not in the bank "
                    "(listed pair disappeared without adjudication)"
                )
            elif mid not in approved:
                errors.append(
                    f"pair {pid}: item {mid} is {by_id[mid]['status']}, not "
                    "approved — retire on the ledger (status=adjudicated + "
                    "reason) before leaving the drawable pool"
                )

    for req in REQUIRED_PAIR_IDS:
        if req not in seen_ids:
            errors.append(
                f"required pair id {req} is missing from the ledger "
                "(deleting a measured pair row is RED; adjudicate in place)"
            )

    seen_kd: set[str] = set()
    for i, row in enumerate(ledger["known_distinct"]):
        if not isinstance(row, dict):
            errors.append(f"known_distinct[{i}]: not a table")
            continue
        kid = row.get("id")
        a = row.get("a")
        b = row.get("b")
        if not isinstance(kid, str) or not kid.strip():
            errors.append(f"known_distinct[{i}]: missing id")
            continue
        if kid in seen_kd:
            errors.append(f"known_distinct {kid}: duplicate id")
            continue
        seen_kd.add(kid)
        if not isinstance(a, str) or not isinstance(b, str) or a == b:
            errors.append(f"known_distinct {kid}: a and b must be two ids")
            continue
        for mid in (a, b):
            if mid not in by_id:
                errors.append(f"known_distinct {kid}: item {mid} missing")
            elif mid not in approved:
                errors.append(
                    f"known_distinct {kid}: item {mid} is "
                    f"{by_id[mid]['status']}, not approved"
                )
        if _canon_ids(a, b) in open_abs:
            errors.append(
                f"known_distinct {kid}: {a}/{b} is also listed as open "
                "paraphrase debt — a known-good pair cannot be open debt"
            )
    for req in REQUIRED_DISTINCT_IDS:
        if req not in seen_kd:
            errors.append(
                f"required known_distinct id {req} is missing "
                "(the known-good leg cannot be dropped)"
            )
    return errors


# ── report (not a verdict) ─────────────────────────────────────────────────


def overlap_report(items: list[dict], ledger: dict) -> list[str]:
    """Print-shaped candidate lines. Never used as a fail condition."""
    approved = [it for it in items if it["status"] == APPROVED]
    debt = {}
    for row in ledger["pairs"]:
        if not isinstance(row, dict):
            continue
        a, b = row.get("a"), row.get("b")
        if isinstance(a, str) and isinstance(b, str):
            debt[_canon_ids(a, b)] = row.get("id", "?")
    distinct = {}
    for row in ledger["known_distinct"]:
        if not isinstance(row, dict):
            continue
        a, b = row.get("a"), row.get("b")
        if isinstance(a, str) and isinstance(b, str):
            distinct[_canon_ids(a, b)] = row.get("id", "?")

    candidates: list[tuple[int, int, str, str, str]] = []
    n = len(approved)
    comparisons = 0
    for i in range(n):
        for j in range(i + 1, n):
            comparisons += 1
            ia, ib = approved[i], approved[j]
            stem = sim_percent(tokens(ia["stem"]), tokens(ib["stem"]))
            ans = sim_percent(tokens(ia["key"]), tokens(ib["key"]))
            ab = _canon_ids(ia["id"], ib["id"])
            tagged = ab in debt or ab in distinct
            paraphrase_shaped = stem >= REPORT_STEM_PCT and ans < C3_ANSWER_PCT
            if not (tagged or paraphrase_shaped):
                continue
            if ab in debt:
                tag = f"known-debt {debt[ab]}"
            elif ab in distinct:
                tag = f"known-distinct {distinct[ab]}"
            else:
                tag = "candidate (not a verdict)"
            candidates.append((stem, ans, ia["id"], ib["id"], tag))
    candidates.sort(key=lambda r: (-r[0], -r[1], r[2], r[3]))

    lines = [
        f"REPORT (stem>={REPORT_STEM_PCT}% and answer<{C3_ANSWER_PCT}%, "
        f"plus ledger pairs; NOT a grader-of-record; {comparisons} comparisons)",
    ]
    shown = 0
    for stem, ans, a, b, tag in candidates:
        # Always show ledger rows; cap untagged candidates.
        is_ledger = tag.startswith("known-")
        if not is_ledger:
            if shown >= REPORT_CAP:
                continue
            shown += 1
        lines.append(f"  {a} <-> {b}  stem {stem}% · answer {ans}%  [{tag}]")
    extra = sum(1 for c in candidates if not c[4].startswith("known-")) - shown
    if extra > 0:
        lines.append(f"  … {extra} more candidate(s) omitted (cap {REPORT_CAP})")
    if not any(l.startswith("  ") for l in lines):
        lines.append("  (no stem-overlap candidates at this cut)")
    return lines


# ── selftest (L4: proven to trip) ──────────────────────────────────────────


def _plant_ledger() -> dict:
    """Well-formed ledger used as the selftest baseline.

    Plants must not depend on `--ledger` still being intact — a RED demo
    that deletes a committed row would otherwise make the selftest
    un-plantable and report a false SELFTEST failure on top of the real one.
    """
    return {
        "source": "selftest-plant",
        "min_pairs": 4,
        "pairs": [
            {
                "id": REQUIRED_PAIR_IDS[0],
                "a": "m09-q111",
                "b": "m09-q242",
                "status": STATUS_OPEN,
            },
            {
                "id": REQUIRED_PAIR_IDS[1],
                "a": "m09-q113",
                "b": "m09-q202",
                "status": STATUS_OPEN,
            },
            {
                "id": REQUIRED_PAIR_IDS[2],
                "a": "m09-q122",
                "b": "m09-q234",
                "status": STATUS_OPEN,
            },
            {
                "id": REQUIRED_PAIR_IDS[3],
                "a": "m09-q140",
                "b": "m09-q209",
                "status": STATUS_OPEN,
            },
        ],
        "known_distinct": [
            {
                "id": REQUIRED_DISTINCT_IDS[0],
                "a": "m09-q206",
                "b": "m09-q207",
            }
        ],
    }


def run_selftest(items: list[dict]) -> list[str]:
    """Plant known-bad ledgers in memory. Exit 0 only if each plant goes RED."""
    fails: list[str] = []
    base = _plant_ledger()

    empty = copy.deepcopy(base)
    empty["pairs"] = []
    empty_errs = check_ledger(empty, items)
    if not any("zero [[pair]]" in e or "empty [[pair]]" in e for e in empty_errs):
        fails.append(
            "selftest EMPTY LEDGER did not ERROR "
            f"(got {empty_errs[:3]!r})"
        )

    dropped = copy.deepcopy(base)
    dropped["pairs"] = [
        p for p in dropped["pairs"] if p.get("id") != REQUIRED_PAIR_IDS[0]
    ]
    drop_errs = check_ledger(dropped, items)
    if not any(REQUIRED_PAIR_IDS[0] in e for e in drop_errs):
        fails.append(
            f"selftest MISSING PAIR {REQUIRED_PAIR_IDS[0]} did not RED "
            f"(got {drop_errs[:3]!r})"
        )

    zero_errs = check_ledger(base, [])
    if not any("zero items" in e for e in zero_errs):
        fails.append(f"selftest ZERO ITEMS did not ERROR (got {zero_errs[:3]!r})")

    one = items[:1] if items else []
    one_errs = check_ledger(base, one)
    if one and not any("fewer than two" in e or "zero items" in e for e in one_errs):
        fails.append(
            f"selftest SINGLE ITEM did not ERROR (got {one_errs[:3]!r})"
        )

    stolen = copy.deepcopy(base)
    stolen["pairs"] = list(stolen["pairs"]) + [
        {
            "id": "pp-should-not-list-known-good",
            "a": "m09-q206",
            "b": "m09-q207",
            "status": STATUS_OPEN,
        }
    ]
    stolen_errs = check_ledger(stolen, items)
    if not any("known-good" in e or "known_distinct" in e for e in stolen_errs):
        fails.append(
            "selftest KNOWN-GOOD listed as open debt did not RED "
            f"(got {stolen_errs[:3]!r})"
        )

    silent = copy.deepcopy(base)
    silent["pairs"][0]["status"] = STATUS_ADJUDICATED
    silent["pairs"][0]["adjudication_reason"] = ""
    silent_errs = check_ledger(silent, items)
    if not any("adjudication_reason" in e for e in silent_errs):
        fails.append(
            "selftest ADJUDICATED-WITHOUT-REASON did not RED "
            f"(got {silent_errs[:3]!r})"
        )

    floor_errs = parse_ledger(
        {
            "schema_version": 1,
            "registry": {"min_pairs": 0},
            "pair": base["pairs"],
            "known_distinct": base["known_distinct"],
        },
        "selftest-min-pairs-0",
    )[1]
    floor = copy.deepcopy(base)
    floor["min_pairs"] = 0
    floor_check = check_ledger(floor, items)
    if not floor_errs and not any("min_pairs" in e for e in floor_check):
        fails.append("selftest min_pairs=0 did not ERROR")

    return fails


# ── main ───────────────────────────────────────────────────────────────────


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n", 1)[0])
    ap.add_argument("--ledger", type=Path, default=DEFAULT_LEDGER)
    ap.add_argument("--bank", type=Path, default=DEFAULT_BANK)
    ap.add_argument(
        "--selftest",
        action="store_true",
        help="run only the in-process known-bad plants (still loads the live bank)",
    )
    args = ap.parse_args(argv)

    name = "verify_paraphrase_pairs"
    errors: list[str] = []

    if not args.ledger.is_file():
        print(f"{name}: FAIL: ledger missing: {args.ledger}", file=sys.stderr)
        return 4
    try:
        raw = load_toml(args.ledger)
    except Exception as exc:  # noqa: BLE001
        print(f"{name}: FAIL: ledger unparseable: {exc}", file=sys.stderr)
        return 4
    ledger, ledger_load_errs = parse_ledger(raw, str(args.ledger))
    errors.extend(ledger_load_errs)

    items, item_errs = load_items(args.bank)
    errors.extend(item_errs)

    live_errs = check_ledger(ledger, items) if not args.selftest else []
    if not args.selftest:
        errors.extend(live_errs)

    approved_n = sum(1 for it in items if it["status"] == APPROVED)
    print(
        f"{name}: {len(items)} scanned · {approved_n} approved · "
        f"{len(ledger['pairs'])} ledger pair(s) · "
        f"{len(ledger['known_distinct'])} known-distinct · "
        f"{sum(1 for p in ledger['pairs'] if isinstance(p, dict) and p.get('status') == STATUS_OPEN)} open",
        flush=True,
    )

    if items and approved_n >= 2:
        for line in overlap_report(items, ledger):
            print(line)

    selftest_fails = run_selftest(items)
    if selftest_fails:
        errors.extend(f"SELFTEST: {f}" for f in selftest_fails)
    else:
        print(
            f"{name}: selftest RED on planted missing pair / empty ledger / "
            "zero items / adjudicated-without-reason / known-good-as-debt"
        )

    if errors:
        for e in errors:
            print(f"{name}: FAIL: {e}", file=sys.stderr)
        print(
            f"{name}: {len(errors)} finding(s) — this is a ledger tripwire, "
            "not a license to delete bank items",
            file=sys.stderr,
        )
        return 2 if any("SELFTEST:" not in e for e in errors) or args.selftest else 2

    print(
        f"{name}: ok — ledger intact; 804/779 is a pool size, not a "
        "distinct-proposition count; report is not a verdict"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
