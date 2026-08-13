# Security Policy

## Supported versions

This repository is a **local-first study tool**. There is no multi-tenant
production SaaS surface in-tree. Security reports still matter for:

- accidental secrets in git history
- XSS or path issues in static web assets
- supply-chain issues in Rust/WASM build scripts

## Reporting a vulnerability

**Do not open a public GitHub issue for security reports.**

Email the maintainer privately (operator contact on file with the repo owner —
Joshua Nowak / ZestStream). Include:

1. Affected path or crate
2. Reproduction steps (minimal)
3. Impact assessment (local only vs network-facing if served)

You should receive an acknowledgment within a few business days when the inbox
is monitored.

## Secrets and study data

- Never commit API keys, Infisical exports, or private PDF corpus beyond the
  free/public allowlist (see `docs/OQ_REGISTER.md` OQ-09 / OQ-10).
- Paid SDO standards are **out of tree** by policy.
- Bank items are original study MCQs — not exam dumps.

## Honesty

This tool does **not** grant EPI/EXIN CDCP certification. Security hygiene here
does not imply a certified exam product.
