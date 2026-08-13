#!/usr/bin/env python3
"""build_reference.py — ship parent reference docs into offline web surface.

Reads parent corpus:
  ../reference/GLOSSARY.md
  ../reference/POWER-AND-REDUNDANCY-CHEATSHEET.md

Emits:
  web/content/reference/GLOSSARY.md
  web/content/reference/POWER-AND-REDUNDANCY-CHEATSHEET.md
  web/reference.html

Markdown is loaded via relative fetch (same static-server model as Learn).
Parent-corpus paths in the shipped copies are rewritten to in-app hrefs.

Re-run after parent reference edits:

  python3 scripts/build_reference.py
"""
from __future__ import annotations

import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PARENT = ROOT.parent
SRC_DIR = PARENT / "reference"
WEB = ROOT / "web"
CONTENT_DIR = WEB / "content" / "reference"
OUT_HTML = WEB / "reference.html"

DOCS = (
    {
        "id": "glossary",
        "title": "Glossary",
        "src_name": "GLOSSARY.md",
        "dest_name": "GLOSSARY.md",
    },
    {
        "id": "power",
        "title": "Power & redundancy",
        "src_name": "POWER-AND-REDUNDANCY-CHEATSHEET.md",
        "dest_name": "POWER-AND-REDUNDANCY-CHEATSHEET.md",
    },
)

HONESTY = (
    "<strong>Study tool only.</strong>\n"
    "      This tool does <strong>not</strong> grant EPI/EXIN certification.\n"
    "      Completing practice here is not a CDCP credential."
)

# Parent markdown → in-app paths (order matters for overlapping prefixes).
LINK_REWRITES: list[tuple[str, str]] = [
    ("../practice/DRILL-CARDS.md", "drill.html"),
    ("../practice/PRACTICE-EXAM.md", "mock.html"),
    ("../modules/06-power.md", "learn/06-power.html"),
    ("../modules/09-cooling.md", "learn/09-cooling.html"),
    ("./GLOSSARY.md", "#glossary"),
    ("GLOSSARY.md", "#glossary"),
    (
        "./POWER-AND-REDUNDANCY-CHEATSHEET.md",
        "#power",
    ),
    (
        "POWER-AND-REDUNDANCY-CHEATSHEET.md",
        "#power",
    ),
]


def rewrite_links(text: str) -> str:
    out = text
    for old, new in LINK_REWRITES:
        # Markdown links: ](old) and ](old "title")
        out = out.replace(f"]({old})", f"]({new})")
        out = re.sub(
            rf"\]\({re.escape(old)}\s+",
            f"]({new} ",
            out,
        )
    return out


def copy_docs() -> list[dict]:
    if not SRC_DIR.is_dir():
        raise SystemExit(f"FAIL: missing parent reference dir {SRC_DIR}")

    CONTENT_DIR.mkdir(parents=True, exist_ok=True)
    shipped: list[dict] = []
    for doc in DOCS:
        src = SRC_DIR / doc["src_name"]
        if not src.is_file():
            raise SystemExit(f"FAIL: missing {src}")
        raw = src.read_text(encoding="utf-8")
        text = rewrite_links(raw)
        dest = CONTENT_DIR / doc["dest_name"]
        dest.write_text(text, encoding="utf-8", newline="\n")
        shipped.append(
            {
                **doc,
                "content_path": f"content/reference/{doc['dest_name']}",
                "bytes": dest.stat().st_size,
            }
        )
    # Drop orphaned copies (only known docs)
    keep = {d["dest_name"] for d in DOCS}
    for stale in CONTENT_DIR.glob("*.md"):
        if stale.name not in keep:
            stale.unlink()
    return shipped


