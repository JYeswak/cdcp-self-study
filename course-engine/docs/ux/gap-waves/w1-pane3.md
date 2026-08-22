# Wave 1 · Pane 3 — flagship product sites

Corpus read: `frankensim_website`, `mcp_agent_mail_website`, and `jeffrey_emanuel_personal_site` on the Studio mirror. The read was source-only; no corpus site was run. Subject: `course-engine/web/index.html`, the learner hub.

The useful comparison is at the decision level: these sites answer orientation, proof, grouping, and next-action questions deliberately. Their framework, network, and visual-runtime choices are not automatically admissible for this offline learner surface.

### G-31-01 · The hero turns the promise into one named first action

- **Seen in:** `frankensim_website/app/page.tsx:218-269` — the headline states “Simulation that returns proofs,” then places “See the Flagships,” “Explore the Kernel,” and “Source” in a visible action cluster.
- **What it decides:** A first-time visitor gets a plain-language promise before being asked to choose a destination, and the primary action is visually distinct from secondary exploration.
- **What ours does:** `course-engine/web/index.html:37-58` — the hub first explains local HTTP, `file://`, and `CDCP_FILE_ORIGIN`, then offers two equal actions in “Now · 90 seconds”; the first learner promise and preferred path are not separated from operating instructions.
- **Class:** JOURNEY
- **Transferable under our constraints?** YES — hierarchy, copy, and static links fit HTML/CSS/vanilla JS; no framework or network is required.
- **Cost:** L (needs a design decision a human must make; choose the preferred first-run path)
- **Regression risk:** F-23 instruction density; preserve the measured 375 px no-horizontal-overflow KEEP while changing the action cluster.

### G-31-02 · Section landmarks explain the job of the next block

- **Seen in:** `mcp_agent_mail_website/app/page.tsx:192-212` and `mcp_agent_mail_website/app/page.tsx:427-452` — “Why Purpose-Built Coordination” and “Coordination Comparison” each pair an eyebrow, heading, and kicker with a distinct content block.
- **What it decides:** A visitor can answer “why am I seeing this section?” before scanning its cards or visualization; the page is a sequence of arguments, not only a sequence of components.
- **What ours does:** `course-engine/web/index.html:60-104` — the main eight-card grid has no owning heading or kicker; card labels identify individual destinations but do not explain the role of the group. The later “Specialist tracks” and “Module mastery” blocks do have headings at `course-engine/web/index.html:106-129`.
- **Class:** HIERARCHY
- **Transferable under our constraints?** YES — a static heading, eyebrow, and one-sentence kicker are sufficient.
- **Cost:** M (< 1d)
- **Regression risk:** F-23 instruction density; the new landmark must reduce scanning ambiguity without adding another paragraph wall, and must preserve the constrained prose column.

### G-31-03 · A proof strip answers “why should I trust this?” at a glance

- **Seen in:** `mcp_agent_mail_website/lib/content.ts:252-256` and `mcp_agent_mail_website/app/page.tsx:160-187` — four named hero metrics (“MCP Tools,” “Resources,” “Stress Gauntlet,” and “Sustained Throughput”) are rendered as a dedicated proof strip immediately after the hero, with contextual hover text.
- **What it decides:** The first screen does not require a visitor to infer scale or reliability from feature prose; it names the evidence and its scope.
- **What ours does:** `NOTHING — no code exists for this question` — the hub has no measured proof strip for learner scope, content coverage, or study-state reliability; `course-engine/web/index.html:131-134` only states implementation properties such as static shell and no CDN.
- **Class:** FEEDBACK
- **Transferable under our constraints?** DEGRADED — a frozen, locally measured strip could transfer, but live counters, hover-only context, or invented efficacy numbers cannot; every value needs a named denominator and measurement.
- **Cost:** L (needs a design decision a human must make; choose metrics that are real and useful)
- **Regression risk:** F-23 instruction density and the dark-theme contrast KEEP measurements (13.35:1 body / 7.73:1 secondary / 5.18:1 faint).

### G-31-04 · Before/after scenarios make the promised change concrete

