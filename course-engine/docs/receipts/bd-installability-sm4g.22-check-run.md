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
