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
command -v python3 >/dev/null 2>&1 || fail "python3 required"
command -v cargo >/dev/null 2>&1 || fail "cargo required"

# Read budgets from slo.toml (python for reliable TOML; no external deps).
read_budgets() {
  python3 - <<'PY'
from pathlib import Path
try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore

data = tomllib.loads(Path("slo.toml").read_text(encoding="utf-8"))
b = data.get("budgets") or data
keys = ("grade_ms", "export_ms", "bank_verify_ms")
missing = [k for k in keys if k not in b]
if missing:
    raise SystemExit(f"slo.toml missing budgets: {missing}")
print(int(b["grade_ms"]))
print(int(b["export_ms"]))
print(int(b["bank_verify_ms"]))
PY
}

BUDGETS="$(read_budgets)" || fail "could not parse slo.toml budgets"
GRADE_MS="$(printf '%s\n' "$BUDGETS" | sed -n '1p')"
EXPORT_MS="$(printf '%s\n' "$BUDGETS" | sed -n '2p')"
VERIFY_MS="$(printf '%s\n' "$BUDGETS" | sed -n '3p')"

if [ "${CDCP_SLO_SELFTEST_TINY:-}" = "1" ]; then
  warn "CDCP_SLO_SELFTEST_TINY=1 — forcing 1ms budgets (expect RED)"
  GRADE_MS=1
  EXPORT_MS=1
  VERIFY_MS=1
fi

echo "smoke_slo: budgets grade_ms=$GRADE_MS export_ms=$EXPORT_MS bank_verify_ms=$VERIFY_MS"

# Prebuild so grade/export/verify walls exclude cold compile (charter cares about path, not rustc).
echo "smoke_slo: prebuild cdcp_cli + cdcp_gate"
cargo build -q -p cdcp_cli -p cdcp_gate --locked || fail "cargo build -p cdcp_cli -p cdcp_gate"

# Prefer built binary for timing (spawn overhead only; not rustc).
CDCP_BIN="$ROOT/target/debug/cdcp"
GATE_BIN="$ROOT/target/debug/cdcp_gate"
if [ ! -x "$CDCP_BIN" ]; then
  # cargo build places it here; fall back to cargo run if missing
  CDCP_BIN=""
fi
# Anti-vacuous: a missing bank verifier is RED, not a skipped wall.
# scripts/verify_bank.py is the differential oracle only — not timed here.
[ -x "$GATE_BIN" ] || fail "missing $GATE_BIN after cargo build -p cdcp_gate (bank verifier required)"
"$GATE_BIN" list | grep -q '^verify-bank' \
  || fail "cdcp_gate binary has no verify-bank subcommand"

# Portable wall-ms: start_ms / elapsed_ms via python.
now_ms() {
  python3 -c 'import time; print(int(time.time() * 1000))'
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
if [ -n "$CDCP_BIN" ]; then
  run_timed "grade" "$GRADE_MS" \
    "$CDCP_BIN" grade \
      --bank bank/items \
      --fixture goldens/fixtures/mock40_seed42.json \
      --mode all-correct
else
  run_timed "grade" "$GRADE_MS" \
    cargo run -q -p cdcp_cli --locked -- grade \
      --bank bank/items \
      --fixture goldens/fixtures/mock40_seed42.json \
      --mode all-correct
fi

echo "==> (2) export-web --seed 42"
TMP_EXPORT="$(mktemp -d "${TMPDIR:-/tmp}/cdcp_slo_export.XXXXXX")"
if [ -n "$CDCP_BIN" ]; then
  run_timed "export" "$EXPORT_MS" \
    "$CDCP_BIN" export-web \
      --bank bank/items \
      --seed 42 \
      --out "$TMP_EXPORT"
else
  run_timed "export" "$EXPORT_MS" \
    cargo run -q -p cdcp_cli --locked -- export-web \
      --bank bank/items \
      --seed 42 \
      --out "$TMP_EXPORT"
fi

echo "==> (3) cdcp_gate verify-bank"
run_timed "bank_verify" "$VERIFY_MS" \
  "$GATE_BIN" verify-bank

echo "smoke_slo: PASSED (all walls under budget)"
exit 0
