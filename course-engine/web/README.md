# CDCP Study — browser surface (`web/`)

The hub is a **local HTTP** product. It is **not** a `file://` page.

- **Supported:** `cargo build -p cdcp_cli --locked` then `./target/debug/cdcp serve` (default `http://127.0.0.1:8766/`). No public network required.
- **Unsupported:** double-clicking `index.html`. That origin fails closed as **`CDCP_FILE_ORIGIN`** — browsers block ES modules, `fetch()`, and WASM from `file://`.

Static HTML + CSS + vanilla JS (+ WASM grade). **No** React / Next / Tailwind / CDN.

This is a **study tool only**. It does **not** grant EPI/EXIN certification. Completing practice is not a CDCP credential.

## Layout

| Path | Role |
|------|------|
| `index.html` | Hub — Learn · Drill · Mock · Reference |
| `learn.html` | Curriculum module hub (L5-S6) |
| `learn/{id}.html` | Per-module reader (markdown via learn_md.js) |
| `content/modules/{id}.md` | Shipped note copies (from `cdcp build-learn`) |
| `reference.html` | Glossary + power/redundancy cheatsheet (L7-S4) |
| `content/reference/*.md` | Shipped parent `reference/` copies (`cdcp build-reference`) |
| `diagrams/power-path.html` | Interactive N vs 2N power path self-check (V11-S2) |
| `diagrams/security-layers.html` | Perimeter → white space layers + fail-safe/fail-secure door toggle (bd-1sd.9.2) |
| `runbooks.html` | Runbook vignette stems → quiz/mock links (V11-S4) |
| `data/modules_index.json` | Machine index of domains / hrefs |
| `assets/js/learn_*.js` | Progress (localStorage), markdown render, reader |
| `quiz.html` | Module quiz 8–12 items (L5-S7) |
| `drill.html` | Missed + short-interval review; `?mode=due` Drill-10 · `?mode=miss` Miss-review (L6-S6) |
| `mock.html` | 40Q mock take flow (L5-S3; multi-seed L6-S5; closed-notes L7-S1) |
| `results.html` | WASM grade results (score, study signal, weak modules, digest) |
| `assets/css/course.css` | Design tokens + exam UI |
| `assets/js/mock.js` | Take flow, sessionStorage, timer |
| `assets/js/grade_bridge.js` | WASM glue |
| `assets/js/results.js` | Results: loadWasm + gradeDigest, keys explanations, missed feed |
| `assets/js/quiz.js` | Module sample + WASM/key-compare grade |
| `assets/js/drill.js` | Mode-aware missed stems + Drill-10 due queue |
| `assets/js/review.js` | Persist/render for 1d/3d ladder; law is `cdcp_schedule` via WASM |
| `assets/js/schedule_bridge.js` | WASM glue for interval + mastery thresholds |
| `assets/js/mastery.js` | Persist/render practiced / mastered; thresholds from WASM |
| `assets/wasm/cdcp_wasm.wasm` | Built via `./scripts/build_web_wasm.sh` |
| `data/mock40_seed{N}.json` | Learner packs (stems + choices, no keys); every seed is a sampler run (no implicit fixture) [[fact:fact-export-web-implicit-fixture-at-seed-42=no]] |

See [`data/README.md`](data/README.md) for export-web packs and answer-key policy.

## Serve (required — this is the product)

`file://` is **CDCP_FILE_ORIGIN** and is not a supported origin. ES modules, `fetch()` of `data/*.json`, and WASM grading all need a loopback HTTP server:

```bash
# Documented path (V11): from course-engine root — loopback default, no auth
cargo build -p cdcp_cli --locked
./target/debug/cdcp serve
# open http://127.0.0.1:8766/
```

Prove the fail-closed: `node scripts/smoke_file_origin.mjs` (from `course-engine/`).

Threat model: static files only, **no auth / no TLS**. Default bind is localhost-only (`127.0.0.1:8766`).

## Smoke steps (L5-S3 mock take)

1. **Hub links**  
   Open `http://127.0.0.1:8766/` → click **Learn**, **Drill**, **Mock**, **Reference** (nav and cards). Each should load with the amber honesty banner and relative assets (no CDN).

2. **Load pack**  
   On Mock, status should clear and show **question 1/40** with four A–D choices. Progress reads `1 / 40`. Timer counts down from `60:00`.

3. **Keyboard answer**  
   Press `A`/`B`/`C`/`D` (or `1`–`4`) to select a choice. Selection highlights and persists when you leave the question. Use `→` / `N` and `←` / `P` to move. Jump chips at the bottom jump to any item.

4. **sessionStorage draft**  
   Answer a few items, refresh the page. Draft answers and progress restore from `sessionStorage` key `cdcp_mock_draft_v1`.

5. **Submit gate**  
   Submit stays **disabled** until all 40 items have a chosen letter. Hint text tracks answered count.

