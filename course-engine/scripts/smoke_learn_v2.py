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


def evaluate(web: Path) -> int:
    errs.clear()
    print("==> smoke_learn_v2 (M8 B/D assets)")
    units_path = web / "data/units_index.json"
    if not units_path.is_file():
        fail("missing units_index.json — run `cdcp build-units`")
    else:
        d = json.loads(units_path.read_text(encoding="utf-8"))
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
        g = json.loads(gloss.read_text(encoding="utf-8"))
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

    m01 = (web / "learn/01-mission-critical.html").read_text(encoding="utf-8")
    for needle in ("learn-unit-shell", "learn_units.js", "learn_glossary.js"):
        if needle not in m01:
            fail(f"M01 missing {needle}")
        else:
            ok(f"M01 {needle}")

    drill = (web / "drill.html").read_text(encoding="utf-8")
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


def _selftest_known_bad() -> int:
    try:
        selftest_empty_units_is_red()
        selftest_empty_m01_is_red()
        selftest_live_tree_is_green()
    except AssertionError as e:
        print(f"SELFTEST FAIL: {e}", file=sys.stderr)
        return 2
    print("smoke_learn_v2 --selftest: PASSED")
    return 0


if __name__ == "__main__":
    if "--selftest" in sys.argv:
        sys.exit(_selftest_known_bad())
    sys.exit(main())
