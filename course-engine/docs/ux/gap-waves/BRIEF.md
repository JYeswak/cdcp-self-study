# Design-gap wave brief — grading our learner surface against the dicklesworthstone corpus

**Controller:** cockpit Claude (session `cdcp`, pane 1)
**Subject:** `course-engine/web/` — the CDCP learner surface (hub, learn ×15, quiz, mock, drill,
results, reference, runbooks, diagrams)
**Oracle:** Jeffrey Emanuel's shipped web work, mirrored on the Studio at
`/Volumes/ZestData/dicklesworthstone-mirror` (212 repos). **This is an external oracle we do not
control** — that is the point. L3 of the artifact rigor standard: the project names an oracle it
does not own and runs a differential harness against it.

---

## Why this exists

`bd-rc-surface-undesigned-0ns7` states the finding: *117 UX observations produced ZERO blockers and
64 majors. Nothing is broken; a great deal is unconsidered.* Broken gets patched. Unconsidered gets
designed. We have no design vocabulary of our own to design against, so we borrow one from a corpus
of shipped artifacts that demonstrably clear the bar.

**This is a differential read, not an imitation exercise.** We are not porting Framer Motion or a
green-monster theme into an offline, no-CDN, no-network study tool. We are extracting the
*decisions* — what a surface tells a user, in what order, with what feedback — and recording where
ours makes no decision at all.

---

## Access

The mirror is on the Studio. From a pane:

```bash
ssh studio 'cd /Volumes/ZestData/dicklesworthstone-mirror/<repo> && <cmd>'
```

Non-interactive ssh does not source `.zshrc`. If you need a tool, export PATH inline:
`ssh studio 'export PATH="$HOME/.local/bin:/opt/homebrew/bin:$PATH"; ...'`

Read source. Do **not** try to run these sites — most are Next.js/BUN and the point is the
decisions in the source, not a running instance.

---

## Output contract (STRICT — a wave that violates this is not admissible)

Append to **your own file only**: `course-engine/docs/ux/gap-waves/w<N>-pane<P>.md`.
Never edit another pane's file. Never edit `UX-FINDINGS-*.md`.

Each finding is one row in this exact shape:

```
### G-<pane><wave>-<nn> · <one-line name>

- **Seen in:** `<repo>/<path>:<line>` — the specific artifact, not "the site generally"
- **What it decides:** the user-facing question that source answers
- **What ours does:** `course-engine/web/<file>:<line>` — cite our line, or write `NOTHING — no
  code exists for this question` and say which surface should own it
- **Class:** JOURNEY | FEEDBACK | STATE | HIERARCHY | MOTION | DENSITY | RECOVERY | AFFORDANCE | COPY
- **Transferable under our constraints?** YES / NO / DEGRADED — we are offline, no CDN, no network
  at runtime, static HTML + vanilla JS + one CSS file, `#![forbid(unsafe_code)]` Rust/WASM grader,
  reduced-motion honoured, dark theme measured at 13.35:1 body contrast. If NO, say so and stop —
  a NO is a real finding and costs nothing to record.
- **Cost:** S (< 1h) | M (< 1d) | L (needs a design decision a human must make)
- **Regression risk:** which of the 34 KEEP measurements in `UX-FINDINGS-DEDUP.md` this could break
```

## Hard requirements

1. **Cite lines.** A finding without `repo/path:line` on both sides is not a finding, it is an
   impression. Impressions do not go in the ledger.
2. **Every wave must contain at least one `Transferable: NO`.** If a pass finds everything
   transferable it has stopped evaluating and started shopping. Say what you rejected and why.
3. **Every wave must name at least one thing we do BETTER**, with our line cited. The corpus is an
   oracle, not a superior. Our honesty banner, measured contrast, byte-exact grader, and
   keyboard-complete exam are real and several of these sites have no equivalent.
4. **`NOTHING — no code exists` is the highest-value finding class.** The epic says the surface was
   never designed. Absences are the evidence. Hunt them deliberately.
5. **Do not change any product file during a wave.** Waves read and record. Implementation happens
   after the controller synthesises and beads are cut. A wave that edits `web/` has broken the
   protocol.
6. **Green-does-not-prove:** finishing a wave proves you read those repos. It does not prove the
   corpus contains the answer to our problem, and it does not prove a finding survives contact with
   our constraints. That is the controller's synthesis job, not yours.

## Return format (paste into the pane when done)

```
WAVE <N> PANE <P> COMPLETE
repos read: <list>
findings: <n> total — JOURNEY <n> FEEDBACK <n> STATE <n> HIERARCHY <n> MOTION <n> DENSITY <n> RECOVERY <n> AFFORDANCE <n> COPY <n>
transferable: YES <n> / DEGRADED <n> / NO <n>
NOTHING-no-code-exists: <n>
we-do-better: <n>
file: course-engine/docs/ux/gap-waves/w<N>-pane<P>.md
biggest single gap: <one sentence>
what this wave did NOT establish: <one sentence — required>
```
