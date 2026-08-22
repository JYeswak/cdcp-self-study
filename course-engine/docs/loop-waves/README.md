# Loop-skill research waves

Repeatable host: `cargo run -p cdcp_loop_wave`.

```sh
cargo run --locked -p cdcp_loop_wave -- \
  --root . \
  --harvest ~/.claude/references/franken-harvest.md \
  --out docs/loop-waves

cargo run --locked -p cdcp_loop_wave -- \
  --root . \
  --harvest ~/.claude/references/franken-harvest.md \
  --prior docs/loop-waves/wave-1.json \
  --out docs/loop-waves
```

Wave 2 must pass `--prior` so it is not a blank-slate rerun.
Harvest missing or empty → exit 4 (fail closed).
Do not edit other panes' dirty `cdcp_gate` / `.flywheel/ALERT` / `web/learn.html`.
`bd-pqi3` is a human design fork — this runner grades the gap; it does not invent the commit choke.
