#!/usr/bin/env python3
"""smoke_learn.py — every domains.toml primary_notes path resolves or is empty-ok;

Also checks the offline Learn surface artifacts:
  - web/data/modules_index.json present and consistent with domains.toml
  - each non-empty domain has web/learn/{id}.html
  - content copy web/content/modules/{id}.md OR primary_notes source resolves
    (build_learn.py ships the copies; reader also falls back to parent corpus)
  - empty-ok domain (ops-adjacent) is listed without a href (no 404)
  - learn hub + module pages carry honesty banner non-grant language
  - hrefs are relative (no absolute http(s) asset roots in shells)

Exit 0 PASS · non-zero FAIL.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
DOMAINS = ROOT / "knowledge" / "domains.toml"
WEB = ROOT / "web"
INDEX_JSON = WEB / "data" / "modules_index.json"
LEARN_HUB = WEB / "learn.html"
LEARN_DIR = WEB / "learn"
CONTENT_DIR = WEB / "content" / "modules"

HONESTY_RE = re.compile(
    r"does.*not.*grant EPI/EXIN certification|does <strong>not</strong> grant EPI/EXIN certification",
    re.I,
)


def main() -> int:
    errors: list[str] = []

    if not DOMAINS.is_file():
        print("FAIL: knowledge/domains.toml missing")
        return 1

    data = tomllib.loads(DOMAINS.read_text(encoding="utf-8"))
    domains = data.get("domain") or []
    if not domains:
        errors.append("domains.toml has zero [[domain]] rows")

    # --- domain path resolution (mirrors verify_knowledge_paths) ---
    checked = 0
    empty_ok = 0
    domain_by_id: dict[str, dict] = {}
    for dom in domains:
        did = dom.get("id") or "<missing-id>"
        domain_by_id[str(did)] = dom
        pn = dom.get("primary_notes")
        if pn is None:
            errors.append(f"{did}: primary_notes field missing")
            continue
        pn_s = str(pn).strip()
        if not pn_s:
            if dom.get("exam_weight_unknown") is True:
                empty_ok += 1
                continue
            errors.append(
                f"{did}: empty primary_notes without exam_weight_unknown=true"
            )
            continue
        candidate = Path(pn_s)
        if not candidate.is_absolute():
            candidate = (ROOT / pn_s).resolve()
        checked += 1
        if not candidate.is_file():
            errors.append(
                f"{did}: primary_notes does not resolve: {pn_s!r} → {candidate}"
            )

    # --- index + artifacts ---
    if not INDEX_JSON.is_file():
        errors.append(f"missing {INDEX_JSON.relative_to(ROOT)} — run scripts/build_learn.py")
        index = None
    else:
        try:
            index = json.loads(INDEX_JSON.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            errors.append(f"modules_index.json invalid JSON: {e}")
            index = None

    if not LEARN_HUB.is_file():
        errors.append("missing web/learn.html")

    if index is not None:
        mods = index.get("modules") or []
        if not mods:
            errors.append("modules_index.json has zero modules")
        index_ids = {m.get("id") for m in mods}
        domain_ids = set(domain_by_id.keys())
        if index_ids != domain_ids:
            missing = domain_ids - index_ids
            extra = index_ids - domain_ids
            if missing:
                errors.append(f"index missing domain ids: {sorted(missing)}")
            if extra:
                errors.append(f"index has unknown domain ids: {sorted(extra)}")

        for m in mods:
            mid = m.get("id") or "<missing>"
            empty = m.get("empty") is True
            if empty:
                if m.get("href") not in (None, ""):
                    errors.append(f"{mid}: empty-ok domain must not have href (got {m.get('href')!r})")
                # must not produce a learn page that 404s as navigable
                if (LEARN_DIR / f"{mid}.html").is_file():
                    # allowed only if we never link to it; still warn as unexpected
                    pass
                continue

            href = m.get("href") or ""
            if not href:
                errors.append(f"{mid}: navigable module missing href")
            elif href.startswith(("http://", "https://", "/")):
                errors.append(f"{mid}: href must be relative offline path, got {href!r}")
            elif not href.startswith("learn/") or not href.endswith(".html"):
                errors.append(f"{mid}: unexpected href shape {href!r}")

            page = LEARN_DIR / f"{mid}.html"
            if not page.is_file():
                errors.append(f"{mid}: missing learn page {page.relative_to(ROOT)}")
            else:
                text = page.read_text(encoding="utf-8")
                if not HONESTY_RE.search(text):
                    errors.append(f"{mid}: learn page missing honesty non-grant banner")
                # relative assets only (css/js)
                if re.search(r'href=["\']https?://', text) and "rel=\"noopener" not in text:
                    # external content links may exist in notes; shell assets must be relative
                    pass
                if not re.search(r'href=["\']\.\./assets/css/course\.css["\']', text):
                    errors.append(f"{mid}: learn page css must be relative ../assets/css/course.css")
                if 'src="../assets/js/learn_progress.js"' not in text:
                    errors.append(f"{mid}: learn page must load relative learn_progress.js")
                if 'src="../assets/js/learn_md.js"' not in text:
                    errors.append(f"{mid}: learn page must load relative learn_md.js")
                if 'src="../assets/js/learn_reader.js"' not in text and 'id="module-md"' not in text:
                    errors.append(
                        f"{mid}: learn page must load learn_reader.js or embed #module-md"
                    )
                has_embed = 'id="module-md"' in text
                has_fetch = f"content/modules/{mid}.md" in text
                if not has_embed and not has_fetch:
                    errors.append(
                        f"{mid}: learn page must embed #module-md or fetch content/modules/{mid}.md"
                    )

            content = CONTENT_DIR / f"{mid}.md"
            # Content copies are produced by build_learn.py. If missing, the
            # parent corpus primary_notes path must still resolve (checked
            # above) so a rebuild can ship copies; learn_reader.js also
            # falls back to ../../../modules/{id}.md when the monorepo root
            # is the static-server CWD.
            if content.is_file() and content.stat().st_size < 32:
                errors.append(f"{mid}: content copy is empty/tiny: {content.relative_to(ROOT)}")
            elif not content.is_file():
                pn = str((domain_by_id.get(mid) or {}).get("primary_notes") or "").strip()
                src = (ROOT / pn).resolve() if pn else None
                if not src or not src.is_file():
                    errors.append(
                        f"{mid}: missing content copy and primary_notes source "
                        f"(run scripts/build_learn.py)"
                    )


    if LEARN_HUB.is_file():
        hub = LEARN_HUB.read_text(encoding="utf-8")
        if not HONESTY_RE.search(hub):
            errors.append("web/learn.html missing honesty non-grant banner")
        if 'href="assets/css/course.css"' not in hub and "href='assets/css/course.css'" not in hub:
            errors.append("web/learn.html css must be relative assets/css/course.css")
        # An empty-ok domain has no page, so linking to it would 404. Derived
        # from the index rather than hardcoded to one module id: 15-ops-adjacent
        # was the only empty-ok domain until it was taught (bd-hardening-c-status-hzs.4),
        # and a gate naming one id cannot notice the next one.
        empty_ids = sorted(
            str(m.get("id"))
            for m in ((index or {}).get("modules") or [])
            if m.get("empty") is True
        )
        for eid in empty_ids:
            if re.search(rf'href=["\']learn/{re.escape(eid)}\.html["\']', hub):
                errors.append(f"hub must not link to empty-ok module page {eid}")
        # navigable modules should appear
        for mid, dom in domain_by_id.items():
            pn = str(dom.get("primary_notes") or "").strip()
            if not pn:
                continue
            if f"learn/{mid}.html" not in hub and f'data-module-id="{mid}"' not in hub:
                errors.append(f"hub does not list navigable module {mid}")

    if errors:
        print("FAIL: smoke_learn")
        for e in errors:
            print(f"  - {e}")
        return 1

    print("PASS: smoke_learn")
    print(f"  primary_notes_checked={checked}")
    print(f"  empty_allowed={empty_ok}")
    if index is not None:
        print(f"  index_modules={len(index.get('modules') or [])}")
        print(f"  navigable={index.get('navigable_count')}")
    print(f"  hub={LEARN_HUB.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
