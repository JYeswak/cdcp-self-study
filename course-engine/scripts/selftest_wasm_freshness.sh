#!/usr/bin/env sh
# selftest_wasm_freshness.sh — L4 plants for the shipped-wasm pin
# [bd-installability-sm4g.4]
#
# Modes:
#   --assert-fresh --committed PATH --built PATH
#       Compare sha256 of the two files. Mismatch is RED and names the wasm
#       path, not a downstream grade digest. Used by check.sh after a
#       `cargo build -p cdcp_wasm --target wasm32-unknown-unknown --release --locked`.
#
#   (no args)
#       Two known-bads, then emit `INJECTIONS=2 SUITE=wasm-freshness`:
#         1) flip one byte of a copy of the committed blob → --assert-fresh RED
#            naming cdcp_wasm.wasm
#         2) rebuild native only with a grade-affecting constant changed
#            (`--cfg cdcp_plant_weak`) → dual-path RED (native != shipped wasm)
#
# Plant 2 does not mutate the live tree. Isolated CARGO_TARGET_DIR + RUSTFLAGS.
# If dual_path started rebuilding wasm from the same source, this plant would
# stay GREEN — that is the certify-the-certifier.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "selftest_wasm_freshness: FAIL: $*" >&2; exit 2; }
ok() { echo "selftest_wasm_freshness: ok: $*"; }

SHIPPED="web/assets/wasm/cdcp_wasm.wasm"

wasm_sha256() {
  [ -f "$1" ] || { echo "selftest_wasm_freshness: missing $1" >&2; return 2; }
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    fail "neither sha256sum nor shasum is available"
  fi
}

assert_fresh() {
  _committed="$1"
  _built="$2"
  if [ ! -f "$_committed" ]; then
    echo "selftest_wasm_freshness: FAIL: missing wasm $_committed" >&2
    return 2
  fi
  if [ ! -f "$_built" ]; then
    echo "selftest_wasm_freshness: FAIL: missing wasm $_built" >&2
    return 2
  fi
  _hc="$(wasm_sha256 "$_committed")" || return 2
  _hb="$(wasm_sha256 "$_built")" || return 2
  if [ "$_hc" != "$_hb" ]; then
    echo "selftest_wasm_freshness: FAIL: wasm artifact $_committed sha256=${_hc} != $_built sha256=${_hb}" >&2
    return 2
  fi
  echo "selftest_wasm_freshness: sha256 match $_committed == $_built ($_hc)"
  return 0
}

# ── --assert-fresh (check.sh live pin + plant 1) ──────────────────────────
if [ "${1:-}" = "--assert-fresh" ]; then
  shift
  _committed=""
  _built=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --committed)
        _committed="${2:-}"
        shift 2
        ;;
      --built)
        _built="${2:-}"
        shift 2
        ;;
      *)
        fail "unknown arg for --assert-fresh: $1"
        ;;
    esac
  done
  [ -n "$_committed" ] || fail "--assert-fresh requires --committed PATH"
  [ -n "$_built" ] || fail "--assert-fresh requires --built PATH"
  assert_fresh "$_committed" "$_built"
  exit $?
fi

# ── plants ────────────────────────────────────────────────────────────────
INJ=0
SUITE_NAME="wasm-freshness"
inject_counted() { INJ=$((INJ + 1)); }

TMP_ROOT=""
PLANT_TARGET=""
restore_all() {
  if [ -n "${TMP_ROOT:-}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}" 2>/dev/null || true
  fi
  if [ -n "${PLANT_TARGET:-}" ]; then
    case "$PLANT_TARGET" in
      "$ROOT/target/cdcp-scratch"/*) rm -rf "$PLANT_TARGET" 2>/dev/null || true ;;
    esac
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

echo "==> selftest_wasm_freshness (shipped-blob pin known-bad)"

[ -f "$SHIPPED" ] || fail "missing $SHIPPED"
[ -f scripts/selftest_wasm_freshness.sh ] || fail "missing self"

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/selftest_wasm_freshness.XXXXXX")"

# ── (1) flip one byte of a copy of the committed blob ─────────────────────
echo "==> (1) flip one byte of committed wasm → RED naming the wasm"
FLIP="$TMP_ROOT/cdcp_wasm.wasm"
cp "$SHIPPED" "$FLIP"
# Byte 8 is past \0asm + version; keep magic intact so this is a hash miss,
# not a "not a wasm module" miss.
_cur="$(dd if="$FLIP" bs=1 skip=8 count=1 2>/dev/null | od -An -tu1 | tr -d ' \n')"
if [ "$_cur" = "255" ]; then
  printf '\000'
else
  printf '\377'
fi | dd of="$FLIP" bs=1 seek=8 conv=notrunc 2>/dev/null
_flip_h="$(wasm_sha256 "$FLIP")"
_orig_h="$(wasm_sha256 "$SHIPPED")"
[ "$_flip_h" != "$_orig_h" ] || fail "plant 1 did not change sha256 (flip was a no-op)"
# Compare flipped copy against the real committed blob — names cdcp_wasm.wasm,
# does not mention a grade digest. Does not need a wasm32 rebuild.
assert_fails_with "flipped-committed-blob" "cdcp_wasm.wasm" \
  sh scripts/selftest_wasm_freshness.sh --assert-fresh \
  --committed "$FLIP" --built "$SHIPPED"
# Must name the wasm, not only a downstream digest file.
# (the needle above is the filename; refuse a report that forgot it)
ok "plant 1 named the wasm (not a golden/digest pin)"

# ── (2) grade-affecting constant, native rebuild only ─────────────────────
echo "==> (2) --cfg cdcp_plant_weak, native only → dual-path mismatch"
# Isolated target so the cfg cannot poison the workspace target/ used by
# concurrent agents. Live sources are not written.
PLANT_TARGET="$ROOT/target/cdcp-scratch/wasm-freshness-$$"
mkdir -p "$PLANT_TARGET"
_before="$(wasm_sha256 "$SHIPPED")"
# Append, do not clobber a caller RUSTFLAGS.
_plant_rustflags="${RUSTFLAGS:-} --cfg cdcp_plant_weak"
assert_fails_with "mutated-grade-native-only" "dual-path mismatch" \
  env CARGO_TERM_COLOR=never \
      CARGO_TARGET_DIR="$PLANT_TARGET" \
      RUSTFLAGS="$_plant_rustflags" \
      cargo test -p cdcp_wasm --test dual_path --locked --manifest-path "$ROOT/Cargo.toml" \
      -- --nocapture --include-ignored shipped_wasm_matches_native_grade
_after="$(wasm_sha256 "$SHIPPED")"
[ "$_before" = "$_after" ] || fail "plant 2 mutated the committed $SHIPPED blob"
ok "plant 2 left $SHIPPED byte-unchanged"

[ "$INJ" -eq 2 ] || fail "INJ=$INJ; expected 2"
echo "INJECTIONS=$INJ SUITE=$SUITE_NAME"
echo "selftest_wasm_freshness: PASSED (flipped blob RED naming wasm · native-only constant RED)"
exit 0
