#!/usr/bin/env python3
"""build_learn.py — export curriculum modules into offline static Learn surface.

Reads knowledge/domains.toml (paths relative to course-engine ROOT → parent
corpus ../modules/*.md). Emits:

  web/data/modules_index.json   machine index for hub + smoke
  web/data/topic_anchors.json   topic_id → learn section anchors (L7-S2)
  web/content/modules/{id}.md   shipped markdown copies
  web/learn/{id}.html           offline readers (fetch content; optional embed)
  web/learn.html                hub list

Empty primary_notes is allowed only when exam_weight_unknown is true
(ops-adjacent). Those domains appear on the hub without a 404 link.

Markdown is loaded from web/content/modules/ via relative fetch (same local
static-server model as mock). Re-run after corpus changes:

  python3 scripts/build_learn.py
  cargo run -q -p cdcp_cli -- smoke-learn
  cargo run -q -p cdcp_cli -- smoke-feedback-links
"""
from __future__ import annotations

import html
import json
import re
import shutil
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
DOMAINS = ROOT / "knowledge" / "domains.toml"
TOPICS = ROOT / "knowledge" / "topics.toml"
WEB = ROOT / "web"
CONTENT_DIR = WEB / "content" / "modules"
LEARN_DIR = WEB / "learn"
INDEX_JSON = WEB / "data" / "modules_index.json"
TOPIC_ANCHORS_JSON = WEB / "data" / "topic_anchors.json"

# Stopwords ignored when fuzzy-matching topic labels to headings.
_STOP = frozenset(
    {
        "a",
        "an",
        "and",
        "as",
        "at",
        "for",
        "in",
        "of",
        "on",
        "or",
        "the",
        "to",
        "vs",
        "with",
    }
)

HONESTY = (
    "<strong>Study tool only.</strong>\n"
    "      This tool does <strong>not</strong> grant EPI/EXIN certification.\n"
    "      Completing practice here is not a CDCP credential."
)


def _header(depth: int = 0) -> str:
    prefix = "../" if depth else ""
    return f"""  <header class="site-header">
    <div class="honesty-banner" role="status">
      {HONESTY}
    </div>
    <div class="site-header__inner">
      <a class="brand" href="{prefix}index.html">
        <span class="brand__title">CDCP Study</span>
        <span class="brand__sub">Self-study · offline</span>
      </a>
      <nav aria-label="Hub">
        <ul class="hub-nav">
          <li><a href="{prefix}learn.html" aria-current="page">Learn</a></li>
          <li><a href="{prefix}drill.html">Drill</a></li>
          <li><a href="{prefix}mock.html">Mock</a></li>
          <li><a href="{prefix}reference.html">Reference</a></li>
        </ul>
      </nav>
    </div>
  </header>"""


