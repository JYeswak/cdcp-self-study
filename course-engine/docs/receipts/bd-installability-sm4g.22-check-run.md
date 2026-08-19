# bd-installability-sm4g.22 — timed local check

## Run evidence

- Tree: `56b82bf`
- Command: `PATH=/Users/josh/.cargo/bin:$PATH /usr/bin/time -p ./scripts/check.sh`
- Environment: local warm Cargo registry/target cache
- Result: no successful full-chain completion; the invocation ran to its first fail and exited 2
- Wall time: `30.45` seconds (`0.51` minutes)
- Interpretation: this is a measured warm-cache floor, not a CI duration estimate

The run reached `near-duplicate-items` and stopped there. The later `cdcp_anki` export failure and stale `EXPECTED_APPROVED_LIVE=879` versus live `931` were not reached in this invocation; they are pane 2's current downstream work and were not changed here.

## Ordered steps and verdicts

1. `check.sh lock selftest` — PASS
2. `cdcp-course check` / knowledge scaffold — PASS
3. `cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked` — PASS
4. `check.sh snapshot selftest` — PASS
5. `cdcp_registry_check` / registry-check — PASS
6. `cdcp check-licence` / licence split — PASS
7. `cargo test -p cdcp_data --test corpus_rights` — PASS
8. `cdcp corpus-rights` — PASS
9. `cdcp load-snapshots` — PASS
10. `cdcp check-osha` — PASS
11. `cdcp verify-data-lock` — PASS
12. `cdcp verify-data-lock --selftest` — PASS
13. `cdcp oracle-check` — PASS
14. stale-plant detector — PASS
15. `cdcp_gate substrate-guard` — PASS
16. `cdcp_gate substrate-guard --prove-wired` — PASS
17. `cdcp_gate install-hooks` — PASS
18. `cdcp_gate install-hooks --check` — PASS
19. `cdcp_gate capability-maturity` — PASS
20. `cdcp_gate goldens-couplings` — PASS
21. `cdcp_gate doc-facts` — PASS
22. exam-form public CDCP format pins — PASS
23. honesty string smoke — PASS
24. standards crosswalk — PASS
25. `topics.toml` count — PASS
26. source `fetch_date` presence — PASS
27. `cdcp_gate verify-bank` — PASS
28. `cdcp_gate answer-key-skew` — PASS
29. `cdcp_gate validate-grounding` — PASS
30. `cdcp_gate grounding-wave` — PASS
31. `cdcp_gate verify-orphans` — PASS
32. `selftest_orphan.sh` — PASS
33. `cdcp_gate near-duplicate-items` — FAIL

Steps after 33 were not executed because `check.sh` fails closed at the first failing step.

## First failure

Exact gate output:

```text
near-duplicate-items: FAIL: bank-m14-q133 (m14-q133.toml) <-> m14-q208 (m14-q208.toml) — answer 72% · distractors 4% · stem 11% [shared-key-text] — review: same proposition twice?
near-duplicate-items: FAIL: bank-m14-q133 (m14-q133.toml) <-> m15-q219 (m15-q219.toml) — answer 87% · distractors 4% · stem 5% [shared-key-text] — review: same proposition twice?
near-duplicate-items: FAIL: bank-m15-q149 (m15-q149.toml) <-> m01-q044 (m01-q044.toml) — answer 100% · distractors 0% · stem 0% [shared-key-text] — review: same proposition twice?
near-duplicate-items: FAIL: m01-q041 (m01-q041.toml) <-> m01-q200 (m01-q200.toml) — answer 83% · distractors 2% · stem 33% [shared-key-text] — review: same proposition twice?
near-duplicate-items: FAIL: m01-q056 (m01-q056.toml) <-> m06-q101 (m06-q101.toml) — answer 100% · distractors 13% · stem 0% [shared-key-text] — review: same proposition twice?
near-duplicate-items: FAIL: m14-q208 (m14-q208.toml) <-> m15-q219 (m15-q219.toml) — answer 72% · distractors 6% · stem 4% [shared-key-text] — review: same proposition twice?
near-duplicate-items: FAIL: m15-q356 (m15-q356.toml) <-> m15-q383 (m15-q383.toml) — answer 63% · distractors 25% · stem 23% [shared-key-text] — review: same proposition twice?
near-duplicate-items: FAIL: 7 violation(s)
check.sh: FAIL: near-duplicate items in the approved pool
```

