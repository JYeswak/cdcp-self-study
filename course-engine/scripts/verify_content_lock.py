#!/usr/bin/env python3
"""verify_content_lock.py — compare content.lock to the live tree (L7-S9, bd-llj).

Exit 0 if bank_hash + every pinned knowledge/module path matches.
Exit non-zero on missing lock, hash drift, or missing pinned file.

Regenerate (human only):
  UPDATE_CONTENT_LOCK=1 python3 scripts/gen_content_lock.py

Optional selftest (does not mutate committed tree):
  CDCP_CONTENT_LOCK_SELFTEST=1 python3 scripts/verify_content_lock.py
  → plants a temp mismatch expectation and asserts RED path is reachable.
"""
from __future__ import annotations

import hashlib
import os
import subprocess
import sys
import tempfile
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
LOCK_PATH = ROOT / "content.lock"
GOLDEN_BANK_HASH = ROOT / "goldens" / "bank_hash.txt"


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def live_bank_hash() -> str:
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
        except (subprocess.CalledProcessError, FileNotFoundError, OSError):
            continue
    if GOLDEN_BANK_HASH.is_file():
        return GOLDEN_BANK_HASH.read_text(encoding="utf-8").strip()
    raise RuntimeError("cannot obtain live bank_hash")


def resolve_pinned(rel: str) -> Path:
    """Resolve a lock path relative to course-engine ROOT, then parent corpus."""
    p = Path(rel)
    if p.is_absolute():
        return p
    cand = (ROOT / p).resolve()
    if cand.exists():
        return cand
    # Parent-relative paths like modules/01-mission-critical.md
    cand2 = (ROOT.parent / p).resolve()
    return cand2


def verify(lock_path: Path = LOCK_PATH) -> list[str]:
    errors: list[str] = []
    if not lock_path.is_file():
        return [f"missing content.lock at {lock_path}"]

    data = tomllib.loads(lock_path.read_text(encoding="utf-8"))
    schema = data.get("schema_version")
    if schema != 1:
        errors.append(f"unsupported schema_version={schema!r} (want 1)")

    pinned_bank = data.get("bank_hash")
    if not pinned_bank or not isinstance(pinned_bank, str):
        errors.append("content.lock missing bank_hash")
    else:
        try:
            live = live_bank_hash()
        except RuntimeError as e:
            errors.append(str(e))
            live = None
        if live is not None and live != pinned_bank:
            errors.append(
                f"bank_hash drift: lock={pinned_bank[:16]}… live={live[:16]}…"
            )

    knowledge = data.get("knowledge") or {}
    modules = data.get("modules") or {}
    if not knowledge:
        errors.append("content.lock [knowledge] empty (vacuous ERROR)")
    if not modules:
        errors.append("content.lock [modules] empty (vacuous ERROR)")

    for section, mapping in (("knowledge", knowledge), ("modules", modules)):
        if not isinstance(mapping, dict):
            errors.append(f"[{section}] must be a table of path = hash")
            continue
        for rel, expected in sorted(mapping.items()):
            path = resolve_pinned(str(rel))
            if not path.is_file():
                errors.append(f"[{section}] missing file: {rel}")
                continue
            actual = sha256_file(path)
            if actual != expected:
                errors.append(
                    f"[{section}] hash mismatch: {rel} "
                    f"lock={str(expected)[:12]}… live={actual[:12]}…"
                )

    return errors


def selftest_mutate() -> int:
    """Optional: prove a mutated lock trips RED without dirtying the tree."""
    if not LOCK_PATH.is_file():
        print("FAIL: content.lock missing; cannot selftest", file=sys.stderr)
        return 1
    text = LOCK_PATH.read_text(encoding="utf-8")
    # Flip last hex nibble of bank_hash line if present.
    lines = text.splitlines()
    mutated = False
    new_lines: list[str] = []
    for line in lines:
        if line.startswith("bank_hash = ") and not mutated:
            # bank_hash = "abc...f" → flip final hex digit
            if line.endswith('"'):
                body = line[:-1]
                last = body[-1]
                flip = "0" if last != "0" else "1"
                line = body[:-1] + flip + '"'
                mutated = True
        new_lines.append(line)
    if not mutated:
        print("FAIL: selftest could not locate bank_hash line", file=sys.stderr)
        return 1

    with tempfile.NamedTemporaryFile(
        mode="w",
        suffix=".lock",
        delete=False,
        encoding="utf-8",
    ) as tmp:
        tmp.write("\n".join(new_lines) + "\n")
        tmp_path = Path(tmp.name)

    try:
        errs = verify(tmp_path)
        if not errs:
            print(
                "FAIL: expected RED on mutated bank_hash but verify was green",
                file=sys.stderr,
            )
            return 1
        if not any("bank_hash drift" in e for e in errs):
            print("FAIL: expected bank_hash drift signal; got:", file=sys.stderr)
            for e in errs:
                print(f"  - {e}", file=sys.stderr)
            return 1
        print("verify_content_lock: ok: mutate-selftest trips RED (bank_hash drift)")
        return 0
    finally:
        tmp_path.unlink(missing_ok=True)


def main() -> int:
    if os.environ.get("CDCP_CONTENT_LOCK_SELFTEST") == "1":
        return selftest_mutate()

    errors = verify()
    if errors:
        print("verify_content_lock: FAIL", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        print(
            "Regenerate (human review): "
            "UPDATE_CONTENT_LOCK=1 python3 scripts/gen_content_lock.py",
            file=sys.stderr,
        )
        return 1

    # Count pins for receipt
    data = tomllib.loads(LOCK_PATH.read_text(encoding="utf-8"))
    nk = len(data.get("knowledge") or {})
    nm = len(data.get("modules") or {})
    bh = str(data.get("bank_hash") or "")[:12]
    print(
        f"verify_content_lock: PASS bank_hash={bh}… "
        f"knowledge={nk} modules={nm}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
