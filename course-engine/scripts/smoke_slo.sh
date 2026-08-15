#!/usr/bin/env sh
# smoke_slo.sh — L7-S8 light SLO-as-code (bd-3pz)
#
# Times three product-path walls against slo.toml budgets:
#   1) grade all-correct (fixture mock40_seed42)
#   2) export-web --seed 42
#   3) cdcp_gate verify-bank (Rust bank pool floors; not the python oracle)
#
# Exit 0 if all under budget; non-zero if any over.
# Skip-honest (local thermal only — never the default CI story):
#   CDCP_SKIP_SLO=1 ./scripts/smoke_slo.sh
#
# Optional known-bad: CDCP_SLO_SELFTEST_TINY=1 forces 1ms budgets → must RED.
#
# EXTRACT-THEN-DELETE (bd-extract-smoke-slo-python-l5ke): wall budgets and
# the epoch-ms clock come from `cdcp slo budgets` / `cdcp slo now-ms`.
# A missing $ROOT/target/debug/cdcp after this script's cargo build is RED
# (no compile-and-run fallback).
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "smoke_slo: FAIL: $*" >&2; exit 2; }
ok() { echo "smoke_slo: ok: $*"; }
warn() { echo "smoke_slo: WARN: $*" >&2; }

if [ "${CDCP_SKIP_SLO:-}" = "1" ]; then
  warn "CDCP_SKIP_SLO=1 — skip-honest receipt (not a green CI claim)"
  echo "smoke_slo: SKIPPED (CDCP_SKIP_SLO=1)"
  exit 0
fi

[ -f slo.toml ] || fail "missing slo.toml"
[ -f goldens/fixtures/mock40_seed42.json ] || fail "missing goldens/fixtures/mock40_seed42.json"
[ -f crates/cdcp_gate/src/gates/verify_bank.rs ] \
  || fail "missing crates/cdcp_gate/src/gates/verify_bank.rs (bank verifier required)"
command -v cargo >/dev/null 2>&1 || fail "cargo required"

# Prebuild so grade/export/verify walls exclude cold compile (charter cares about path, not rustc).
echo "smoke_slo: prebuild cdcp_cli + cdcp_gate"
cargo build -q -p cdcp_cli -p cdcp_gate --locked || fail "cargo build -p cdcp_cli -p cdcp_gate"

CDCP_BIN="$ROOT/target/debug/cdcp"
GATE_BIN="$ROOT/target/debug/cdcp_gate"
# Missing binary after our own cargo build is RED. There is no fallback.
[ -x "$CDCP_BIN" ] || fail "missing $CDCP_BIN after cargo build -p cdcp_cli (no fallback)"
[ -x "$GATE_BIN" ] || fail "missing $GATE_BIN after cargo build -p cdcp_gate (bank verifier required)"
"$GATE_BIN" list | grep -q '^verify-bank' \
  || fail "cdcp_gate binary has no verify-bank subcommand"

# Read budgets from slo.toml via the product binary (typed [budgets] table).
read_budgets() {
  "$CDCP_BIN" slo budgets --file slo.toml
}

BUDGETS="$(read_budgets)" || fail "could not parse slo.toml budgets"
GRADE_MS="$(printf '%s\n' "$BUDGETS" | sed -n '1p')"
EXPORT_MS="$(printf '%s\n' "$BUDGETS" | sed -n '2p')"
VERIFY_MS="$(printf '%s\n' "$BUDGETS" | sed -n '3p')"

# Three integers, or the helper printed noise / dropped a wall.
case "$GRADE_MS" in *[!0-9]*|"") fail "slo budgets line 1 is not an integer: $GRADE_MS" ;; esac
case "$EXPORT_MS" in *[!0-9]*|"") fail "slo budgets line 2 is not an integer: $EXPORT_MS" ;; esac
case "$VERIFY_MS" in *[!0-9]*|"") fail "slo budgets line 3 is not an integer: $VERIFY_MS" ;; esac

if [ "${CDCP_SLO_SELFTEST_TINY:-}" = "1" ]; then
  warn "CDCP_SLO_SELFTEST_TINY=1 — forcing 1ms budgets (expect RED)"
  GRADE_MS=1
  EXPORT_MS=1
  VERIFY_MS=1
fi

echo "smoke_slo: budgets grade_ms=$GRADE_MS export_ms=$EXPORT_MS bank_verify_ms=$VERIFY_MS"

# Portable wall-ms: start / elapsed via the product binary.
now_ms() {
  "$CDCP_BIN" slo now-ms
}

run_timed() {
  # run_timed LABEL BUDGET_MS cmd...
  label="$1"
  budget="$2"
  shift 2
  start="$(now_ms)"
  rc=0
  out="$("$@" 2>&1)" || rc=$?
  end="$(now_ms)"
  elapsed=$((end - start))
  if [ "$rc" -ne 0 ]; then
    printf '%s\n' "$out" >&2
    fail "$label command failed (rc=$rc) after ${elapsed}ms"
  fi
  if [ "$elapsed" -gt "$budget" ]; then
    printf '%s\n' "$out" >&2
    fail "$label over budget: ${elapsed}ms > ${budget}ms"
  fi
  ok "$label ${elapsed}ms ≤ ${budget}ms"
  # Keep last line of useful stdout for humans
  printf '%s\n' "$out" | tail -n 3 | sed 's/^/  | /'
}

TMP_EXPORT=""
cleanup() {
  if [ -n "${TMP_EXPORT:-}" ] && [ -d "${TMP_EXPORT}" ]; then
    rm -rf "${TMP_EXPORT}" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM HUP

echo "==> (1) grade all-correct"
run_timed "grade" "$GRADE_MS" \
  "$CDCP_BIN" grade \
    --bank bank/items \
    --fixture goldens/fixtures/mock40_seed42.json \
    --mode all-correct

echo "==> (2) export-web --seed 42"
TMP_EXPORT="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_slo_export.XXXXXX")"
run_timed "export" "$EXPORT_MS" \
  "$CDCP_BIN" export-web \
    --bank bank/items \
    --seed 42 \
    --out "$TMP_EXPORT"

echo "==> (3) cdcp_gate verify-bank"
# scripts/verify_bank.py is the differential oracle only — not timed here.
run_timed "bank_verify" "$VERIFY_MS" \
  "$GATE_BIN" verify-bank

echo "smoke_slo: PASSED (all walls under budget)"
exit 0
