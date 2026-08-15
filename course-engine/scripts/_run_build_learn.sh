#!/usr/bin/env sh
# Convenience: build Learn surface + smoke.
set -eu
ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
python3 scripts/build_learn.py
cargo run -q -p cdcp_cli -- smoke-learn
cargo run -q -p cdcp_cli -- smoke-feedback-links
