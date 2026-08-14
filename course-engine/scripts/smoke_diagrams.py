#!/usr/bin/env python3
"""smoke_diagrams.py — bind every PRESENT registry row to its actual artifact.

WHAT THIS GATE CLAIMS
---------------------
For every row the inventory table of ``docs/DIAGRAM-REGISTRY.md`` marks
``**present**``, the file ``web/diagrams/<id>.html`` exists, parses as HTML, and
contains BOTH structural landmarks a present diagram is defined by:

  * an element carrying the class ``honesty-banner``, whose own text says the
    tool does *not* grant certification, and
  * an element carrying the attribute ``data-diagram="<id>"``.

and that the number of present rows equals the pinned ``EXPECTED_PRESENT``.

FLOOR-RAISE — what this gate MOVED
----------------------------------
Two earlier versions of this file were each a fooled certificate.

  1. The check set was a Python literal naming the three P0 diagrams. Four P1
     diagrams shipped 2026-08-14 and the gate stayed green having looked at none
     of them. Fix: DERIVE the set from the registry.
  2. Deriving fixed WHICH items are checked and left HOW they are checked
     untouched, so the registry became a new place to write the same lie. A row
     could name ``docs/DIAGRAM-REGISTRY.md`` as its path — that file exists and
     contains the substrings "not", "certif" and "fire-sequence", so every
     predicate passed and the real diagram was never opened. Confirmed by
     injection: exit 0, "PASS (7 present diagrams from the registry)".

The lesson this file now encodes: deriving a reference is necessary and not
sufficient — the derived reference has to be BOUND to the artifact, and the
predicates applied to it have to be structural. Concretely, this version:

  * pins a present row's path to exactly ``web/diagrams/<id>.html``, so the row
    cannot redirect the check at a file that is not the diagram;
  * pins the expected present count, so a row leaving the PRESENT set is RED
    rather than invisible (partial coverage loss used to report identically to
    full coverage);
  * parses the inventory table STRUCTURALLY — every data row between the header
    and the end of the table must parse, so dropping the backticks off an ID no
    longer makes a row vanish from the check set;
  * rejects any status value outside a closed enum, so a status typo is an ERROR
    and never a silent exclusion;
  * parses the HTML for the honesty-banner element and the ``data-diagram``
    attribute instead of grepping the file for "not"/"certif". A file whose whole
    content is ``not certif fire-sequence`` now fails.

WHAT THIS GATE CANNOT DECIDE
----------------------------
  * REGISTRY COMPLETENESS. A diagram that ships with no registry row is still
    unchecked. The set is derived from a reviewed document, not from the
    filesystem; this gate moves the omission from a Python literal into that
    document and prints the count, it does not prove the document is complete.
  * PEDAGOGICAL OR FACTUAL CORRECTNESS. The presence of an honesty banner is not
    evidence that the banner's claim is true, that the diagram teaches the right
    model, or that its content is accurate. This gate is a structural floor.
  * THAT THE DIAGRAM WORKS. No JavaScript is executed, no interaction is driven,
    no rendering is verified. A diagram whose stepper is broken passes.
  * THE PIN'S CORRECTNESS. ``EXPECTED_PRESENT`` is a tripwire against silent
    drift, not a source of truth. Deliberately adding a diagram means editing it
    — that edit is the point.

EXIT CODES
----------
  0  every present row bound to a conforming artifact
  1  FAIL — a registry row is well-formed but its artifact is missing or does not
     carry the required structure (an artifact defect)
  2  ERROR — the registry itself cannot be trusted: unreadable, no inventory
     table, unexpected columns, a malformed row, an unrecognised status, a path
     that is not the row's own diagram, zero present rows, or a present count
     that disagrees with the pin

This file is on the Rust migration list (bd-substrate-rust-migration-jhd.14) and
is intended to serve as a byte-exact differential oracle for the port: every
message below is part of the specified behaviour, not incidental phrasing.
"""
from __future__ import annotations

import re
import sys
from html.parser import HTMLParser
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs" / "DIAGRAM-REGISTRY.md"

# The directory a present diagram must live in. A row's path is bound to
# f"{DIAGRAM_DIR}/{id}.html" — nothing else is accepted, so the row cannot
# redirect the check at some other existing file.
DIAGRAM_DIR = "web/diagrams"

# Pinned so that a row silently leaving the PRESENT set is RED, not invisible.
# Raise this in the same commit that ships a new present diagram.
EXPECTED_PRESENT = 7

# The inventory table is located by heading and validated by column names, so a
# reformat is an ERROR rather than a regex that quietly stops matching.
INVENTORY_HEADING = "## Inventory"
EXPECTED_COLUMNS = ["ID", "Title", "Modules", "Priority", "Status", "Path"]

