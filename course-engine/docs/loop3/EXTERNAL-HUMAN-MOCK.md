# Q14 external-human mock handoff

This is the instrument for the Loop-3 external signal. It is **not** a gate,
and it is not complete until a person who did not build this bank actually uses
it. An agent run, a maintainer self-run, `cdcp demo`, and a synthetic answer
file are useful internal checks but do **not** satisfy Q14.

**Honesty:** study signal only — not EPI/EXIN certification
[[claim:claim-not-epi-certified]] [[claim:claim-study-signal-27]]
[[claim:claim-interview-study-signal]]

## Facilitator: start the product

Use a current installed bundle and run the learner entry point:

```text
cdcp study
```

For a source checkout, the equivalent local-only command is:

```text
./target/debug/cdcp study --root /path/to/course-engine --no-open
```

Open the printed local URL in the participant's browser. Do not use
`cdcp demo`: demo deliberately submits planted all-correct and all-wrong
attempts and is an installation/grading smoke test, not an external learner
run. If the browser cannot open the bundle, record that as a product failure;
do not silently switch to a PDF, a source file, or a maintainer explanation.

## Participant: the 40-question session

1. Open **Mock exam** and use the displayed seed/form (the default handoff is
   seed 42).
2. Answer all 40 questions without coaching, an answer key, or a standards
   search. A facilitator may explain the controls, but must not explain the
   content.
3. Mark the option you chose and a short confusion note while the question is
   fresh. “None” is a valid note; do not invent a confusion point.
4. Submit the form and copy the score shown by the product. The displayed score
   is the score of record for this run.

The 27/40 value is a **study signal only**, not an EPI/EXIN certification or
an official exam pass. The participant should not be shown the answer key
before the notes are complete.

## Capture file

Copy this file to `docs/loop3/runs/YYYY-MM-DD-external-human.md` after the
session. Use a role or pseudonym rather than unnecessary personal data.

```markdown
# Loop-3 external-human run — YYYY-MM-DD

- **tier:** Q14 / external human
- **external_human:** (role or pseudonym; not an agent)
- **facilitator:** (role or pseudonym)
- **product_entry:** `cdcp study`
- **bundle_root:** (installed bundle or source-checkout path)
- **seed / mode:** mock seed 42 / 40 questions
- **bank_hash:** (record from the product/release)
- **started_utc:**
- **finished_utc:**
- **score:** __/40
- **study_signal:** pass | fail (27/40 study threshold only; not a credential)
- **browser_or_surface_issue:** none | (describe)
- **weak_modules:** (participant's or facilitator's evidence-based list)
- **what_confused_them:** (participant's words, or `none reported`)
- **what_the_product_helped_with:** (participant's words)
- **still_open_gap:** (participant's words)
- **epi_claim:** none
- **agent_run:** false

## Answer and confusion capture

| # | chosen option | confidence (low/medium/high) | confusion or reason |
|---:|:---:|:---:|---|
| 1 |  |  |  |
| 2 |  |  |  |
| 3 |  |  |  |
| 4 |  |  |  |
| 5 |  |  |  |
| 6 |  |  |  |
| 7 |  |  |  |
| 8 |  |  |  |
| 9 |  |  |  |
| 10 |  |  |  |
| 11 |  |  |  |
| 12 |  |  |  |
| 13 |  |  |  |
| 14 |  |  |  |
| 15 |  |  |  |
| 16 |  |  |  |
| 17 |  |  |  |
| 18 |  |  |  |
| 19 |  |  |  |
| 20 |  |  |  |
| 21 |  |  |  |
| 22 |  |  |  |
| 23 |  |  |  |
| 24 |  |  |  |
| 25 |  |  |  |
| 26 |  |  |  |
| 27 |  |  |  |
| 28 |  |  |  |
| 29 |  |  |  |
| 30 |  |  |  |
| 31 |  |  |  |
| 32 |  |  |  |
| 33 |  |  |  |
| 34 |  |  |  |
| 35 |  |  |  |
| 36 |  |  |  |
| 37 |  |  |  |
| 38 |  |  |  |
| 39 |  |  |  |
| 40 |  |  |  |
```

Do not fill this template from an agent's imagined answers. A missing human
run is a loud gap, not permission to manufacture an external signal.
