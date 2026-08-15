#!/usr/bin/env sh
# selftest_l7_objectives.sh — L7-S7 objective coverage gate proven to trip
#
# TEMP-only known-bad (never mutates live registries/ or bank/items):
#   a) empty objectives registry → RED
#   b) planted missing claim_id ref → RED
#   c) empty bank → RED (vacuous domain coverage)
#   d) live tree still GREEN
#   e) DECLARED module with zero items → RED, naming the module
#   f) exemption without a reason → RED, and the module STAYS required
#   g) [[domain_min]] for an undeclared module → RED (cross-source drift)
#   h) topic in an undeclared domain → RED (cross-source drift)
#   i) recorded exemption WITH a reason → GREEN control, NOT counted
#
# (e)–(i) are the bd-lt7 rebase: this gate's module set is DERIVED from
# knowledge/domains.toml instead of `range(1, 15)`. (e) is the regression
# itself — under the old literal a registry declaring module 15 with an empty
# bank for it was GREEN — and it must never be dropped.
#
# (i) is a known-GOOD control and is deliberately checked with a plain rc/grep
# rather than assert_fails_with, so it cannot increment INJ. An attack-only
# suite ships an over-strict gate, and over-strict gates get routed around.
#
# Plants run the Rust binary (same helper contract as check.sh):
#   $CDCP_BIN_DIR/cdcp_gate verify-objectives
# scripts/verify_objectives.py is the cargo-test differential oracle only.
#
# Trap cleans TEMP. Never leaves registries/bank dirty.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_l7_objectives: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_l7_objectives: ok: $*"; }

# -- L4 drift guard: self-reported RED-injection count ----------------------
# INJ counts the injections this run actually asserted RED (green controls are
# NOT counted). Emitted once, on the success path only, as a machine-readable
# line that scripts/verify_injection_count.py aggregates. A suite that stops
# emitting the line is an ERROR to that gate, never a silent zero.
INJ=0
SUITE_NAME="selftest_l7_objectives"
inject_counted() { INJ=$((INJ + 1)); }