6. **Submit → attempt**  
   After all 40 answered, click **Submit attempt**. Browser navigates to `results.html`, which reads `cdcp_mock_attempt_v1`, grades via WASM, and shows score, study signal, weak modules, digest, and per-item keys. Wrong item_ids are written to Drill/SRS localStorage.

7. **Attempt shape** (DevTools → Application → sessionStorage → `cdcp_mock_attempt_v1`):

   ```json
   {
     "exam_id": "mock40",
     "seed": 42,
     "bank_hash": "<64-hex>",
     "answers": [
       { "item_id": "bank-m15-q142", "chosen": "B" }
     ]
   }
   ```

   Exactly 40 `{item_id, chosen}` rows when fully completed. `chosen` is `"A"` | `"B"` | `"C"` | `"D"`.

8. **Timer soft expiry**  
   Timer turns amber under 5 minutes and red at `00:00`. Submit remains available after expiry (study tool — not a hard lock).

9. **Honesty**  
   Banner always visible; no green credential treatment; no EPI/EXIN certification claims.

10. **Closed-notes mode (L7-S1)**  
    Toggle **Closed notes** in the mock toolbar (preference: `sessionStorage` key `cdcp_mock_closed_notes_v1` = `"1"` / `"0"`).
    While an attempt is active (pack loaded, not yet submitted):

    - **Learn / Drill / Reference** hub links are locked (disabled + strikethrough; clicks show a brief status message — turn the toggle off to study).
    - Module readers, quiz, and future reference/cheatsheet URLs are treated the same.
    - Leaving via Hub, brand, seed change, refresh, or tab close shows a **soft warning** (`confirm` / `beforeunload`). Cancel keeps you on the mock; draft answers remain in `cdcp_mock_draft_v1`.
    - **Submit attempt → `results.html` still works** — submit sets an allow-leave flag so grading is not blocked.
    - Toggle off anytime to unlock study nav without losing the draft.

    Closed-notes is a study aid only. It does **not** claim proctoring, exam integrity, or EPI/EXIN certification.

## sessionStorage keys

| Key | When | Payload |
|-----|------|---------|
| `cdcp_mock_draft_v1` | During take | answers map, index, timer start |
| `cdcp_mock_attempt_v1` | After submit | `ExamAttempt` JSON for S4 grade |
| `cdcp_mock_closed_notes_v1` | Mock mode pref | `"1"` = closed notes preferred; `"0"` / absent = open |
| `cdcp_quiz_draft_v1` | During module quiz | module, item_ids, answers, index |

## localStorage keys (Learn + Drill / short-interval review + Mastery)

| Key | When | Payload |
|-----|------|---------|
| `cdcp.learn.visited.v1` | Learn hub/reader | `string[]` module ids |
| `cdcp.drill.missed.v1` | After mock/quiz grade | missed feed (see schema below) |
| `cdcp.srs.v1` | After wrongs + reviews | review cards map (historical key name; law is not SRS) |
| `cdcp.mastery.v1` | After module quiz grade | per-module quiz attempts (see schema below) |

### Missed feed schema (`cdcp.drill.missed.v1`)

Written by `results.js` (mock) and `quiz.js` (module quiz) via `review.recordGradedWrongs`.

```json
{
  "schema_version": 1,
  "source": "mock",
  "exam_id": "mock40",
  "seed": 42,
  "bank_hash": "<64-hex>",
  "saved_at": 1700000000000,
  "item_ids": ["bank-m12-q041", "m07-q209"]
}
```

- `source`: `"mock"` | `"quiz"`
- `item_ids`: incorrect item ids from the last graded attempt (overwrites previous feed)
- Drill lists these ids, loads stems from `data/bank_items_seed42.json`, flips explanations from keys/bank

### Review schema (`cdcp.srs.v1`)

Short-interval ladder: **1 day → 3 days** (cap). Law is `cdcp_schedule::next_interval_days` via WASM. Not SRS.

```json
{
  "schema_version": 1,
  "cards": {
    "bank-m12-q041": {
      "item_id": "bank-m12-q041",
      "interval_days": 1,
      "due_at": 1700086400000,
      "reps": 0,
      "lapses": 1,
      "updated_at": 1700000000000
    }
  }
}
```

| Field | Meaning |
|-------|---------|
| `interval_days` | Last scheduled step (`1` or `3`; `0` = never stepped) |
| `due_at` | Epoch ms when card is due |
| `reps` | Successful reviews |
| `lapses` | Failed reviews / re-misses |

**Interval law**

| Current | Outcome | Next |
|---------|---------|------|
| 0 / new | wrong or correct | 1d |
| 1d | correct | 3d |
| 3d | correct | 3d (cap) |
| any | wrong | 1d |

Missed items from mock/quiz are scheduled at 1d (re-miss resets to 1d and increments `lapses`). State survives reload. Tests: `node scripts/smoke_srs.mjs` (loads WASM).

### Mastery schema (`cdcp.mastery.v1`)

Written by `quiz.js` after every graded module quiz (WASM or key-compare path) via `mastery.recordQuizResult`. **Study signal only** — practiced / mastered never means EPI/EXIN certified.

