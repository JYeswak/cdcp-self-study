#!/usr/bin/env sh
# Convenience: build Learn surface + smoke.
set -eu
ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
python3 scripts/build_learn.py
python3 scripts/smoke_learn.py
python3 scripts/smoke_feedback_links.py
