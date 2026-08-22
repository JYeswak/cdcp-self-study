# Wave 2, pane 3 — interactive explainers and the pedagogy bar

This is a differential read of six assigned artifacts against the shipped Learn
surface. The corpus is not a trivial-demo corpus: `letter_learning_game` has a
real repetition, correction, adaptive-selection, and review loop; RaptorQ has
several staged manipulable models; the other artifacts make explicit decisions
about prerequisites, analogies, comparison, and implementation steps. Those
decisions are evidence of what the source chose, not proof that the same choice
improves CDCP exam outcomes.

## Findings

### G-32-01 · A beginner is oriented before the first attempt

- **Seen in:** `letter_learning_game/index.html:5081-5092` — the start screen says what the game teaches, asks for sound, explains the reward and round loop, then offers “Let's Start!”; the first prompt is only generated later at `letter_learning_game/index.html:9371-9429`.
- **What it decides:** A cold beginner should know what the activity is, what to attend to, and what will happen before being asked to identify an item.
- **What ours does:** `course-engine/web/learn/01-mission-critical.html:54-79` — the page exposes a unit shell and “Quick check” host around asynchronously loaded prose; `course-engine/web/assets/js/learn_units.js:498-524` shows the unit and check after the rendered article section, but there is no first-attempt orientation or concept-to-attempt handoff.
- **Class:** JOURNEY
- **Transferable under our constraints?** YES — the orientation and explicit boundary are static copy and vanilla DOM state; no source dependency needs to be imported.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: prose column stays constrained; KEEP: skip-to-main-content link; KEEP: the honesty banner must remain visible while adding orientation.

### G-32-02 · A wrong answer keeps the learner in the same concept

- **Seen in:** `letter_learning_game/index.html:9632-9670` — a wrong choice is recorded, the system names both the chosen and target letters, says “Try again,” and enters a correction path; `letter_learning_game/index.html:9671-9694` highlights the correct choice, restarts the timer, and re-enables the same question.
- **What it decides:** Wrong is a teachable state, not a terminal score event: identify the distinction, show the correction, and give the learner another attempt before advancing.
- **What ours does:** `course-engine/web/assets/js/learn_units.js:400-418` — the first click sets `data-done`, disables every choice, marks the selected and correct buttons, and appends an explanation; there is no retry action for that item.
- **Class:** FEEDBACK
- **Transferable under our constraints?** YES — same-item retry, explanation, and local state are implementable in the existing vanilla module shell.
- **Cost:** L (needs a design decision a human must make)
- **Regression risk:** KEEP: diagram self-checks use fieldset/legend with text-bearing labels; KEEP: reduced-motion is honoured; KEEP: “No cards due” must not be repurposed as a misleading mastery signal.

### G-32-03 · The next item is chosen from demonstrated weakness

- **Seen in:** `letter_learning_game/index.html:7441-7479` and `letter_learning_game/index.html:7497-7553` — due letters are preferred, low success and error counts increase weight, recent items are down-weighted, and shape peers are deliberately reinforced after an error.
- **What it decides:** “Next” is not merely the next row; it is a computed response to what this learner just did.
- **What ours does:** `course-engine/web/assets/js/learn_units.js:544-551` advances by `idx++`; `course-engine/web/assets/js/learn_units.js:317-323` persists the current unit position, but it does not select the next unit from check outcomes.
- **Class:** STATE
- **Transferable under our constraints?** YES — the decision can use the existing localStorage-only state; no network or service is required.
- **Cost:** L (needs a design decision a human must make)
- **Regression risk:** KEEP: “No cards due” empty state names the next action; KEEP: dark-theme contrast measured; KEEP: prose column stays constrained.

### G-32-04 · Bounded rounds turn repetition into a visible effort unit

- **Seen in:** `letter_learning_game/index.html:5771-5776` — the game declares a 15-question round; `letter_learning_game/index.html:9963-9969` inserts a possible break and then a completion boundary; `letter_learning_game/index.html:10256-10311` reports average time, accuracy, score, problem letters, and the next round.
- **What it decides:** Repetition needs a finish line and a review moment, not only a stream of attempts.
- **What ours does:** `course-engine/web/assets/js/learn_units.js:498-531` gives the current Learn surface a 5–8 minute unit path and “unit N / total” bar, but `course-engine/web/assets/js/learn_units.js:419-429` only reports aggregate completion for the local micro-check and does not turn misses into a round review boundary.
- **Class:** DENSITY
- **Transferable under our constraints?** YES — a local unit/round boundary and review state fit static HTML plus vanilla JS.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: 375 px has no horizontal overflow; KEEP: prose column stays constrained; KEEP: reduced-motion is honoured.

