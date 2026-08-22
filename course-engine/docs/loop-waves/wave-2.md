loop-wave wave=2
harvest_sha256=9b7e3e793a19e40ac665e0e7e600e5e3d6642108750cc25434b3a97167b2a662 harvest_path=/Users/josh/.claude/references/franken-harvest.md
prior_artifact_path=/Users/josh/cdcp-self-study/course-engine/docs/loop-waves/wave-1.json prior_artifact_sha256=d88cc9886436f76a683a877d79e0b23cd4c6af2fa6cce7cc6c0eebd7357ebff4
wave-2 consumed wave-1 artifact (not a blank-slate rerun)

GRADE skill=loop-enforcement
FINDING comment-only emit-tick choke: scripts/check.sh calls emit-tick 'not a verdict-producing gate'; hooks/pre-commit execs substrate-guard --staged only. emit_tick / tick_guard exist in crates/cdcp_bank/src/tick_emitter.rs but a commit can land with no ledger row. BUILT ≠ WIRED.
PROBE id=emit-tick-forbidden-phrases path=crates/cdcp_bank/src/tick_emitter.rs wiring=Wired citation=standing by
PROBE id=check-sh-comment-only-choke path=scripts/check.sh wiring=Contradicted citation=not a verdict-producing gate
PROBE id=pre-commit-is-substrate-guard path=hooks/pre-commit wiring=Unwired citation=substrate-guard --staged (no emit-tick)
PROBE id=gate-wrapper-emit-tick path=crates/cdcp_gate/src/gates/emit_tick.rs wiring=Wired citation=tick_emitter

GRADE skill=loop-engineering
FINDING Rule Zero is Charter-stated and product_moved is COMPUTED in tick_emitter.rs (product_moved_disagreement when claim ≠ compute). A tick still counts only if it changed the product; the classifier decides, the agent proposes. The choke that would refuse a no-tick commit is unwired — see loop-enforcement grade.
PROBE id=emit-tick-computes-product-moved path=crates/cdcp_bank/src/tick_emitter.rs wiring=Wired citation=computed_product_moved
PROBE id=charter-rule-zero path=.flywheel/CHARTER.md wiring=Wired citation=PRODUCT MOVED
PROBE id=tick-ledger-exists path=.flywheel/tick-ledger.jsonl wiring=Wired citation=zs.tick-receipt
PROBE id=harvest-ledger-is-standing path=__HARVEST__ wiring=Wired citation=FRANKEN HARVEST
