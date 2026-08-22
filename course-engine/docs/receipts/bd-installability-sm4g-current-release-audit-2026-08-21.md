# Installability P0 current-release audit — 2026-08-21

This is an acceptance measurement for `bd-installability-sm4g`, not a close. It
uses a clean detached worktree at the current committed source revision and
does not modify or publish the shared checkout.

## Pinned producer

- Source revision: `e0c333fa0d5b0abf17029ce0dab7e2d1801a565c`.
- Source tree: `5d40da762f4fb8099a76d64e2b256267c8ee81c0`.
- Target: `aarch64-apple-darwin`.
- Producer command: `cdcp release build --target aarch64-apple-darwin --out dist/acceptance-20260821`.
- Archive: one root-level regular `cdcp` member.
- Archive SHA-256: `aee96ea8f1a9840352ccb1086d0f8e402a1a8ccf8d56784aae7432b0f125918b`.
- `cdcp release verify`: PASS.
- Installer selftest: PASS, `INJECTIONS=5 SUITE=installer`.

## Stranger-path measurements

The archive was installed into an isolated temporary home. All subsequent
commands ran from a separate empty directory with `HOME` isolated and
`PATH=/usr/bin:/bin:/usr/sbin:/sbin` (no Cargo or Rust toolchain on `PATH`).

| Leg | Result | Evidence |
|---|---|---|
| Installed `cdcp --version` | PASS | rc=0, `0.1.0` |
| Installed `cdcp doctor --json` | PASS | rc=0, `ok=true`, installed layer, 4/4 probes |
| Installed `cdcp study --no-open --bind 127.0.0.1:0` | PASS | served `http://127.0.0.1:53619/`; `curl -fsS` rc=0 and returned the learner HTML |
| Remove installed `web/`, then `cdcp doctor --json` | RED as required | rc=4, `cdcp: bundle not found: /tmp/cdcp-missing-bundle.xji4wP/home/.local/share/cdcp/web` |
| Installed binary path disclosure | RED — acceptance criterion not met | `strings <installed cdcp> | grep -c '/Users/'` = `165` |

The first four rows establish the current binary/archive can install and serve
the learner surface from an empty working directory without a Rust toolchain on
the execution `PATH`, and that a missing bundle fails closed. The last row is a
real release defect: the current producer build does not satisfy the epic's
zero-local-path requirement. No attempt was made to hide or reinterpret it.

## Disposition

The epic remains **OPEN**. The stranger install/serve and missing-bundle legs
are now measured green at the current revision; the binary path-leak criterion
is red at 165 occurrences. This receipt does not claim a Rust-free operating
system or Windows parity, and it does not close the bead. The release/installer
source was not changed in this audit because that surface is owned by pane 3.

## Path-remap diagnosis

As a controlled follow-up, the same producer was run at the same source revision
with the inherited build flag
`RUSTFLAGS='--remap-path-prefix=/Users/josh='`.

- Remapped archive SHA-256: `f4322d779830723c1bb90807e32b911f06ae249e75fe34bc8a0bf8041f3dcd17`.
- `cdcp release verify`: PASS.
- Installer selftest: PASS, `INJECTIONS=5 SUITE=installer`.
- `strings <release cdcp> | grep -c '/Users/'`: `0`.
- `strings <release cdcp> | grep -c '/private/tmp/cdcp-installability-audit'`: `0`.

This isolates the 165-string failure to the producer's missing path-remap build
flag. The flag was supplied externally for diagnosis; the producer source was not
changed, so the epic remains open until the release command applies and verifies
the remap itself.
