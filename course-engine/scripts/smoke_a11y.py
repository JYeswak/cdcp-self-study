#!/usr/bin/env python3
"""smoke_a11y.py — L7-S5: a11y baseline smoke for primary product pages.

Asserts for each primary HTML page under web/ (index, mock, results, learn,
drill, quiz; reference only when present):
  1. skip link (class=skip-link or Skip-to-main pattern)
  2. honesty banner (class=honesty-banner) OR meta honesty non-grant language
  3. main / content landmark (<main> or role=main)
  4. course.css stylesheet link

Also asserts course.css carries:
  - :focus-visible rule(s)
  - --touch-min design token

Exit 0 PASS · non-zero FAIL.
Zero pages found / missing required page / empty CSS = ERROR (no vacuous green).
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WEB = ROOT / "web"
CSS = WEB / "assets" / "css" / "course.css"

# Required primary shells (must exist).
REQUIRED_PAGES: tuple[str, ...] = (
    "index.html",
    "mock.html",
    "results.html",
    "learn.html",
    "drill.html",
    "quiz.html",
)

# Optional primary shells — checked only when present (L7-S4 reference panel).
OPTIONAL_PAGES: tuple[str, ...] = (
    "reference.html",
)

SKIP_RE = re.compile(
    r'class=["\'][^"\']*\bskip-link\b[^"\']*["\']'
    r'|href=["\']#main["\'][^>]*>\s*Skip\b'
    r'|>\s*Skip to (main )?content\s*<',
    re.I,
)

HONESTY_BANNER_RE = re.compile(
    r'class=["\'][^"\']*\bhonesty-banner\b[^"\']*["\']',
    re.I,
)

# Non-grant honesty language (banner body or meta description).
HONESTY_COPY_RE = re.compile(
    r"does\s+(?:<strong>)?not(?:</strong>)?\s+grant\s+EPI/EXIN\s+certification"
    r"|not\s+EPI/EXIN\s+certification"
    r"|study\s+(?:tool|signal)\s+only",
    re.I,
)

# Meta description that itself carries honesty language.
HONESTY_META_RE = re.compile(
    r'<meta[^>]+name=["\'](?:description|honesty)["\'][^>]*'
    r'content=["\'][^"\']*'
    r'(?:does\s+not\s+grant|not\s+EPI/EXIN|study\s+(?:tool|signal)\s+only)',
    re.I,
)

# Main/content landmark — require real landmark element or role (not id alone).
MAIN_LANDMARK_RE = re.compile(
    r"<main\b"
    r'|role=["\']main["\']',
    re.I,
)

COURSE_CSS_RE = re.compile(
    r"""href=["'][^"']*assets/css/course\.css["']""",
    re.I,
)

FOCUS_VISIBLE_RE = re.compile(r":focus-visible\b")
TOUCH_MIN_RE = re.compile(r"--touch-min\b")


def check_page(rel: str, text: str) -> list[str]:
    errors: list[str] = []

    if not SKIP_RE.search(text):
        errors.append(f"{rel}: missing skip link (.skip-link or Skip to main content)")

    has_banner = bool(HONESTY_BANNER_RE.search(text))
    has_meta = bool(HONESTY_META_RE.search(text))
    has_copy = bool(HONESTY_COPY_RE.search(text))
    if not (has_banner or has_meta or has_copy):
        errors.append(
            f"{rel}: missing honesty banner (.honesty-banner) and meta honesty language"
        )
    elif has_banner and not has_copy and not has_meta:
        # Banner class without non-grant language is a hollow shell.
        errors.append(
            f"{rel}: honesty-banner present but no non-grant / meta honesty language"
        )

    if not MAIN_LANDMARK_RE.search(text):
        if re.search(r'id=["\']main["\']', text, re.I):
            errors.append(
                f"{rel}: #main present but missing landmark element (<main> or role=main)"
            )
        else:
            errors.append(f"{rel}: missing main/content landmark (<main> or role=main)")

    if not COURSE_CSS_RE.search(text):
        errors.append(f"{rel}: missing course.css stylesheet link")

    return errors


def check_css(text: str) -> list[str]:
    errors: list[str] = []
    if not FOCUS_VISIBLE_RE.search(text):
        errors.append("course.css: missing :focus-visible rule")
    if not TOUCH_MIN_RE.search(text):
        errors.append("course.css: missing --touch-min token")
    return errors


def main() -> int:
    errors: list[str] = []
    pages_checked = 0

    if not WEB.is_dir():
        print("FAIL: smoke_a11y — missing web/")
        return 1

    # --- CSS baseline ---
    if not CSS.is_file():
        errors.append(f"missing {CSS.relative_to(ROOT)}")
    else:
        css_text = CSS.read_text(encoding="utf-8")
        if not css_text.strip():
            errors.append("course.css is empty — refusing vacuous green")
        else:
            errors.extend(check_css(css_text))

    # --- Required primary pages ---
    for name in REQUIRED_PAGES:
        path = WEB / name
        rel = f"web/{name}"
        if not path.is_file():
            errors.append(f"missing required primary page {rel}")
            continue
        text = path.read_text(encoding="utf-8")
        if not text.strip():
            errors.append(f"{rel}: empty file — refusing vacuous green")
            continue
        pages_checked += 1
        errors.extend(check_page(rel, text))

    # --- Optional primary pages (only when present) ---
    optional_checked = 0
    for name in OPTIONAL_PAGES:
        path = WEB / name
        if not path.is_file():
            continue
        rel = f"web/{name}"
        text = path.read_text(encoding="utf-8")
        if not text.strip():
            errors.append(f"{rel}: empty file — refusing vacuous green")
            continue
        pages_checked += 1
        optional_checked += 1
        errors.extend(check_page(rel, text))

    # Anti-vacuous: zero pages checked is always an error.
    if pages_checked == 0:
        errors.append(
            "zero primary HTML pages checked — refusing vacuous green "
            f"(expected at least {len(REQUIRED_PAGES)} required pages)"
        )

    if errors:
        print("FAIL: smoke_a11y")
        for e in errors:
            print(f"  - {e}")
        return 1

    print("PASS: smoke_a11y")
    print(f"  pages_checked={pages_checked}")
    print(f"  required={len(REQUIRED_PAGES)}")
    print(f"  optional_present={optional_checked}")
    print(f"  css={CSS.relative_to(ROOT)}")
    print("  checks=skip-link · honesty · main landmark · course.css · :focus-visible · --touch-min")
    for name in REQUIRED_PAGES:
        print(f"  ok web/{name}")
    for name in OPTIONAL_PAGES:
        if (WEB / name).is_file():
            print(f"  ok web/{name} (optional)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