def render_module_page(
    *,
    mod_id: str,
    title: str,
    order: int,
    prev_mod: dict | None,
    next_mod: dict | None,
) -> str:
    prev_link = ""
    next_link = ""
    if prev_mod:
        prev_link = (
            f'<a class="mod-nav__link" href="{html.escape(prev_mod["id"])}.html">'
            f'← {int(prev_mod["order"]):02d}. '
            f'{html.escape(prev_mod["epi_heading"])}</a>'
        )
    if next_mod:
        next_link = (
            f'<a class="mod-nav__link" href="{html.escape(next_mod["id"])}.html">'
            f'{int(next_mod["order"]):02d}. '
            f'{html.escape(next_mod["epi_heading"])} →</a>'
        )

    diagram_cta = ""
    if order == 6 or mod_id == "06-power":
        diagram_cta = """
        <aside class="diagram-cta" aria-label="Interactive diagram">
          <p class="diagram-cta__tag mono">DIAGRAM</p>
          <h2 class="diagram-cta__title">Power path N vs 2N</h2>
          <p class="diagram-cta__body">Interactive label quiz for single-path vs dual-path topology.
          Interview one-liner: dual cords only protect you if upstream paths are independent.</p>
          <p><a class="diagram-cta__link" href="../diagrams/power-path.html">Open power-path self-check →</a></p>
        </aside>"""
    elif order == 1 or mod_id == "01-mission-critical":
        diagram_cta = """
        <aside class="diagram-cta" aria-label="Interactive diagram">
          <p class="diagram-cta__tag mono">DIAGRAM</p>
          <h2 class="diagram-cta__title">Site dependency stack</h2>
          <p class="diagram-cta__body">Click layers from business impact down to MEP.
          Interview one-liner: white space is not enough — availability is manufactured in grey space.</p>
          <p><a class="diagram-cta__link" href="../diagrams/site-stack.html">Open site-stack →</a></p>
        </aside>"""
    elif order == 9 or mod_id == "09-cooling":
        diagram_cta = """
        <aside class="diagram-cta" aria-label="Interactive diagram">
          <p class="diagram-cta__tag mono">DIAGRAM</p>
          <h2 class="diagram-cta__title">Heat path chip → outdoors</h2>
          <p class="diagram-cta__body">Stepper: IT load → rack → room → plant → outdoors.
          Interview one-liner: every watt to IT becomes heat that must leave the building.</p>
          <p><a class="diagram-cta__link" href="../diagrams/heat-path.html">Open heat-path →</a></p>
        </aside>"""

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="CDCP Learn — {html.escape(title)}. Does not grant EPI/EXIN certification.">
  <title>CDCP Study — {html.escape(title)}</title>
  <link rel="stylesheet" href="../assets/css/course.css">
</head>
<body data-module-id="{html.escape(mod_id)}">
  <a class="skip-link" href="#main">Skip to main content</a>
  <div id="learn-progress-bar" class="learn-progress-bar" role="progressbar"
       aria-valuemin="0" aria-valuemax="100" aria-valuenow="0" aria-label="Reading progress">
    <div class="learn-progress-bar__fill"></div>
    <span class="learn-progress-bar__label mono">0%</span>
  </div>

{_header(1)}

  <main id="main" class="wrap wrap-learn" tabindex="-1">
    <p class="breadcrumb">
      <a href="../learn.html">Learn</a>
      <span aria-hidden="true"> / </span>
      <span>Module {order:02d}</span>
    </p>
    <noscript>
      <p class="lede">JavaScript is required to render module markdown. Source:
      <span class="mono">web/content/modules/{html.escape(mod_id)}.md</span>.</p>
    </noscript>
    <div class="learn-layout">
      <nav id="learn-toc" class="learn-toc" aria-label="On this page" hidden></nav>
      <div class="learn-layout__main">
        <div id="learn-unit-shell" class="learn-unit-shell" hidden>
          <p class="learn-unit-shell__status mono"></p>
          <h2 class="learn-unit-shell__title"></h2>
          <div class="learn-unit-shell__controls">
            <button type="button" data-unit-prev>← Prev unit</button>
            <button type="button" data-unit-next>Next unit →</button>
            <button type="button" data-unit-full>Full article</button>
            <button type="button" data-unit-mode>Unit mode</button>
          </div>
          <p class="meta">Unit mode shows one section + quick check. Full article = entire module. Study signals only.</p>
        </div>
        <article
          class="prose"
          id="module-prose"
          data-module-id="{html.escape(mod_id)}"
          data-content-href="../content/modules/{html.escape(mod_id)}.md"
          aria-busy="true"
        >
          <p class="lede">Loading module…</p>
        </article>
        <div id="learn-unit-check" class="unit-check" hidden></div>
{diagram_cta}
        <nav class="mod-nav" aria-label="Module sequence">
          <div class="mod-nav__prev">{prev_link}</div>
          <div class="mod-nav__hub"><a href="../learn.html">All modules</a></div>
          <div class="mod-nav__next">{next_link}</div>
        </nav>
        <p class="meta">
          <a href="../quiz.html?module={order}&amp;mode=learn15">Learn-15 (5 check Q)</a>
          · <a href="../quiz.html?module={order}">Module {order:02d} quiz (8–12)</a>
          · study notes only · not an EPI/EXIN credential · progress in this browser.
        </p>
      </div>
    </div>
  </main>
  <script src="../assets/js/learn_md.js"></script>
  <script src="../assets/js/learn_progress.js"></script>
  <script src="../assets/js/learn_chrome.js"></script>
  <script src="../assets/js/learn_units.js"></script>
  <script src="../assets/js/learn_glossary.js"></script>
  <script src="../assets/js/learn_reader.js"></script>
  <script>
    if (window.CdcpLearnReader) {{
      CdcpLearnReader.loadAndRender({json.dumps(mod_id)});
    }}
    document.addEventListener("DOMContentLoaded", function () {{
      /* units/glossary mount after async render via learn_reader hooks */
    }});
  </script>