```json
{
  "schema_version": 1,
  "modules": {
    "6": {
      "module": 6,
      "attempts": [
        {
          "correct": 9,
          "total": 10,
          "ratio": 0.9,
          "at_ms": 1700000000000
        }
      ],
      "best_ratio": 0.9
    }
  }
}
```

| Field | Meaning |
|-------|---------|
| `module` | Bank module number (same as `quiz.html?module=N`) |
| `attempts[]` | Chronological quiz outcomes |
| `attempts[].ratio` | `correct / total` clamped to [0, 1] |
| `attempts[].at_ms` | Epoch ms of the attempt |
| `best_ratio` | Max ratio across attempts |

**Laws**

| State | Rule |
|-------|------|
| **practiced** | `best_ratio ≥ 0.80` |
| **mastered** | ≥2 attempts with `ratio ≥ 0.90` whose timestamps are **≥ 24h** apart |

- A single 90% score is practiced, not mastered.
- Two 90% attempts on the same day (&lt;24h gap) stay not mastered.
- Spacing uses fixed `DAY_MS = 86400000` (no DST).
- Pure tests: `node scripts/smoke_mastery.mjs`.

## Module quiz (L5-S7)

```bash
# From course-engine/
cargo build -p cdcp_cli --locked
./target/debug/cdcp serve
# open http://127.0.0.1:8766/quiz.html?module=6
# or Learn → module page → "Module NN quiz"
```

- Filters `data/bank_items_seed42.json` by numeric **`module`** field (`BankItem.module`).
- Samples **8–12** items (or all if the module pool is smaller than 8); deterministic seed `42 + module*1000`.
- **Grading:** prefers WASM `grade_bridge.gradeDigest` (same GradeExact letter law as mock). If WASM fails to load, falls back to **key-compare pedagogy score only** — no invented GradeExact digest, no cert claim.
- Wrong item_ids → `cdcp.drill.missed.v1` + short-interval review schedule.
- Score → `cdcp.mastery.v1` (`recordQuizResult`) for practiced / mastered study state.
- **No LLM grading.**

## Drill / short-interval review (L5-S7)

```bash
# After a mock with wrongs (or a quiz):
open http://127.0.0.1:8766/drill.html
# Missed list shows item_ids + stems; flip for explanation from keys/bank.
# Due queue: Again (1d) / Good (next step). Reload keeps cards.
```

Headless interval + mastery smoke:

```bash
# From course-engine/
node scripts/smoke_srs.mjs
node scripts/smoke_mastery.mjs
```

## Learn surface (L5-S6)

```bash
# From course-engine/: copy notes + regenerate hub/pages/index
cargo build -p cdcp_cli --locked
./target/debug/cdcp build-learn
./target/debug/cdcp smoke-learn

# Serve web/ (fetch needs http)
./target/debug/cdcp serve
# open http://127.0.0.1:8766/learn.html
```

- Hub lists all `knowledge/domains.toml` domains; empty `primary_notes` with
  `exam_weight_unknown` (ops-adjacent) is listed without a link.
- Module pages use relative assets only; honesty banner on every page.
- Each module page links to `quiz.html?module={order}`.
- Visited modules stored in `localStorage` key `cdcp.learn.visited.v1`.
- If content copies are missing, `learn_reader.js` tries parent-corpus
  relative paths when the monorepo root is the static-server CWD.

## Reference / cheatsheet (L7-S4)

```bash
# From course-engine/: copy parent reference/*.md + regenerate panel
cargo build -p cdcp_cli --locked
./target/debug/cdcp build-reference

# Serve web/ (fetch needs http)
./target/debug/cdcp serve
# open http://127.0.0.1:8766/reference.html
# tabs: #glossary · #power
```

- Sources: parent `reference/GLOSSARY.md` and
  `reference/POWER-AND-REDUNDANCY-CHEATSHEET.md`.
- Shipped copies under `web/content/reference/`; parent practice/module links
  rewritten to in-app hrefs at build time.
- Same course.css shell + amber honesty banner; no CDN.

## Results / WASM grade (L5-S4)

```bash
# From course-engine/
./scripts/build_web_wasm.sh
cargo build -p cdcp_cli --locked
./target/debug/cdcp serve
# open mock → submit → results, or inject all-correct attempt in console:
#   const keys = await (await fetch('data/keys_seed42.json')).json();
#   sessionStorage.setItem('cdcp_mock_attempt_v1',
#     JSON.stringify(CdcpResults.buildAllCorrectAttempt(keys)));
#   location.href = 'results.html';
# window.__cdcp_last_digest should equal goldens/mock40_seed42_all_correct.sha256
```

If WASM fails, results show a clear error and **do not** invent scores in JS. CLI fallback only:

```bash
cargo build -p cdcp_cli --locked
./target/debug/cdcp grade \
  --fixture goldens/fixtures/mock40_seed42.json --mode all-correct
```

## Non-goals (remaining)

- Digest match e2e harness in check.sh (S5)
- Full Anki export / multi-device SRS sync