def render_page() -> str:
    tab_buttons: list[str] = []
    panels: list[str] = []
    for i, doc in enumerate(DOCS):
        selected = "true" if i == 0 else "false"
        hidden = "" if i == 0 else " hidden"
        tab_buttons.append(
            f'        <button type="button" class="ref-tabs__btn" role="tab"\n'
            f'          id="tab-{doc["id"]}" data-ref-id="{doc["id"]}"\n'
            f'          aria-controls="panel-{doc["id"]}" aria-selected="{selected}">\n'
            f'          {doc["title"]}\n'
            f"        </button>"
        )
        panels.append(
            f'      <section class="ref-panel" role="tabpanel"\n'
            f'        id="panel-{doc["id"]}" data-ref-id="{doc["id"]}"\n'
            f'        aria-labelledby="tab-{doc["id"]}"{hidden}\n'
            f'        data-content-href="content/reference/{doc["dest_name"]}">\n'
            f'        <article class="prose" id="prose-{doc["id"]}" aria-busy="true">\n'
            f'          <p class="lede">Loading {doc["title"].lower()}…</p>\n'
            f"        </article>\n"
            f"      </section>"
        )

    tabs_html = "\n".join(tab_buttons)
    panels_html = "\n".join(panels)

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="CDCP reference — glossary and power/redundancy cheatsheet. Study tool only; does not grant EPI/EXIN certification.">
  <title>CDCP Study — Reference</title>
  <link rel="stylesheet" href="assets/css/course.css">
</head>
<body>
  <a class="skip-link" href="#main">Skip to main content</a>

  <header class="site-header">
    <div class="honesty-banner" role="status">
      {HONESTY}
    </div>
    <div class="site-header__inner">
      <a class="brand" href="index.html">
        <span class="brand__title">CDCP Study</span>
        <span class="brand__sub">Self-study · offline</span>
      </a>
      <nav aria-label="Hub">
        <ul class="hub-nav">
          <li><a href="learn.html">Learn</a></li>
          <li><a href="drill.html">Drill</a></li>
          <li><a href="mock.html">Mock</a></li>
          <li><a href="reference.html" aria-current="page">Reference</a></li>
        </ul>
      </nav>
    </div>
  </header>

  <main id="main" class="wrap wrap-learn" tabindex="-1">
    <h1>Reference</h1>
    <p class="lede">
      Offline glossary and power/redundancy cheatsheet from the parent study corpus.
      Interview-ready shorthand only — not an official EPI dictionary and not a design manual.
    </p>

    <div class="ref-tabs" role="tablist" aria-label="Reference documents">
{tabs_html}
    </div>

{panels_html}

    <p class="meta">
      <a href="index.html">← Hub</a>
      · <a href="learn.html">Learn</a>
      · <a href="learn/06-power.html">Module 06 power</a>
      · source <span class="mono">web/content/reference/</span>
      · study notes only · not an EPI/EXIN credential.
    </p>
  </main>

  <script src="assets/js/learn_md.js"></script>
  <script src="assets/js/reference.js"></script>
</body>
</html>
"""


def main() -> int:
    shipped = copy_docs()
    OUT_HTML.write_text(render_page(), encoding="utf-8", newline="\n")

    # Lightweight self-check (no network)
    html = OUT_HTML.read_text(encoding="utf-8")
    errors: list[str] = []
    if "does <strong>not</strong> grant EPI/EXIN certification" not in html:
        errors.append("reference.html missing honesty non-grant language")
    if 'href="assets/css/course.css"' not in html:
        errors.append("reference.html missing course.css")
    if "cdn." in html.lower() or "https://cdn" in html.lower():
        errors.append("reference.html must not pull a CDN")
    for doc in shipped:
        if f'content/reference/{doc["dest_name"]}' not in html:
            errors.append(f"reference.html missing panel for {doc['dest_name']}")
        if not (CONTENT_DIR / doc["dest_name"]).is_file():
            errors.append(f"missing content copy {doc['dest_name']}")

    if errors:
        print("FAIL: build_reference")
        for e in errors:
            print(f"  - {e}")
        return 1

    print("PASS: build_reference")
    for doc in shipped:
        print(f"  {doc['id']}: {doc['content_path']} ({doc['bytes']} bytes)")
    print(f"  page={OUT_HTML.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
