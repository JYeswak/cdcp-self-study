# Wave assignments — disjoint corpus split

Read `BRIEF.md` first. Mirror root: `/Volumes/ZestData/dicklesworthstone-mirror` on the Studio
(`ssh studio`). Assignments are disjoint by repo so two panes never read the same source.

---

## WAVE 1 — flagship product sites · the polish bar

What to extract: the **scroll narrative**. How does a first-time visitor get from "what is this" to
"I know what to do next"? Where is hierarchy created? What is the first interactive thing offered,
and how soon? How does a section signal it is a section?

Our subject for this wave: `web/index.html` (the hub) — 8 equal-weight cards, a "Now · 90 seconds"
panel that currently reads "No cards due", and a 15-row module list with no progress anywhere.

| pane | repos |
|---|---|
| **2** | `frankentui_website` · `frankensqlite_website` · `asupersync_website` |
| **3** | `frankensim_website` · `mcp_agent_mail_website` · `jeffrey_emanuel_personal_site` |

Start with each repo's `lib/content.ts` (or equivalent) and its homepage. The content file is where
the decisions are legible — what got a name, what got a number, what got a stat, what got cut.

---

## WAVE 2 — interactive explainers · the pedagogy bar

**This is the wave that matters most.** These are the closest artifacts in the corpus to what we
actually are: things built to make a reader *understand* something, not to sell them a binary.

What to extract: how a concept is introduced before it is tested; where the artifact lets you
*manipulate* rather than read; what it does when you get something wrong; how it paces; whether it
ever tells you what to do next.

Our subject for this wave: `web/learn/*.html` (15 modules — **our least-observed surface, ~12 of
117 findings**, and where a learner spends most of their time) plus `web/diagrams/power-path.html`.

| pane | repos |
|---|---|
| **2** | `cmaes_explainer` · `phage_explorer` · `interactive_reversible_cellular_automata` · `cellular_automata_snowflake_simulator` · `visual_astar_python` |
| **3** | `letter_learning_game` · `jazz_chord_progression_editor_html` · `hoeffdings_d_explainer` · `raptorq_article` · `paxos_vs_raft` · `introduction_to_temporal_logic` |

`letter_learning_game` is small and looks trivial. Read it anyway: it is the only artifact in the
corpus whose entire job is *teaching a beginner through repetition*, which is literally our drill
loop.

---

## WAVE 3 — data & state surfaces · the dashboard bar

What to extract: how a screen renders *someone's accumulated state* — progress, history, weakness,
what changed. What does an empty state say? What does a bad state say? How is "what to do next"
computed and displayed?

Our subject for this wave: `web/results.html` (a 6/40 renders **15,233 px** with the recovery link
at y=15,152), `web/index.html`'s Module-mastery block (bare `M01`–`M15`, no miss counts), and
`assets/js/hub_mastery.js` / `mastery.js` / `review.js`.

| pane | repos |
|---|---|
| **2** | `beads_viewer` · `beads_viewer_rust` · `mindmap-generator` · `github-diff-viewer` · `coding_agent_usage_tracker` |
| **3** | `asimposium.org` · `classic-patents.com` · `eidetic-engine-website-project` · `llm-docs` · `nextjs-github-markdown-blog` · `markdown_web_browser` |

`beads_viewer` is the highest-signal repo in this wave: it is a static, offline-capable HTML view of
a dependency graph with state per node — structurally the same problem as our module-mastery list,
solved by someone who decided to actually solve it.

---

## Sequencing

Waves run in order. A pane finishing early does **not** start the next wave — it reports and holds,
because the controller synthesises between waves and the synthesis can change the next assignment.
That is the loop-#2 injection point.