### G-32-05 · Familiar ground is named before formal machinery

- **Seen in:** `raptorq_article/index.html:189-197` — “What You May Already Know” uses PAR2 and RAID as a bridge before the article turns to equations at `raptorq_article/index.html:225-249`; the first matrix interaction follows that bridge at `raptorq_article/index.html:251-289`.
- **What it decides:** The reader should receive a mental hook and a concrete vocabulary before manipulating the formal model.
- **What ours does:** `course-engine/web/learn/01-mission-critical.html:70-79` renders the article and its check host; `course-engine/web/assets/js/learn_units.js:184-216` can reveal one heading-bounded unit, but it does not declare a “you may already know this” bridge before the check.
- **Class:** HIERARCHY
- **Transferable under our constraints?** YES — analogy-before-formalism is a content and sequencing decision, not a framework dependency.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: instruction density is already a measured concern; KEEP: prose column stays constrained; KEEP: dark-theme contrast measured.

### G-32-06 · Manipulation exposes a model’s changing state

- **Seen in:** `raptorq_article/index.html:251-289` offers “Add Equation” and “Reset,” exposes rank state, and explains what a new independent packet changes; `raptorq_article/index.html:585-610` adds Back/Next/Reset/Auto for a toy decode, while `raptorq_article/index.html:631-645` makes each peel step observable.
- **What it decides:** The learner can alter a model, inspect the changed state, and proceed one causal step at a time instead of only reading a finished diagram.
- **What ours does:** `course-engine/web/diagrams/power-path.html:236-258` has the one shipped interactive self-check and `course-engine/web/diagrams/power-path.html:431-452` supports placement, wrong-slot feedback, and retry; the 15 module pages otherwise expose diagram links/artifacts rather than a sequence of manipulable models, as shown by `course-engine/web/learn/01-mission-critical.html:81-92`.
- **Class:** AFFORDANCE
- **Transferable under our constraints?** YES — the interaction pattern is achievable with the current static HTML, vanilla JS, and one CSS file; RaptorQ’s visual styling is not part of the transfer.
- **Cost:** L (needs a design decision a human must make)
- **Regression risk:** KEEP: diagram self-checks use fieldset/legend with text-bearing labels; KEEP: reduced-motion is honoured; KEEP: 375 px has no horizontal overflow.

### G-32-07 · The artifact makes the learner author and hear a result

- **Seen in:** `jazz_chord_progression_editor_html/jazz_chord_progression_editor.html:49-65` — the artifact describes a concrete loop: type changes like a lead sheet, hear the band play them, and share the chart; it explicitly defines the studio as deterministic and offline.
- **What it decides:** A learner-facing tool can make the learner produce an object and immediately perceive its result, rather than only consume explanatory text.
- **What ours does:** `course-engine/web/learn/01-mission-critical.html:81-85` asks the learner to speak a 60-second site tour on paper; it is a useful produced artifact, but it is not an on-device authored object with immediate playback or state feedback.
- **Class:** AFFORDANCE
- **Transferable under our constraints?** YES — the decision can be implemented with local HTML, vanilla JS, Web Audio, and a URL fragment; the corpus’s larger studio scope is a cost concern, not a reason to reject the interaction pattern.
- **Cost:** L (needs a design decision a human must make)
- **Regression risk:** KEEP: honesty banner remains claim-not-credential; KEEP: prose column stays constrained; KEEP: reduced-motion is honoured.

### G-32-08 · A prerequisite can be explicit and skippable

- **Seen in:** `introduction_to_temporal_logic/README.md:3-7` — the explainer says FOL is a prerequisite, explains why it helps, and explicitly permits a familiar reader to skip to temporal logic.
- **What it decides:** The cold learner gets a prerequisite decision without being forced through known material.
- **What ours does:** `NOTHING — no code exists for this question` in the Learn page shell; `course-engine/web/learn/01-mission-critical.html:41-50` has only breadcrumb and a noscript source note, so the Learn surface should own a prerequisite/skip cue.
- **Class:** JOURNEY
- **Transferable under our constraints?** YES — a local prerequisite note and skip link need no runtime service.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: skip-to-main-content link is first in tab order; KEEP: instruction density is measured; KEEP: prose column stays constrained.

### G-32-09 · One concept is repeated as notation, English, and intuition

