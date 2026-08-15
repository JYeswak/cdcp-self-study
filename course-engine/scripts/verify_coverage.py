#!/usr/bin/env python3
"""verify_coverage.py — L6 domain coverage oracle.

# CLAIM: FLOOR-RAISE

Every module the course DECLARES must carry at least N bank items:
  - N from knowledge/bank_policy.toml [[domain_min]] min_items when the
    policy FILE is present and names the module
  - else N=1 (OQ-05 ASSUMED floor) — only when that file is present
A missing policy file is an ERROR (bd-j98g). The sized floors live in
that file; falling back to N=1 would lower them (fail-open). That is the
opposite of verify_objectives, where absence removes exemptions and makes
the gate stricter. A present file with empty [[domain_min]] is the honest
N=1 default and is distinguishable from a missing file.

The module set is DERIVED from knowledge/domains.toml — the same registry
build_learn.py turns into web/data/modules_index.json (the Learn index) and the
same one bank_policy.toml's [[domain_min]] rows are keyed against. It is not a
range literal.

## Why the derivation, and not `range(1, 15)` (bd-lt7)

Until 2026-08-14 this gate read `PRIMARY_MODULES = range(1, 15)` and module 15
was exempted from the floor with the comment "may appear in counts but is not
required for green". Module 15 was, at that time, assessed but never taught —
so the gate had written a KNOWN DEFECT down as a rule. When module 15 was
taught, three sibling gates went RED for the fix being correct. This one did not
go red; it stayed green by LUCK, because an exemption cannot fail. That is worse:
a gate that cannot notice the thing it was supposed to check.

The defect was never "someone hardcoded 14". It was that the bound was derived
from OBSERVED STATE rather than from a stated contract. domains.toml is the
contract. If a module must be exempt, that is a RECORDED row —
`[[coverage_exempt]] module = N, reason = "..."` in bank_policy.toml, with a
reason string — and an exemption without a reason is an ERROR, not a default.

## WHICH POOL THE FLOOR MEASURES (bd-coverage-counts-retired-items-49jh)

The floor is measured against the **approved** pool — `status == "approved"` —
and never against the file set. C1 restricts assembly to approved items
(`cdcp_assemble::sample_item_ids`), so a floor counted over every file is a
floor over a population no learner is ever assessed from, and it fails in the
OPEN direction: the file count can only ever be >= the number that matters, so
the gate can only ever be too generous.

Until 2026-08-14 this gate counted files. That was invisible because the bank
held exactly ONE non-approved item, so the two numbers differed by one in a
single module and no reader would notice. bd-tetz then retired 24 duplicates and
the gap became 25 across ten modules — measured, on the day of the fix:

    m05 31->30  m06 136->130  m07 32->31  m08 34->32  m09 121->117
    m10 35->33  m11 72->69    m12 63->61  m13 48->46  m14 44->42

No module breached its floor on the approved pool, so there was no incident —
but the claim had been unearned since the first retirement, and every future
adjudication widens the gap. The report therefore names BOTH numbers on every
line that carries a count, so the two populations can never be confused again.

A status outside `approved`/`draft`/`retired` is an ERROR naming the item, not a
silent drop into "not approved". `cdcp_bank` rejects an unknown status at load
for the same reason: a value nobody modelled must not be bucketed by guess.

## VERDICT SHAPE (bd-verify-coverage-verdict-before-write-rk9n)

**No line claiming success is emitted on a path that can still return non-zero.**
This script used to `print(status)` at the top of the report and then, 57 lines
later, run an unguarded `mkdir` + `write_text` under `--write-json`. An OSError
there (read-only destination, ENOSPC, a path whose parent is a file) propagated
out of `main()` and CPython exited 1 with a traceback underneath a stdout that
already said PASS. The command is the one printed in the artifact's own `note`
field, so the docs told operators to run exactly that path.

The report is now COMPOSED into a buffer and printed once, after the write has
succeeded. A failed write therefore prints NOTHING and exits non-zero, which is
the same shape the third instance of this class was fixed into
(`build_glossary_json.py`, bd-builder-verdict-shape-qm65) and the second
(`build_units.py`, bd-lt7): the side effect depends on the verdict, never the
reverse.

The write is also atomic — temp file in the destination directory, then
`Path.replace` — so a failed or torn write leaves NO partial artifact for a
later reader to mistake for the residue of a passing run.

## Anti-vacuous

Zero modules discovered is an ERROR. Zero items loaded is an ERROR. Zero
APPROVED items is an ERROR even when the bank is full of files, because that is
precisely the state in which a file-counting floor would have reported green. An
empty scan set must never report like a scan that ran and came back clean. That
rule holds at FILE granularity too: a single bank file whose `items[]` yields
zero items is named and is RED, because the aggregate count would otherwise stay
healthy on the strength of the files around it (bd-0czh).

## What this gate cannot decide

It counts items, not coverage: twenty near-identical approved items satisfy a
floor of twenty. It says nothing about whether an item is correct, well written,
or mapped to the right topic, and nothing about exam pass probability. A module
above its floor is a module that is not STARVED, which is all that is claimed.

Exit 0 with per-module counts; non-zero if the bank is empty, nothing in it is
approved, the registry is empty, the policy file is missing, an exemption is
malformed, any required module is below N, or the `--write-json` summary could
not be written.

Optional: --write-json PATH writes a machine-readable summary. Not a shipped product input (bd-smvb).

Omitted --policy means "bank_policy.toml beside the domains file this run
loaded", never the shipped knowledge/bank_policy.toml. A live-tree invocation
(no --domains, or --domains knowledge/domains.toml) still lands on the live
policy. An isolated --bank/--domains fixture does not (bd-conu). If the
resolved path is not a file, that is an ERROR, not an N=1 fallback (bd-j98g).
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BANK = ROOT / "bank" / "items"
DEFAULT_POLICY = ROOT / "knowledge" / "bank_policy.toml"
DEFAULT_DOMAINS = ROOT / "knowledge" / "domains.toml"
# DEFAULT_POLICY is the LIVE-TREE location. It is NOT the argparse default
# (bd-conu): omitted --policy resolves to bank_policy.toml beside the domains
# file that this run actually loaded. A fixture that passed isolated
# --bank/--domains used to pick up the shipped [[domain_min]] rows and go
# RED (or GREEN) for a reason it did not inject.

# OQ-05 ASSUMED floor. Applied only when the policy FILE is present and a
# module has no [[domain_min]] row. File-absent is ERROR (bd-j98g), not this.
DEFAULT_N = 1

# C1 lifecycle. `APPROVED` is the ONLY status `cdcp_assemble` may draw, so it is
# the only population a floor may be measured against. A missing status is
# `draft` by C1's default — silence never publishes — and anything outside this
# set is an ERROR rather than a guess.
APPROVED = "approved"
KNOWN_STATUSES = ("approved", "draft", "retired")


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_declared_modules(domains_path: Path) -> tuple[dict[int, str], list[str]]:
    """The module set, derived from the domain registry. Never a range literal.

    Returns ({order: domain_id}, errors). A registry that is missing, malformed,
    or empty yields zero modules AND an error — never a silent empty set that
    would make every floor below vacuously satisfied.
    """
    errors: list[str] = []
    declared: dict[int, str] = {}
    if not domains_path.is_file():
        return declared, [f"domain registry missing: {domains_path}"]
    try:
        data = load_toml(domains_path)
    except Exception as e:  # noqa: BLE001 — fail-closed on a bad registry
        return declared, [f"domain registry parse error: {e}"]

    for row in data.get("domain") or []:
        if not isinstance(row, dict):
            errors.append(f"domains.toml: [[domain]] row is not a table: {row!r}")
            continue
        did = str(row.get("id") or "").strip()
        try:
            order = int(row["order"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"domains.toml: {did or row!r} has no usable order")
            continue
        if order in declared:
            errors.append(
                f"domains.toml: duplicate order {order} ({declared[order]} and {did})"
            )
            continue
        declared[order] = did or f"module-{order}"

    if not declared:
        errors.append(
            "domain registry declares zero modules (vacuous coverage is ERROR)"
        )
    return declared, errors


def load_exemptions(
    policy_path: Path, declared: dict[int, str]
) -> tuple[dict[int, str], list[str]]:
    """Recorded coverage exemptions: `[[coverage_exempt]] module, reason`.

    An exemption is the ONLY sanctioned way to hold a declared module out of the
    floor, and it must say why. A row without a non-empty reason, for an
    undeclared module, or contradicting an explicit [[domain_min]] floor, is an
    ERROR — the escape hatch may not be quieter than the rule it escapes.
    """
    errors: list[str] = []
    exempt: dict[int, str] = {}
    # Missing file: empty exemptions (stricter — same direction as
    # verify_objectives). Do NOT copy that gate's ABSENT-OK sentence here:
    # the floors path ERRORs on the same absence, because absence would
    # lower sized [[domain_min]] rows. See load_domain_mins (bd-j98g).
    if not policy_path.is_file():
        return exempt, errors
    bp = load_toml(policy_path)
    floors = {
        int(r["module"])
        for r in (bp.get("domain_min") or [])
        if isinstance(r, dict) and str(r.get("module", "")).strip().lstrip("-").isdigit()
    }
    for row in bp.get("coverage_exempt") or []:
        if not isinstance(row, dict):
            errors.append(f"bank_policy.toml: coverage_exempt row is not a table: {row!r}")
            continue
        try:
            mod = int(row["module"])
        except (KeyError, TypeError, ValueError):
            errors.append(
                f"bank_policy.toml: coverage_exempt row has no usable module: {row!r}"
            )
            continue
        reason = str(row.get("reason") or "").strip()
        if not reason:
            errors.append(
                f"bank_policy.toml: coverage_exempt module {mod} has no reason "
                f"(an exemption without a reason is a schema error)"
            )
            continue
        if mod not in declared:
            errors.append(
                f"bank_policy.toml: coverage_exempt module {mod} is not in the "
                f"domain registry"
            )
            continue
        if mod in floors:
            errors.append(
                f"bank_policy.toml: module {mod} is both coverage_exempt and has a "
                f"[[domain_min]] floor — pick one"
            )
            continue
        exempt[mod] = reason
    return exempt, errors


def load_domain_mins(
    policy_path: Path, required: list[int]
) -> tuple[dict[int, int], list[str]]:
    """Per-module floors from [[domain_min]]; default N=1 when the FILE is
    present but a module has no row.

    Absence of the file is an ERROR, not a fallback (bd-j98g). The sized
    floors live here; defaulting to N=1 would lower them (fail-open). That
    is the opposite of verify_objectives, where absence removes exemptions
    and makes the gate stricter. A present file with empty [[domain_min]]
    is the honest N=1 default — distinguishable from a missing file.

    A [[domain_min]] row for a module the registry does not declare is an ERROR:
    the two sources of truth for "which modules exist" have drifted, and that
    drift is exactly how module 15 came to be assessed without being taught.
    """
    errors: list[str] = []
    mins: dict[int, int] = {m: DEFAULT_N for m in required}
    if not policy_path.is_file():
        errors.append(
            f"bank_policy.toml missing: {policy_path} "
            f"(absence would lower sized [[domain_min]] floors to N=1)"
        )
        return mins, errors
    bp = load_toml(policy_path)
    rows = bp.get("domain_min") or []
    for row in rows:
        try:
            mod = int(row["module"])
            need = int(row["min_items"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"bank_policy.toml: unusable [[domain_min]] row {row!r}")
            continue
        if mod in mins:
            mins[mod] = max(1, need)  # never below OQ-05 floor of 1
        else:
            errors.append(
                f"bank_policy.toml: [[domain_min]] module {mod} is not a required "
                f"module in the domain registry"
            )
    return mins, errors


def load_items(bank_dir: Path) -> tuple[list[tuple[str, dict]], list[str]]:
    """Load item dicts from *.toml under bank_dir. Returns (loaded, errors)."""
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
                # Anti-vacuous at FILE granularity (bd-0czh, the class sweep of
                # bd-2kr). `items = []` — or an items[] holding nothing this loop
                # can read as an item — takes the list branch, adds nothing, and
                # never reaches the `no id or items[]` leg below, because `elif`
                # cannot run once `if` has. Without this line a file that was
                # never really checked reports exactly like one that passed, and
                # the aggregate `empty bank` check below stays satisfied because
                # the other files carry the count.
                errors.append(
                    f"{path.name}: items[] yielded zero items "
                    "(vacuous file scan is ERROR)"
                )
        elif "id" in data:
            loaded.append((path.name, data))
        else:
            errors.append(f"{path.name}: no id or items[]")
    return loaded, errors


def count_modules(
    loaded: list[tuple[str, dict]],
) -> tuple[Counter[int], Counter[int], list[str]]:
    """Two populations, and the report carries both.

    Returns `(approved, scanned, errors)`. `approved` is what the floors are
    measured against — the pool C1 restricts assembly to. `scanned` is every
    item that loaded, whatever its status, and exists so the report can name
    both numbers side by side; a report that showed only one of them is how a
    floor came to be checked against a set no learner draws from.
    """
    approved: Counter[int] = Counter()
    scanned: Counter[int] = Counter()
    errors: list[str] = []
    for fname, it in loaded:
        mod = it.get("module")
        try:
            mi = int(mod)
        except (TypeError, ValueError):
            iid = it.get("id") or fname
            errors.append(f"{iid}: bad module {mod!r}")
            continue
        scanned[mi] += 1
        status = it.get("status", "draft")
        if status == APPROVED:
            approved[mi] += 1
        elif status not in KNOWN_STATUSES:
            # Fail-closed AND loud. Dropping an unmodelled status silently into
            # "not approved" would be the same defect one level down: a bucket
            # decided by guess rather than by the recorded lifecycle.
            iid = it.get("id") or fname
            errors.append(f"{iid}: unknown status {status!r}")
    return approved, scanned, errors


def write_summary(out: Path, body: str) -> None:
    """Write the `--write-json` summary so that a FAILED write leaves NOTHING.

    Temp file in the DESTINATION directory, then `Path.replace`, which is atomic
    on the same filesystem. A torn or refused write therefore never leaves a
    partial `coverage.json` behind for a later reader to mistake for the ledger
    of a run that passed — and on any failure the exception propagates, so the
    caller never gets to print a verdict over it.

    Any exception is re-raised after the temp file is removed. The removal is
    best-effort: if the directory is unwritable the temp file was never created
    in the first place, and if it cannot be unlinked the original exception is
    still the one that reaches the operator.
    """
    out.parent.mkdir(parents=True, exist_ok=True)
    tmp = out.with_name(out.name + ".tmp")
    try:
        tmp.write_text(body, encoding="utf-8")
        tmp.replace(out)
    except BaseException:
        try:
            tmp.unlink()
        except OSError:
            pass
        raise


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="L6 domain coverage oracle (module set derived from domains.toml)"
    )
    ap.add_argument(
        "--bank",
        type=Path,
        default=DEFAULT_BANK,
        help=f"bank items directory (default: {DEFAULT_BANK})",
    )
    ap.add_argument(
        "--policy",
        type=Path,
        default=None,
        help=(
            "bank_policy.toml path (default: bank_policy.toml beside the "
            "domains registry; omitted never falls back to the shipped tree)"
        ),
    )
    ap.add_argument(
        "--domains",
        type=Path,
        default=DEFAULT_DOMAINS,
        help=f"domains.toml registry path (default: {DEFAULT_DOMAINS})",
    )
    ap.add_argument(
        "--write-json",
        type=Path,
        default=None,
        help="optional path to write coverage.json summary",
    )
    args = ap.parse_args(argv)

    bank_dir = args.bank
    if not bank_dir.is_absolute():
        bank_dir = (ROOT / bank_dir).resolve()
    domains_path = args.domains
    if not domains_path.is_absolute():
        domains_path = (ROOT / domains_path).resolve()
    # Same root as the domains this run loaded. Never ROOT/knowledge/ unless
    # that is where --domains (or its default) actually points.
    if args.policy is None:
        policy_path = domains_path.parent / DEFAULT_POLICY.name
    else:
        policy_path = args.policy
        if not policy_path.is_absolute():
            policy_path = (ROOT / policy_path).resolve()

    errors: list[str] = []

    declared, declared_errors = load_declared_modules(domains_path)
    errors.extend(declared_errors)
    exempt, exempt_errors = load_exemptions(policy_path, declared)
    errors.extend(exempt_errors)

    required = sorted(m for m in declared if m not in exempt)
    domain_mins, min_errors = load_domain_mins(policy_path, required)
    errors.extend(min_errors)

    loaded, load_errors = load_items(bank_dir)
    module_counts, scanned_counts, mod_errors = count_modules(loaded)
    errors.extend(load_errors)
    errors.extend(mod_errors)

    n = len(loaded)
    approved_n = sum(module_counts.values())
    # Vacuous empty = ERROR (anti-vacuous: empty scan set must not pass)
    if n == 0:
        errors.append("empty bank: zero items loaded (vacuous coverage is ERROR)")
    # A bank FULL of files and empty of drawable items is the exact state a
    # file-counting floor reported green on. It is named separately from the
    # empty-bank leg because it is a different failure with the same verdict.
    elif approved_n == 0:
        errors.append(
            f"zero approved items ({n} scanned): the floors measure a pool no "
            "learner can be assessed from (vacuous coverage is ERROR)"
        )
    # …and so is a run with nothing left to require. A gate whose required set
    # emptied out reports exactly like one that checked everything and found it
    # sound, which is the failure this whole rebase exists to remove.
    if not required:
        errors.append(
            "zero required modules after exemptions (vacuous coverage is ERROR)"
        )

    shortfalls: list[dict] = []
    for mod in required:
        need = domain_mins[mod]
        have = module_counts.get(mod, 0)
        seen = scanned_counts.get(mod, 0)
        if have < need:
            # Both numbers, deliberately: `44 scanned` under a floor of 24 is
            # exactly the reading that made this fail open for a week.
            msg = (
                f"module {mod}: {have} approved < min {need} "
                f"({seen} scanned, {seen - have} not approved)"
            )
            errors.append(msg)
            shortfalls.append(
                {"have": have, "min": need, "module": mod, "scanned": seen}
            )

    # Report: every required module, then recorded exemptions, then anything the
    # bank carries that the registry never declared.
    #
    # COMPOSED, NOT PRINTED AS IT GOES. See the header: the verdict is emitted
    # once, at the end, after the `--write-json` side effect has succeeded, so
    # no success token can ever land above a failure that came later.
    status = "PASS" if not errors else "FAIL"
    lines: list[str] = [status]
    lines.append(f"  bank={bank_dir}")
    lines.append(
        f"  items={n} scanned, {approved_n} approved "
        f"(floors count the approved pool only)"
    )
    lines.append(
        f"  policy={'present' if policy_path.is_file() else 'absent'}"
    )
    lines.append(f"  registry={domains_path.name} declares={len(declared)}")
    lines.append(
        f"  modules ({len(required)} required, derived from the domain registry):"
    )
    for mod in required:
        have = module_counts.get(mod, 0)
        seen = scanned_counts.get(mod, 0)
        need = domain_mins[mod]
        flag = "ok" if have >= need and n > 0 else "SHORT"
        lines.append(
            f"    m{mod:02d}: {have} approved of {seen} scanned (min {need}) [{flag}]"
        )
    if exempt:
        lines.append("  recorded exemptions (bank_policy.toml [[coverage_exempt]]):")
        for mod in sorted(exempt):
            have = module_counts.get(mod, 0)
            seen = scanned_counts.get(mod, 0)
            lines.append(
                f"    m{mod:02d}: {have} approved of {seen} scanned — exempt: {exempt[mod]}"
            )
    # Drift is a property of the FILE SET, not of the drawable pool: a retired
    # item filed under a module the registry never declared is still drift, and
    # counting extras on the approved pool would hide it.
    extras = sorted(m for m in scanned_counts if m not in declared)
    if extras:
        lines.append("  undeclared modules present in the bank (not required for green):")
        for mod in extras:
            lines.append(
                f"    m{mod:02d}: {scanned_counts[mod]} scanned "
                f"(not in the domain registry)"
            )

    # Prefer repo-relative bank path in JSON for portable commits
    try:
        bank_rel = str(bank_dir.resolve().relative_to(ROOT.resolve()))
    except ValueError:
        bank_rel = str(bank_dir)

    summary = {
        # v3: `counts` changed population. It was the file set and is now the
        # APPROVED pool, which is a semantic change no consumer could detect
        # from the numbers alone — so the version moves with it. `item_count`
        # keeps its old meaning (everything scanned) and `approved_count` and
        # `scanned_counts` are added, so both populations are in the ledger.
        "schema_version": 3,
        "gate": "l6-domain-coverage",
        "status": status.lower(),
        "bank": bank_rel,
        "item_count": n,
        "approved_count": approved_n,
        "module_source": domains_path.name,
        "declared_modules": sorted(declared),
        "primary_modules": required,
        "exemptions": {str(k): v for k, v in sorted(exempt.items())},
        "domain_min": {str(k): v for k, v in sorted(domain_mins.items())},
        "counts": {str(k): module_counts.get(k, 0) for k in required},
        "scanned_counts": {str(k): scanned_counts.get(k, 0) for k in required},
        "extra_counts": {str(k): scanned_counts[k] for k in extras},
        "shortfalls": shortfalls,
        "oq05_default_n": DEFAULT_N,
        # The note must not name a tracked path or a regenerate command that
        # can go stale when the producer changes (bd-smvb). --write-json is an
        # optional operator dump, not a shipped product input.
        "note": (
            "Coverage ≠ exam pass probability; study signal only. "
            "Optional --write-json operator summary; not a shipped "
            "product input."
        ),
    }

    # THE SIDE EFFECT RUNS BEFORE THE VERDICT IS PRINTED, not after it. Nothing
    # above this point has reached stdout, so an OSError here exits 1 with an
    # EMPTY stdout and a traceback — never with a PASS a reader would have
    # believed. The `status` baked into the summary is the pre-write verdict,
    # which is sound precisely because the file only exists when the write
    # succeeded, and when it succeeded the pre-write verdict is the final one.
    if args.write_json is not None:
        out = args.write_json
        if not out.is_absolute():
            out = (ROOT / out).resolve()
        write_summary(out, json.dumps(summary, indent=2, sort_keys=True) + "\n")
        lines.append(f"  wrote {out}")

    if errors:
        lines.append("  failures:")
        for e in errors[:40]:
            lines.append(f"    - {e}")
        if len(errors) > 40:
            lines.append(f"    ... +{len(errors) - 40} more")
        print("\n".join(lines))
        return 1

    # Enumerated, not spanned: an exemption can leave a gap, and "1–15" would
    # read as covering a module that was held out.
    span = " ".join(f"m{m:02d}" for m in required)
    lines.append(
        f"  coverage GREEN ({len(required)} required modules ≥ domain_min: {span})"
    )
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main())