TMP_ROOT=""
restore_all() {
  if [ -n "${TMP_ROOT:-}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}" 2>/dev/null || true
  fi
}
trap restore_all EXIT INT TERM HUP

# The output of the most recent assert_fails_with, so a case can make a SECOND
# assertion about the same RED run without paying for a second run and without
# counting a second injection. Used by (f): "has no reason" is the finding, and
# "the module stays required" is the consequence that makes the finding matter.
LAST_OUT=""

assert_fails_with() {
  label="$1"
  needle="$2"
  shift 2
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  LAST_OUT="$out"
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

# ── fixture builders for (e)–(i) ───────────────────────────────────────────
# The gate already takes --domains --policy --topics --bank, so every case
# below is a real registry written into TEMP; nothing is patched and nothing
# live is touched. registries/objectives.toml and registries/claims.toml stay
# LIVE on purpose: these cases are about the MODULE SET, and a fixture
# objectives registry would only add a second thing that could be wrong.

# write_domains <path> <order>...
write_domains() {
  _out="$1"
  shift
  printf 'schema_version = 1\n' >"$_out"
  for _o in "$@"; do
    printf '\n[[domain]]\nid = "%02d-fixture"\norder = %d\nepi_heading = "Fixture domain %d"\n' \
      "$_o" "$_o" "$_o" >>"$_out"
  done
}

# write_topics <path> <domain-id>...  (one topic per domain, in order)
write_topics() {
  _out="$1"
  shift
  printf 'schema_version = 1\n' >"$_out"
  _n=0
  for _d in "$@"; do
    _n=$((_n + 1))
    printf '\n[[topic]]\nid = "t-fixture-%d"\ndomain = "%s"\nlabel = "fixture topic %d"\n' \
      "$_n" "$_d" "$_n" >>"$_out"
  done
}

# write_bank <dir> <module>...
write_bank() {
  _dir="$1"
  shift
  rm -rf "$_dir"
  mkdir -p "$_dir"
  for _m in "$@"; do
    printf 'id = "sel-m%02d"\nmodule = %d\nstatus = "approved"\ntopic_ids = ["t-fixture-1"]\n' \
      "$_m" "$_m" >"$_dir/m$_m.toml"
  done
}

# run_objectives <domains> <topics> <bank> <policy>
# Every case passes its own policy file — defaulting to the LIVE
# knowledge/bank_policy.toml would make its [[domain_min]] rows drift against
# a fixture registry and turn each case RED for a reason it did not plant.
run_objectives() {
  verify_objectives \
    --objectives registries/objectives.toml \
    --claims registries/claims.toml \
    --domains "$1" \
    --topics "$2" \
    --bank "$3" \
    --policy "$4" \
    --skip-topic-coverage
}

echo "==> selftest_l7_objectives (L7-S7 objective coverage known-bad)"

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

verify_objectives() {
  "$CDCP_BIN_DIR/cdcp_gate" verify-objectives "$@"
}

[ -f registries/objectives.toml ] || fail "missing registries/objectives.toml"
[ -f registries/claims.toml ] || fail "missing registries/claims.toml"
[ -d bank/items ] || fail "missing bank/items"

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/selftest_l7_objectives.XXXXXX")"

# --- (a) empty objectives registry → ERROR ---
echo "==> (a) empty objectives → ERROR"
empty_obj="$TMP_ROOT/objectives_empty.toml"
cat >"$empty_obj" <<'EOF'
schema_version = 1

[registry]
name = "objectives"
description = "selftest empty — must RED"
EOF
assert_fails_with "empty-objectives" "zero [[objective]]" \
  verify_objectives \
    --objectives "$empty_obj" \
    --claims registries/claims.toml \
    --bank bank/items \
    --skip-topic-coverage
ok "empty objectives temp only"

# --- (b) planted missing claim ref → RED ---
echo "==> (b) missing claim_id ref → RED"
bad_obj="$TMP_ROOT/objectives_bad_claim.toml"
cat >"$bad_obj" <<'EOF'
schema_version = 1

[registry]
name = "objectives"
description = "selftest planted unresolved claim"

[[objective]]
id = "obj-selftest-unresolved"
text = "planted objective with missing claim ref"
claim_ids = ["claim-does-not-exist-selftest-only"]
EOF
assert_fails_with "missing-claim-ref" "unresolved claim_id" \
  verify_objectives \
    --objectives "$bad_obj" \
    --claims registries/claims.toml \
    --bank bank/items \
    --skip-topic-coverage
ok "missing claim ref trips RED"

# --- (b2) empty claim_ids on objective → RED ---
echo "==> (b2) empty claim_ids → RED"
empty_claims_obj="$TMP_ROOT/objectives_empty_claims.toml"
cat >"$empty_claims_obj" <<'EOF'
schema_version = 1

[registry]
name = "objectives"
description = "selftest empty claim_ids"

[[objective]]
id = "obj-selftest-no-claims"
text = "planted objective with empty claim_ids"
claim_ids = []
EOF
assert_fails_with "empty-claim-ids" "claim_ids empty" \
  verify_objectives \
    --objectives "$empty_claims_obj" \
    --claims registries/claims.toml \
    --bank bank/items \
    --skip-topic-coverage
ok "empty claim_ids trips RED"

# --- (c) empty bank → vacuous domain ERROR ---
echo "==> (c) empty bank → ERROR"
empty_bank="$TMP_ROOT/empty_bank"
mkdir -p "$empty_bank"
assert_fails_with "empty-bank" "empty bank" \
  verify_objectives \
    --objectives registries/objectives.toml \
    --claims registries/claims.toml \
    --bank "$empty_bank" \
    --skip-topic-coverage
ok "empty bank trips RED"

# --- (d) live tree GREEN ---
echo "==> (d) live tree objective coverage GREEN"
rc=0
live_out="$(verify_objectives \
  --objectives registries/objectives.toml \
  --claims registries/claims.toml \
  --bank bank/items 2>&1)" || rc=$?
printf '%s\n' "$live_out"
[ "$rc" -eq 0 ] || fail "live verify_objectives exited $rc (selftest must not dirty tree)"
printf '%s\n' "$live_out" | grep -q 'objective coverage GREEN' \
  || fail "live output missing 'objective coverage GREEN'"
ok "live tree still GREEN"

# ── (e)–(i): the bd-lt7 rebase, in the suite instead of in a scratchpad ─────
# Shared fixture pieces. An "empty" policy is a real file with no rows, so the
# exemption ledger is READ and found to hold nothing — not absent, which is a
# different code path.
empty_policy="$TMP_ROOT/policy_empty.toml"
printf '# fixture policy: no rows\n' >"$empty_policy"
topics_ok="$TMP_ROOT/topics_ok.toml"
write_topics "$topics_ok" "01-fixture"

# --- (e) a DECLARED module with zero bank items → RED, naming the module ---
# THE bd-lt7 regression. Under `PRIMARY_MODULES = range(1, 15)` this exact
# tree was GREEN: module 15 was declared, assessed nowhere, and exempt by a
# literal nobody had written down as a decision.
echo "==> (e) declared module starved of items → RED"
dom_1_15="$TMP_ROOT/domains_1_15.toml"
write_domains "$dom_1_15" 1 15
bank_1="$TMP_ROOT/bank_m1"
write_bank "$bank_1" 1
assert_fails_with "declared-module-starved" "domain module 15: 0 approved < min 1" \
  run_objectives "$dom_1_15" "$topics_ok" "$bank_1" "$empty_policy"

# --- (f) an exemption without a reason → RED, and the module stays required ---
echo "==> (f) exemption without a reason → RED"
pol_no_reason="$TMP_ROOT/policy_no_reason.toml"
printf '[[coverage_exempt]]\nmodule = 15\n' >"$pol_no_reason"
assert_fails_with "exemption-without-reason" \
  "coverage_exempt module 15 has no reason" \
  run_objectives "$dom_1_15" "$topics_ok" "$bank_1" "$pol_no_reason"
# The finding alone is not the guarantee. A rejected exemption that still held
# the module out of the floor would be the escape hatch working while being
# reported as broken — quieter than the rule it escapes.
printf '%s\n' "$LAST_OUT" | grep -q 'domain module 15: 0 approved < min 1' \
  || fail "(f) a rejected exemption silently held module 15 out of the floor"
ok "(f) rejected exemption leaves module 15 REQUIRED (not a second injection)"

# --- (g) a [[domain_min]] floor for an undeclared module → RED ---
echo "==> (g) [[domain_min]] for an undeclared module → RED"
dom_1="$TMP_ROOT/domains_1.toml"
write_domains "$dom_1" 1
pol_stray_min="$TMP_ROOT/policy_stray_min.toml"
printf '[[domain_min]]\nmodule = 1\nmin_items = 1\n\n[[domain_min]]\nmodule = 15\nmin_items = 16\n' \
  >"$pol_stray_min"
assert_fails_with "domain-min-undeclared" \
  "[[domain_min]] module 15 is not declared in the domain registry" \
  run_objectives "$dom_1" "$topics_ok" "$bank_1" "$pol_stray_min"

# --- (h) a topic in a domain the registry never declared → RED ---
# The fixture carries one GOOD topic as well, so the finding proves the drift
# detector fired rather than the "zero topics in a required domain" floor.
echo "==> (h) topic in an undeclared domain → RED"
topics_drift="$TMP_ROOT/topics_drift.toml"
write_topics "$topics_drift" "01-fixture" "99-never-declared"
assert_fails_with "topic-undeclared-domain" \
  "topics.toml: topic in an undeclared domain" \
  run_objectives "$dom_1" "$topics_drift" "$bank_1" "$empty_policy"

# --- (i) a RECORDED exemption WITH a reason → GREEN control, NOT counted ---
# Plain rc/grep by design: assert_fails_with is the only thing that increments
# INJ, and a green leg must never inflate the advertised known-bad count.
echo "==> (i) recorded exemption with a reason → GREEN (control, not counted)"
pol_with_reason="$TMP_ROOT/policy_with_reason.toml"
printf '[[coverage_exempt]]\nmodule = 15\nreason = "fixture: module not yet authored"\n' \
  >"$pol_with_reason"
rc=0
exempt_out="$(run_objectives "$dom_1_15" "$topics_ok" "$bank_1" "$pol_with_reason" 2>&1)" || rc=$?
if [ "$rc" -ne 0 ]; then
  printf '%s\n' "$exempt_out" >&2
  fail "(i) a recorded exemption with a reason must be honoured, exited $rc"
fi
printf '%s\n' "$exempt_out" | grep -q 'objective coverage GREEN' \
  || fail "(i) exited 0 without the 'objective coverage GREEN' receipt"
printf '%s\n' "$exempt_out" | grep -q 'exempt: fixture: module not yet authored' \
  || fail "(i) the exemption must be PRINTED, so the hole is visible"
ok "(i) recorded exemption honoured and printed (GREEN control, NOT counted)"

# Confirm no planted files under live registries/bank
if [ -f registries/objectives_empty.toml ] || [ -f bank/items/planted-obj.toml ]; then
  fail "planted file leaked into live tree"
fi

# Anti-vacuous: a suite that discovered zero plants reports like a pass.
# Eight RED plants are the contract (a,b,b2,c,e,f,g,h). Dropping one is RED here,
# not a quieter receipt for verify_injection_count to notice later.
[ "$INJ" -gt 0 ] || fail "zero plants discovered (vacuous known-bad suite is ERROR)"
[ "$INJ" -eq 8 ] || fail "expected 8 RED plants, got $INJ (do not drop a plant)"

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_l7_objectives: PASSED (a empty RED · b missing-claim RED · b2 empty-claims RED · c empty-bank RED · d live GREEN · e starved-module RED · f reasonless-exemption RED · g stray domain_min RED · h undeclared topic domain RED · i recorded exemption GREEN)"
exit 0