</body>
</html>
"""


def render_hub(modules: list[dict]) -> str:
    rows: list[str] = []
    for m in modules:
        order = int(m["order"])
        heading = html.escape(m["epi_heading"])
        mid = html.escape(m["id"])
        if m.get("empty"):
            rows.append(
                f"""      <li class="module-list__item module-list__item--empty" data-module-id="{mid}">
        <span class="module-list__order mono">{order:02d}</span>
        <span class="module-list__body">
          <span class="module-list__title">{heading}</span>
          <span class="module-list__status">Notes not shipped (exam weight unknown)</span>
        </span>
        <span class="module-list__badge" data-progress-for="{mid}" hidden>Visited</span>
      </li>"""
            )
        else:
            href = html.escape(m["href"])
            rows.append(
                f"""      <li class="module-list__item" data-module-id="{mid}">
        <a class="module-list__link" href="{href}">
          <span class="module-list__order mono">{order:02d}</span>
          <span class="module-list__body">
            <span class="module-list__title">{heading}</span>
            <span class="module-list__status mono">{mid}</span>
          </span>
          <span class="module-list__badge" data-progress-for="{mid}" hidden>Visited</span>
          <span class="module-list__mastery" data-mastery-for="{order}" hidden></span>
        </a>
      </li>"""
            )

    list_html = "\n".join(rows)
    slim = [
        {
            "id": m["id"],
            "order": m["order"],
            "epi_heading": m["epi_heading"],
            "empty": bool(m.get("empty")),
            "href": m.get("href"),
            "word_count": m.get("word_count") or 0,
            "estimate_minutes": m.get("estimate_minutes") or 0,
        }
        for m in modules
    ]
    index_embed = json.dumps({"schema_version": 1, "modules": slim}, indent=2)

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="CDCP Learn — study modules. Does not grant EPI/EXIN certification.">
  <title>CDCP Study — Learn</title>
  <link rel="stylesheet" href="assets/css/course.css">
</head>
<body>
  <a class="skip-link" href="#main">Skip to main content</a>

{_header(0)}

  <main id="main" class="wrap" tabindex="-1">
    <h1>Learn</h1>
    <p class="lede">
      Fourteen EPI CDCP curriculum domains plus partner ops expansions.
      Open a module to study offline. Progress is stored only in this browser.
      Completing modules here is a study signal — not a CDCP credential.
    </p>

    <p class="learn-progress-summary" id="learn-progress-summary" aria-live="polite"></p>

    <p id="learn-continue" class="learn-continue" hidden>
      <a class="learn-continue__link" href="#">Continue</a>
      <span class="meta"> · last module in this browser (study signal only)</span>
    </p>

    <ol class="module-list" id="module-list">
{list_html}
    </ol>

    <p class="meta">
      <a href="index.html">← Hub</a>
      · <a href="quiz.html">Module quiz</a>
      · <a href="drill.html">Drill</a>
      · Generated from <span class="mono">knowledge/domains.toml</span>
    </p>
  </main>

  <script type="application/json" id="modules-index">
{index_embed}
  </script>
  <script src="assets/js/learn_progress.js"></script>
  <script src="assets/js/learn_chrome.js"></script>
  <script type="module" src="assets/js/hub_mastery.js"></script>
  <script>
    if (window.CdcpLearn) {{
      CdcpLearn.paintHub();
    }}
    if (window.CdcpLearnChrome) {{
      CdcpLearnChrome.loadHubExtras();
    }}
  </script>
</body>
</html>
"""


