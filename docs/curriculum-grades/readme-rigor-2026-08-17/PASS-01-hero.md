# PASS 01 — Hero / one-liner / badges

**Bead:** `bd-readme-public-rigor-8y0r` · **Skill:** `readme-writing` · **Date:** 2026-08-17
**Scope:** `README.md:1-36` (title, hero image, badge block, one-liner, not-a-cert paragraph, Run-it block).
**Ledger only.** No README rewrite. No `cargo`. No `CHARTER.md` edit. No `br close`. No `ntm send --all`.

> Completing this program does not certify anyone. 27/40 is a study signal, not a pass mark.

**Oracle for this pass:** the tree, measured. Where `course-engine/docs/FEATURE_SURFACE.md` already states a number, I confirmed it independently rather than inheriting it.

---

## Measurement baseline (run from repo root)

```bash
# modules
ls modules/*.md | wc -l                                    # -> 15
wc -w modules/*.md | tail -1                               # -> 81860 total
grep -c '^\[\[domain\]\]' course-engine/knowledge/domains.toml   # -> 15

# bank
ls course-engine/bank/items/*.toml | wc -l                 # -> 846
grep -h '^status = ' course-engine/bank/items/*.toml | sort | uniq -c
#   -> 821 status = "approved"   /   25 status = "retired"
```

| Quantity | FEATURE_SURFACE | Measured | Agree? |
|---|---|---|---|
| Modules | 15 (`docs/FEATURE_SURFACE.md:19`) | **15** | ✅ |
| Bank files | 846 (`:9`) | **846** | ✅ |
| Approved | 821 (`:9`) | **821** | ✅ |
| Retired | 25 (`:9`) | **25** | ✅ |

FEATURE_SURFACE is accurate. **The README is the drifted surface.**

---

## H1 — One-liner says "Fourteen modules" (CRITICAL)

**Section:** `README.md:20`, sentence 2.

**Current:**
> Fourteen modules of original writing (~54,000 words) covering the publicly advertised EPI® CDCP® syllabus domains, an 804-item question bank (779 approved — a **pool size**, not a count of distinct propositions), …

**Three false numbers in one clause.** Measured: **15** modules, **~82,000** words, **846/821**. The README contradicts *itself* at `:44` ("15-module … curriculum") and `:103` ("~62k words" — a **third** word count).

**Exact replacement** (`README.md:20`, full paragraph):

```markdown
**A free, offline, self-hosted course that teaches the data-centre facilities domain — and a Rust engine that grades you the same way twice.** Fifteen modules of original writing (~82,000 words) — the fourteen publicly advertised EPI® CDCP® facility domains plus one ops-adjacent supplement — an 846-item question bank (821 approved — a **pool size**, not a count of distinct propositions), and a browser course whose grader is a pure-Rust core compiled to WASM, pinned so the native and browser paths produce **byte-identical** result digests. No account, no telemetry, no network at runtime, no LLM in the grading path.
```

Why "fifteen … plus one ops-adjacent supplement" and not just "fifteen": the 14/15 split is load-bearing honesty — Module 15 is **not** an EPI facility domain and carries `exam_weight_unknown = true`. Flattening to "fifteen EPI domains" would manufacture a claim EPI's published outline does not support.

**Measure command:**
```bash
ls modules/*.md | wc -l && wc -w modules/*.md | tail -1 && \
  ls course-engine/bank/items/*.toml | wc -l && \
  grep -h '^status = ' course-engine/bank/items/*.toml | sort | uniq -c
```

---

## H2 — Word count is stated twice more, differently (HIGH)

**Section:** `README.md:20` (~54,000) · `README.md:103` (~62k) · measured **81,860**.

`:103` is inside the system-map code block and belongs to whichever pass owns §"System map" — **not this pass**. Recorded here so the two are fixed to the same measured number and not to each other.

**Exact replacement** for `:103` (hand-off, not applied by this pass):
```
├── modules/                  14 EPI domains + ops-adjacent · ~82k words · original writing
```

**Measure command:**
```bash
wc -w modules/*.md | tail -1                                        # 81860  (modules only)
cat modules/*.md practice/*.md reference/*.md | wc -w                # 89204  (whole corpus)
```
Recommend quoting **modules only (~82k)** in both sites, since both sentences describe `modules/`.

---

## H3 — Badge block: verified sound, two deferrals (PASS)

**Section:** `README.md:9-16`. Eight badges. All eight anchors resolve to a real heading; the hero image exists.

| Badge | Line | Claim | Verdict |
|---|---|---|---|
| code MIT | 9 | `./LICENSE` | ✅ file exists, dual-license documented |
| content CC BY-NC-SA 4.0 | 10 | `./LICENSE` | ✅ |
| gate: 85 ordered steps | 11 | `#the-gate` | ⏸ **anchor OK — number is PASS 4** |
| known-bad: 72 injections | 12 | `#gates-proven-to-trip` | ⏸ **anchor OK — number is PASS 4** |
| grading byte-exact | 13 | `#how-grading-works` | ⏸ anchor OK — claim is `claim-grade-byte-exact`, PASS 4 |
| unsafe: forbidden | 14 | safety-dance | ✅ **verified true this pass** |
| offline | 15 | `#running-it` | ✅ anchor OK |
| not a certification | 16 | `#the-honesty-constitution` | ✅ anchor OK |

