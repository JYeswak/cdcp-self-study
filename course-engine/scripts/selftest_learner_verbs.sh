#!/usr/bin/env sh
# selftest_learner_verbs.sh — L4 plants for study / demo / test [bd-installability-sm4g.16]
#
# BUILT != WIRED. The verbs shipped; check.sh did not invoke them.
# This suite proves the live wrappers are fail-closed:
#   1) missing/corrupt wasm → `cdcp test` RED, names test/wasm
#   2) missing bundle      → `cdcp demo` RED, names the verb / bundle
#   3) missing bundle      → `cdcp study` exit 4, names bundle
#   4) ignore-exit is RED  — check.sh must contain `run_cdcp_cli <verb> || fail`
#      A wrapper that runs the verb and continues is the vacuous pass already
#      shipped once (build-learn as a generator, .10).
#
# Not via run_selftest: same as selftest_install.sh (N.7). Registering a new
# suite grows cdcp_gate past gate_shrink and moves the 72-count. Plants still
# fire. Does not mutate the live tree.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_learner_verbs: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_learner_verbs: ok: $*"; }

INJ=0
SUITE_NAME=learner_verbs
inject_counted() { INJ=$((INJ + 1)); }

CDCP="${CDCP_BIN_DIR:-$ROOT/target/debug}/cdcp"
[ -x "$CDCP" ] || fail "cdcp binary missing at $CDCP — check.sh must build first"

CHECK="$ROOT/scripts/check.sh"
[ -f "$CHECK" ] || fail "missing $CHECK"

TMP=
cleanup() { [ -n "${TMP:-}" ] && rm -rf "$TMP"; return 0; }
trap cleanup EXIT INT TERM HUP
TMP=$(mktemp -d "${TMPDIR:-/tmp}/cdcp-selftest-learner-verbs.XXXXXX")

assert_red() {
  label=$1
  needle=$2
  shift 2
  rc=0
  out=$("$@" 2>&1) || rc=$?
  if [ "$rc" -eq 0 ]; then
    printf '%s\n' "$out" >&2
    fail "expected RED for $label but command exited 0"
  fi
  case "$out" in
    *"$needle"*) ;;
    *)
      printf '%s\n' "$out" >&2
      fail "expected RED for $label to name '$needle' (rc=$rc)"
      ;;
  esac
  inject_counted
  ok "$label trips RED (rc=$rc, saw: $needle)"
}

require_wired() {
  verb=$1
  pattern=$2
  # The live step must invoke the helper AND fail closed. A line that runs
  # the verb and ignores the status is the vacuous pass.
  if ! grep -F "$pattern" "$CHECK" >/dev/null 2>&1; then
    fail "check.sh is missing fail-closed wiring for $verb (expected: $pattern)"
  fi
  if grep -E "run_cdcp_cli[[:space:]]+$verb([^[:alnum:]_-].*)?[[:space:]]*\\|\\|[[:space:]]*true" "$CHECK" >/dev/null 2>&1; then
    fail "check.sh runs $verb and ignores its exit code (\`|| true\`) — vacuous pass"
  fi
  inject_counted
  ok "check.sh wires $verb fail-closed ($pattern)"
}

# ── 4) static anti-vacuity (must fire even if plants cannot spawn) ────────
require_wired "test" 'run_cdcp_cli test || fail "cdcp test"'
require_wired "demo" 'run_cdcp_cli demo --no-open || fail "cdcp demo"'
if ! grep -F 'fail "cdcp study"' "$CHECK" >/dev/null 2>&1; then
  fail "check.sh is missing fail-closed wiring for study (expected: fail \"cdcp study\")"
fi
if ! grep -E 'curl .*cdcp study|cdcp study.*curl|_study_code' "$CHECK" >/dev/null 2>&1; then
  fail "check.sh study step never curls the bound URL — printed-URL-only is vacuous"
fi
inject_counted
ok "check.sh study step curls the bound URL and fail-closes"

