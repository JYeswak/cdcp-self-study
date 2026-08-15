#!/usr/bin/env sh
# selftest_doc_consistency.sh — the roadmap-truth gate, proven to trip.
#
# Contract (mirrors scripts/selftest_l6_coverage.sh and scripts/selftest_orphan.sh):
#   write a real, self-contained specimen roadmap into TEMP, inject one known-bad
#   at a time, assert `$CDCP_BIN_DIR/cdcp_gate verify-doc-consistency` goes RED
#   with the expected signal, then restore the specimen. The live
#   CHARTER/README/PHASE-NEXT are NEVER mutated and no patch is ever applied
#   to the working tree — the specimens are files this script writes, so they
#   cannot silently no-op the way `git apply` does on an index mismatch.
#
# Plants run the Rust binary (same helper contract as check.sh).
# scripts/verify_doc_consistency.py is the cargo-test differential oracle only.
#
# Cases:
#   a) clean specimen                       → GREEN (baseline; injections are the only defect)
#   b) one milestone twice in one table     → RED (duplicate row)
#   c) same milestone, two statuses         → RED (cross-doc contradiction)
#   d) status cell in unknown vocabulary    → RED (fail-closed, not fail-open)
#   e) "going public is pending" line       → RED (repo is public; unmarked)
#   f) root with zero markdown              → ERROR (anti-vacuous)
#   g) roadmap doc missing                  → ERROR (cannot verify agreement)
#   h) row too short to reach its Status    → RED (ragged row, fail-closed)
#   i) table that names the detector        → GREEN (bd-1sd.12; not counted)
#   j) <!-- doc-truth: describes-detector --> → GREEN (not counted)
#   k) closed fence quoting the trigger     → GREEN (not counted)
# GREEN controls do not increment INJ.
#
# The live-tree run is check.sh's own preceding step; this script proves the
# checker bites. If ANY injection stays GREEN, this exits non-zero.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_doc_consistency: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_doc_consistency: ok: $*"; }

# -- L4 drift guard: self-reported RED-injection count ----------------------
# INJ counts the injections this run actually asserted RED (green controls are
# NOT counted). Emitted once, on the success path only, as a machine-readable
# line that scripts/verify_injection_count.py aggregates. A suite that stops
# emitting the line is an ERROR to that gate, never a silent zero.
INJ=0
SUITE_NAME="selftest_doc_consistency"
inject_counted() { INJ=$((INJ + 1)); }

TMP_ROOT=""
restore_all() {
  if [ -n "${TMP_ROOT:-}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}" 2>/dev/null || true
  fi
}
trap restore_all EXIT INT TERM HUP

# assert_fails_with <label> <needle> cmd...
assert_fails_with() {
  label="$1"
  needle="$2"
  shift 2
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  if [ "$rc" -eq 0 ]; then
    printf '%s\n' "$out" >&2
    fail "expected RED for $label but command exited 0"
  fi
  case "$out" in
    *"$needle"*)
      inject_counted
      ok "$label trips RED (rc=$rc, saw: $needle)"
      ;;
    *)
      printf '%s\n' "$out" >&2
      fail "$label exited $rc but missing expected signal '$needle'"
      ;;
  esac
}

assert_green() {
  label="$1"
  shift
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  if [ "$rc" -ne 0 ]; then
    printf '%s\n' "$out" >&2
    fail "$label expected GREEN but exited $rc"
  fi
  case "$out" in
    *"roadmap GREEN"*) ok "$label GREEN" ;;
    *)
      printf '%s\n' "$out" >&2
      fail "$label exited 0 without the 'roadmap GREEN' receipt"
      ;;
  esac
}

echo "==> selftest_doc_consistency (roadmap-truth known-bad)"

# Same binary contract as check.sh: honour CARGO_TARGET_DIR, never cargo run.
if [ -z "${CDCP_BIN_DIR:-}" ]; then
  if [ -n "${CARGO_TARGET_DIR:-}" ]; then
    CDCP_BIN_DIR="${CARGO_TARGET_DIR%/}/debug"
  else
    CDCP_BIN_DIR="$ROOT/target/debug"
  fi
fi
[ -n "$CDCP_BIN_DIR" ] \
  || fail "CDCP_BIN_DIR unset — cargo build -p cdcp_gate -p cdcp_cli --locked must run first (no fallback to cargo run)"
[ -x "$CDCP_BIN_DIR/cdcp_gate" ] \
  || fail "cdcp_gate binary absent at $CDCP_BIN_DIR/cdcp_gate — cargo build -p cdcp_gate -p cdcp_cli --locked did not produce it (no fallback to cargo run)"

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/selftest_doc_consistency.XXXXXX")"
SPEC="$TMP_ROOT/specimen"

write_specimen() {
  rm -rf "$SPEC"
  mkdir -p "$SPEC/course-engine/docs"

  cat >"$SPEC/CHARTER.md" <<'EOF'
# Specimen charter

## 9. Milestones

| ID | Milestone | Status |
|----|-----------|--------|
| **M0–M2** | scaffold · registries · bank | **DONE** |
| **V11** | stretch surfaces | **DONE** |
| **M8** | learn v2 | **GREEN** |
| **M9** | publicize | **DONE** |
EOF

  cat >"$SPEC/README.md" <<'EOF'
# Specimen readme

## Roadmap

| ID | Milestone | Status |
|---|---|---|
| M0–M2 | scaffold · registries · bank | **done** |
| V11 | stretch surfaces | **done** |
| M8 | learn v2 | **done** |
| M9 | publicize | **DONE** (2026-08-12) |
EOF

  cat >"$SPEC/course-engine/docs/PHASE-NEXT.md" <<'EOF'
# Specimen phase-next

## Done (do not re-plan)

| Wave | Outcome |
|------|---------|
| **V11** | stretch surfaces |
| **M8** | learn v2 |
| **M9-S1/S2** | bar + OSS meta |
EOF
}