## Evidence-based option

Pick: **(b) split a fast PR subset from the full scheduled chain**.

The warm-cache chain reached its first correctness failure in 30.45 seconds, so this run does not evidence a local throughput problem. A fast PR subset would provide timely change feedback while the full scheduled chain retains the complete correctness and product-surface coverage. Option (a) may reduce cold-run compile cost, but the measured floor is already short and does not establish that Cargo startup is the limiting factor. Options (c) and (d) change queue pressure without addressing the observed correctness stop. This is a recommendation only; no CI configuration was changed.

## Run 2 — after choice restoration

- Tree: `123affa`
- Command: `PATH=/Users/josh/.cargo/bin:$PATH /usr/bin/time -p ./scripts/check.sh`
- Environment: local warm Cargo registry/target cache
- Result: no successful full-chain completion; the invocation reached step 36 and exited 2
- Wall time: `51.23` seconds (`0.85` minutes)
- Interpretation: this is a measured warm-cache floor to the next correctness failure, not a complete-run time or CI duration estimate

This run passed the step-33 near-duplicate gate, its known-bad selftest, and paraphrase-pair verification. It failed at the formatting substep before clippy/test. The pane-2 formatting drift was not changed here.

## Run 2 ordered steps and verdicts

1. `check.sh lock selftest` — PASS
2. `cdcp-course check` / knowledge scaffold — PASS
3. `cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked` — PASS
4. `check.sh snapshot selftest` — PASS
5. `cdcp_registry_check` / registry-check — PASS
6. `cdcp check-licence` / licence split — PASS
7. `cargo test -p cdcp_data --test corpus_rights` — PASS
8. `cdcp corpus-rights` — PASS
9. `cdcp load-snapshots` — PASS
10. `cdcp check-osha` — PASS
11. `cdcp verify-data-lock` — PASS
12. `cdcp verify-data-lock --selftest` — PASS
13. `cdcp oracle-check` — PASS
14. stale-plant detector — PASS
15. `cdcp_gate substrate-guard` — PASS
16. `cdcp_gate substrate-guard --prove-wired` — PASS
17. `cdcp_gate install-hooks` — PASS
18. `cdcp_gate install-hooks --check` — PASS
19. `cdcp_gate capability-maturity` — PASS
20. `cdcp_gate goldens-couplings` — PASS
21. `cdcp_gate doc-facts` — PASS
22. exam-form public CDCP format pins — PASS
23. honesty string smoke — PASS
24. standards crosswalk — PASS
25. `topics.toml` count — PASS
26. source `fetch_date` presence — PASS
27. `cdcp_gate verify-bank` — PASS
28. `cdcp_gate answer-key-skew` — PASS
29. `cdcp_gate validate-grounding` — PASS
30. `cdcp_gate grounding-wave` — PASS
31. `cdcp_gate verify-orphans` — PASS
32. `selftest_orphan.sh` — PASS
33. `cdcp_gate near-duplicate-items` — PASS
34. near-duplicate-items selftest — PASS
35. `cdcp verify-paraphrase-pairs` — PASS
36. `cargo fmt/clippy/test` — FAIL at `cargo fmt`

Steps after 36 were not executed because `check.sh` fails closed at the formatting step.

## Run 2 first failure

`cargo fmt` reported formatting diffs at these exact locations:

