# Local-CI scratch lifecycle (bd-qlxp)

`./scripts/check.sh` is the CI boundary; GitHub Actions is intentionally off.
Every probe that materialises a tree owns a child of:

```text
target/cdcp-scratch/<label>-<pid>-<attempt>/
```

Rust probes use `cdcp_registry_check::scratch::ScratchDir`, whose `Drop` removes
the owned child on ordinary return, early error, and panic unwinding. Shell
probes use the same namespace and an `EXIT INT TERM HUP` trap. A process killed
with `SIGKILL` cannot run a destructor or trap, so the preflight
`scripts/reap_scratch.sh --selftest` is the process-level backstop. It also
removes the legacy direct `target/*` probe trees.

The reaper is fail-closed: it reports the number discovered and removed,
raises an error if entries exist but discovery reports zero, and warns (without
turning the warning into a false green) when pre-reap usage exceeds the
50-GiB limit in `registries/scratch_lifecycle.toml`. It only removes children
of the named scratch root and disposable direct `target/` trees. It never
removes `target/debug`, `target/release`, or
`target/wasm32-unknown-unknown`; these are warm build artifacts.

Proof on 2026-08-19/20: the reaper found and removed 109 legacy trees using
76,649,324,544 bytes, warned above the 53,687,091,200-byte limit, removed a
planted tree, made the zero-find leg error, and preserved `target/debug`.
