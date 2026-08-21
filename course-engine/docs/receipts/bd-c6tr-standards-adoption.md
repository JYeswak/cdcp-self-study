# bd-c6tr — measured standards adoption audit

Date: 2026-08-20. Source payload: franken-harvest standards ledger installed
locally at `~/.local/bin/fh` (content id recorded on the bead). This receipt is
an audit of the two findings assigned to this bead; it does not claim that the
remaining standards beads are implemented.

## G1 — verdict stream contract

Finding: present, currently contained, not fixed.

`scripts/check.sh` centralizes failures in `fail()`. The function writes
`check.sh: FAIL: ...` to stderr. A stdout-only capture therefore contains the
ordinary progress/OK lines but no failure line. The local CI path is safe today:
it invokes diagnostic check with `>... 2>&1` and preserves the non-zero result in
its receipt. A future caller that redirects stdout only is not safe.

Known-bad reproduction of the exact stream failure:

```text
$ sh -c 'fail() { echo "check.sh: FAIL: planted" >&2; exit 2; }; fail planted' > /tmp/cdcp-stdout-only.log
$ rc=$?; printf 'exit=%s stdout_bytes=' "$rc"; wc -c < /tmp/cdcp-stdout-only.log
exit=2 stdout_bytes=0
```

The command fails, but the captured stdout transcript is empty and cannot show
the reason. This is a source-level reproduction of the contract, not a claim
that the full chain was run in this audit. The durable fix belongs with the
check.sh owner: either emit a machine-readable verdict on stdout as well, or
make stdout-only capture impossible. No check.sh change is made here because
pane 2 owns that path.

## G2 — input-set law

Finding: precondition present; no current violation found.

The live product scan domain is bounded by named paths such as:

| Consumer | Measured root(s) |
|---|---|
| bank verification/orphans/coverage | `bank/items`, `knowledge/topics.toml`, `knowledge/domains.toml`, `knowledge/bank_policy.toml` |
| grounding | `bank/items`, `knowledge`, `knowledge/corpus/public` |
| construction and duplicate checks | `bank/items` |
| pack freshness | `bank/items`, the three named `web/data/*_seed42*` artifacts |
| corpus rights | `knowledge/corpus/public`, `knowledge/corpus/free-pdfs` |
| learner/build paths | `web/content/modules`, `knowledge`, `bank/items` |
| registry checks | `tracks`, `scripts`, `crates/cdcp_gate/src/gates` and named registry/doc roots |

The scan code receives those bounded directories as arguments; no current
load-bearing scanner is configured to walk the repository root or an
untracked-state directory. The measured repository state was:

```text
target/                         35G
.beads/.br_history              210M
tracked product inputs          1,255 files
working product tree files      1,910 files
tracked target/.br_history      0 files
```

The large difference is precisely why the domain must remain explicit. This
receipt does not establish a permanent guard against a future gate adding a
repository-root walk. The acceptance work still needed is an in-tree test that
fails when a scan root resolves to the repository root or an untracked-state
directory, and reports the stated and actual scan counts for each scanner.

## Boundary and limitation

G1 currently detects ordinary check failures when invoked through the local-CI
path, but cannot protect an arbitrary stdout-only caller until its contract is
changed. G2 currently establishes bounded roots by inspection; it cannot prove
that future code will preserve those roots without the structural test described
above. A non-stale, bounded input set is not evidence that the scanned product
content is correct.