```text
Diff in /Users/josh/cdcp-self-study/course-engine/crates/cdcp_bank/src/validate_grounding.rs:239
Diff in /Users/josh/cdcp-self-study/course-engine/crates/cdcp_bank/src/validate_grounding.rs:255
Diff in /Users/josh/cdcp-self-study/course-engine/crates/cdcp_bank/src/verify_bank.rs:514
Diff in /Users/josh/cdcp-self-study/course-engine/crates/cdcp_bank/src/verify_coverage.rs:490
Diff in /Users/josh/cdcp-self-study/course-engine/crates/cdcp_gate/tests/diff_validate_grounding.rs:609
Diff in /Users/josh/cdcp-self-study/course-engine/crates/cdcp_gate/tests/diff_verify_bank.rs:1282
Diff in /Users/josh/cdcp-self-study/course-engine/crates/cdcp_gate/tests/diff_verify_coverage.rs:1216
Diff in /Users/josh/cdcp-self-study/course-engine/crates/cdcp_gate/tests/diff_verify_coverage.rs:1260
check.sh: FAIL: cargo fmt
```

## Run 2 option revisit

The new measurement is `51.23` seconds to a later correctness failure, versus `30.45` seconds to the earlier near-duplicate failure. There is still no successful complete run, so the throughput question remains unanswered and this number must not be treated as a full-chain runtime. The earlier option (b)—a fast PR subset plus the full scheduled chain—remains a provisional feedback-flow recommendation, not a measured throughput fix. There is not enough evidence to elevate option (a) caching, or any other option, as the highest-leverage throughput fix until the chain completes successfully. No CI configuration was changed.

## Run 3 — after formatter fix

- Tree: `9954b37`
- Command: `PATH=/Users/josh/.cargo/bin:$PATH /usr/bin/time -p ./scripts/check.sh`
- Environment: local warm Cargo registry/target cache
- Result: no successful full-chain completion; the invocation reached step 16 and exited 2
- Wall time: `19.34` seconds (`0.32` minutes)
- Interpretation: this is a measured warm-cache floor to an index/probe correctness failure, not a complete-run time or CI duration estimate

The six-file formatter fix was committed and `cargo fmt --all -- --check` passed. Run 3 passed the ordinary substrate scan, but its nested prove-wired check materialised the committed index snapshot and failed the gate-shrink ceiling before the planted substrate file could be evaluated. The top-level worktree registry check was green at `37272/37275`; the nested index snapshot measured `cdcp_gate=37280/37275`.

## Run 3 ordered steps and verdicts

1. `check.sh lock selftest` — PASS
2. `cdcp-course check` / knowledge scaffold — PASS
3. `cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked` — PASS
4. `check.sh snapshot selftest` — PASS
5. `cdcp_registry_check` / registry-check — PASS (`37272/37275` in the worktree)
6. `cdcp check-licence` / licence split — PASS
7. `cargo test -p cdcp_data --test corpus_rights` — PASS
8. `cdcp corpus-rights` — PASS
9. `cdcp load-snapshots` — PASS
10. `cdcp check-osha` — PASS
11. `cdcp verify-data-lock` — PASS
12. `cdcp verify-data-lock --selftest` — PASS
13. `cdcp oracle-check` — PASS
14. stale-plant detector — PASS
15. `cdcp_gate substrate-guard` — PASS
16. `cdcp_gate substrate-guard --prove-wired` — FAIL

Steps after 16 were not executed because `check.sh` fails closed at the prove-wired step.

## Run 3 first failure and blocker

The top-level wrapper reported:

```text
substrate-guard: ERROR: the behavioural wiring leg could not be evaluated — check.sh ended with exit 2 without the guard ever reporting on scripts/__cdcp_probe_unlisted__.py; the failure cannot be attributed to the substrate step. ERROR, not a pass.
check.sh: FAIL: substrate-guard wiring does not stop check.sh
```

The nested transcript gives the exact underlying error:

```text
cdcp_registry_check: gate-shrink: cdcp_gate 37280 > ceiling 37275 — the crate GREW. Raising ceiling_lines is weakening a gate (escalation-only). Extract or delete; do not transcribe.
check.sh: FAIL: registry-check
```