run_checker() {
  "$CDCP_BIN_DIR/cdcp_gate" verify-doc-consistency --repo-root "$1"
}

# ── (a) clean specimen must be GREEN ────────────────────────────────────────
echo "==> (a) clean specimen → GREEN"
write_specimen
assert_green "clean-specimen" run_checker "$SPEC"

# ── (b) duplicate milestone row inside one table ────────────────────────────
echo "==> (b) duplicate milestone row → RED"
write_specimen
printf '%s\n' '| **M9** | publicize (stale copy) | **open** |' >>"$SPEC/CHARTER.md"
assert_fails_with "duplicate-row" "appears twice in the same table" \
  run_checker "$SPEC"

# ── (c) same milestone, contradictory status across docs ────────────────────
echo "==> (c) conflicting status across docs → RED"
write_specimen
sed 's/| M8 | learn v2 | \*\*done\*\* |/| M8 | learn v2 | **ongoing** |/' \
  "$SPEC/README.md" >"$SPEC/README.tmp" && mv "$SPEC/README.tmp" "$SPEC/README.md"
grep -q 'ongoing' "$SPEC/README.md" || fail "injection (c) did not apply — specimen drifted"
assert_fails_with "conflicting-status" "conflicting status across the roadmap docs" \
  run_checker "$SPEC"

# ── (d) status vocabulary the gate cannot read must FAIL CLOSED ─────────────
echo "==> (d) unreadable status vocabulary → RED"
write_specimen
sed 's/| M8 | learn v2 | \*\*done\*\* |/| M8 | learn v2 | mostly there |/' \
  "$SPEC/README.md" >"$SPEC/README.tmp" && mv "$SPEC/README.tmp" "$SPEC/README.md"
grep -q 'mostly there' "$SPEC/README.md" || fail "injection (d) did not apply — specimen drifted"
assert_fails_with "unreadable-status" "unrecognised status vocabulary" \
  run_checker "$SPEC"

# ── (e) a doc that still calls publication pending ──────────────────────────
echo "==> (e) publication-pending assertion → RED"
write_specimen
printf '\n%s\n' 'Going public is pending.' \
  >>"$SPEC/course-engine/docs/PHASE-NEXT.md"
assert_fails_with "publication-pending" "publication described as not done" \
  run_checker "$SPEC"

# ── (f) anti-vacuous: a root with zero markdown is an ERROR, not a pass ─────
echo "==> (f) zero markdown → ERROR"
empty_root="$TMP_ROOT/empty_root"
mkdir -p "$empty_root"
assert_fails_with "empty-root" "zero markdown files scanned" \
  run_checker "$empty_root"

# ── (g) a missing roadmap doc is an ERROR, not a silent skip ────────────────
echo "==> (g) missing roadmap doc → ERROR"
write_specimen
rm -f "$SPEC/course-engine/docs/PHASE-NEXT.md"
assert_fails_with "missing-roadmap-doc" "roadmap doc missing" \
  run_checker "$SPEC"

# ── (h) a row too short to reach its own Status column ──────────────────────
# The table declares a Status column; this row has two cells and never reaches
# it. Until bd-hw3 that row read as `None` and the gate printed PASS and then
# raised. A row whose status cannot be read is RED, never a row without a
# status — the escape hatch may not be quieter than the rule.
echo "==> (h) ragged milestone row → RED"
write_specimen
printf '%s\n' '| **M10** | ragged row |' >>"$SPEC/CHARTER.md"
assert_fails_with "ragged-row" "row is shorter than its Status column" \
  run_checker "$SPEC"

# ── (i) a table that NAMES the detector must stay GREEN (bd-1sd.12) ─────────
# This is the measured false positive: README listed the injection as
# "publication described as pending" next to `selftest_doc_consistency`.
echo "==> (i) table naming the detector → GREEN"
write_specimen
cat >>"$SPEC/README.md" <<'EOF'

| Suite | n | Injections |
|---|---|---|
| `selftest_doc_consistency` | 7 | publication described as pending |
EOF
assert_green "describe-detector-table" run_checker "$SPEC"

# ── (j) explicit per-line opt-out; unmarked sibling still trips in (e) ──────
echo "==> (j) describes-detector marker → GREEN"
write_specimen
printf '\n%s\n' 'Going public is pending. <!-- doc-truth: describes-detector -->' \
  >>"$SPEC/README.md"
assert_green "describe-detector-marker" run_checker "$SPEC"

# ── (k) a CLOSED fence quoting the trigger is a quotation, not a claim ──────
echo "==> (k) fenced trigger phrase → GREEN"
write_specimen
cat >>"$SPEC/README.md" <<'EOF'

```
Going public is pending.
```
EOF
assert_green "describe-detector-fence" run_checker "$SPEC"

# ── nothing may have leaked out of TEMP ─────────────────────────────────────
write_specimen
assert_green "specimen-restored" run_checker "$SPEC"
[ -d "$TMP_ROOT" ] || fail "TEMP root vanished mid-run"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_doc_consistency: PASSED (a clean GREEN · b duplicate row · c cross-doc conflict · d unreadable status · e publication pending · f zero markdown · g missing doc · h ragged row · i describe-detector table GREEN · j marker GREEN · k fence GREEN)"
exit 0
