#!/usr/bin/env python3
"""smoke_learn_chrome.py — M8 Wave A learn chrome (TOC/math/continue/power embed).

Checks static artifacts exist and key strings are wired. Does not drive a browser.
Exit 0 = pass; non-zero = fail.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WEB = ROOT / "web"
errs: list[str] = []


def ok(msg: str) -> None:
    print(f"  ok: {msg}")


def fail(msg: str) -> None:
    errs.append(msg)
    print(f"  FAIL: {msg}")


def main() -> int:
    print("==> smoke_learn_chrome (M8-A)")

    chrome = WEB / "assets/js/learn_chrome.js"
    mdjs = WEB / "assets/js/learn_md.js"
    if not chrome.is_file():
        fail("missing learn_chrome.js")
    else:
        ok("learn_chrome.js")
    if not mdjs.is_file():
        fail("missing learn_md.js")
    else:
        text = mdjs.read_text(encoding="utf-8")
        if "latexToHtml" not in text or "math-block" not in text:
            fail("learn_md.js missing latex/math-block path")
        else:
            ok("learn_md formula path")

    # Module 01 page has TOC host + progress bar + chrome script
    m01 = WEB / "learn/01-mission-critical.html"
    if not m01.is_file():
        fail("missing learn/01-mission-critical.html — run build_learn.py")
    else:
        h = m01.read_text(encoding="utf-8")
        for needle in (
            'id="learn-toc"',
            'id="learn-progress-bar"',
            "learn_chrome.js",
        ):
            if needle not in h:
                fail(f"M01 page missing {needle}")
            else:
                ok(f"M01 has {needle}")

    # M06 embeds power-path CTA
    m06 = WEB / "learn/06-power.html"
    if not m06.is_file():
        fail("missing learn/06-power.html")
    else:
        h = m06.read_text(encoding="utf-8")
        if "diagrams/power-path.html" not in h:
            fail("M06 missing power-path diagram CTA")
        else:
            ok("M06 power-path CTA")
        if "diagram-cta" not in h:
            fail("M06 missing diagram-cta class")
        else:
            ok("M06 diagram-cta")

    # Learn hub continue + chrome
    hub = WEB / "learn.html"
    if not hub.is_file():
        fail("missing learn.html")
    else:
        h = hub.read_text(encoding="utf-8")
        if 'id="learn-continue"' not in h:
            fail("learn.html missing #learn-continue")
        else:
            ok("learn.html continue chip host")
        if "learn_chrome.js" not in h:
            fail("learn.html missing learn_chrome.js")
        else:
            ok("learn.html learn_chrome.js")

    # modules_index estimates
    idx = WEB / "data/modules_index.json"
    if not idx.is_file():
        fail("missing modules_index.json")
    else:
        import json

        data = json.loads(idx.read_text(encoding="utf-8"))
        mods = [m for m in data.get("modules") or [] if not m.get("empty")]
        if not mods:
            fail("no navigable modules in index")
        else:
            missing_eta = [
                m["id"]
                for m in mods
                if not m.get("estimate_minutes") and not m.get("word_count")
            ]
            if missing_eta:
                fail(f"modules missing word_count/eta: {missing_eta[:3]}")
            else:
                ok(f"modules_index eta fields on {len(mods)} modules")

    # power-path still present
    pp = WEB / "diagrams/power-path.html"
    if not pp.is_file():
        fail("missing diagrams/power-path.html")
    else:
        ok("power-path diagram file")

    # CSS hooks
    css = (WEB / "assets/css/course.css").read_text(encoding="utf-8")
    for cls in (".learn-toc", ".math-block", ".learn-continue", ".diagram-cta"):
        if cls not in css:
            fail(f"course.css missing {cls}")
        else:
            ok(f"css {cls}")

    if errs:
        print(f"smoke_learn_chrome: FAIL ({len(errs)} errors)")
        return 1
    print("smoke_learn_chrome: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