def load_domains() -> list[dict]:
    if not DOMAINS.is_file():
        raise SystemExit(f"FAIL: missing {DOMAINS}")
    data = tomllib.loads(DOMAINS.read_text(encoding="utf-8"))
    domains = data.get("domain") or []
    if not domains:
        raise SystemExit("FAIL: domains.toml has zero [[domain]] rows")
    return sorted(domains, key=lambda d: int(d.get("order") or 0))


def slugify_heading(text: str) -> str:
    """Stable heading slug — must match learn_md.js CdcpLearnMd.slugify."""
    s = str(text or "").lower()
    s = re.sub(r"[*_`]", "", s)
    s = re.sub(r"[^\w\s-]", "", s, flags=re.UNICODE)
    s = s.strip()
    s = re.sub(r"[\s_]+", "-", s)
    s = re.sub(r"-+", "-", s).strip("-")
    return s or "section"


def extract_headings(md_text: str) -> list[dict]:
    """Parse ATX ## / ### (and #…######) headings with collision-safe ids."""
    used: dict[str, int] = {}
    out: list[dict] = []
    in_fence = False
    for raw in md_text.replace("\r\n", "\n").replace("\r", "\n").split("\n"):
        line = raw.rstrip()
        stripped = line.strip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        m = re.match(r"^(#{1,6})\s+(.*)$", stripped)
        if not m:
            continue
        level = len(m.group(1))
        title = re.sub(r"\s+#*\s*$", "", m.group(2)).strip()
        plain = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", title)
        plain = re.sub(r"[*_`]", "", plain)
        base = slugify_heading(plain)
        if base not in used:
            used[base] = 1
            hid = base
        else:
            n = used[base] + 1
            while f"{base}-{n}" in used:
                n += 1
            used[base] = n
            used[f"{base}-{n}"] = 1
            hid = f"{base}-{n}"
        out.append({"level": level, "text": title, "id": hid})
    return out


def _significant_words(text: str) -> list[str]:
    words = re.findall(r"[a-z0-9]+", text.lower())
    return [w for w in words if w not in _STOP and len(w) > 1]