The nested index snapshot contains the pre-existing committed `crates/cdcp_gate/src/vcs.rs` at 434 lines. The live worktree has an uncommitted pane-2 change to that file at 421 lines, but committing or otherwise taking that pane-2 change is outside this tick. The ceiling was not raised.

## Run 3 option status

Run 3 is another correctness/index failure at 19.34 seconds, so there is still no successful complete-run time and the throughput question remains unanswered. Option (b) remains a provisional feedback-flow recommendation; no evidence now supports elevating caching, batching, queue draining, or any option as the highest-leverage throughput fix. No CI configuration was changed.

## Run 4 — after doc-facts extraction

- Tree: `fb040bb`
- Command: `PATH=/Users/josh/.cargo/bin:$PATH /usr/bin/time -p ./scripts/check.sh`
- Environment: local warm Cargo registry/target cache
- Result: no successful full-chain completion; the invocation reached step 36 and exited 2
- Wall time: `66.50` seconds (`1.11` minutes)
- Interpretation: this is a measured warm-cache floor to the next correctness/formatting failure, not a complete-run time or CI duration estimate

This run includes the committed `doc_facts` extraction. `cdcp_registry_check` and its nested gate-shrink receipt agree at `35579/35579`, and `cdcp_gate doc-facts` passed through the stable subcommand. The run passed the bank, answer-key-skew, grounding, orphan, near-duplicate, and paraphrase legs before the formatting chain.

## Run 4 ordered steps and verdicts

1. `check.sh lock selftest` — PASS
2. `cdcp-course check` / knowledge scaffold — PASS
3. `cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked` — PASS
4. `check.sh snapshot selftest` — PASS
5. `cdcp_registry_check` / registry-check — PASS (`35579/35579`)
6. `cdcp check-licence` / licence split — PASS
7. `cargo test -p cdcp_data --test corpus_rights` — PASS
8. `cdcp corpus-rights` — PASS
9. `cdcp load-snapshots` — PASS
10. `cdcp check-osha` — PASS
11. `cdcp verify-data-lock` — PASS
12. `cdcp verify-data-lock --selftest` — PASS
13. `cdcp oracle-check` — PASS
14. stale-plant detector — PASS
15. `cdcp_gate substrate-guard` — PASS
16. `cdcp_gate substrate-guard --prove-wired` — PASS
17. `cdcp_gate install-hooks` — PASS
18. `cdcp_gate install-hooks --check` — PASS
19. `cdcp_gate capability-maturity` — PASS
20. `cdcp_gate goldens-couplings` — PASS
21. `cdcp_gate doc-facts` — PASS
22. exam-form public CDCP format pins — PASS
23. honesty string smoke — PASS
24. standards crosswalk — PASS
25. `topics.toml` count — PASS
26. source `fetch_date` presence — PASS
27. `cdcp_gate verify-bank` — PASS
28. `cdcp_gate answer-key-skew` — PASS
29. `cdcp_gate validate-grounding` — PASS
30. `cdcp_gate grounding-wave` — PASS
31. `cdcp_gate verify-orphans` — PASS
32. `selftest_orphan.sh` — PASS
33. `cdcp_gate near-duplicate-items` — PASS
34. near-duplicate-items selftest — PASS
35. `cdcp verify-paraphrase-pairs` — PASS
36. `cargo fmt/clippy/test` — FAIL at the custom gate-module rustfmt check

Steps after 36 were not executed because `check.sh` fails closed at the formatting step.

## Run 4 first failure and blocker

Exact error:

```text
rustfmt_gate_modules: rustfmt --check failed: crates/cdcp_gate/src/gates/answer_key_skew.rs
check.sh: FAIL: rustfmt over crates/cdcp_gate/src/gates (cargo fmt cannot see these)
```

The failing dispatcher is the existing three-line compressed `answer_key_skew` gate file, with its one-line `#[rustfmt::skip]` runner. Formatting its first two lines would add two gate lines, taking the committed `35579` count over the ratcheted `ceiling_lines = 35579`; raising the ceiling is forbidden, and shortening prose or renaming locals to compensate would be gate-count golf. It was therefore recorded as the next blocker rather than changed in this extraction tick.

