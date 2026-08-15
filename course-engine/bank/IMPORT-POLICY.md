# Bank import policy — one pool, one gate

**Decision, 2026-08-14 (bd-tetz).** Recorded because 25 near-duplicate pairs were not 25
authoring slips. They were the predictable output of two bulk additions into an ungated pool.

## The decision

**Practice content may keep entering `bank/items`. There is no second pool, and no import
quarantine.** What changes is that an import is no longer allowed to land un-adjudicated:

1. **`cdcp_gate near-duplicate-items` is the admission gate for any bulk addition.** Run it
   BEFORE the addition and AFTER it. Every pair the second run flags that the first did not is
   a pair the addition created, and it is adjudicated in the same commit that adds the items.
   An import that leaves the gate RED is not landed; it is not "landed with a backlog".
2. **When an import copy duplicates a module-bank item, the IMPORT copy is retired** — unless
   the published seed-42 form actually draws it, in which case the form wins and the other copy
   goes. This is mechanical on purpose. Deciding 24 pairs one at a time is how the class was
   allowed to grow in the first place.
3. **Retire, never delete.** The bank is content-addressed; the retired file is the only
   surviving record that the duplication happened, and its header carries the argument.
4. **Every retirement gets a row in `cdcp_bank::SANCTIONED_RETIRED`** with a real reason. The
   allowlist is checked in both directions, so an unexplained retirement and a stale row are
   both RED.

## Why a second pool was rejected

Splitting practice content into its own pool would stop the cross-pool duplicates and stop
nothing else. The 24 pairs adjudicated under bd-tetz split 18 / 6: eighteen put a `mock40-*`
import against a module-bank item, and **six were module-bank against module-bank** — the
`q2xx` "bank expansion (depth/apply)" wave duplicating items the same bank already held. A
separate pool would not have caught one of those six. The defect is an ungated bulk write, not
the particular source directory it came from, and a second pool would additionally have to
answer which pool a mock form samples — reintroducing the assessment question the single pool
already answers with `status`.

## The two bulk additions that produced the class

| Wave | Marker | What it added | Pairs it created (of 24) |
|------|--------|---------------|--------------------------|
| `practice/PRACTICE-EXAM.md` import | `id = "mock40-*"`, first-line comment `imported from practice/PRACTICE-EXAM.md` | 40 items, filenames numbered by practice-exam question order | 18 |
| `q2xx` bank expansion | ids ending `-q2xx`, first-line comment `bank expansion (depth/apply)` | the depth/apply expansion | 6 |

`bank/MANIFEST.toml` already names both: `generated_from = ["../practice/PRACTICE-EXAM.md",
"generated-expansions"]`. It records that they happened; it did not gate them.

## What this policy does NOT claim

It does not claim the pool now holds N distinct propositions, and nothing here should be read
as certifying the item count. `near-duplicate-items` measures textual closeness of the correct
answer; a **paraphrased** duplicate — same proposition, freshly worded answer — passes it
silently. Four such pairs are listed in `registries/paraphrase_pairs.toml` and asserted by
`cdcp verify-paraphrase-pairs` (`cdcp_bank::paraphrase`, bd-e1yt / jhd.21): deleting a listed
pair without an adjudication reason is RED. The command also prints a stem-overlap REPORT of
candidates; that report is not a grader-of-record and does not retire items. A green C3 verdict
means "no cosmetic duplicates in the approved pool". It is a floor, not a census. 804/779 is a
pool size.

## Module vs `topic_ids` prefix (bd-mock40-q37-cross-module-topic-76vs)

**Decision, 2026-08-15.** A `topic_ids` prefix that does not match `module` is **legal**.
Topics may name a secondary domain. `mock40-q11` is module 4 (removing a floor tile) and
still cites `m09-raised-floor-cooling`. A gate that required every topic prefix to equal
`module` would have **passed** `mock40-q37` (it carried `m13-safety-components` next to
the MOP stem) and **failed** those cross-cuts. Prefix equality is the wrong axis.

What is **not** legal: an *approved* item whose `module` disagrees with the stem's
subject. The practice-exam import numbered files by question order and copied that
number into `module`. That is a mechanism, not a slip.

Scan, 2026-08-15, every `mock40-*` item, stem vs `module` (not filename):

| | n |
|---|---|
| checked | 40 |
| content-wrong | 1 (`mock40-q37`: MOP / module 15, filed as 13) |
| live (approved) misfiles | 0 |

`mock40-q37` stays retired. The misfile stays on the file (`module = 13`) so the
import numbering defect remains visible. The machine check is
`cdcp_bank::mock40_module_audit` / `MOCK40_CONTENT_MODULE`: zero `mock40-*` items
is ERROR; a new `mock40-*` id without a ledger row is RED; an approved item whose
`module` ≠ stem-subject is RED. This is not a `cdcp_gate` row — prefix
disagreement is legal, so it does not belong in `verify-orphans`.
