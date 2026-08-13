#!/usr/bin/env sh
# selftest_l7_objectives.sh — L7-S7 objective coverage gate proven to trip
#
# TEMP-only known-bad (never mutates live registries/ or bank/items):
#   a) empty objectives registry → RED
#   b) planted missing claim_id ref → RED
#   c) empty bank → RED (vacuous domain coverage)
#   d) live tree still GREEN
#
# Trap cleans TEMP. Never leaves registries/bank dirty.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_l7_objectives: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_l7_objectives: ok: $*"; }

TMP_ROOT=""
restore_all() {
  if [ -n "${TMP_ROOT:-}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}" 2>/dev/null || true
  fi
}
trap restore_all EXIT INT TERM HUP

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
    *"$needle"*) ok "$label trips RED (rc=$rc, saw: $needle)" ;;
    *)
      printf '%s\n' "$out" >&2
      fail "$label exited $rc but missing expected signal '$needle'"
      ;;
  esac
}

echo "==> selftest_l7_objectives (L7-S7 objective coverage known-bad)"

[ -f scripts/verify_objectives.py ] || fail "missing scripts/verify_objectives.py"
[ -f registries/objectives.toml ] || fail "missing registries/objectives.toml"
[ -f registries/claims.toml ] || fail "missing registries/claims.toml"
[ -d bank/items ] || fail "missing bank/items"
command -v python3 >/dev/null 2>&1 || fail "python3 required"

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
  python3 scripts/verify_objectives.py \
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
  python3 scripts/verify_objectives.py \
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
  python3 scripts/verify_objectives.py \
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
  python3 scripts/verify_objectives.py \
    --objectives registries/objectives.toml \
    --claims registries/claims.toml \
    --bank "$empty_bank" \
    --skip-topic-coverage
ok "empty bank trips RED"

# --- (d) live tree GREEN ---
echo "==> (d) live tree objective coverage GREEN"
rc=0
live_out="$(python3 scripts/verify_objectives.py \
  --objectives registries/objectives.toml \
  --claims registries/claims.toml \
  --bank bank/items 2>&1)" || rc=$?
printf '%s\n' "$live_out"
[ "$rc" -eq 0 ] || fail "live verify_objectives exited $rc (selftest must not dirty tree)"
printf '%s\n' "$live_out" | grep -q 'objective coverage GREEN' \
  || fail "live output missing 'objective coverage GREEN'"
ok "live tree still GREEN"

# Confirm no planted files under live registries/bank
if [ -f registries/objectives_empty.toml ] || [ -f bank/items/planted-obj.toml ]; then
  fail "planted file leaked into live tree"
fi

echo "selftest_l7_objectives: PASSED (a empty RED · b missing-claim RED · b2 empty-claims RED · c empty-bank RED · d live GREEN)"
exit 0