## Run 4 option status

The chain now reaches 66.50 seconds before a formatting correctness failure, but still does not complete. This remains a warm-cache floor, not a CI runtime. The throughput premise is still unmeasured; the earlier option (b)—a fast PR subset plus the full scheduled chain—remains a provisional feedback-flow recommendation, not an evidenced throughput fix. No CI configuration was changed.

## Run 5 — ratchet held at zero slack

- Tree: `c57ef03`
- Command: `PATH=/Users/josh/.cargo/bin:$PATH /usr/bin/time -p ./scripts/check.sh`
- Environment: local warm Cargo registry/target cache
- Result: no successful full-chain completion; the invocation reached step 36 and exited 2
- Wall time: `51.26` seconds (`0.85` minutes)
- Interpretation: this is a measured warm-cache floor to the same formatting failure, not a complete-run time or CI duration estimate

The requested `ceiling_lines = 35620` change was not made: it would raise the committed monotone-decreasing ratchet from `35579`, which is prohibited by the gate policy. The honest temporary-line probe confirmed the current state instead: adding one comment to the dispatcher made the measured gate count `35581`, and `cdcp_registry_check` rejected it with `cdcp_gate 35581 > ceiling 35579`. Removing the probe restored `35579/35579` and registry-check GREEN. There is therefore no spendable slack under the current ratchet; obtaining slack requires another real extraction or an authorized ratchet policy change.

## Run 5 ordered steps and verdicts

1. `check.sh lock selftest` — PASS
2. `cdcp-course check` / knowledge scaffold — PASS
3. `cargo build -p cdcp_gate -p cdcp_cli -p cdcp_registry_check --locked` — PASS
4. `check.sh snapshot selftest` — PASS
5. `cdcp_registry_check` / registry-check — PASS (`35579/35579`)
6. `cdcp check-licence` / licence split — PASS
7. `cargo test -p cdcp_data --test corpus_rights` — PASS
8. `cdcp corpus-rights` — PASS
9. `cdcp load-snapshots` — PASS
10. `cdcp check-osha` — PASS
11. `cdcp verify-data-lock` — PASS
12. `cdcp verify-data-lock --selftest` — PASS
13. `cdcp oracle-check` — PASS
14. stale-plant detector — PASS
15. `cdcp_gate substrate-guard` — PASS
16. `cdcp_gate substrate-guard --prove-wired` — PASS
17. `cdcp_gate install-hooks` — PASS
18. `cdcp_gate install-hooks --check` — PASS
19. `cdcp_gate capability-maturity` — PASS
20. `cdcp_gate goldens-couplings` — PASS
21. `cdcp_gate doc-facts` — PASS
22. exam-form public CDCP format pins — PASS
23. honesty string smoke — PASS
24. standards crosswalk — PASS
25. `topics.toml` count — PASS
26. source `fetch_date` presence — PASS
27. `cdcp_gate verify-bank` — PASS
28. `cdcp_gate answer-key-skew` — PASS
29. `cdcp_gate validate-grounding` — PASS
30. `cdcp_gate grounding-wave` — PASS
31. `cdcp_gate verify-orphans` — PASS
32. `selftest_orphan.sh` — PASS
33. `cdcp_gate near-duplicate-items` — PASS
34. near-duplicate-items selftest — PASS
35. `cdcp verify-paraphrase-pairs` — PASS
36. `cargo fmt/clippy/test` — FAIL at the custom gate-module rustfmt check

## Run 5 first failure

```text
rustfmt_gate_modules: rustfmt --check failed: crates/cdcp_gate/src/gates/answer_key_skew.rs
check.sh: FAIL: rustfmt over crates/cdcp_gate/src/gates (cargo fmt cannot see these)
```

The step remains unresolved because formatting that existing compressed dispatcher adds lines above the exact `35579` ceiling. No pane-2 test invocation or `required_tests` registry was changed.
