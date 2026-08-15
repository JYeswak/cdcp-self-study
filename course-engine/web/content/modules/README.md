# Shipped module notes

Markdown copies of parent-corpus notes (`cdcp-self-study/modules/*.md`),
generated from `knowledge/domains.toml` via:

```bash
# from course-engine/
cargo build -p cdcp_cli --locked
./target/debug/cdcp build-learn
./target/debug/cdcp smoke-learn
```

Until this directory is populated, the Learn reader falls back to parent-corpus
relative paths when the static server’s CWD is the monorepo root
(`cdcp-self-study/`). Prefer shipping copies so `./target/debug/cdcp serve`
from `course-engine/` is enough.
