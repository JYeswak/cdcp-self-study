#!/usr/bin/env python3
"""Regenerate bank/MANIFEST.toml from bank/items/*.toml."""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ITEMS = ROOT / "bank" / "items"
OUT = ROOT / "bank" / "MANIFEST.toml"

names = sorted(p.name for p in ITEMS.glob("*.toml"))
lines = [
    "# Auto-regenerated library manifest",
    "schema_version = 1",
    f"item_count = {len(names)}",
    'bank_id = "cdcp-library"',
    'source_class = "original"',
    'description = "Original CDCP study bank (~20x exam); mocks sample 40 without replacement"',
    "pool_min_items = 400",
    "pool_target_items = 800",
    'generated_from = ["../practice/PRACTICE-EXAM.md", "generated-expansions"]',
    "",
    "items = [",
]
for n in names:
    lines.append(f'  "{n}",')
lines.append("]")
lines.append("")
OUT.write_text("\n".join(lines), encoding="utf-8")
print(f"MANIFEST item_count={len(names)} → {OUT}")
