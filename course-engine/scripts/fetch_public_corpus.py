#!/usr/bin/env python3
"""fetch_public_corpus.py — snapshot free/public sources for grounding.

Only fetches sources with access in {public_summary, free, local}.
Writes knowledge/corpus/public/<source_id>.txt + manifest.json.
Does NOT fetch access=paid.
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
import urllib.request
from datetime import date
from html.parser import HTMLParser
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore

ROOT = Path(__file__).resolve().parents[1]
SOURCES = ROOT / "knowledge" / "sources.toml"
OUT_DIR = ROOT / "knowledge" / "corpus" / "public"
MANIFEST = OUT_DIR / "manifest.json"

ALLOWED_ACCESS = frozenset({"public_summary", "free", "local"})


class TextExtractor(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self._chunks: list[str] = []
        self._skip = 0

    def handle_starttag(self, tag: str, attrs) -> None:
        if tag in {"script", "style", "noscript"}:
            self._skip += 1

    def handle_endtag(self, tag: str) -> None:
        if tag in {"script", "style", "noscript"} and self._skip:
            self._skip -= 1

    def handle_data(self, data: str) -> None:
        if self._skip:
            return
        t = data.strip()
        if t:
            self._chunks.append(t)

    def text(self) -> str:
        return "\n".join(self._chunks)


def load_sources() -> list[dict]:
    data = tomllib.loads(SOURCES.read_text(encoding="utf-8"))
    # tomllib may not parse [[source]] as list named source — use regex fallback
    text = SOURCES.read_text(encoding="utf-8")
    blocks = re.split(r"\n\[\[source\]\]\n", text)
    out: list[dict] = []
    for b in blocks[1:]:
        d: dict = {}
        for key in ("id", "org", "title", "url", "access", "fetch_date", "note"):
            m = re.search(rf'(?m)^{key}\s*=\s*"(.*)"\s*$', b)
            if m:
                d[key] = m.group(1)
        if d.get("id") and d.get("url"):
            out.append(d)
    return out


def fetch_url(url: str, timeout: int = 25) -> bytes:
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "cdcp-course-grounding/0.1 (educational; local)"},
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read()


def html_to_text(raw: bytes) -> str:
    try:
        html = raw.decode("utf-8", errors="replace")
    except Exception:
        html = raw.decode("latin-1", errors="replace")
    p = TextExtractor()
    try:
        p.feed(html)
        return p.text()
    except Exception:
        # strip tags crudely
        return re.sub(r"<[^>]+>", " ", html)


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    sources = load_sources()
    manifest: list[dict] = []
    ok = 0
    skip = 0
    fail = 0

    for s in sources:
        access = s.get("access", "")
        if access not in ALLOWED_ACCESS:
            skip += 1
            continue
        sid = s["id"]
        url = s["url"]
        if url.startswith("file://"):
            # local path relative to knowledge or repo
            rel = url.replace("file://", "")
            path = (ROOT / rel).resolve()
            if not path.is_file():
                # try parent study root
                path = (ROOT.parent / rel.lstrip("./")).resolve()
            if path.is_file():
                text = path.read_text(encoding="utf-8", errors="replace")
                raw = text.encode("utf-8")
            else:
                print(f"skip local missing {sid}: {url}")
                skip += 1
                continue
        else:
            try:
                raw = fetch_url(url)
                if url.lower().endswith(".pdf"):
                    # store metadata only — avoid large binary / license issues
                    text = (
                        f"[PDF not extracted in-repo]\nurl={url}\n"
                        f"title={s.get('title','')}\n"
                        f"note=Use link-only for grounding; binary not stored.\n"
                    )
                    raw = text.encode("utf-8")
                else:
                    text = html_to_text(raw)
            except Exception as e:
                print(f"FAIL fetch {sid}: {e}")
                fail += 1
                continue

        out_path = OUT_DIR / f"{sid}.txt"
        # normalize whitespace lightly
        body = re.sub(r"\n{3,}", "\n\n", text)
        header = (
            f"# source_id: {sid}\n"
            f"# url: {url}\n"
            f"# access: {access}\n"
            f"# fetched: {date.today().isoformat()}\n"
            f"# title: {s.get('title','')}\n\n"
        )
        out_path.write_text(header + body, encoding="utf-8")
        digest = hashlib.sha256(out_path.read_bytes()).hexdigest()
        manifest.append(
            {
                "id": sid,
                "url": url,
                "access": access,
                "path": str(out_path.relative_to(ROOT)),
                "sha256": digest,
                "bytes": out_path.stat().st_size,
            }
        )
        ok += 1
        print(f"ok {sid} ({out_path.stat().st_size} bytes)")

    MANIFEST.write_text(
        json.dumps(
            {"schema": "cdcp.corpus.manifest.v1", "date": date.today().isoformat(), "sources": manifest},
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    print(f"done ok={ok} skip={skip} fail={fail} manifest={MANIFEST}")
    # fail only if zero free/public snapshots
    if ok == 0:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