- **Seen in:** `introduction_to_temporal_logic/README.md:68-87` — each equality axiom is presented with formal representation, English translation, and explanation; `introduction_to_temporal_logic/README.md:132-140` repeats the pattern for inference with a formulation, example, and intuition.
- **What it decides:** A formal claim should have at least one readable translation and one reason it matters before the reader is asked to use it.
- **What ours does:** `course-engine/web/assets/js/learn_md.js:1-7` supports headings, lists, tables, fences, and inline content, and `course-engine/web/learn/01-mission-critical.html:70-79` mounts the rendered prose/check surface; neither establishes a repeated concept/formalism/intuition contract.
- **Class:** COPY
- **Transferable under our constraints?** YES — this is a content schema and authoring rule that the existing Markdown renderer can carry.
- **Cost:** L (needs a design decision a human must make)
- **Regression risk:** KEEP: dark-theme contrast measured at 13.35:1 body / 7.73:1 secondary / 5.18:1 faint; KEEP: prose column stays constrained; KEEP: instruction density is measured.

### G-32-10 · Comparison is staged from analogy to table to independent subproblems

- **Seen in:** `paxos_vs_raft/README.md:13-27` begins with a restaurant analogy; `paxos_vs_raft/README.md:38-49` turns the analogy into a step table; `paxos_vs_raft/README.md:207-212` explains that Raft divides the problem into independent subproblems to improve understandability.
- **What it decides:** Complexity is made navigable by changing representation in deliberate stages, not by presenting one undifferentiated article.
- **What ours does:** `course-engine/web/assets/js/learn_units.js:184-216` segments by existing `h2` headings and `course-engine/web/assets/js/learn_units.js:498-524` places a quick check after a unit; it does not choose or label a representation sequence such as analogy → comparison → isolated mechanism.
- **Class:** HIERARCHY
- **Transferable under our constraints?** YES — the sequence can be authored in Markdown and revealed by the existing unit path.
- **Cost:** L (needs a design decision a human must make)
- **Regression risk:** KEEP: instruction density is measured; KEEP: prose column stays constrained; KEEP: 375 px has no horizontal overflow.

### G-32-11 · A long explainer tells the reader what step comes next

- **Seen in:** `hoeffdings_d_explainer/README.md:48-64` explicitly orders words and one formula, piece-by-piece intuition, and an implementation; `hoeffdings_d_explainer/README.md:106-122` then makes the implementation concrete with a small dataset and displayed intermediate values.
- **What it decides:** The learner gets a declared progression from orientation to procedure to runnable/concrete evidence.
- **What ours does:** `course-engine/web/assets/js/learn_reader.js:96-123` renders the full module and mounts units after the async render; `course-engine/web/assets/js/learn_units.js:527-565` provides fixed Prev/Next and Full article controls, but does not compute a next step from the learner’s evidence.
- **Class:** JOURNEY
- **Transferable under our constraints?** NO — the artifact’s runnable handoff points to Google Colab at `hoeffdings_d_explainer/README.md:122-127`, which violates our offline/no-network runtime constraint. The sequencing decision survives as a separate design observation; the external execution path does not.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: reduced-motion is honoured; KEEP: “No cards due” empty state names the next action; KEEP: honesty banner remains claim-not-credential.

### G-32-12 · The source’s visual progress is tied to narrative progress

- **Seen in:** `raptorq_article/index.html:127-155` gives the reader a persistent scroll-progress bar and a “Scroll to Explore” cue; the article then marks distinct sections such as `raptorq_article/index.html:225-229` and `raptorq_article/index.html:574-583` before each interactive sequence.
- **What it decides:** Progress is both “where am I in the narrative?” and “what kind of effort comes next?”
- **What ours does:** `course-engine/web/learn/01-mission-critical.html:13-17` and `course-engine/web/assets/js/learn_chrome.js:240-270` do provide measured scroll progress, while `course-engine/web/assets/js/learn_chrome.js:206-237` builds “On this page”; the remaining gap is that these signals track scroll/headings, not a learner-facing sequence of concept, manipulation, check, and recovery.
- **Class:** DENSITY
- **Transferable under our constraints?** YES — the existing progress bar and TOC can carry a small narrative-state label without a build step or network.
- **Cost:** M (< 1d)
- **Regression risk:** KEEP: reduced-motion is honoured; KEEP: prose column stays constrained; KEEP: dark-theme contrast measured.

### G-32-13 · The learning surface preserves the claim boundary while teaching