def match_topic_to_heading(
    label: str, topic_id: str, headings: list[dict]
) -> str | None:
    """Best-effort map topic label/id → heading id. None if no credible match."""
    if not headings:
        return None
    # Prefer h2/h3 for section anchors (L7-S2); fall back to any level.
    section_heads = [h for h in headings if h["level"] in (2, 3)] or headings
    label_slug = slugify_heading(label)
    by_id = {h["id"]: h for h in section_heads}

    # 1) exact slug match
    if label_slug in by_id:
        return label_slug

    # 2) exact casefold text match
    label_cf = label.casefold().strip()
    for h in section_heads:
        if h["text"].casefold().strip() == label_cf:
            return h["id"]

    # 3) topic id tail as token (m06-transformers → transformers)
    tail = topic_id.split("-", 1)[-1] if topic_id.startswith("m") else topic_id
    tail_tokens = [t for t in re.split(r"[-_]", tail) if t and t not in _STOP]
    # Prefer longer, more specific headings that contain all tail tokens.
    candidates: list[tuple[int, int, str]] = []  # (score, -len, id)
    label_words = _significant_words(label)

    for h in section_heads:
        hid = h["id"]
        htext = h["text"].casefold()
        hslug = hid
        score = 0
        if label_slug and (label_slug in hslug or hslug in label_slug):
            score = max(score, 80 if label_slug == hslug else 55)
        if label_words:
            hits = sum(1 for w in label_words if w in htext or w in hslug)
            if hits == len(label_words) and hits > 0:
                score = max(score, 70 + min(hits, 5))
            elif hits >= max(2, (len(label_words) + 1) // 2):
                score = max(score, 40 + hits)
        if tail_tokens:
            t_hits = sum(1 for t in tail_tokens if t in hslug or t in htext)
            if t_hits == len(tail_tokens) and t_hits > 0:
                score = max(score, 60 + t_hits)
            elif t_hits >= 1 and len(tail_tokens) == 1:
                # single-token topic tail (e.g. transformers) is strong
                score = max(score, 50)
        if score > 0:
            # Prefer h3 over h2 slightly for depth; shorter slug as tie-break.
            level_bonus = 1 if h["level"] == 3 else 0
            candidates.append((score + level_bonus, -len(hid), hid))

    if not candidates:
        return None
    candidates.sort(reverse=True)
    best_score, _, best_id = candidates[0]
    # Threshold: avoid weak noise matches
    if best_score < 50:
        return None
    return best_id


def build_topic_anchors(navigable: list[dict]) -> dict:
    """topic_id → learn href fragment map for results feedback links."""
    topics: list[dict] = []
    if TOPICS.is_file():
        tdata = tomllib.loads(TOPICS.read_text(encoding="utf-8"))
        topics = list(tdata.get("topic") or [])

    domain_by_id = {m["id"]: m for m in navigable}
    headings_by_domain: dict[str, list[dict]] = {}
    for m in navigable:
        md_path = CONTENT_DIR / f"{m['id']}.md"
        if md_path.is_file():
            headings_by_domain[m["id"]] = extract_headings(
                md_path.read_text(encoding="utf-8")
            )
        else:
            headings_by_domain[m["id"]] = []

    topic_map: dict[str, dict] = {}
    matched = 0
    for t in topics:
        tid = str(t.get("id") or "").strip()
        if not tid:
            continue
        domain = str(t.get("domain") or "").strip()
        label = str(t.get("label") or tid).strip()
        mod = domain_by_id.get(domain)
        if not mod:
            topic_map[tid] = {
                "topic_id": tid,
                "domain": domain,
                "label": label,
                "module": None,
                "slug": None,
                "anchor": None,
                "href": None,
            }
            continue
        order = int(mod["order"])
        slug = mod["id"]
        heads = headings_by_domain.get(domain) or []
        anchor = match_topic_to_heading(label, tid, heads)
        href = f"learn/{slug}.html"
        if anchor:
            href = f"{href}#{anchor}"
            matched += 1
        topic_map[tid] = {
            "topic_id": tid,
            "domain": domain,
            "label": label,
            "module": order,
            "slug": slug,
            "anchor": anchor,
            "href": href,
        }

    return {
        "schema_version": 1,
        "generated_by": "scripts/build_learn.py",
        "slug_algorithm": "learn_md.js CdcpLearnMd.slugify / slugify_heading",
        "topic_count": len(topic_map),
        "topics_with_anchor": matched,
        "modules": {
            m["id"]: {
                "order": m["order"],
                "headings": [
                    h
                    for h in headings_by_domain.get(m["id"], [])
                    if h["level"] in (2, 3)
                ],
            }
            for m in navigable
        },
        "topics": topic_map,
    }


def main() -> int:
    domains = load_domains()
    CONTENT_DIR.mkdir(parents=True, exist_ok=True)
    LEARN_DIR.mkdir(parents=True, exist_ok=True)
    INDEX_JSON.parent.mkdir(parents=True, exist_ok=True)

    modules: list[dict] = []
    errors: list[str] = []

    for dom in domains:
        did = str(dom.get("id") or "").strip()
        if not did:
            errors.append("domain missing id")
            continue
        order = int(dom.get("order") or 0)
        heading = str(dom.get("epi_heading") or did)
        pn = dom.get("primary_notes")
        pn_s = "" if pn is None else str(pn).strip()
        exam_unknown = dom.get("exam_weight_unknown") is True

        entry: dict = {
            "id": did,
            "order": order,
            "epi_heading": heading,
            "primary_notes": pn_s,
            "exam_weight_unknown": exam_unknown,
        }

        if not pn_s:
            if not exam_unknown:
                errors.append(
                    f"{did}: empty primary_notes without exam_weight_unknown=true"
                )
                continue
            entry["empty"] = True
            entry["href"] = None
            entry["content_path"] = None
            modules.append(entry)
            continue

        src = Path(pn_s)
        if not src.is_absolute():
            src = (ROOT / pn_s).resolve()
        if not src.is_file():
            errors.append(f"{did}: primary_notes missing: {pn_s} → {src}")
            continue

        dest_md = CONTENT_DIR / f"{did}.md"
        shutil.copyfile(src, dest_md)
        text = dest_md.read_text(encoding="utf-8")
        words = len(re.findall(r"\b\w+\b", text))
        # ~200 wpm + 35% drill buffer; clamp 15–55 min (M8-A4)
        eta = max(15, min(55, round(words / 200 * 1.35)))

        entry["empty"] = False
        entry["href"] = f"learn/{did}.html"
        entry["content_path"] = f"content/modules/{did}.md"
        entry["source_path"] = pn_s
        entry["word_count"] = words
        entry["estimate_minutes"] = eta
        modules.append(entry)

    if errors:
        print("FAIL: build_learn")
        for e in errors:
            print(f"  - {e}")
        return 1

    navigable = [m for m in modules if not m.get("empty")]
    for idx, m in enumerate(navigable):
        prev_m = navigable[idx - 1] if idx > 0 else None
        next_m = navigable[idx + 1] if idx + 1 < len(navigable) else None
        page = render_module_page(
            mod_id=m["id"],
            title=m["epi_heading"],
            order=m["order"],
            prev_mod=prev_m,
            next_mod=next_m,
        )
        (LEARN_DIR / f"{m['id']}.html").write_text(page, encoding="utf-8", newline="\n")

    keep = {f"{m['id']}.html" for m in navigable}
    for stale in LEARN_DIR.glob("*.html"):
        if stale.name not in keep:
            stale.unlink()

    # Drop orphaned content copies.
    # README.md is hand-written documentation for this directory, not a generated
    # copy — it is tracked in git and must survive the sweep. Without this guard
    # every build_learn.py run silently deletes it (observed 2026-08-15 while
    # adding module 15); the deletion then rides along in whatever commit follows.
    keep_md = {f"{m['id']}.md" for m in navigable} | {"README.md"}
    for stale in CONTENT_DIR.glob("*.md"):
        if stale.name not in keep_md:
            stale.unlink()

    index = {
        "schema_version": 1,
        "generated_by": "scripts/build_learn.py",
        "module_count": len(modules),
        "navigable_count": len(navigable),
        "empty_ok_count": sum(1 for m in modules if m.get("empty")),
        "modules": modules,
    }
    INDEX_JSON.write_text(
        json.dumps(index, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )

    topic_anchors = build_topic_anchors(navigable)
    TOPIC_ANCHORS_JSON.write_text(
        json.dumps(topic_anchors, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )

    (WEB / "learn.html").write_text(render_hub(modules), encoding="utf-8", newline="\n")

    print("PASS: build_learn")
    print(
        f"  modules={len(modules)} navigable={len(navigable)} "
        f"empty_ok={index['empty_ok_count']}"
    )
    print(f"  index={INDEX_JSON.relative_to(ROOT)}")
    print(f"  topic_anchors={TOPIC_ANCHORS_JSON.relative_to(ROOT)}")
    print(
        f"  topics_with_anchor={topic_anchors['topics_with_anchor']}/"
        f"{topic_anchors['topic_count']}"
    )
    print(f"  learn_pages={LEARN_DIR.relative_to(ROOT)}/")
    print(f"  content={CONTENT_DIR.relative_to(ROOT)}/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