**`unsafe: forbidden` is honest — measured, not assumed.** All **18** crates carry `#![forbid(unsafe_code)]` and there are **0** `unsafe {` blocks under `crates/*/src/`.

```bash
cd course-engine
for d in crates/*/; do grep -rq 'forbid(unsafe_code)' "$d"src/ || echo "MISSING: $d"; done   # -> no output
grep -rn 'unsafe {' crates/*/src/ | wc -l                                                    # -> 0
ls -d crates/*/ | wc -l                                                                      # -> 18
```

**Anchor check:**
```bash
for a in the-gate gates-proven-to-trip how-grading-works running-it the-honesty-constitution; do
  printf '%s -> ' "$a"; grep -ci "^#.*$(echo $a | tr '-' ' ')" README.md
done   # -> 1 each
ls -la visual/hero.jpg   # -> present, 511242 bytes
```

**No badge change proposed by this pass.** Gate/injection numbers are Pass 4's cell; duplicating them here would put two passes on one claim — the failure mode `registries/doc-facts.toml:75-78` explicitly rejects.

**Optional, deferred to Pass 6:** there is no CI badge, though `.github/workflows/` exists. Adding one is a maintenance commitment (skill anti-pattern: "displaying outdated or broken badges"). Recommend Pass 6 decide.

---

## H4 — Not-a-cert paragraph: KEEP AS IS (PASS)

**Section:** `README.md:22`. No change proposed.

It names the enforcing mechanism (`claim-not-epi-certified`) rather than asserting good intent, which is the strongest form this paragraph can take. It is also externally correct: EXIN's public page lists **"Training: Mandatory"**, so "only the official EXIN/EPI exam after authorised training grants the credential" holds.

**Adjacent defect, NOT this pass:** `README.md:403-405` claims 27/40 is "not affiliated with any official cut score," while EPI's public course page states *"Passing Mark: 27 out of 40 questions."* That is the FAQ section — filed in `docs/curriculum-grades/franken-research-2026-08-17/FINDINGS-OPUS.md` F1. **Do not fix it in this pass**; flagged so Pass 3 (Limitations/FAQ) or a dedicated bead owns it.

---

## H5 — Run-it block: one placeholder, one deferral (MED)

**Section:** `README.md:27-32`.

**Current line 28:** `git clone <this-repo> cdcp-self-study && cd cdcp-self-study`

`<this-repo>` is an unresolved placeholder in a copy-paste block — skill anti-pattern *"all code blocks are copy-paste ready."* `README.md:367` states the repo is public at `github.com/JYeswak/cdcp-self-study`; the local tree's only remote is a temp path, so the URL cannot be confirmed from here.

**Exact replacement** (`README.md:28`) — **apply only after confirming the remote**:
```bash
git clone https://github.com/JYeswak/cdcp-self-study.git && cd cdcp-self-study
```

**Measure command:**
```bash
git remote -v                                  # local tree shows only `engine` -> temp path
grep -n "github.com/JYeswak" README.md         # :367 asserts the public URL
gh repo view JYeswak/cdcp-self-study --json url,visibility   # run to confirm before applying
```

**Deferred to Pass 5 (install path) — do not act here:**
- The skill requires a **curl one-liner above the fold** (Critical Rule 2). There is none; the hero opens with a three-line clone-and-build.
- `cdcp study` **exists in source** (`crates/cdcp_cli/src/main.rs:259,799`), but `bd-installability-sm4g.2` is `in_progress` and its acceptance (port retry, `--no-open`, exit 4 on missing bundle) is unverified. **Do not advertise `cdcp study` in the hero until Pass 5 verifies behavior.** No `cargo` was run this pass, so I make no claim either way.

---

## Apply list — Pass 1

| # | Site | Change | Blocking? |
|---|---|---|---|
| H1 | `README.md:20` | Replace paragraph (fourteen→fifteen, 54,000→~82,000, 804/779→846/821) | **Apply** |
| H2 | `README.md:103` | `~62k` → `~82k` | Hand-off to system-map owner |
| H3 | `README.md:9-16` | No change | — |
| H4 | `README.md:22` | No change | — |
| H5 | `README.md:28` | Resolve `<this-repo>` → real URL | **Apply after `gh repo view`** |

**Not applied by this pass. Ledger only.**

---

## Verification after apply (whoever applies)

```bash
# the three numbers must not reappear anywhere in the hero
sed -n '1,36p' README.md | grep -nE 'Fourteen modules|54,000|804|779' && echo "STILL DRIFTED" || echo "hero clean"
# and the corrected ones must be present
sed -n '20p' README.md | grep -qE 'Fifteen modules.*82,000.*846.*821' && echo "hero measured-true"
```

An empty grep here is the pass condition; a match is a fail. Do not treat "no output" from the second command as success — it must print `hero measured-true`.

*Pass 1 ledger only. No README edit, no cargo, no CHARTER edit, no bead closed, no commit.*
