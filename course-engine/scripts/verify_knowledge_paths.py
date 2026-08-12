#!/usr/bin/env python3
"""verify_knowledge_paths.py — every non-empty primary_notes path must resolve.

Paths in knowledge/*.toml are relative to the course-engine ROOT (parent of
knowledge/), pointing at the parent corpus ../modules/*.md — never at a
missing course-engine/modules/ tree.

Empty primary_notes is allowed only when exam_weight_unknown is true
(ops-adjacent expansions without study notes).
"""
from __future__ import annotations

import sys
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
DOMAINS = ROOT / "knowledge" / "domains.toml"
KNOWLEDGE_DIR = ROOT / "knowledge"


def main() -> int:
    errors: list[str] = []
    if not DOMAINS.is_file():
        print("FAIL: knowledge/domains.toml missing")
        return 1

    data = tomllib.loads(DOMAINS.read_text(encoding="utf-8"))
    domains = data.get("domain") or []
    if not domains:
        print("FAIL: domains.toml has zero [[domain]] rows")
        return 1

    checked = 0
    empty_ok = 0
    for dom in domains:
        did = dom.get("id") or "<missing-id>"
        pn = dom.get("primary_notes")
        if pn is None:
            errors.append(f"{did}: primary_notes field missing")
            continue
        pn_s = str(pn).strip()
        if not pn_s:
            if dom.get("exam_weight_unknown") is True:
                empty_ok += 1
                continue
            errors.append(
                f"{did}: empty primary_notes without exam_weight_unknown=true"
            )
            continue

        # Resolve relative to course-engine ROOT (not knowledge/).
        candidate = Path(pn_s)
        if not candidate.is_absolute():
            candidate = (ROOT / pn_s).resolve()
        else:
            candidate = candidate.resolve()

        checked += 1
        if not candidate.is_file():
            errors.append(
                f"{did}: primary_notes does not resolve to a file: {pn_s!r} "
                f"(resolved {candidate})"
            )
            continue

        # Guard: must not live under a missing/wrong course-engine/modules/
        # when parent corpus is the intended target.
        try:
            candidate.relative_to(ROOT / "modules")
            errors.append(
                f"{did}: primary_notes resolves under course-engine/modules/ "
                f"({candidate}); parent corpus is ../modules/ relative to ROOT"
            )
        except ValueError:
            pass

        # Soft sanity: prefer real files under parent modules/
        parent_modules = (ROOT.parent / "modules").resolve()
        try:
            candidate.relative_to(parent_modules)
        except ValueError:
            # Allow other absolute/shared paths if they exist; warn via error
            # only if they also fail the is_file check (already handled).
            pass

    # Also scan other knowledge/*.toml for primary_notes / module path keys
    for path in sorted(KNOWLEDGE_DIR.glob("*.toml")):
        if path.name == "domains.toml":
            continue
        text = path.read_text(encoding="utf-8")
        for i, line in enumerate(text.splitlines(), 1):
            stripped = line.strip()
            if not stripped.startswith("primary_notes"):
                continue
            # unexpected elsewhere — still validate if present
            if "=" not in stripped:
                continue
            raw = stripped.split("=", 1)[1].strip().strip('"').strip("'")
            if not raw:
                continue
            cand = (ROOT / raw).resolve() if not Path(raw).is_absolute() else Path(raw)
            if not cand.is_file():
                errors.append(
                    f"{path.name}:{i}: primary_notes {raw!r} does not resolve"
                )

    if errors:
        print("FAIL")
        for e in errors:
            print(f"  - {e}")
        return 1

    print("PASS")
    print(f"  primary_notes_checked={checked}")
    print(f"  empty_allowed={empty_ok}")
    print(f"  root={ROOT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
