# Contributing

Thanks for interest in the CDCP course-engine study tool.

## Before you start

1. Read `README.md` (start path + honesty banners) and `LICENSE` (code is MIT; curriculum content is CC BY-NC-SA 4.0).
2. Read `course-engine/AGENTS.md` / `CHARTER.md` if changing gates or claims.
3. Run `cd course-engine && ./scripts/check.sh` and keep W0–L7 + V11 + M8 green.

## Local setup

```bash
# Rust toolchain (stable)
cargo build -p cdcp_cli

# Serve static web + WASM (use a free port)
cargo run -p cdcp_cli -- serve --bind 127.0.0.1:8766
```

Do not open `web/` as `file://` when testing quiz/WASM.

## What we welcome

- Bug fixes with a failing smoke or test when practical
- Learn/unit content improvements grounded in free/public sources
- Accessibility and UX polish that preserve honesty copy
- Diagrams registered in `docs/DIAGRAM-REGISTRY.md` with `smoke_diagrams`

## What we do not accept

- Exam dumps or copyrighted paid SDO PDF bodies
- Claims that this tool certifies anyone as CDCP
- Secrets, PII, or private client context
- Weakening `check.sh` gates or known-bad selftests without a tracked reason

## PR hygiene

- Conventional commits preferred: `feat(scope): …`, `fix(scope): …`
- One logical change per PR
- Update `docs/FEATURE_SURFACE.md` when adding a product surface
- New present diagrams must pass `cdcp smoke-diagrams`

## Code of conduct

See `CODE_OF_CONDUCT.md`.