- **Seen in:** `mcp_agent_mail_website/app/page.tsx:457-485` — “What Changes” renders each workflow delta with “Without Agent Mail,” “With Agent Mail,” and an explicit impact line.
- **What it decides:** A newcomer can compare the current pain with the proposed outcome without translating feature names into a use case.
- **What ours does:** `course-engine/web/index.html:99-102` — “Runbook vignettes” is named and described as having links into quiz and mock practice, but the hub does not show the scenario’s before state, after state, or consequence.
- **Class:** JOURNEY
- **Transferable under our constraints?** YES — static paired panels or details blocks fit the shipped path; the scenario content, not the source site’s animation, is the transferable decision.
- **Cost:** M (< 1d)
- **Regression risk:** F-23 instruction density; avoid duplicating the full runbook prose and preserve the 375 px no-horizontal-overflow KEEP.

### G-31-05 · A persona rail lets the visitor choose a route by situation

- **Seen in:** `mcp_agent_mail_website/app/page.tsx:563-606` — “Built for How You Work” presents target-audience labels, audience-specific headlines and sublines, and a CTA per route.
- **What it decides:** The first-time visitor is not forced to understand the product taxonomy before choosing a useful entry point; the route is selected by context.
- **What ours does:** `course-engine/web/index.html:60-118` — the hub offers eight equal cards and one CDCS specialist-track card, but no “I am new / I am revising misses / I want interview direction” route with a distinct next action.
- **Class:** AFFORDANCE
- **Transferable under our constraints?** YES — a small static route rail can link to existing Learn, Drill, Mock, and Reference surfaces.
- **Cost:** L (needs a design decision a human must make; choose route labels and precedence)
- **Regression risk:** F-28 Drill naming and F-23 instruction density; do not create another near-duplicate Drill surface or make the honesty banner less prominent.

### G-31-06 · A compact inventory previews the ecosystem before the full catalog

- **Seen in:** `jeffrey_emanuel_personal_site/components/hero.tsx:253-301` and `jeffrey_emanuel_personal_site/lib/content.ts:67-101` — five named tools receive short taglines/descriptions in a compact, horizontally scrollable mobile row, followed by a single “Explore all tools” route.
- **What it decides:** The visitor sees representative substance quickly, understands that the visible examples are not the whole inventory, and gets one explicit expansion path.
- **What ours does:** `course-engine/web/index.html:60-104` — all eight hub cards are presented as the same grid-level unit, with no representative subset and no “these are examples; browse the full course” distinction.
- **Class:** DENSITY
- **Transferable under our constraints?** YES — a CSS overflow row or compact static preview can work offline; the 70vw implementation is not required.
- **Cost:** M (< 1d)
- **Regression risk:** 375 px no-horizontal-overflow and constrained-prose-column KEEP measurements; test any horizontal affordance with keyboard focus and reduced motion.

### G-31-07 · Authored identity and audience context answer “who is this for?”

- **Seen in:** `jeffrey_emanuel_personal_site/lib/content.ts:67-121` and `jeffrey_emanuel_personal_site/components/hero.tsx:145-169` — the hero names the creator’s role, states the flywheel’s origin, gives representative tools, and identifies the “Founder & CEO” context before the main CTAs.
- **What it decides:** A first-time visitor gets audience and provenance context, not only a product noun; the site explains why this person is making the thing.
- **What ours does:** `course-engine/web/index.html:37-45` — “Course hub” and “Local HTTP practice for CDCP topics” establish the artifact, but no concise learner-audience sentence says who should start here or what successful study use looks like.
- **Class:** COPY
- **Transferable under our constraints?** DEGRADED — borrow a short audience-and-use sentence, not a personal biography or marketing claim; the copy must preserve “study signal only.”
- **Cost:** L (needs a design decision a human must make; choose the intended learner and honest promise)
- **Regression risk:** F-23 instruction density and the honesty-banner KEEP; the added context must not become an exam-certification implication.

### G-31-08 · A 3D/live hero is a visual decision, not an admissible implementation here

