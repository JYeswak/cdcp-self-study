# Reading-pass template — read-only sources → citation registry

**Use this while a read-only standard is open in the viewer.** It captures everything the
evidence layer needs and nothing that would violate the terms you accepted.

> ## ⛔ Per-publisher AI rules differ. Check before you read.
>
> | Publisher | Read-only? | **AI processing** | Lane |
> |---|---|---|---|
> | **NFPA** | yes — no download/print | not specifically restricted | may use this template with AI assistance |
> | **ASHRAE** | yes | **PROHIBITED** — no entry of ASHRAE IP into any AI tool; no AI-created derivative works without written permission | **HUMAN-ONLY. Fill this in by hand. Do not paste ASHRAE text into any agent.** |
> | **Uptime / TIA / BICSI / EN** | paid | assume prohibited | cite by clause only |
> | **US Gov (NIST, OSHA/eCFR, NOAA…)** | public domain | unrestricted | quote freely, AI-process freely |
>
> "Read-only" is not one permission. ASHRAE's terms add an AI restriction NFPA's do not.
> A locator (`ASHRAE 90.4-2025 §x.y`) is a fact about a document and remains recordable;
> the *content* may not be entered into a tool.

## The rule, once

| Capture | Why it is fine |
|---|---|
| Standard number, edition year, clause locator | facts, not expression |
| **Your own paraphrase** of the requirement | your words |
| Whether one of our items agrees, conflicts, or is unsupported | our analysis |
| Scope boundaries — what the standard says it does NOT cover | usually stated in §1, and it's the highest-value line in any standard |

| Never capture | Why |
|---|---|
| Verbatim body text, tables, figures | copyrighted expression; the repo is **public** |
| Screenshots of pages | same |
| Any bulk extraction of the viewer | violates the terms accepted on open |

Three ASHRAE PDFs were purged from this repo's HEAD **and history** on 2026-08-12 for exactly
this reason. Do not recreate that problem on a public repo.

---

## Per-standard header

```
authority:      NFPA
document:       75
title:          Standard for the Fire Protection of Information Technology Equipment
edition:        2024
access:         read-only (free access viewer)
read_by:        <your name>
read_at:        <ISO date>
scope_verbatim_short: "protection of ITE and ITE areas from fire damage by fire or its
                       associated effects — smoke, corrosion, heat, and water"
scope_excludes: <what §1 says is out of scope — capture this, it is the most useful line>
```

## Per-claim row (one per requirement that touches our bank)

```
citation_id:    nfpa75-2024-<clause>
clause_locator: §<n.n.n>
subject:        <one line, your words — e.g. "suppression agent selection for ITE areas">
paraphrase:     <your words. NOT the standard's sentence.>
our_items:      [m12-qNNN, ...]        # items this governs
verdict:        supported | conflicts | unsupported | out-of-scope
note:           <if conflicts: what our item says vs what the clause requires>
confidence:     EXTRACTED   # you read it directly — this is the one place 1.0 is honest
```

`verdict` meanings, kept strict:
- **supported** — our item's claim is consistent with the clause
- **conflicts** — our item asserts something the clause contradicts → **file a bead**
- **unsupported** — our item makes a normative claim with no clause behind it → not proven
  wrong, but not groundable either. This is the expected result for most of the 60 uncited
  module-12 items.
- **out-of-scope** — the clause does not reach our claim; record so nobody re-checks

---

## Priority queue for NFPA 75 (module 12 has 63 items, **0** currently cite it)

1. Scope + exclusions (§1) — capture what it does not cover
2. ITE area construction / fire-resistive separation
3. Detection requirements
4. Suppression — agent selection, and **water as a damage vector** (in scope per the title line)
5. Emergency response, orderly shutdown, recovery
6. Housekeeping / combustible controls

Then run the same pass for **70E** (arc flash, LOTO — the safety layer the plant model may
never compute), **110** (genset classes/testing), **111** (UPS/battery), **37** (generators),
**3 / 4** (Cx and integrated systems testing — the bank has 6 commissioning items total).

---

## What this produces

Rows here become the citation-registry backbone: `{citation_id, authority, edition,
clause_locator, verified_by, verified_at, access}`. That is **evidence conformance** — it
proves a human checked the locator. It does **not** prove the claim is true, and must never
be described as a factual oracle (review rounds 2 and 3 both killed that framing).

`conflicts` rows are the highest-value output: each one is a defect in a live bank item,
found against an authority we do not control.
