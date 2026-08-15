#!/usr/bin/env python3
"""smoke_learn_v2.py — units index + unit shell + glossary + concept card assets.

Anti-vacuous (bd-learnv2-vacuous-coverage-dad8): an empty `units` array or an
empty M01 list is ERROR, never `ok: check_item_ids coverage 0/0` / a pass of
the M01 check-item floor over nothing.

`unit_count` is a DECLARED field, not `len(units)`. This smoke does not
cross-check them. Empty `units` is independently ERROR. A non-empty array
with a lying declared count is a different defect: coverage and the module
floors already run over the actual arrays, and the `unit_count` floor is a
collapse detector on the declared field (same shape as glossary `term_count`).

Fail-closed inputs (bd-learnv2-unguarded-reads-0f7g): a missing Learn/Drill
page, malformed JSON, or a non-UTF-8 file is a FAIL row + verdict, never a
CPython traceback. Type mismatches inside a parsed object still raise.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WEB = ROOT / "web"
errs: list[str] = []


def fail(m: str) -> None:
    errs.append(m)
    print("  FAIL:", m)


def ok(m: str) -> None:
    print("  ok:", m)


def _read_utf8(path: Path, label: str) -> str | None:
    """Read a file as UTF-8 or emit a FAIL row. Caller already checked is_file()."""
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        fail(f"{label} is not valid UTF-8")
        return None
    except OSError as e:
        fail(f"{label} unreadable ({e})")
        return None


def _load_json_object(path: Path, label: str) -> dict | None:
    text = _read_utf8(path, label)
    if text is None:
        return None
    try:
        data = json.loads(text)
    except json.JSONDecodeError:
        fail(f"{label} is not valid JSON")
        return None
    if not isinstance(data, dict):
        fail(f"{label} is not a JSON object")
        return None
    return data


def evaluate(web: Path) -> int:
    errs.clear()
    print("==> smoke_learn_v2 (M8 B/D assets)")
    units_path = web / "data/units_index.json"
    if not units_path.is_file():
        fail("missing units_index.json — run `cdcp build-units`")
    else:
        d = _load_json_object(units_path, "units_index.json")
        if d is not None:
            by = d.get("by_module") or {}
            for mid, need in (("01-mission-critical", 4), ("06-power", 3)):
                n = len(by.get(mid) or [])
                if n < need:
                    fail(f"{mid} units={n} need ≥{need}")
                else:
                    ok(f"{mid} units={n}")
            if (d.get("unit_count") or 0) < 50:
                fail("unit_count too low")
            else:
                ok(f"unit_count={d.get('unit_count')}")
            # Quick-check coverage: units must carry real bank item ids.
            # Empty units is ERROR, not a ZeroDivisionError-avoiding ok 0/0.
            units = d.get("units") or []
            with_checks = 0
            thin = []
            for u in units:
                n = len(u.get("check_item_ids") or [])
                if n >= 2:
                    with_checks += 1
                else:
                    thin.append(u.get("id"))
            if not units:
                fail("zero units in units_index.json (vacuous coverage is ERROR)")
            elif with_checks / len(units) < 0.8:
                fail(
                    f"check_item_ids coverage {with_checks}/{len(units)} < 80% "
                    f"(sample thin: {thin[:5]})"
                )
            else:
                ok(f"check_item_ids coverage {with_checks}/{len(units)}")
            m01 = (d.get("by_module") or {}).get("01-mission-critical") or []
            if not m01:
                fail(
                    "zero M01 units in units_index.json "
                    "(vacuous M01 check-item floor is ERROR)"
                )
            elif min(len(u.get("check_item_ids") or []) for u in m01) < 2:
                fail("M01 unit missing ≥2 check_item_ids")
            else:
                ok("M01 every unit has ≥2 check items")

    gloss = web / "data/glossary.json"
    if not gloss.is_file():
        fail("missing glossary.json")
    else:
        g = _load_json_object(gloss, "glossary.json")
        if g is not None:
            if (g.get("term_count") or 0) < 15:
                fail("glossary term_count < 15")
            else:
                ok(f"glossary terms={g.get('term_count')}")

    for rel in (
        "assets/js/learn_units.js",
        "assets/js/learn_glossary.js",
        "assets/js/concept_card.js",
    ):
        if not (web / rel).is_file():
            fail(f"missing {rel}")
        else:
            ok(rel)

    m01_path = web / "learn/01-mission-critical.html"
    if not m01_path.is_file():
        fail("missing learn/01-mission-critical.html")
    else:
        m01 = _read_utf8(m01_path, "learn/01-mission-critical.html")
        if m01 is not None:
            for needle in ("learn-unit-shell", "learn_units.js", "learn_glossary.js"):
                if needle not in m01:
                    fail(f"M01 missing {needle}")
                else:
                    ok(f"M01 {needle}")

    drill_path = web / "drill.html"
    if not drill_path.is_file():
        fail("missing drill.html")
    else:
        drill = _read_utf8(drill_path, "drill.html")
        if drill is not None:
            if "concept_card.js" not in drill:
                fail("drill.html missing concept_card.js")
            else:
                ok("drill concept_card")

    if errs:
        print("smoke_learn_v2: FAIL")
        return 1
    print("smoke_learn_v2: PASS")
    return 0


def main() -> int:
    return evaluate(WEB)


def _unit(uid: str, checks: int) -> dict:
    return {"id": uid, "check_item_ids": [f"{uid}-c{k}" for k in range(checks)]}


def _good_payload() -> dict:
    m01 = [_unit(f"u01-{k}", 2) for k in range(4)]
    m06 = [_unit(f"u06-{k}", 2) for k in range(3)]
    return {
        "unit_count": 60,
        "by_module": {"01-mission-critical": m01, "06-power": m06},
        "units": m01 + m06,
    }


def _plant_web(tmp: Path, payload: dict) -> Path:
    web = tmp / "web"
    (web / "data").mkdir(parents=True)
    (web / "assets/js").mkdir(parents=True)
    (web / "learn").mkdir(parents=True)
    (web / "data/units_index.json").write_text(
        json.dumps(payload), encoding="utf-8"
    )
    (web / "data/glossary.json").write_text(
        '{"term_count": 40}\n', encoding="utf-8"
    )
    for name in ("learn_units.js", "learn_glossary.js", "concept_card.js"):
        (web / "assets/js" / name).write_text("// script\n", encoding="utf-8")
    (web / "learn/01-mission-critical.html").write_text(
        '<div class="learn-unit-shell"></div>'
        '<script src="learn_units.js"></script>'
        '<script src="learn_glossary.js"></script>\n',
        encoding="utf-8",
    )
    (web / "drill.html").write_text(
        '<script src="concept_card.js"></script>\n', encoding="utf-8"
    )
    return web


def _capture(web: Path) -> tuple[int, str]:
    import contextlib
    import io

    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        rc = evaluate(web)
    return rc, buf.getvalue()


def selftest_empty_units_is_red() -> None:
    """Known-bad: by_module populated, units=[], must be RED — never ok 0/0."""
    import tempfile

    m01 = [_unit(f"u01-{k}", 2) for k in range(4)]
    m06 = [_unit(f"u06-{k}", 2) for k in range(3)]
    payload = {
        "unit_count": 60,
        "by_module": {"01-mission-critical": m01, "06-power": m06},
        "units": [],
    }
    with tempfile.TemporaryDirectory(prefix="smoke_learn_v2_empty_units_") as td:
        rc, out = _capture(_plant_web(Path(td), payload))
    if rc == 0:
        raise AssertionError(f"empty units stayed GREEN:\n{out}")
    needle = "  FAIL: zero units in units_index.json (vacuous coverage is ERROR)\n"
    if needle not in out:
        raise AssertionError(f"missing vacuous-coverage fail:\n{out}")
    if "  ok: check_item_ids coverage 0/0\n" in out:
        raise AssertionError(f"vacuous ok 0/0 still printed:\n{out}")
    print("selftest_empty_units_is_red: RED ok")


def selftest_empty_m01_is_red() -> None:
    """Known-bad: M01 list empty, units otherwise populated — floor must not mask."""
    import tempfile

    m06 = [_unit(f"u06-{k}", 2) for k in range(3)]
    payload = {
        "unit_count": 60,
        "by_module": {"01-mission-critical": [], "06-power": m06},
        "units": m06,
    }
    with tempfile.TemporaryDirectory(prefix="smoke_learn_v2_empty_m01_") as td:
        rc, out = _capture(_plant_web(Path(td), payload))
    if rc == 0:
        raise AssertionError(f"empty M01 stayed GREEN:\n{out}")
    needle = (
        "  FAIL: zero M01 units in units_index.json "
        "(vacuous M01 check-item floor is ERROR)\n"
    )
    if needle not in out:
        raise AssertionError(f"missing vacuous-M01 fail:\n{out}")
    if "  ok: M01 every unit has ≥2 check items\n" in out:
        raise AssertionError(f"vacuous M01 ok still printed:\n{out}")
    print("selftest_empty_m01_is_red: RED ok")


def selftest_live_tree_is_green() -> None:
    rc, out = _capture(WEB)
    if rc != 0:
        raise AssertionError(f"live tree not GREEN:\n{out}")
    if "smoke_learn_v2: PASS" not in out:
        raise AssertionError(f"live tree missing PASS token:\n{out}")
    print("selftest_live_tree_is_green: GREEN ok")


def _assert_fail_row(rc: int, out: str, needle: str, *, continued: str | None = None) -> None:
    if rc == 0:
        raise AssertionError(f"known-bad stayed GREEN:\n{out}")
    if f"  FAIL: {needle}\n" not in out:
        raise AssertionError(f"missing FAIL row {needle!r}:\n{out}")
    if "smoke_learn_v2: FAIL" not in out:
        raise AssertionError(f"known-bad must reach the verdict line, not raise:\n{out}")
    if "smoke_learn_v2: PASS" in out:
        raise AssertionError(f"known-bad reached PASS:\n{out}")
    if continued is not None and continued not in out:
        raise AssertionError(f"grading after the defect was skipped:\n{out}")


def selftest_missing_m01_page_is_red() -> None:
    """Known-bad: missing Learn page is a FAIL row, not FileNotFoundError."""
    import tempfile

    with tempfile.TemporaryDirectory(prefix="smoke_learn_v2_missing_m01_") as td:
        web = _plant_web(Path(td), _good_payload())
        (web / "learn/01-mission-critical.html").unlink()
        rc, out = _capture(web)
    _assert_fail_row(
        rc, out, "missing learn/01-mission-critical.html",
        continued="  ok: drill concept_card\n",
    )
    for token in (
        "  ok: M01 learn-unit-shell\n",
        "  ok: M01 learn_units.js\n",
        "  ok: M01 learn_glossary.js\n",
    ):
        if token in out:
            raise AssertionError(f"M01 needle graded despite missing page:\n{out}")
    print("selftest_missing_m01_page_is_red: RED ok")


def selftest_missing_drill_is_red() -> None:
    """Known-bad: missing drill.html is a FAIL row, not FileNotFoundError."""
    import tempfile

    with tempfile.TemporaryDirectory(prefix="smoke_learn_v2_missing_drill_") as td:
        web = _plant_web(Path(td), _good_payload())
        (web / "drill.html").unlink()
        rc, out = _capture(web)
    _assert_fail_row(
        rc, out, "missing drill.html",
        continued="  ok: M01 learn_glossary.js\n",
    )
    if "  ok: drill concept_card\n" in out:
        raise AssertionError(f"drill graded despite missing page:\n{out}")
    print("selftest_missing_drill_is_red: RED ok")


def selftest_malformed_units_json_is_red() -> None:
    """Known-bad: unguarded json.loads on units_index must become a FAIL row."""
    import tempfile

    with tempfile.TemporaryDirectory(prefix="smoke_learn_v2_bad_units_json_") as td:
        web = _plant_web(Path(td), _good_payload())
        (web / "data/units_index.json").write_text('{"unit_count": }\n', encoding="utf-8")
        rc, out = _capture(web)
    _assert_fail_row(
        rc, out, "units_index.json is not valid JSON",
        continued="  ok: glossary terms=40\n",
    )
    if "  ok: unit_count=" in out:
        raise AssertionError(f"malformed units still graded:\n{out}")
    print("selftest_malformed_units_json_is_red: RED ok")


def selftest_malformed_glossary_json_is_red() -> None:
    """Known-bad: unguarded json.loads on glossary must become a FAIL row."""
    import tempfile

    with tempfile.TemporaryDirectory(prefix="smoke_learn_v2_bad_gloss_json_") as td:
        web = _plant_web(Path(td), _good_payload())
        (web / "data/glossary.json").write_text("not json at all\n", encoding="utf-8")
        rc, out = _capture(web)
    _assert_fail_row(
        rc, out, "glossary.json is not valid JSON",
        continued="  ok: unit_count=60\n",
    )
    if "  ok: glossary terms=" in out:
        raise AssertionError(f"malformed glossary still graded:\n{out}")
    print("selftest_malformed_glossary_json_is_red: RED ok")


def selftest_units_json_array_is_red() -> None:
    """Known-bad: a JSON array has no .get — grade it, do not AttributeError."""
    import tempfile

    with tempfile.TemporaryDirectory(prefix="smoke_learn_v2_units_array_") as td:
        web = _plant_web(Path(td), _good_payload())
        (web / "data/units_index.json").write_text("[1, 2, 3]\n", encoding="utf-8")
        rc, out = _capture(web)
    _assert_fail_row(
        rc, out, "units_index.json is not a JSON object",
        continued="  ok: glossary terms=40\n",
    )
    print("selftest_units_json_array_is_red: RED ok")


def selftest_undecodable_utf8_is_red() -> None:
    """Known-bad: every read_text(utf-8) site FAIL-closes, never UnicodeDecodeError."""
    import tempfile

    plants = (
        ("data/units_index.json", "units_index.json is not valid UTF-8", "  ok: glossary terms=40\n"),
        ("data/glossary.json", "glossary.json is not valid UTF-8", "  ok: unit_count=60\n"),
        (
            "learn/01-mission-critical.html",
            "learn/01-mission-critical.html is not valid UTF-8",
            "  ok: drill concept_card\n",
        ),
        ("drill.html", "drill.html is not valid UTF-8", "  ok: M01 learn_glossary.js\n"),
    )
    bad = bytes([ord("{"), 0x80, ord("}")])
    for rel, needle, continued in plants:
        with tempfile.TemporaryDirectory(prefix="smoke_learn_v2_bad_utf8_") as td:
            web = _plant_web(Path(td), _good_payload())
            (web / rel).write_bytes(bad)
            rc, out = _capture(web)
        _assert_fail_row(rc, out, needle, continued=continued)
    print("selftest_undecodable_utf8_is_red: RED ok")


def _selftest_known_bad() -> int:
    try:
        selftest_empty_units_is_red()
        selftest_empty_m01_is_red()
        selftest_missing_m01_page_is_red()
        selftest_missing_drill_is_red()
        selftest_malformed_units_json_is_red()
        selftest_malformed_glossary_json_is_red()
        selftest_units_json_array_is_red()
        selftest_undecodable_utf8_is_red()
        selftest_live_tree_is_green()
    except Exception as e:
        print(f"SELFTEST FAIL: {type(e).__name__}: {e}", file=sys.stderr)
        return 2
    print("smoke_learn_v2 --selftest: PASSED")
    return 0


if __name__ == "__main__":
    if "--selftest" in sys.argv:
        sys.exit(_selftest_known_bad())
    sys.exit(main())