# Closed enum. An unrecognised status is an ERROR, never a silent exclusion.
STATUS_PRESENT = "present"
STATUS_PLANNED = "planned"
KNOWN_STATUSES = (STATUS_PRESENT, STATUS_PLANNED)

# A planned row must name no path. em dash, en dash, hyphen or empty.
NO_PATH_CELL = {"—", "–", "-", ""}

ID_CELL = re.compile(r"^`([a-z0-9-]+)`$")
PATH_CELL = re.compile(r"^`([^`]+)`$")
SEPARATOR_CELL = re.compile(r"^:?-{3,}:?$")
NOT_WORD = re.compile(r"\bnot\b")

# HTML elements that never have an end tag; they must not open a nesting level.
VOID_TAGS = frozenset(
    "area base br col embed hr img input link meta param source track wbr".split()
)


class DiagramHTML(HTMLParser):
    """Collects the two structural landmarks a present diagram must carry.

    Structural, not substring: ``data-diagram`` is read as a parsed attribute of
    a real element, and the honesty text is read from inside the element that
    carries class ``honesty-banner``. A plain text file yields zero tags, no
    banner and no markers, and therefore fails — which is the whole point.
    """

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.tags = 0
        self.markers: set[str] = set()
        self.saw_banner = False
        self._banner_text: list[str] = []
        self._depth = 0  # nesting level inside the honesty banner; 0 = outside

    def _open(self, tag: str, attrs: list[tuple[str, str | None]], void: bool) -> None:
        self.tags += 1
        attr = {k.lower(): (v or "") for k, v in attrs}
        marker = attr.get("data-diagram")
        if marker is not None:
            self.markers.add(marker.strip())
        if self._depth:
            if not void:
                self._depth += 1
        elif "honesty-banner" in attr.get("class", "").split():
            self.saw_banner = True
            if not void:
                self._depth = 1

    def handle_starttag(self, tag, attrs):  # noqa: D102
        self._open(tag, attrs, tag.lower() in VOID_TAGS)

    def handle_startendtag(self, tag, attrs):  # noqa: D102
        self._open(tag, attrs, True)

    def handle_endtag(self, tag):  # noqa: D102
        if tag.lower() in VOID_TAGS:
            return
        if self._depth:
            self._depth -= 1

    def handle_data(self, data):  # noqa: D102
        if self._depth:
            self._banner_text.append(data)

    @property
    def banner_text(self) -> str:
        return " ".join("".join(self._banner_text).split())


def split_row(line: str) -> list[str] | None:
    """Cells of a markdown table row, or None if the line is not a table row."""
    s = line.strip()
    if not s.startswith("|") or not s.endswith("|") or len(s) < 2:
        return None
    return [c.strip() for c in s[1:-1].split("|")]


