# CHARTER — cdcp-self-study loop-engineering run

**One charter, ever. Defects are fixed by editing THIS file in place — never by a successor.**

```
shipped_means:  A learner opens the offline hub, is assessed ONLY on material this course
                teaches, from items whose keys are traceable to a named authority, and the
                score means what the repo says it means. LOOP-#3 SIGNAL REQUIRED: a data-centre
                operations practitioner who did not build this works through a module and a mock
                and reports whether the assessment measured anything they use at work.
                A green check.sh is loop-#1 evidence only.

non_goals:      EPI/EXIN affiliation or any certification claim · LLM as grader-of-record ·
                SaaS · React LMS · reproducing proprietary syllabus body text · shipping any
                artifact whose recorded redistribution != permitted · psychometrics before
                there is attempt data · a full-scope simulator (round 3: borrowed prestige).

milestones:     A truth patch                      [S] DONE 467b429   reversible
                B machine ledgers                   [S] reversible
                C item status + assembly filter     [S] reversible  <- THE GATE
                D evidence spine (cdcp_evidence)    [F] reversible
                E public data corpus                [S] reversible   (rights review = decision contract)
                F differential oracle               [F] reversible   <- first honest L3
                G typed assessment                  [S] reversible
                H curriculum from graph             [S] reversible   BLOCKED ON C
                I scheduling                        [S] reversible
                J PX plant model                    [M] BEHIND A FLAG, off the critical path
                K scenario capstone                 [F] reversible
                L attempt capture                   [S] reversible
                M distribution                      [S] IRREVERSIBLE (publish) -> decision contract

ambition_tags:  see milestones. INVARIANT HOLDS: the only [M] is J (PX), which is gated behind a
                pre-registered falsification experiment OUTSIDE the production bank and is not on
                the critical path to shipped_means.

value_bar:      A tick is GREEN iff EITHER
                (a) PRODUCT MOVED — a learner-visible surface changed (an item pool, a module, a
                    question type, a rendered diagram, a score's meaning); OR
                (b) A GUARD DENIED A REAL ATTEMPT — a gate was installed AND observed refusing a
                    genuine bad input, not merely authored (CLAUDE.md §12).
                PLUS in both cases: the gate for that change exited clean, the bead advanced, and
                `value_added` names the change in one sentence.
                A gate that is written but has never denied anything scores ZERO. A doc, schema,
                review, or graph fragment with no route behind it scores ZERO. Partial never
                rounds up.

gauntlet:       L1 claims constitution (registry-check, WIRED) · L2 slo.toml · L4 gates-proven-to-
                trip: 9 suites / 36 injections, drift-guarded, every new gate ships a known-bad
                injection AND a meta-test AND a known-GOOD leg · L5 property/fuzz floor ·
                L7 content.lock.
                META-TEST, STATED CORRECTLY (corrected in place 2026-08-14): the earlier wording
                here — "delete the assertion -> selftest non-zero" — is INCOHERENT for a
                differential or oracle-backed test, and two port agents refused it on the same
                day rather than fake the result. Deleting an assertion WEAKENS a test; it cannot
                make it fail. On an unmutated tree there is nothing left for it to catch.
                The meaningful form is a PAIR: (1) MUTATE THE GATE — one byte of its output, its
                exit code, or its anti-vacuous branch — and confirm the suite goes non-zero;
                (2) with that mutation STILL IN PLACE, delete the assertion and confirm the suite
                returns to zero. Leg (1) proves the guard bites. Leg (2) proves THAT assertion is
                what bit, rather than some neighbour. Record both TRUE exit codes; never read an
                exit code through a pipe. A bead whose acceptance says only "delete the assertion"
                has not specified a meta-test and must not be closed on one.
                (3) RESTORE — AND THIS IS THE STEP THAT ROTS (added 2026-08-14,
                bd-stale-binary-mtime-trap-p65w). `cp f f.bak` … `mv f.bak f` puts the CONTENT
                back but hands the file the BACKUP'S mtime, older than the artifact cargo already
                built from the perturbed source; cargo compares mtimes, skips the rebuild, and the
                next run reads its verdict off the PERTURBED binary. A bad step (3) does not spoil
                the pair that performed it — IT SPOILS THE NEXT ONE, which is why it survives
                review. Measured 2026-08-14 (goldens/PROVENANCE.md, with both mtimes) as a false
                RED; the identical mechanism yields a FALSE GREEN when what you restore is the
                deleted assertion of leg (2), and that direction is silent. Reproduced on demand
                in both directions by crates/cdcp_gate/tests/restore_rebuild_trap.rs.
                THE RULE: restore by WRITING BYTES into the existing file — `git checkout -- f`,
                `cp bak f`, `printf … > f`, or `cdcp_restore_safe dest bak` — NEVER a rename;
                then prove the rebuild with one command whose exit code is the receipt
                (bd-stale-artifact-gate-urj0):
                `sh scripts/restore_safe.inc.sh prove-rebuild --artifact <bin> -- cargo test -p
                <crate> --offline --no-run`. That command compares the artifact's mtime BEFORE
                the build with its mtime AFTER and is non-zero when nothing was rebuilt.
                "Just run cargo build first" is NOT the fix and is itself vacuous: after a
                rename-restore cargo exits 0, prints `Finished in 0.00s` and rebuilds nothing —
                it passes in exactly the case it was written to catch. A BUILD THAT FINDS
                NOTHING TO REBUILD, WHEN THE FILE DEMONSTRABLY CHANGED, IS AN ERROR AND NOT A
                PASS. Do not "prove" freshness with `artifact_mtime > source_mtime`: the
                poisoned tree satisfies it. Pattern and argument:
                crates/cdcp_gate/tests/support/rebuild.rs. `mv backup dest` is not expressible
                in scripts/restore_safe.inc.sh.
                L3 IS CURRENTLY **NO** (CHARTER §5a) — F3 is the tick that flips it, and the
                capability-maturity row must point at F3's test.
                CLAIM DISCIPLINE: every gate states its claim as a FLOOR-RAISE plus what it
                cannot decide. "guarantees / proves / makes impossible" in a gate header is a
                defect.
                ANCHORING: >=1 verification command per close must touch something outside our
                own gate stack (a product test, a real dataset, a published reference value).

budgets:        Per tick: one bead, <=5 files, agent budget <=250k tokens. Escalate if a single
                bead exceeds two ticks.

external_validation:
                A practitioner who did not build this reports whether the assessment measured
                anything real. Until that exists, "shipped" is unpronounceable. F3's differential
                harness against published reference values is the first NON-CIRCULAR anchor and
                is a loop-#3 CANDIDATE, not the signal itself.

context_advantage_points:
                (1) OQ-10 paid SDO spend (TIA-942 ~$653) — escalation-class, changes what F/D can
                    assert. (2) C5 module-15: teach or exclude is a product-scope call.
                (3) M publish: irreversible. (4) What "good" means for an assessment item — the
                    reviewer P1 needs and we do not have. BATCHED to milestone boundaries.

skill_embedding:
                planning-workflow (plan) · beads-workflow + beads-north-star (materialize) ·
                beads-compliance-and-completion-verification (GRADE EVERY CLOSE) ·
                research-to-graph (research ticks dual-emit) · codex adversarial review each pass ·
                socraticode before substrate edits · dcg/slb for destructive+deploy.

design_plan:    docs/PLAN-A-TO-Z.md — the A→Z execution plan; docs/ROADMAP-PHASES.md scope
                rationale; docs/ROADMAP-WAVES.md review history (v1→v4, two theses killed);
                docs/FRANKENSIM-ADOPTION.md the borrowed patterns.

doc_practices:  CHANGELOG on learner-visible change · CHARTER edited IN PLACE on defect ·
                .flywheel/STATE.md every tick · tick-ledger.jsonl one row per tick.

decalogue:      1. Determinism is a contract: same inputs => byte-identical canonical JSON.
                2. A gate with no known-bad injection is a suggestion, and the loop treats it as
                   unwired.
                3. An empty input set is an ERROR, never a pass.
                4. A claim in prose must resolve to a registry row (L1) or it is not a claim.
                5. Rights are decided per artifact by a RECORDED licence line; blank is an ERROR,
                   never default-permissive. `access=free` is a PRICE fact only.
                6. ai_ingestion=PROHIBITED sources are never entered into any AI tool, and a
                   locator is a fact about a document, not its content.
                7. A remedy verifiable only by another of our documents is UNGROUNDED (U1).
                8. Every gate states its claim as a FLOOR-RAISE plus what it cannot decide (U4).
                9. Repetition is not corroboration — two sources circulating one unsourced claim
                   raise no confidence.
               10. BUILT != WIRED: no bead closes until the mechanism fires without a human.

danger_list:    spend (incl. OQ-10) · publishing/pushing to the public remote · deleting or
                overwriting another agent's work · git history rewrite · weakening or bypassing
                any gate (strengthening is autonomous) · entering ASHRAE or any
                ai_ingestion=PROHIBITED content into an AI tool · vendoring anything whose
                redistribution != permitted · credential rotation.

red_streak_pause: 3 consecutive RED ticks => STOP, surface the trend, name the single decision.

counter_plan:   The seductive-wrong version is a loop that drains B (ledgers/gates) at high
                velocity, reports 9 GREEN ticks, and never changes what a learner sees — the
                80%-documents pathology wearing a bead graph as a badge. Pre-empted by the value
                bar: a written-but-never-fired gate scores ZERO, and C1/C3/C5 are ordered ahead
                of B in the ready queue.

risk_register:  SEV-1 gate-on-gate drift (mitigate: value bar (b) requires an OBSERVED denial)
                SEV-1 no reviewer for item quality (mitigate: P1 blueprint; escalate at boundary)
                SEV-2 scope creep (mitigate: anything new must displace something or wait)
                SEV-2 stale volatile facts in the graph (mitigate: recorded_at on every such node)
                SEV-2 agent collision on shared files (mitigate: one bead per tick, file ownership)

target_ticks:   A measure, not the goal. Track product-move RATE; if it nears zero, STOP.

campaign:       N/A — no incumbent to beat yet; the core does not exist.

signed_off:     PENDING — reversible local building proceeds without it (RULE ZERO). Required
                before any danger_list action, notably M (publish) and OQ-10 spend.
```
