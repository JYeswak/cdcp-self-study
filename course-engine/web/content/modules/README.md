# Shipped module notes

Markdown copies of parent-corpus notes (`cdcp-self-study/modules/*.md`),
generated from `knowledge/domains.toml` via:

```bash
# from course-engine/
python3 scripts/build_learn.py
cargo run -q -p cdcp_cli -- smoke-learn
```

Until this directory is populated, the Learn reader falls back to parent-corpus
relative paths when the static server’s CWD is the monorepo root
(`cdcp-self-study/`). Prefer shipping copies so `python3 -m http.server` from
`web/` alone is enough.