def parse_registry() -> tuple[list[tuple[str, str]], list[str]] | None:
    """Return (rows, errors) for the inventory table, or None if unreadable.

    ``rows`` is a list of (id, status). None means ERROR-unreadable, and is
    deliberately a different return value from an empty list: conflating "I could
    not read the registry" with "the registry lists nothing" is exactly how an
    unreadable input becomes a pass.

    Every data row between the header and the end of the table MUST parse. There
    is no "skip the lines that do not match" leg — that leg is what let a row
    with unbackticked cells disappear from the check set while the gate stayed
    green on the survivors.
    """
    if not REGISTRY.is_file():
        print(f"smoke_diagrams: ERROR: missing registry {REGISTRY}")
        return None
    try:
        lines = REGISTRY.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeDecodeError) as exc:
        print(f"smoke_diagrams: ERROR: unreadable registry {REGISTRY}: {exc}")
        return None

    start = next(
        (i for i, ln in enumerate(lines) if ln.strip() == INVENTORY_HEADING), None
    )
    if start is None:
        print(f"smoke_diagrams: ERROR: no '{INVENTORY_HEADING}' heading in {REGISTRY}")
        return None

    head = next((i for i in range(start + 1, len(lines)) if split_row(lines[i])), None)
    if head is None:
        print(f"smoke_diagrams: ERROR: no table under '{INVENTORY_HEADING}'")
        return None

    columns = split_row(lines[head]) or []
    if columns != EXPECTED_COLUMNS:
        print(
            "smoke_diagrams: ERROR: inventory columns changed: "
            f"expected {EXPECTED_COLUMNS}, found {columns}"
        )
        return None

    body = head + 1
    sep = split_row(lines[body]) if body < len(lines) else None
    if not sep or not all(SEPARATOR_CELL.match(c) for c in sep):
        print(f"smoke_diagrams: ERROR: inventory header not followed by a separator row")
        return None

    rows: list[tuple[str, str]] = []
    errors: list[str] = []
    seen: set[str] = set()
    for lineno in range(body + 1, len(lines)):
        cells = split_row(lines[lineno])
        if cells is None:
            break  # end of the table
        where = f"{REGISTRY.name}:{lineno + 1}"
        if len(cells) != len(EXPECTED_COLUMNS):
            errors.append(f"{where}: expected {len(EXPECTED_COLUMNS)} cells, got {len(cells)}")
            continue
        raw_id, _title, _modules, _priority, raw_status, raw_path = cells

        m = ID_CELL.match(raw_id)
        if not m:
            errors.append(f"{where}: malformed ID cell {raw_id!r} (want `lower-kebab-id`)")
            continue
        did = m.group(1)
        if did in seen:
            errors.append(f"{where}: duplicate ID `{did}`")
            continue
        seen.add(did)

        status = raw_status.replace("*", "").replace("`", "").strip().lower()
        if status not in KNOWN_STATUSES:
            errors.append(
                f"{where}: `{did}` unrecognised status {raw_status!r} "
                f"(known: {', '.join(KNOWN_STATUSES)})"
            )
            continue

        if status == STATUS_PLANNED:
            if raw_path.replace("`", "").strip() not in NO_PATH_CELL:
                errors.append(f"{where}: `{did}` is planned but names path {raw_path!r}")
            else:
                rows.append((did, status))
            continue

        pm = PATH_CELL.match(raw_path)
        if not pm:
            errors.append(f"{where}: `{did}` malformed path cell {raw_path!r} (want `path`)")
            continue
        want = f"{DIAGRAM_DIR}/{did}.html"
        got = pm.group(1).strip()
        if got != want:
            # Bind the row to its own artifact. A present row may not point at
            # any other file — including this registry — and a mismatch is an
            # ERROR, never a skip.
            errors.append(f"{where}: `{did}` path {got!r} is not {want!r}")
            continue
        rows.append((did, status))

    return rows, errors


def check_diagram(did: str) -> list[str]:
    """Structural failures for one present diagram. Empty list means conforming."""
    path = ROOT / PurePosixPath(f"{DIAGRAM_DIR}/{did}.html")
    if not path.is_file():
        return [f"{did}: missing {path.relative_to(ROOT)}"]
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        return [f"{did}: unreadable {path.relative_to(ROOT)}: {exc}"]

    parser = DiagramHTML()
    try:
        parser.feed(text)
        parser.close()
    except Exception as exc:  # malformed beyond html.parser's tolerance
        return [f"{did}: unparseable HTML: {exc}"]

    bad: list[str] = []
    if parser.tags == 0:
        return [f"{did}: not HTML (zero elements parsed)"]
    if not parser.saw_banner:
        bad.append(f'{did}: no element with class="honesty-banner"')
    else:
        banner = parser.banner_text.lower()
        if not banner:
            bad.append(f"{did}: honesty-banner element is empty")
        elif not (NOT_WORD.search(banner) and "certif" in banner):
            # Scoped to the parsed banner element, not to the whole file: the
            # claim under test is that THE BANNER disclaims certification.
            bad.append(f"{did}: honesty-banner does not disclaim certification")
    if did not in parser.markers:
        bad.append(f'{did}: no element with data-diagram="{did}"')
    return bad


def main() -> int:
    print("==> smoke_diagrams")
    parsed = parse_registry()
    if parsed is None:
        return 2  # unreadable input — ERROR, distinct from FAIL
    rows, errors = parsed
    for err in errors:
        print(f"  ERROR: {err}")

    present = [did for did, status in rows if status == STATUS_PRESENT]

    # Anti-vacuous: an empty set is an ERROR, never a pass. A parser that
    # silently stopped matching after a table reformat would otherwise report
    # PASS on zero rows.
    if not present:
        print("smoke_diagrams: ERROR: zero present diagrams parsed from the registry")
        return 2
    if len(present) != EXPECTED_PRESENT:
        print(
            f"  ERROR: present count {len(present)} != pinned {EXPECTED_PRESENT} "
            f"({', '.join(present)})"
        )
        errors.append("count")
    if errors:
        print(f"smoke_diagrams: ERROR ({len(errors)}) — registry not trustworthy")
        return 2

    fails: list[str] = []
    for did in present:
        bad = check_diagram(did)
        if bad:
            fails.extend(bad)
            for msg in bad:
                print(f"  FAIL: {msg}")
        else:
            print(f"  ok: {did}")
    if fails:
        print(f"smoke_diagrams: FAIL ({len(fails)})")
        return 1
    print(f"smoke_diagrams: PASS ({len(present)} present diagrams from the registry)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