# ── plant trees ───────────────────────────────────────────────────────────
WEB_SRC="$ROOT/web"
[ -f "$WEB_SRC/index.html" ] || fail "missing $WEB_SRC/index.html"
[ -f "$WEB_SRC/data/mock40_seed42.json" ] || fail "missing seed-42 pack"
[ -f "$WEB_SRC/assets/wasm/cdcp_wasm.wasm" ] || fail "missing shipped wasm"

copy_seed42() {
  dest=$1
  mkdir -p "$dest/web/data" "$dest/web/assets/wasm"
  cp "$WEB_SRC/index.html" "$dest/web/index.html"
  cp "$WEB_SRC/data/mock40_seed42.json" "$dest/web/data/mock40_seed42.json"
  cp "$WEB_SRC/data/bank_items_seed42.json" "$dest/web/data/bank_items_seed42.json"
  cp "$WEB_SRC/data/keys_seed42.json" "$dest/web/data/keys_seed42.json"
}

# 1) test: seed-42 assets present, wasm is not a wasm → RED naming test/wasm
TEST_PLANT="$TMP/test-nowasm"
copy_seed42 "$TEST_PLANT"
printf 'notw' >"$TEST_PLANT/web/assets/wasm/cdcp_wasm.wasm"
assert_red "test missing-wasm" "test" \
  env -u CDCP_HOME -u CDCP_REPO_ROOT \
  "$CDCP" test --root "$TEST_PLANT"

# Prove ignore-exit would GREEN this plant (the .10 defect). Then require
# the real wrapper is not that pattern — already asserted above — and that
# the plant itself is actually RED (already asserted). This block exists so
# a future editor who deletes assert_red and keeps only the grep still has
# a live RED against the verb.
vac_rc=0
"$CDCP" test --root "$TEST_PLANT" >/dev/null 2>&1 || vac_rc=$?
if [ "$vac_rc" -eq 0 ]; then
  fail "test plant is GREEN — the known-bad no longer breaks the verb"
fi
# Vacuous wrapper: run + ignore. This MUST exit 0. If it does not, the
# demonstration that ignore-exit hides the plant is itself broken.
if ! ( "$CDCP" test --root "$TEST_PLANT" >/dev/null 2>&1 || true ); then
  fail "vacuous \`|| true\` wrapper did not stay GREEN — cannot certify anti-vacuity"
fi
inject_counted
ok "ignore-exit would GREEN the test plant (|| true stays 0); real wiring does not"

# 2) demo: no web/index.html → RED naming bundle / demo
DEMO_PLANT="$TMP/demo-empty"
mkdir -p "$DEMO_PLANT/empty"
assert_red "demo missing-bundle" "bundle not found" \
  env -u CDCP_HOME -u CDCP_REPO_ROOT \
  "$CDCP" demo --no-open --root "$DEMO_PLANT/empty"

# 3) study: same missing bundle → exit 4, does not block
STUDY_PLANT="$TMP/study-empty"
mkdir -p "$STUDY_PLANT/empty"
study_rc=0
study_out=$(
  env -u CDCP_HOME -u CDCP_REPO_ROOT \
    "$CDCP" study --no-open --bind 127.0.0.1:0 --root "$STUDY_PLANT/empty" 2>&1
) || study_rc=$?
if [ "$study_rc" -eq 0 ]; then
  printf '%s\n' "$study_out" >&2
  fail "study missing-bundle exited 0 (must be 4)"
fi
if [ "$study_rc" -ne 4 ]; then
  printf '%s\n' "$study_out" >&2
  fail "study missing-bundle rc=$study_rc (must be 4)"
fi
case "$study_out" in
  *"bundle not found"*) ;;
  *)
    printf '%s\n' "$study_out" >&2
    fail "study missing-bundle did not name 'bundle not found'"
    ;;
esac
inject_counted
ok "study missing-bundle exits 4 naming bundle (did not block)"

if [ "$INJ" -lt 7 ]; then
  fail "suite asserted $INJ RED(s); floor is 7 (3 static wirings + study-curl + 3 plants + ignore-exit)"
fi

echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_learner_verbs: PASSED ($INJ injections)"
