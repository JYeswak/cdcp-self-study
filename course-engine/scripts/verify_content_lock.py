#!/usr/bin/env python3
"""verify_content_lock.py — compare content.lock to the live tree (L7-S9, bd-llj).

CLAIM: FLOOR-RAISE (bd-z3v, bd-hw3).

This gate raises one floor: **the content under the locked roots is exactly the
content content.lock pins, in both directions.** It goes RED when any of these
is true —

  1. content.lock is absent, unparseable, or schema_version is not 1;
  2. bank_hash is absent/empty, or the live bank digest differs from it;
  3. [knowledge] or [modules] is empty — a lock that pins nothing must not
     report like a lock whose pins all held;
  4. a pinned path no longer resolves to a file, or its sha256 drifted;
  5. a file exists under a locked root and NO row in content.lock pins it —
     this is the tree-side walk, and it is what makes deleting a row RED
     instead of silently narrowing the gate's own coverage (bd-z3v);
  6. a locked root is missing or matches zero files — a root that checked
     nothing must not report like a root whose every file matched.

The locked roots are exactly the roots scripts/gen_content_lock.py writes rows
from: knowledge/*.toml, web/content/modules/*.md, and ../modules/*.md. They are
named on stdout on the GREEN path together with the file counts, so a reader of
a green verdict is told what the verdict does not mean.

WHAT THIS GATE CANNOT DECIDE. It says nothing about any path outside those three
globs: bank/items is covered only transitively through bank_hash; the knowledge/
subdirectories (corpus, citations, graph, schema), non-.md files under the module
roots, scripts/, registries/, and the rest of web/ are not walked at all. It
cannot decide that a pinned digest is the RIGHT digest — a lock regenerated over
corrupted content is internally consistent and reports green. It cannot decide
that the content is correct, current, accurate, or well written, only that it is
byte-for-byte what someone pinned. And when it falls back to
goldens/bank_hash.txt for the live bank digest it is comparing a pin against
another pin, which is weaker than comparing a pin against a freshly computed
digest.

Exit 0 on green; exit 1 on any of the above.

Regenerate (human only):
  UPDATE_CONTENT_LOCK=1 python3 scripts/gen_content_lock.py

Optional selftest (does not mutate committed tree):
  CDCP_CONTENT_LOCK_SELFTEST=1 python3 scripts/verify_content_lock.py
  → plants a temp mismatch expectation and asserts RED path is reachable.

CDCP_BANK_HASH_TIMEOUT_S overrides the bank-hash subprocess timeout. It can only
shorten how long this gate is willing to wait, never turn a RED into a pass; an
unparseable or non-positive value is an ERROR, not a silent default.
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

# The bank-hash subprocess budget, and the env var that may shorten it.
BANK_HASH_TIMEOUT_S = 300.0
TIMEOUT_ENV = "CDCP_BANK_HASH_TIMEOUT_S"

# (section, display label, directory, glob pattern).
#
# These are EXACTLY the roots scripts/gen_content_lock.py enumerates when it
# writes the lock. Keeping the two lists identical is what makes "present in the
# tree but absent from the lock" a decidable question rather than an opinion: a
# regenerated lock is green here by construction, and any divergence is a real
# unpinned file. If gen_content_lock.py grows a root, this tuple must grow the
# same root or the gate goes RED on the first regenerated lock — which is the
# intended failure direction.
LOCKED_ROOTS: tuple[tuple[str, str, Path, str], ...] = (
    ("knowledge", "knowledge/*.toml", ROOT / "knowledge", "*.toml"),
    (
        "modules",
        "web/content/modules/*.md",
        ROOT / "web" / "content" / "modules",
        "*.md",
    ),
    ("modules", "../modules/*.md", ROOT.parent / "modules", "*.md"),
)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def bank_hash_timeout() -> float:
    """Seconds to wait on the bank-hash subprocess. Fail closed on garbage."""
    raw = os.environ.get(TIMEOUT_ENV)
    if raw is None or raw == "":
        return BANK_HASH_TIMEOUT_S
    try:
        val = float(raw)
    except ValueError:
        raise RuntimeError(
            f"invalid {TIMEOUT_ENV} (want a positive number of seconds)"
        ) from None
    if not val > 0:
        raise RuntimeError(f"invalid {TIMEOUT_ENV} (want a positive number of seconds)")
    return val


def live_bank_hash() -> str:
    timeout = bank_hash_timeout()
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
                timeout=timeout,
            )
        except subprocess.TimeoutExpired:
            # bd-hw3: this used to escape as an uncaught TimeoutExpired, i.e. a
            # traceback where a verdict belonged. A hung oracle is an ERROR.
            raise RuntimeError(
                "bank-hash timed out (cannot obtain live bank_hash)"
            ) from None
        except (subprocess.CalledProcessError, FileNotFoundError, OSError):
            continue
        lines = out.strip().splitlines()
        if not lines:
            # bd-hw3: `out.strip().splitlines()[-1]` on empty output raised
            # IndexError. A bank-hash that exits 0 saying nothing is a broken
            # oracle; falling through to the next candidate (or to the golden)
            # would let a broken oracle read as a pass.
            raise RuntimeError(
                "bank-hash exited 0 with no output (cannot obtain live bank_hash)"
            )
        hx = lines[-1].strip()
        if len(hx) == 64 and all(c in "0123456789abcdef" for c in hx):
            return hx
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


def lock_key(path: Path) -> str:
    """The spelling a content.lock row would use for a discovered file.

    Inverse of resolve_pinned: ROOT-relative when the file is under ROOT, else
    ROOT.parent-relative, else absolute (which no row can match, so it goes RED).
    """
    p = path.resolve()
    try:
        return p.relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        pass
    try:
        return p.relative_to(ROOT.parent.resolve()).as_posix()
    except ValueError:
        return p.as_posix()


def discover(directory: Path, pattern: str) -> list[Path]:
    """Files matching `pattern` directly under `directory`, in lock-key order."""
    if not directory.is_dir():
        return []
    return sorted((p for p in directory.glob(pattern) if p.is_file()), key=lock_key)


def root_counts() -> list[tuple[str, int]]:
    """(label, file count) per locked root, for the GREEN receipt."""
    return [
        (label, len(discover(directory, pattern)))
        for _section, label, directory, pattern in LOCKED_ROOTS
    ]


def tree_side_errors(pinned: dict[str, set[str]]) -> list[str]:
    """The walk the lock cannot narrow: every file under a locked root must be pinned.

    This is the half of the gate whose coverage is NOT defined by the artifact
    being checked. Without it, deleting a row deletes the check, and the removal
    is indistinguishable from a pass (bd-z3v).
    """
    errors: list[str] = []
    for section, label, directory, pattern in LOCKED_ROOTS:
        if not directory.is_dir():
            errors.append(
                f"[{section}] locked root is not a directory: {label} "
                "(nothing was checked there — vacuous ERROR)"
            )
            continue
        found = discover(directory, pattern)
        if not found:
            errors.append(
                f"[{section}] locked root matched zero files: {label} "
                "(nothing was checked there — vacuous ERROR)"
            )
            continue
        for p in found:
            key = lock_key(p)
            if key not in pinned[section]:
                errors.append(
                    f"[{section}] in the tree but not pinned in content.lock: {key}"
                )
    return errors


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

    pinned: dict[str, set[str]] = {"knowledge": set(), "modules": set()}
    for section, mapping in (("knowledge", knowledge), ("modules", modules)):
        if not isinstance(mapping, dict):
            errors.append(f"[{section}] must be a table of path = hash")
            continue
        pinned[section] = {str(k) for k in mapping}
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

    errors.extend(tree_side_errors(pinned))

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
    # A green verdict must say what it ranges over, or the reader supplies their
    # own optimistic scope. These are the roots that were walked file-by-file.
    print(
        "verify_content_lock: covered roots (every file found under these is "
        "pinned and matched): "
        + " ".join(f"{label}={n}" for label, n in root_counts())
    )
    print(
        "verify_content_lock: NOT covered: anything outside those roots — "
        "bank/items only through bank_hash, the knowledge/ subdirectories "
        "(corpus, citations, graph, schema), non-.md files under the module "
        "roots, scripts/, registries/, and the rest of web/"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