- **Seen in:** `letter_learning_game/index.html:5081-5088` frames the activity as a game with stars and unlocked challenges, but does not provide an equivalent certification boundary in its start copy.
- **What it decides:** A motivating first-run frame can be made explicit without letting an activity imply a credential.
- **What ours does:** `course-engine/web/learn/01-mission-critical.html:19-24` keeps the always-visible study-only/not-a-credential banner, and `course-engine/web/learn/01-mission-critical.html:68-68` repeats that unit signals are not a credential.
- **Class:** COPY
- **Transferable under our constraints?** NO — importing the source’s “unlock challenges” framing as a green achievement/certification treatment would violate the load-bearing honesty constraint. Our amber claim boundary is the better decision and must not be softened.
- **Cost:** S (< 1h)
- **Regression risk:** KEEP: honesty banner is load-bearing; KEEP: amber claim boundary; KEEP: dark-theme contrast measured.

### G-32-14 · Offline and reduced-motion behavior are stronger in our shipped surface

- **Seen in:** `letter_learning_game/index.html:9-29` loads analytics, Tailwind, GSAP, animate.css, TensorFlow.js, MobileNet, and Google Fonts from network URLs; its source therefore makes runtime availability part of the learning experience.
- **What it decides:** The artifact chose convenience and richer runtime dependencies for the interaction layer.
- **What ours does:** `course-engine/web/assets/css/course.css:1-3` declares a static/no-CDN surface, `course-engine/web/assets/css/course.css:22-39` defines the local dark token system, and `course-engine/web/assets/css/course.css:1141-1155` collapses animation and smooth scrolling under reduced motion; this is a case where our shipped decision is better.
- **Class:** MOTION
- **Transferable under our constraints?** NO — the source’s CDN-dependent runtime is killed by our offline/no-CDN/no-network constraint. Extract the interaction decisions, not those dependencies.
- **Cost:** S (< 1h)
- **Regression risk:** KEEP: reduced-motion is honoured; KEEP: dark-theme contrast measured at 13.35:1 body / 7.73:1 secondary / 5.18:1 faint; KEEP: no CDN at runtime.

## The five required questions

1. **Concept before test:** `letter_learning_game` answers this with its start orientation and spoken prompt (G-32-01); our unit path does stage prose before a micro-check, but does not make the cold-start boundary explicit enough. The first-run hub therefore needs a learner route before a due-card return loop.
2. **Manipulate rather than read:** RaptorQ answers this repeatedly with state-changing matrix, simulation, toy-decode, and peel interactions (G-32-06). We do have a legitimate counterexample to “nothing interactive”: `power-path.html` supports placement, reveal, wrong-slot feedback, and retry. The differential gap is breadth and causal sequencing across the 15 modules, not the absence of any interaction.
3. **When wrong:** the strongest corpus decision is same-item correction and retry (G-32-02). Our micro-check marks the card done and disables every option after one answer (G-32-02); power-path is better than the generic unit check because its wrong placement can be retried (G-32-06).
4. **Pacing:** our current unit path, 5–8 minute estimate, unit progress bar, and TOC are real pacing improvements (G-32-04 and G-32-12), but they measure position/scroll rather than a learner’s concept → manipulation → check → recovery effort. The corpus shows why a visible round or step boundary still matters.
5. **What to do next:** the letter game computes future items from due state, errors, success rate, recency, and shape confusion (G-32-03), and computes practice recommendations from performance (G-32-14’s counterpart in `letter_learning_game/index.html:11837-11850`). Our Continue state restores a unit position (`course-engine/web/assets/js/learn_chrome.js:171-189`), and our next button increments the unit index (`course-engine/web/assets/js/learn_units.js:544-551`); neither is yet an evidence-driven recommendation.

## The hub disagreement: which reading wins?

For the hub’s primary first-run experience, the “single worst thing” reading wins. “No cards due” names a sensible return-state action, and the KEEP measurement “No cards due” empty state names the next action records that strength, but the corpus’s beginner surface demonstrates that a cold visitor first needs orientation and an immediate, bounded learning action. A null due-state panel cannot be the hub’s only wayfinding route for someone with no prior state. This is not a compromise verdict: preserve the phrase for returning learners, but treat it as insufficient and harmful as the first-run route.

## What this wave did better / did not prove

We do better on two constraints the corpus did not preserve as consistently: the always-visible claim boundary (`course-engine/web/learn/01-mission-critical.html:19-24`) and the offline/reduced-motion contract (`course-engine/web/assets/css/course.css:1-3`, `course-engine/web/assets/css/course.css:1141-1155`). Those strengths are not evidence that our pedagogy is complete; they are constraints any later interaction must preserve.

Green-does-not-prove: reading six explainers proves these artifacts made these pedagogical decisions, but it does not establish improved CDCP exam outcomes, that every decision belongs in an offline facilities-exam tool, or that the current Learn surface is worse for every learner because it lacks the full corpus sequence.
