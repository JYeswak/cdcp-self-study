# bd-c6tr — measured standards adoption audit

Date: 2026-08-20. Source payload: franken-harvest standards ledger installed
locally at `~/.local/bin/fh` (content id recorded on the bead). This receipt is
an audit of the two findings assigned to this bead; it does not claim that the
remaining standards beads are implemented.

## G1 — verdict stream contract

Finding: present and fixed in the local-CI path; the contract now also holds
for a caller that captures stdout without merging stderr.

`scripts/check.sh` centralizes failures in `fail()`. It now writes the same
machine-readable `check.sh: FAIL: ...` marker to stdout and stderr. The local CI
path remains safe because it captures both streams and preserves the non-zero
result in its receipt; a stdout-only caller now also sees the failure marker.

Known-bad reproduction of the exact stream failure, retained as a regression
leg:

```text
$ sh -c 'fail() { echo "check.sh: FAIL: planted" >&2; exit 2; }; fail planted' > /tmp/cdcp-stdout-only.log
$ rc=$?; printf 'exit=%s stdout_bytes=' "$rc"; wc -c < /tmp/cdcp-stdout-only.log
exit=2 stdout_bytes=0
```

The planted source specimen still demonstrates why the old contract was unsafe.
The in-tree regression test
`failure_visibility::check_failure_is_visible_on_stdout_and_stderr` runs
`scripts/check.sh --selftest-failure-visibility`, asserts exit 2, and requires
the marker in both captured streams. It passed with one test and no filtered
tests.

## G2 — input-set law

Finding: precondition present; the current declared domains are bounded and
non-empty, and the registry now makes that boundary executable.

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
untracked-state directory. `registries/scan_domains.toml` declares the product
domains; `cdcp_registry_check` rejects `.`/the engine root, `target/`, `.beads/`,
`.flywheel/`, `.git/`, parent escapes and symlink escapes, and prints the stated
root plus live file count for each row. At this revision it reported:

```text
bank-items=957 knowledge=47 web-data=10 web-content-modules=16 tracks=37
scripts=42 gate-dispatchers=26 bank-assertion-owners=18
registry-assertion-owners=11 learn-assertion-owners=17 docs=94 readme=1
```

The measured repository state was:

```text
target/                         35G
.beads/.br_history              210M
tracked product inputs          1,255 files
working product tree files      1,910 files
tracked target/.br_history      0 files
```

The large difference is precisely why the domain must remain explicit. The G2
tests cover the known-bad paths: repository root and untracked-state roots are
rejected, a symlink escaping the engine is rejected, blank/empty registries are
schema errors, and an empty declared domain is an ERROR rather than a pass. The
live registry test proves every declared domain is non-empty.

## Boundary and limitation

G1 proves stream visibility, not the substantive truth of any individual gate.
G2 proves the declared input domains are bounded and reports their live sizes;
it cannot automatically discover a future scanner whose code invents an
undeclared root, nor can it tell whether the bytes inside a bounded domain are
correct. A non-stale, bounded input set is not evidence that the scanned product
content is correct.
