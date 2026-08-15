#!/usr/bin/env python3
"""export_anki.py — export approved bank items to Anki TSV and minimal .apkg.

V11-S1 (bd-j54). Study aid only — not a credential and not an exam dump.
bank/seed42 sources drop retired and draft (bd-anki-ships-retired-bbdr).
Receipt names both populations: "N scanned, M exported".
.apkg bytes are a pure function of the bank (bd-anki-apkg-not-reproducible-e13a):
collection timestamps and zip entry mtimes come from SOURCE_DATE_EPOCH or a
pinned epoch, never time.time() or a temp file's mtime.

Usage:
  python3 scripts/export_anki.py
  python3 scripts/export_anki.py --source seed42 --out dist/anki
  python3 scripts/export_anki.py --source bank --module 6 --format tsv,apkg
  python3 scripts/export_anki.py --source bank --limit 40 --seed 42
  python3 scripts/export_anki.py --check

Sources:
  bank    — all bank/items/*.toml (default)
  seed42  — web/data/bank_items_seed42.json if present, else bank sample
  keys    — join keys_seed42.json + mock40_seed42.json stems (40-item deck)

Formats (comma-separated):
  tsv     — Anki-importable tab-separated (Front/Back/Explanation/Module)
  csv     — same fields, comma-separated with quoting
  apkg    — minimal Anki package (zip + collection.anki2)

.apkg path:
  Prefer genanki if installed; otherwise pure stdlib zip + sqlite3.
  Determinism is guaranteed on the pure-sqlite path (the shipped path).
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import random
import sqlite3
import sys
import tempfile
import time
import zipfile
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
ITEMS_DIR = ROOT / "bank" / "items"
WEB_DATA = ROOT / "web" / "data"
DEFAULT_OUT = ROOT / "dist" / "anki"

# Anki field separator inside notes.flds
FSEP = "\x1f"

# Note type / deck ids (stable for re-import friendliness)
MODEL_ID = 1699990001001
DECK_ID = 1699990002001

# Collection + zip clock. NEVER time.time(). NEVER filesystem mtime.
# SOURCE_DATE_EPOCH (reproducible-builds) wins; else this pinned epoch.
# 2023-11-14T22:13:20Z — same era as MODEL_ID / DECK_ID (1699990001xxx).
# Zip DOS dates only cover 1980–2107; deck_clock() rejects values outside that.
PINNED_EPOCH = 1_700_000_000
_ZIP_DOS_MIN = 315_532_800  # 1980-01-01T00:00:00Z
_ZIP_DOS_MAX = 4_354_812_799  # 2107-12-31T23:59:59Z


def deck_clock() -> int:
    """Unix seconds stamped into col/notes/cards and every ZipInfo.

    Wall clock is not an input. The deck is a function of the bank (and
    optionally SOURCE_DATE_EPOCH), so two runs on the same inputs match.
    """
    raw = os.environ.get("SOURCE_DATE_EPOCH")
    if raw is None or str(raw).strip() == "":
        return PINNED_EPOCH
    try:
        value = int(raw)
    except ValueError as exc:
        raise ValueError(f"SOURCE_DATE_EPOCH is not an integer: {raw!r}") from exc
    if value < _ZIP_DOS_MIN or value > _ZIP_DOS_MAX:
        raise ValueError(
            f"SOURCE_DATE_EPOCH {value} is outside the zip DOS range "
            f"[{_ZIP_DOS_MIN}, {_ZIP_DOS_MAX}]"
        )
    return value


def zip_date_time(epoch: int) -> tuple[int, int, int, int, int, int]:
    """UTC calendar tuple for ZipInfo. TZ-independent (never localtime)."""
    t = time.gmtime(epoch)
    return (t.tm_year, t.tm_mon, t.tm_mday, t.tm_hour, t.tm_min, t.tm_sec)


def write_deterministic_zip(
    path: Path, entries: list[tuple[str, bytes]], epoch: int
) -> None:
    """ZIP_DEFLATED archive whose entry mtimes are `epoch`, not file mtimes.

    ZipFile.write() would copy the temp collection's filesystem mtime into
    the local-file header (and avalanche DEFLATE). writestr + ZipInfo pins
    date_time, create_system, and Unix mode so the host and the temp dir
    cannot leak into the bytes.
    """
    dt = zip_date_time(epoch)
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as zf:
        for name, data in entries:
            info = zipfile.ZipInfo(filename=name, date_time=dt)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3  # Unix, even if we ever run this on win32
            info.external_attr = 0o644 << 16
            info.extra = b""
            info.comment = b""
            zf.writestr(info, data)


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def letter_to_index(letter: str) -> int:
    return ord(letter.upper()) - ord("A")


def format_answer(item: dict) -> str:
    correct = str(item.get("correct") or "").strip().upper()
    choices = item.get("choices") or []
    idx = letter_to_index(correct) if correct in "ABCD" else -1
    if 0 <= idx < len(choices):
        return f"{correct}) {choices[idx]}"
    return correct or "?"


def load_bank_items() -> list[dict]:
    items: list[dict] = []
    for path in sorted(ITEMS_DIR.glob("*.toml")):
        data = load_toml(path)
        if "id" in data:
            items.append(data)
    return items


def is_drawable(item: dict) -> bool:
    """C1 / bd-anki-ships-retired-bbdr: retired and draft never ship.

    Missing status is drawable so synthetic fixtures and the keys/mock pack
    (no status field) still export; the live bank always writes status.
    """
    return str(item.get("status") or "").strip().lower() not in {"retired", "draft"}


def approved_only(items: list[dict]) -> list[dict]:
    return [it for it in items if is_drawable(it)]


def load_seed42_bank_items() -> list[dict] | None:
    path = WEB_DATA / "bank_items_seed42.json"
    if not path.is_file():
        return None
    data = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(data, list):
        return data
    if isinstance(data, dict) and isinstance(data.get("items"), list):
        return data["items"]
    return None


def load_keys_seed42_pack() -> list[dict] | None:
    """Build 40 cards from learner mock + keys (seed42)."""
    mock_path = WEB_DATA / "mock40_seed42.json"
    keys_path = WEB_DATA / "keys_seed42.json"
    if not mock_path.is_file() or not keys_path.is_file():
        # fall back to golden fixture (has stems + correct)
        fix = ROOT / "goldens" / "fixtures" / "mock40_seed42.json"
        if fix.is_file():
            fix_data = json.loads(fix.read_text(encoding="utf-8"))
            rows = []
            for it in fix_data.get("items") or []:
                rows.append(
                    {
                        "id": it["id"],
                        "stem": it["stem"],
                        "choices": it.get("choices") or [],
                        "correct": it["correct"],
                        "explanation": it.get("explanation") or "",
                        "module": it.get("module", ""),
                    }
                )
            # explanations often missing on fixture — enrich from bank
            by_id = {x["id"]: x for x in load_bank_items()}
            for r in rows:
                if not r["explanation"] and r["id"] in by_id:
                    r["explanation"] = by_id[r["id"]].get("explanation") or ""
                    if r["module"] == "" or r["module"] is None:
                        r["module"] = by_id[r["id"]].get("module", "")
            return rows if rows else None
        return None

    mock = json.loads(mock_path.read_text(encoding="utf-8"))
    keys = json.loads(keys_path.read_text(encoding="utf-8"))
    key_map = {k["item_id"]: k for k in keys.get("keys") or []}
    rows: list[dict] = []
    for it in mock.get("items") or []:
        k = key_map.get(it["id"], {})
        rows.append(
            {
                "id": it["id"],
                "stem": it.get("stem") or "",
                "choices": it.get("choices") or [],
                "correct": k.get("correct") or "",
                "explanation": k.get("explanation") or "",
                "module": it.get("module", ""),
            }
        )
    # module may be missing on learner pack — enrich from bank if needed
    if any(r.get("module") in ("", None) for r in rows):
        by_id = {x["id"]: x for x in load_bank_items()}
        for r in rows:
            if r.get("module") in ("", None) and r["id"] in by_id:
                r["module"] = by_id[r["id"]].get("module", "")
    return rows


def filter_items(
    items: list[dict],
    *,
    module: int | None,
    tag: str | None,
    limit: int | None,
    seed: int | None,
) -> list[dict]:
    out = list(items)
    if module is not None:
        out = [it for it in out if int(it.get("module") or -1) == module]
    if tag:
        want = tag.lower()
        filtered = []
        for it in out:
            tags = it.get("tags") or []
            if isinstance(tags, str):
                tags = [tags]
            topic_blob = " ".join(str(t) for t in (it.get("topic_ids") or [])).lower()
            tag_blob = " ".join(str(t) for t in tags).lower()
            if want in tag_blob or want in topic_blob:
                filtered.append(it)
        out = filtered
    out.sort(key=lambda x: str(x.get("id") or ""))
    if limit is not None and limit > 0 and len(out) > limit:
        if seed is not None:
            rng = random.Random(seed)
            out = list(out)
            rng.shuffle(out)
            out = out[:limit]
            out.sort(key=lambda x: str(x.get("id") or ""))
        else:
            out = out[:limit]
    return out


def card_fields(item: dict) -> tuple[str, str, str, str]:
    stem = str(item.get("stem") or "").strip()
    answer = format_answer(item)
    explanation = str(item.get("explanation") or "").strip()
    module = str(item.get("module") if item.get("module") is not None else "")
    return stem, answer, explanation, module


def write_tsv(path: Path, items: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        # Anki text import: first line can be headers if # is used — plain header row
        # is fine when user maps columns; we emit a commented header Anki ignores
        # when using "fields separated by tabs" with no header option — so no # header.
        # Include a first metadata comment line for operators:
        f.write("# CDCP Study Anki export — stem / answer / explanation / module\n")
        f.write("# Not a credential. Import as Basic (or map 4 fields).\n")
        writer = csv.writer(f, delimiter="\t", lineterminator="\n", quoting=csv.QUOTE_MINIMAL)
        for it in items:
            stem, answer, explanation, module = card_fields(it)
            # collapse newlines so TSV stays one row per card
            stem = stem.replace("\t", " ").replace("\n", " ")
            answer = answer.replace("\t", " ").replace("\n", " ")
            explanation = explanation.replace("\t", " ").replace("\n", " ")
            writer.writerow([stem, answer, explanation, module])


def write_csv(path: Path, items: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f, lineterminator="\n")
        writer.writerow(["stem", "answer", "explanation", "module"])
        for it in items:
            stem, answer, explanation, module = card_fields(it)
            writer.writerow([stem, answer, explanation, module])


def _guid_for(item_id: str) -> str:
    h = hashlib.sha1(f"cdcp-anki:{item_id}".encode()).hexdigest()[:10]
    return h


def _csum(sfld: str) -> int:
    # Anki uses first 8 hex digits of sha1 of stripped field as int
    return int(hashlib.sha1(sfld.encode("utf-8")).hexdigest()[:8], 16)


def write_apkg_genanki(path: Path, items: list[dict], deck_name: str) -> bool:
    try:
        import genanki  # type: ignore
    except ImportError:
        return False

    model = genanki.Model(
        MODEL_ID,
        "CDCP Study Basic",
        fields=[
            {"name": "Stem"},
            {"name": "Answer"},
            {"name": "Explanation"},
            {"name": "Module"},
        ],
        templates=[
            {
                "name": "Card 1",
                "qfmt": "{{Stem}}<br><br><i>Module {{Module}}</i>",
                "afmt": "{{FrontSide}}<hr id=answer>{{Answer}}<br><br>{{Explanation}}",
            }
        ],
    )
    deck = genanki.Deck(DECK_ID, deck_name)
    for it in items:
        stem, answer, explanation, module = card_fields(it)
        note = genanki.Note(
            model=model,
            fields=[stem, answer, explanation, module],
            guid=_guid_for(str(it.get("id") or stem[:40])),
            tags=["cdcp-study", f"module{module}"] if module else ["cdcp-study"],
        )
        deck.add_note(note)
    path.parent.mkdir(parents=True, exist_ok=True)
    genanki.Package(deck).write_to_file(str(path))
    return True


def write_apkg_pure(
    path: Path, items: list[dict], deck_name: str, now: int | None = None
) -> None:
    """Minimal Anki 2 collection.anki2 packaged as .apkg (no media).

    `now` is the collection clock (col.crt/mod/scm, notes.mod, cards.mod,
    model/deck JSON `mod`, and every zip entry date_time). Default is
    deck_clock() — never time.time(). Pass an explicit value only from
    --check's planted-leak trip.
    """
    if now is None:
        now = deck_clock()
    model = {
        str(MODEL_ID): {
            "id": MODEL_ID,
            "name": "CDCP Study Basic",
            "type": 0,
            "mod": now,
            "usn": -1,
            "sortf": 0,
            "did": DECK_ID,
            "tmpls": [
                {
                    "name": "Card 1",
                    "ord": 0,
                    "qfmt": "{{Stem}}<br><br><i>Module {{Module}}</i>",
                    "afmt": "{{FrontSide}}<hr id=answer>{{Answer}}<br><br>{{Explanation}}",
                    "bqfmt": "",
                    "bafmt": "",
                    "did": None,
                    "bfont": "",
                    "bsize": 0,
                }
            ],
            "flds": [
                {
                    "name": "Stem",
                    "ord": 0,
                    "sticky": False,
                    "rtl": False,
                    "font": "Arial",
                    "size": 20,
                    "media": [],
                },
                {
                    "name": "Answer",
                    "ord": 1,
                    "sticky": False,
                    "rtl": False,
                    "font": "Arial",
                    "size": 20,
                    "media": [],
                },
                {
                    "name": "Explanation",
                    "ord": 2,
                    "sticky": False,
                    "rtl": False,
                    "font": "Arial",
                    "size": 16,
                    "media": [],
                },
                {
                    "name": "Module",
                    "ord": 3,
                    "sticky": False,
                    "rtl": False,
                    "font": "Arial",
                    "size": 14,
                    "media": [],
                },
            ],
            "css": ".card { font-family: arial; font-size: 18px; text-align: left; color: black; background-color: white; }",
            "latexPre": "",
            "latexPost": "",
            "latexsvg": False,
            "req": [[0, "all", [0]]],
            "tags": [],
            "vers": [],
        }
    }
    decks = {
        "1": {
            "id": 1,
            "name": "Default",
            "mod": now,
            "usn": -1,
            "collapsed": False,
            "browserCollapsed": False,
            "desc": "",
            "dyn": 0,
            "conf": 1,
            "extendNew": 0,
            "extendRev": 0,
        },
        str(DECK_ID): {
            "id": DECK_ID,
            "name": deck_name,
            "mod": now,
            "usn": -1,
            "collapsed": False,
            "browserCollapsed": False,
            "desc": "CDCP self-study cards. Study signal only — not EPI/EXIN certification.",
            "dyn": 0,
            "conf": 1,
            "extendNew": 0,
            "extendRev": 0,
        },
    }
    conf = {
        "nextPos": 1,
        "estTimes": True,
        "activeDecks": [DECK_ID],
        "sortType": "noteFld",
        "timeLim": 0,
        "sortBackwards": False,
        "addToCur": True,
        "curDeck": DECK_ID,
        "newBury": True,
        "newSpread": 0,
        "dueCounts": True,
        "curModel": MODEL_ID,
        "collapseTime": 1200,
    }
    dconf = {
        "1": {
            "id": 1,
            "name": "Default",
            "mod": 0,
            "usn": 0,
            "maxTaken": 60,
            "autoplay": True,
            "timer": 0,
            "replayq": True,
            "new": {
                "bury": True,
                "delays": [1, 10],
                "initialFactor": 2500,
                "ints": [1, 4, 0],
                "order": 1,
                "perDay": 20,
            },
            "rev": {
                "bury": True,
                "ease4": 1.3,
                "ivlFct": 1,
                "maxIvl": 36500,
                "perDay": 200,
                "hardFactor": 1.2,
            },
            "lapse": {
                "delays": [10],
                "leechAction": 0,
                "leechFails": 8,
                "minInt": 1,
                "mult": 0,
            },
        }
    }

    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="cdcp-anki-") as td:
        td_path = Path(td)
        db_path = td_path / "collection.anki2"
        conn = sqlite3.connect(str(db_path))
        cur = conn.cursor()
        cur.executescript(
            """
            CREATE TABLE col (
              id integer primary key,
              crt integer not null,
              mod integer not null,
              scm integer not null,
              ver integer not null,
              dty integer not null,
              usn integer not null,
              ls integer not null,
              conf text not null,
              models text not null,
              decks text not null,
              dconf text not null,
              tags text not null
            );
            CREATE TABLE notes (
              id integer primary key,
              guid text not null,
              mid integer not null,
              mod integer not null,
              usn integer not null,
              tags text not null,
              flds text not null,
              sfld text not null,
              csum integer not null,
              flags integer not null,
              data text not null
            );
            CREATE TABLE cards (
              id integer primary key,
              nid integer not null,
              did integer not null,
              ord integer not null,
              mod integer not null,
              usn integer not null,
              type integer not null,
              queue integer not null,
              due integer not null,
              ivl integer not null,
              factor integer not null,
              reps integer not null,
              lapses integer not null,
              left integer not null,
              odue integer not null,
              odid integer not null,
              flags integer not null,
              data text not null
            );
            CREATE TABLE revlog (
              id integer primary key,
              cid integer not null,
              usn integer not null,
              ease integer not null,
              ivl integer not null,
              lastIvl integer not null,
              factor integer not null,
              time integer not null,
              type integer not null
            );
            CREATE TABLE graves (
              usn integer not null,
              oid integer not null,
              type integer not null
            );
            CREATE INDEX ix_notes_usn on notes (usn);
            CREATE INDEX ix_cards_usn on cards (usn);
            CREATE INDEX ix_revlog_usn on revlog (usn);
            CREATE INDEX ix_cards_nid on cards (nid);
            CREATE INDEX ix_cards_sched on cards (did, queue, due);
            CREATE INDEX ix_revlog_cid on revlog (cid);
            CREATE INDEX ix_notes_csum on notes (csum);
            """
        )
        cur.execute(
            "INSERT INTO col (id,crt,mod,scm,ver,dty,usn,ls,conf,models,decks,dconf,tags) "
            "VALUES (1,?,?,?,11,0,0,0,?,?,?,?,?)",
            (
                now,
                now * 1000,
                now * 1000,
                json.dumps(conf),
                json.dumps(model),
                json.dumps(decks),
                json.dumps(dconf),
                json.dumps({}),
            ),
        )

        # Stable-ish note/card ids from content hash so re-exports don't thrash
        for i, it in enumerate(items):
            stem, answer, explanation, module = card_fields(it)
            iid = str(it.get("id") or f"row-{i}")
            note_id = int(hashlib.sha1(f"note:{iid}".encode()).hexdigest()[:12], 16) % (10**13)
            card_id = int(hashlib.sha1(f"card:{iid}".encode()).hexdigest()[:12], 16) % (10**13)
            # ensure uniqueness on collision
            note_id = note_id + i
            card_id = card_id + i
            flds = FSEP.join([stem, answer, explanation, module])
            tags = f" cdcp-study module{module} " if module else " cdcp-study "
            cur.execute(
                "INSERT INTO notes (id,guid,mid,mod,usn,tags,flds,sfld,csum,flags,data) "
                "VALUES (?,?,?,?,?,?,?,?,?,?,?)",
                (
                    note_id,
                    _guid_for(iid),
                    MODEL_ID,
                    now,
                    -1,
                    tags,
                    flds,
                    stem,
                    _csum(stem),
                    0,
                    "",
                ),
            )
            cur.execute(
                "INSERT INTO cards (id,nid,did,ord,mod,usn,type,queue,due,ivl,factor,"
                "reps,lapses,left,odue,odid,flags,data) "
                "VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                (
                    card_id,
                    note_id,
                    DECK_ID,
                    0,
                    now,
                    -1,
                    0,  # new
                    0,  # new queue
                    i + 1,  # due order
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    "",
                ),
            )
        conn.commit()
        conn.close()

        # Read the image and zip via ZipInfo so the temp file's mtime cannot
        # enter the archive. VACUUM is not required: a fresh file + the same
        # inserts is already byte-stable on this sqlite (probed 2026-08-14).
        db_bytes = db_path.read_bytes()
        write_deterministic_zip(
            path,
            [("collection.anki2", db_bytes), ("media", b"{}")],
            now,
        )


def write_apkg(path: Path, items: list[dict], deck_name: str) -> str:
    if write_apkg_genanki(path, items, deck_name):
        return "genanki"
    write_apkg_pure(path, items, deck_name)
    return "pure-sqlite"


def planted_clock_leak_trips() -> None:
    """L4: two different clocks MUST change the .apkg, or --check is vacuous."""
    items = [
        {
            "id": "plant-a",
            "stem": "s",
            "choices": ["x"],
            "correct": "A",
            "explanation": "e",
            "module": 1,
        },
        {
            "id": "plant-b",
            "stem": "t",
            "choices": ["x"],
            "correct": "A",
            "explanation": "e",
            "module": 1,
        },
    ]
    with tempfile.TemporaryDirectory(prefix="cdcp-anki-plant-") as td:
        a = Path(td) / "a.apkg"
        b = Path(td) / "b.apkg"
        write_apkg_pure(a, items, "CDCP Study", now=PINNED_EPOCH)
        write_apkg_pure(b, items, "CDCP Study", now=PINNED_EPOCH + 1)
        if a.read_bytes() == b.read_bytes():
            raise RuntimeError(
                "planted clock leak did not change .apkg bytes — --check is vacuous"
            )


def peek_apkg(path: Path) -> tuple[int, int, int, tuple[int, ...]]:
    """Return (col.crt, col.mod, col.scm, zip date_time of collection.anki2)."""
    with zipfile.ZipFile(path) as zf:
        info = zf.getinfo("collection.anki2")
        raw = zf.read("collection.anki2")
        date_time = info.date_time
    conn = sqlite3.connect(":memory:")
    try:
        conn.deserialize(raw)
        crt, mod, scm = conn.execute("SELECT crt, mod, scm FROM col WHERE id = 1").fetchone()
    finally:
        conn.close()
    return int(crt), int(mod), int(scm), date_time


def run_repro_check(items: list[dict], deck_name: str) -> int:
    """Two successive exports of the same items must be byte-identical.

    Sleeps across a whole-second boundary so a leftover time.time() would
    flip col.crt and avalanche DEFLATE — the measurement that opened
    bd-anki-apkg-not-reproducible-e13a.
    """
    try:
        planted_clock_leak_trips()
    except RuntimeError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    print("export_anki check: planted clock leak trips (bytes differ)")

    try:
        clock = deck_clock()
    except ValueError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="cdcp-anki-check-") as td:
        td_path = Path(td)
        paths: list[Path] = []
        for i in range(2):
            dest = td_path / f"run{i}"
            dest.mkdir()
            apkg = dest / "cdcp_bank.apkg"
            tsv = dest / "cdcp_bank.tsv"
            write_apkg_pure(apkg, items, deck_name)
            write_tsv(tsv, items)
            paths.append(dest)
            if i == 0:
                time.sleep(1.1)

        a_apkg = (td_path / "run0" / "cdcp_bank.apkg").read_bytes()
        b_apkg = (td_path / "run1" / "cdcp_bank.apkg").read_bytes()
        a_tsv = (td_path / "run0" / "cdcp_bank.tsv").read_bytes()
        b_tsv = (td_path / "run1" / "cdcp_bank.tsv").read_bytes()
        if a_tsv != b_tsv:
            print("FAIL: TSV differed across two successive exports", file=sys.stderr)
            return 1
        if a_apkg != b_apkg:
            print(
                f"FAIL: .apkg differed across two successive exports "
                f"({sum(x != y for x, y in zip(a_apkg, b_apkg, strict=False))} of "
                f"{max(len(a_apkg), len(b_apkg))} bytes)",
                file=sys.stderr,
            )
            return 1

        crt, mod, scm, date_time = peek_apkg(td_path / "run0" / "cdcp_bank.apkg")
        expect_dt = zip_date_time(clock)
        if crt != clock or date_time != expect_dt:
            print(
                f"FAIL: clock leak in artifact: col.crt={crt} zip_date_time={date_time} "
                f"expected crt={clock} date_time={expect_dt}",
                file=sys.stderr,
            )
            return 1
        if mod != clock * 1000 or scm != clock * 1000:
            print(
                f"FAIL: col.mod/scm not derived from the pinned clock: "
                f"mod={mod} scm={scm} expected {clock * 1000}",
                file=sys.stderr,
            )
            return 1

    digest = hashlib.sha256(a_apkg).hexdigest()
    print(f"export_anki check: two runs identical sha256={digest}")
    print(f"export_anki check: col.crt={crt} col.mod={mod} zip_date_time={date_time}")
    print(f"export_anki check: ok cards={len(items)}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description="Export CDCP bank items for Anki (study only).")
    ap.add_argument(
        "--source",
        choices=["bank", "seed42", "keys"],
        default="bank",
        help="bank=all items; seed42=web bank_items export; keys=seed42 mock+keys (40 cards)",
    )
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT, help="output directory")
    ap.add_argument(
        "--format",
        default="tsv,apkg",
        help="comma-separated: tsv,csv,apkg (default: tsv,apkg)",
    )
    ap.add_argument("--module", type=int, default=None, help="filter by module number")
    ap.add_argument("--tag", default=None, help="filter by tag or topic_id substring")
    ap.add_argument("--limit", type=int, default=None, help="max cards")
    ap.add_argument("--seed", type=int, default=None, help="shuffle seed when --limit set")
    ap.add_argument(
        "--deck-name",
        default="CDCP Study",
        help="Anki deck name for .apkg",
    )
    ap.add_argument(
        "--check",
        action="store_true",
        help="export twice to temp dirs and FAIL if .apkg bytes differ "
        "(does not write --out)",
    )
    args = ap.parse_args()

    if args.source == "bank":
        items = load_bank_items()
        stem = "cdcp_bank"
    elif args.source == "seed42":
        items = load_seed42_bank_items()
        if items is None:
            print("WARN: bank_items_seed42.json missing — falling back to bank", file=sys.stderr)
            items = load_bank_items()
            stem = "cdcp_bank"
        else:
            stem = "cdcp_seed42_bank"
    else:  # keys
        items = load_keys_seed42_pack()
        if items is None:
            print("FAIL: keys/seed42 packs not found", file=sys.stderr)
            return 1
        stem = "cdcp_seed42_mock40"

    if not items:
        print("FAIL: zero items to export", file=sys.stderr)
        return 1

    scanned = len(items)
    # Bank and seed42 packs carry `status`. The keys/mock40 source is already
    # the approved-only draw and often has no status field — do not strip it.
    if args.source in ("bank", "seed42"):
        items = approved_only(items)
        if not items:
            print("FAIL: zero approved items to export", file=sys.stderr)
            return 1

    items = filter_items(
        items,
        module=args.module,
        tag=args.tag,
        limit=args.limit,
        seed=args.seed,
    )
    if not items:
        print("FAIL: filter removed all items", file=sys.stderr)
        return 1

    if args.check:
        return run_repro_check(items, args.deck_name)

    formats = {f.strip().lower() for f in args.format.split(",") if f.strip()}
    unknown = formats - {"tsv", "csv", "apkg"}
    if unknown:
        print(f"FAIL: unknown format(s): {sorted(unknown)}", file=sys.stderr)
        return 1
    if not formats:
        print("FAIL: no formats requested", file=sys.stderr)
        return 1

    out_dir: Path = args.out
    out_dir.mkdir(parents=True, exist_ok=True)
    written: list[str] = []

    if "tsv" in formats:
        p = out_dir / f"{stem}.tsv"
        write_tsv(p, items)
        written.append(str(p))
    if "csv" in formats:
        p = out_dir / f"{stem}.csv"
        write_csv(p, items)
        written.append(str(p))
    if "apkg" in formats:
        p = out_dir / f"{stem}.apkg"
        try:
            backend = write_apkg(p, items, args.deck_name)
        except ValueError as exc:
            print(f"FAIL: {exc}", file=sys.stderr)
            return 1
        written.append(f"{p} ({backend})")

    # Operator note next to outputs
    note = out_dir / "README.txt"
    note.write_text(
        "CDCP Study — Anki export\n"
        "========================\n"
        "Study tool only. Does NOT grant EPI/EXIN certification.\n"
        "Not an exam dump; original educational bank content.\n\n"
        "Fields: stem | answer | explanation | module\n"
        "TSV: import in Anki → File → Import → map 4 fields (or use Basic + Extra).\n"
        "APKG: double-click / File → Import to load the deck.\n"
        f"Cards: {len(items)}\n"
        f"Source: {args.source}\n",
        encoding="utf-8",
    )
    written.append(str(note))

    print("export_anki ok")
    print(f"  cards={len(items)}")
    print(f"  {scanned} scanned, {len(items)} exported")
    print(f"  source={args.source}")
    for w in written:
        print(f"  wrote {w}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
