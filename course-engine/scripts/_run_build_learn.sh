#!/usr/bin/env sh
# Convenience: build Learn surface + smoke.
# EXTRACT-THEN-DELETE (bd-substrate-rust-migration-jhd.28): python3 scripts/build_learn.py
# and `cargo run` are gone. This wrapper invokes the prebuilt cdcp binary.
set -eu
ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if [ -z "${CDCP_BIN_DIR:-}" ]; then
  if [ -n "${CARGO_TARGET_DIR:-}" ]; then
    CDCP_BIN_DIR="${CARGO_TARGET_DIR%/}/debug"
  else
    CDCP_BIN_DIR="$ROOT/target/debug"
  fi
fi
[ -x "$CDCP_BIN_DIR/cdcp" ] || {
  echo "FAIL: cdcp binary absent at $CDCP_BIN_DIR/cdcp — cargo build -p cdcp_cli --locked first (no cargo run, no python3)" >&2
  exit 1
}
"$CDCP_BIN_DIR/cdcp" build-learn
"$CDCP_BIN_DIR/cdcp" smoke-learn
"$CDCP_BIN_DIR/cdcp" smoke-feedback-links
