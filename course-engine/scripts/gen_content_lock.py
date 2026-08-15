#!/usr/bin/env python3
"""gen_content_lock.py — write content.lock (L7-S9 ecosystem pin, bd-llj).

Pins:
  - bank_hash (via `cdcp bank-hash`, fallback to goldens/bank_hash.txt)
  - knowledge pack top-level *.toml file hashes
  - module markdown hashes under web/content/modules/ (product surface)
    and parent ../modules/*.md (source corpus) when present

Usage (from course-engine/):
  python3 scripts/gen_content_lock.py
  UPDATE_CONTENT_LOCK=1 python3 scripts/gen_content_lock.py   # same; always writes

Regenerate only with human review of `git diff content.lock` before commit.
"""
from __future__ import annotations

import hashlib
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LOCK_PATH = ROOT / "content.lock"
KNOWLEDGE_DIR = ROOT / "knowledge"
WEB_MODULES = ROOT / "web" / "content" / "modules"
PARENT_MODULES = ROOT.parent / "modules"
GOLDEN_BANK_HASH = ROOT / "goldens" / "bank_hash.txt"

SCHEMA_VERSION = 1
# Domain tag for the bank hash. AUTHORITATIVE COPY IS cdcp_core::BANK_HASH_DOMAIN;
# this is a label that must name it, and content.lock `canonical` is written from
# here. All three move in ONE commit — a partial bump creates a third state.
# crates/cdcp_core/tests/bank_hash_domain_agreement.rs keys on the constant and
# goes RED naming both sides. v1 excluded status/objective_ids/citation_ids/tags;
# v2 covers them (see the table on BANK_HASH_DOMAIN and goldens/PROVENANCE.md).
CANONICAL = "cdcp-bank-v2"
HASH_ALG = "sha256"


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def rel_posix(path: Path) -> str:
    """Path relative to course-engine ROOT when under ROOT; else parent-relative."""
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        try:
            return path.resolve().relative_to(ROOT.parent.resolve()).as_posix()
        except ValueError:
            return path.resolve().as_posix()


def bank_hash() -> str:
    """Live CLI bank-hash; fall back to goldens/bank_hash.txt if cargo fails.

    Prefer the built binary when present (faster, same answer as cargo run).
    """
    candidates: list[list[str]] = []
    bin_path = ROOT / "target" / "debug" / "cdcp"
    if bin_path.is_file():
        candidates.append([str(bin_path), "bank-hash", "--bank", "bank/items"])
    candidates.append(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "cdcp_cli",
            "--locked",
            "--",
            "bank-hash",
            "--bank",
            "bank/items",
        ]
    )

    last_err: Exception | None = None
    for cmd in candidates:
        try:
            out = subprocess.check_output(
                cmd,
                cwd=ROOT,
                stderr=subprocess.STDOUT,
                text=True,
                timeout=300,
            )
            hx = out.strip().splitlines()[-1].strip()
            if len(hx) == 64 and all(c in "0123456789abcdef" for c in hx):
                return hx
            last_err = RuntimeError(f"unexpected bank-hash output: {out!r}")
        except (subprocess.CalledProcessError, FileNotFoundError, OSError) as e:
            last_err = e
            continue

    if GOLDEN_BANK_HASH.is_file():
        hx = GOLDEN_BANK_HASH.read_text(encoding="utf-8").strip()
        if len(hx) == 64:
            print(
                f"gen_content_lock: WARN: live bank-hash failed ({last_err}); "
                f"using goldens/bank_hash.txt",
                file=sys.stderr,
            )
            return hx
    raise SystemExit(f"FAIL: cannot obtain bank_hash: {last_err}")


def collect_hashes(paths: list[Path]) -> dict[str, str]:
    out: dict[str, str] = {}
    for p in sorted(paths, key=lambda x: rel_posix(x)):
        if not p.is_file():
            continue
        out[rel_posix(p)] = sha256_file(p)
    return out


def knowledge_files() -> list[Path]:
    if not KNOWLEDGE_DIR.is_dir():
        return []
    # ONE LEVEL (bd-zhnd). Path.glob("*.toml") does not recurse.
    # knowledge/corpus/*.toml is deliberately unpinned (external blobs).
    # verify_content_lock::discover matches this depth. Deepening either
    # side without the other makes a regenerated lock un-green, or hides
    # a nested file. The depth pin is crates/cdcp_learn/tests/stated_limits.rs.
    return sorted(KNOWLEDGE_DIR.glob("*.toml"))


def module_files() -> list[Path]:
    found: list[Path] = []
    if WEB_MODULES.is_dir():
        # ONE LEVEL, same contract as knowledge_files (bd-zhnd).
        found.extend(sorted(WEB_MODULES.glob("*.md")))
    if PARENT_MODULES.is_dir():
        found.extend(sorted(PARENT_MODULES.glob("*.md")))
    return found


def render_lock(
    bank: str,
    knowledge: dict[str, str],
    modules: dict[str, str],
) -> str:
    lines: list[str] = [
        "# content.lock — ecosystem pin for bank + knowledge + modules (L7-S9)",
        "# Generated by scripts/gen_content_lock.py — do not hand-edit hashes.",
        "# Regenerate: UPDATE_CONTENT_LOCK=1 python3 scripts/gen_content_lock.py",
        "# Verify:     python3 scripts/verify_content_lock.py",
        "",
        f"schema_version = {SCHEMA_VERSION}",
        f'canonical = "{CANONICAL}"',
        f'hash_alg = "{HASH_ALG}"',
        f'bank_hash = "{bank}"',
        "",
        "[knowledge]",
    ]
    if not knowledge:
        lines.append("# (empty — no knowledge/*.toml found)")
    else:
        for path, hx in knowledge.items():
            lines.append(f'"{path}" = "{hx}"')
    lines.append("")
    lines.append("[modules]")
    if not modules:
        lines.append("# (empty — no module markdown found)")
    else:
        for path, hx in modules.items():
            lines.append(f'"{path}" = "{hx}"')
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    # Always write when invoked; UPDATE_CONTENT_LOCK=1 documents human regen path.
    _ = os.environ.get("UPDATE_CONTENT_LOCK")

    bank = bank_hash()
    knowledge = collect_hashes(knowledge_files())
    modules = collect_hashes(module_files())

    if not knowledge:
        print("FAIL: zero knowledge/*.toml files to pin", file=sys.stderr)
        return 1
    if not modules:
        print(
            "FAIL: zero module markdown files "
            f"(looked in {WEB_MODULES} and {PARENT_MODULES})",
            file=sys.stderr,
        )
        return 1

    text = render_lock(bank, knowledge, modules)
    LOCK_PATH.write_text(text, encoding="utf-8")
    print(
        f"gen_content_lock: wrote {LOCK_PATH.relative_to(ROOT)} "
        f"bank_hash={bank[:12]}… "
        f"knowledge={len(knowledge)} modules={len(modules)}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
