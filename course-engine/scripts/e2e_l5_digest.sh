#!/usr/bin/env sh
# e2e_l5_digest.sh — L5 UI e2e digest match harness (headless WASM vs goldens)
#
# Grades frozen seed42 all-correct / all-wrong attempts via grade_bridge +
# cdcp_wasm and compares digests to goldens pins. S8 will wire this into
# check.sh; this script is the callable stage (exit 0 clean, non-zero mismatch).
#
# Usage (from course-engine/ or any cwd):
#   ./scripts/e2e_l5_digest.sh
#   ./scripts/e2e_l5_digest.sh --golden-dir /tmp/alt_goldens
#
# Env:
#   CDCP_WASM_PATH   — override wasm artifact
#   CDCP_GOLDEN_DIR  — override golden pin directory (same as --golden-dir)
#   CDCP_BANK_JSON / CDCP_KEYS_JSON — override fixtures
#
# Anti-vacuous:
#   Missing wasm, missing bank/keys, missing golden pins, empty keys → ERROR
#   (never exit 0 with zero fixtures).
#
# Prefer TEMP golden dir + --golden-dir for known-bad selftests; do not mutate
# committed goldens/.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "e2e_l5_digest: FAIL: $*" >&2; exit 2; }
ok() { echo "e2e_l5_digest: ok: $*"; }

GOLDEN_DIR=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --golden-dir)
      shift
      [ "$#" -gt 0 ] || fail "--golden-dir requires a path"
      GOLDEN_DIR="$1"
      shift
      ;;
    --golden-dir=*)
      GOLDEN_DIR="${1#--golden-dir=}"
      shift
      ;;
    -h|--help)
      sed -n '2,28p' "$0"
      exit 0
      ;;
    *)
      fail "unknown arg: $1"
      ;;
  esac
done

if [ -n "$GOLDEN_DIR" ]; then
  export CDCP_GOLDEN_DIR="$GOLDEN_DIR"
fi
# Default when neither flag nor env set
: "${CDCP_GOLDEN_DIR:=$ROOT/goldens}"
export CDCP_GOLDEN_DIR

echo "==> e2e_l5_digest (WASM grade vs golden pins)"
echo "e2e_l5_digest: golden_dir=$CDCP_GOLDEN_DIR"

command -v node >/dev/null 2>&1 || fail "node required (Node 18+ for ESM + WebAssembly)"

# Required product surfaces (UI path that S5 gates)
[ -d web ] || fail "missing web/"
[ -f web/assets/js/grade_bridge.js ] || fail "missing web/assets/js/grade_bridge.js"
[ -f web/assets/js/results.js ] || fail "missing web/assets/js/results.js"
[ -f scripts/smoke_results_wasm.mjs ] || fail "missing scripts/smoke_results_wasm.mjs"

# Fixtures — zero of these is ERROR (no vacuous green)
BANK_JSON="${CDCP_BANK_JSON:-$ROOT/web/data/bank_items_seed42.json}"
KEYS_JSON="${CDCP_KEYS_JSON:-$ROOT/web/data/keys_seed42.json}"
G_CORRECT="$CDCP_GOLDEN_DIR/mock40_seed42_all_correct.sha256"
G_WRONG="$CDCP_GOLDEN_DIR/mock40_seed42_all_wrong.sha256"

missing=0
for label_path in \
  "bank_json:$BANK_JSON" \
  "keys_json:$KEYS_JSON" \
  "golden_all_correct:$G_CORRECT" \
  "golden_all_wrong:$G_WRONG"
do
  label="${label_path%%:*}"
  path="${label_path#*:}"
  if [ ! -f "$path" ]; then
    echo "e2e_l5_digest: missing fixture $label: $path" >&2
    missing=$((missing + 1))
  fi
done
if [ "$missing" -gt 0 ]; then
  fail "zero/missing fixtures ($missing absent) — refusing vacuous green"
fi
ok "fixtures present (bank · keys · 2 golden pins)"

# WASM artifact must exist (resolve same candidates as smoke_results_wasm.mjs)
wasm_found=0
if [ -n "${CDCP_WASM_PATH:-}" ] && [ -f "$CDCP_WASM_PATH" ]; then
  wasm_found=1
  ok "wasm via CDCP_WASM_PATH=$CDCP_WASM_PATH"
else
  for c in \
    "$ROOT/web/assets/wasm/cdcp_wasm.wasm" \
    "$ROOT/target/wasm32-unknown-unknown/release/cdcp_wasm.wasm" \
    "$ROOT/target/wasm32-unknown-unknown/debug/cdcp_wasm.wasm"
  do
    if [ -f "$c" ]; then
      wasm_found=1
      ok "wasm found: $c"
      break
    fi
  done
fi
[ "$wasm_found" -eq 1 ] || fail "no cdcp_wasm.wasm — run ./scripts/build_web_wasm.sh (missing wasm = ERROR)"

# Headless grade + digest compare
rc=0
out="$(node scripts/smoke_results_wasm.mjs --golden-dir "$CDCP_GOLDEN_DIR" 2>&1)" || rc=$?
printf '%s\n' "$out"

if [ "$rc" -ne 0 ]; then
  # Preserve smoke's exit code semantics: 1 = GOLDEN MISMATCH, 2 = setup/error
  if printf '%s\n' "$out" | grep -q 'GOLDEN MISMATCH'; then
    echo "e2e_l5_digest: GOLDEN MISMATCH (rc=$rc)" >&2
    exit 1
  fi
  fail "smoke_results_wasm.mjs exited $rc"
fi

# Require explicit match markers (anti-success-theater if smoke goes silent)
printf '%s\n' "$out" | grep -q 'ok: all-correct matches golden' \
  || fail "missing all-correct match confirmation"
printf '%s\n' "$out" | grep -q 'ok: all-wrong matches golden' \
  || fail "missing all-wrong match confirmation"
printf '%s\n' "$out" | grep -q 'matched digests: all-correct=' \
  || fail "missing matched digests summary"

ok "all-correct + all-wrong digests match goldens"
echo "e2e_l5_digest: PASS"
exit 0