- **Seen in:** `jeffrey_emanuel_personal_site/components/hero.tsx:22-26` and `jeffrey_emanuel_personal_site/components/hero.tsx:401-418` — the homepage lazy-loads a Three.js scene, uses a loading/fallback boundary, and mounts it only after visibility/idle checks.
- **What it decides:** The hero carries an ambient, interactive visual identity alongside the copy, with explicit loading and failure states.
- **What ours does:** `course-engine/web/index.html:132-137` — the shipped hub declares a static shell, relative assets, no CDN, and local HTTP; no 3D or live-rendered hero exists.
- **Class:** MOTION
- **Transferable under our constraints?** NO — the constraint that kills it is fully offline runtime with static HTML + vanilla JS + one CSS file and no build/CDN/network path; Three.js and the dynamic scene cannot be introduced without violating the assignment. Reduced-motion and the measured dark-theme contrast also remain non-negotiable.
- **Cost:** S (< 1h to reject; no implementation should be opened from this comparison)
- **Regression risk:** Reduced-motion is honoured; dark-theme contrast is measured. Treat the rejection as a guard against importing the wrong proof class, not as a gap to patch.

### G-31-09 · Remote preview failure recovery does not transfer to a local learner hub

- **Seen in:** `jeffrey_emanuel_personal_site/components/featured-sites.tsx:35-57` and `jeffrey_emanuel_personal_site/components/featured-sites.tsx:60-99` — each featured-site card builds a proxy URL for a remote OG image and falls back to a local gradient/icon when the image errors.
- **What it decides:** A remote-content card has a graceful visual fallback instead of collapsing when an external preview is unavailable.
- **What ours does:** `course-engine/web/index.html:60-104` — hub cards use local relative routes and text; there is no remote preview image or external-card recovery state to implement.
- **Class:** RECOVERY
- **Transferable under our constraints?** NO — the constraint that kills the pattern is no network at runtime; adding remote OG previews and a proxy would violate offline operation. The transferable lesson is only “never let a missing optional asset erase the destination,” which the local text cards already satisfy.
- **Cost:** S (< 1h to reject)
- **Regression risk:** 375 px no-horizontal-overflow and no-CDN KEEP constraints; do not add remote image dependencies under a recovery label.

### G-31-10 · Our empty state is more honest and more actionable than a generic CTA

- **Seen in:** `mcp_agent_mail_website/app/page.tsx:109-127` — the hero offers generic “SEE AGENT MAIL IN ACTION” and “GET STARTED” routes, but the homepage source does not define a no-data state or a next action conditional on visitor state.
- **What it decides:** The corpus site sends every visitor to a broad exploration or setup route; it does not claim to know whether work is waiting.
- **What ours does:** `course-engine/web/index.html:47-57` plus `course-engine/web/assets/js/learn_chrome.js:146-165` — the hub’s “Now · 90 seconds” action is paired with a local due-card check, and the empty state says “No cards due. Take a mock or quiz, then come back for a 90-second loop.”
- **Class:** FEEDBACK
- **Transferable under our constraints?** YES — this is already a static-shell/vanilla-JS decision and is the stronger learner-facing pattern.
- **Cost:** S (< 1h to preserve and regression-test)
- **Regression risk:** The KEEP measurement that the “No cards due” empty state names the next action; do not replace it with a blank or generic success state.

### G-31-11 · Our non-certification boundary is clearer than the corpus promise language

- **Seen in:** `mcp_agent_mail_website/lib/content.ts:2-5` — the product title and description make strong infrastructure and coordination claims, but the homepage content model has no equivalent load-bearing learner/certification boundary.
- **What it decides:** The corpus optimizes for a confident product promise; it does not need to distinguish study practice from a professional credential.
- **What ours does:** `course-engine/web/index.html:14-19` — the honesty banner explicitly says “Study tool only,” denies EPI/EXIN certification, and states that practice is not a CDCP credential.
- **Class:** COPY
- **Transferable under our constraints?** YES — keep this exact boundary while borrowing only the corpus’s clarity and hierarchy.
- **Cost:** S (< 1h to preserve)
- **Regression risk:** The load-bearing honesty-banner KEEP: it stays visible, is registered as claim-not-epi-certified, and never receives a certified-green treatment.

## Synthesis boundary

These findings establish decisions visible in three flagship homepages; they do not establish that the corpus’s marketing conversion sequence is the correct learner pedagogy, that any proposed copy improves study outcomes, or that the cited source sites are independently effective. They also do not authorize Wave 2 or implementation.
