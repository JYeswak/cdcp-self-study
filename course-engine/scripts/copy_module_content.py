#!/usr/bin/env python3
"""Deprecated thin wrapper — use build_learn.py (copies + hub + pages)."""
from __future__ import annotations

import runpy
import sys
from pathlib import Path

if __name__ == "__main__":
    target = Path(__file__).with_name("build_learn.py")
    sys.argv[0] = str(target)
    runpy.run_path(str(target), run_name="__main__")
