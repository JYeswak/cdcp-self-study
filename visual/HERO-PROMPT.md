# Hero image spec — Operator Yuzu in the data hall

> ## ⚠️ THE CANONICAL YUZU IS IDENTIFIED BY HASH, NOT BY FILENAME
>
> ```
> canonical: ~/.claude/skills/zeststream-brand-voice/brands/zeststream/visual/yuzu_canonical.jpg
> sha256:    52fb1b09922f9892e53e290279253e07eeb2a6452dc4464c10efbbfe9891f918
> locked:    2026-05-17T04:55:45Z   grader: yuzu_identity_grader.v1   threshold: 70
> ```
>
> **There are at least two different images named `yuzu_canonical.jpg`.** The one at
> `~/Developer/zeststream-brand-voice/visual/` (sha `b1a726c7…`) is NOT canonical and
> produced three rejected heroes. **Verify the sha before using any reference.**
>
> Authorities, in order:
> 1. **The hash above** — `identity-grades-all.json` names it as `canonical_path`
> 2. **`yuzu-anchor-library-v1`** — 12 approved assets graded against that hash, incl.
>    `archetype-teacher.png` (80.4), the pose exemplar for this repo
> 3. `character-bible.md` — useful for wardrobe and palette, but its FACE prose
>    ("medium eyes", "gentle eyebrows") describes a **different image**. The true
>    canonical has **NO eyebrows** and large round eyes. When the bible and the
>    hash disagree, **the hash wins**.
>
> **Forever-rule** (from `yuzu_identity_grader.py`): *every Yuzu candidate touching
> the anchor library MUST pass this grader.* Run it before shipping any Yuzu.


- **Output:** `visual/hero.jpg` (16:9, ≥1600 px wide)
- **Character anchor:** the sha-`52fb1b09…` file named in the banner above
  (`~/.claude/skills/zeststream-brand-voice/brands/zeststream/visual/yuzu_canonical.jpg`)
  — pass as `--cref` / IP-Adapter / edit-chain reference, never text-only.
  **Verify the sha before use.** Do NOT use the same-named file under
  `~/Developer/zeststream-brand-voice/visual/` — wrong character.
- **Pose exemplar:** `archetype-teacher.png` from `yuzu-anchor-library-v1`
  (approved, identity 80.4)
- **Phase:** **PEEL** (discovery · research · audit · learn) — this repo is a
  study tool, so per the bible's scene pattern it is a PEEL-phase image

## Character locks — from the sha-verified canonical, NOT from prose

⚠️ The rows marked **HASH** override the character bible, whose face prose
describes a different image.

| Aspect | Lock |
|---|---|
| **Brows** | **HASH: NONE. The canonical Yuzu has NO eyebrows.** The bible says "gentle eyebrows" — that is the wrong file. Three heroes were rejected for drawing brows |
| **Eyes** | **HASH: LARGE and ROUND**, generous white sclera, green iris with a darker outer ring, big dark pupil, one white catchlight **plus a warm amber secondary glow**. The bible's "medium-sized… small highlights" is the wrong file |
| **Cheeks** | HASH: soft warm blush where the rind warms to orange-tan low on the face |
| Smile | Small, subtle, closed-mouth |
| Head | Yuzu citrus, slightly bumpy, yellow-green, subsurface scattering; ~40% of figure height; **one** leaf on a short stem |
| Wardrobe | Cream henley rolled to elbows **under** a natural canvas apron with tool pockets |
| Signature prop | Wood-handled clipboard with **visible receipt / score pages** |
| Render | 3D Pixar / DreamWorks CG · subsurface scattering · soft global illumination · **warm rim lighting** · shallow DoF |
| Palette | `#CEE741` peel · `#6B8E23` leaf · `#F5F0E1` cream · `#1A1B1F` ink · `#E8A94B` amber · `#5B7553` sage |

**Bible's banned face variants** — treat with care, two of these conflict with
the hash-verified canonical: chibi open smile with teeth · ~~blushing pink
cheeks~~ (the canonical HAS warm cheeks) · ~~wide anime sparkle-eyes~~ (the
canonical HAS large round eyes with catchlights) · stern or angry · eyes closed
or winking. **Where the bible and the hash disagree, the hash wins.**

**Bible's banned palette:** corporate SaaS blue · pure magenta · **cyberpunk
cyan** · neon purple gradient · Jeff's blue-robot colours.

## Scene (repo-specific — this is the only part this file owns)

Per the bible: *"Yuzu always does the thing the repo/tool does. The work IS the
image."* This repo teaches someone to walk a data hall and explain what they
see, so Yuzu is doing exactly that.

Operator Yuzu stands in a **data-hall aisle**, mid-gesture toward a rack,
clipboard in the other hand, in the posture of explaining something to you.

- **Setting:** believable working data hall. Perforated raised-floor tiles, one
  tile lifted to show the plenum and underfloor cabling. Racks with tidy
  structured cabling and cable-management arms.
- **Light:** **PEEL phase — early morning / dawn.** Warm amber (`#E8A94B`) key
  light raking down the aisle, as if from a doorway or high window at the end of
  the room. Soft global illumination. Subtle warm rim on Yuzu per the bible.
- **Palette discipline (the hard part):** a real data hall is full of blue LEDs
  and cool white light — both **banned**. Render the room in **warm neutral
  greys and sage** (`#5B7553`), amber accents, ink-black rack frames. Indicator
  LEDs warm amber/green, never blue or cyan. Cabling in sage/cream/amber, never
  the usual blue.
- **Clipboard:** a site-inspection sheet with visible tick marks — the bible's
  "visible receipt / score pages" requirement, in this repo's domain.

## Avoid

- **A glowing halo or luminous outline around Yuzu.** The bible calls for *warm
  rim lighting* — subtle cinematic separation, not a light source behind his
  head. See Failure log.
- Blue / cyan LEDs, blue cabling, cool white room light, neon, holograms
- Hard hat or hi-vis vest — the canvas apron IS the character
- Readable vendor names, brands, or logos on hardware
- **Any text asserting certification.** No "CDCP", no "Certified", no diploma,
  no mortarboard. This repo certifies nobody; the hero must not imply otherwise
- Faces of real people, or any second character

## Identity grade of the shipped hero

| | phash | palette | score | verdict |
|---|---|---|---|---|
| `visual/hero.jpg` (v4) | 27 | 0.862 | **46.9** | below 70 |
| `archetype-teacher.png` — **approved, officially 80.4** | 34 | 0.922 | **55.7** | below 70 |

**Neither clears 70 under `--judge phash_only`, including a known-approved asset.**
The 70 threshold assumes the gpt-4o-mini vision judge, which could not run:
`OPENAI_API_KEY` returns 401 (so do `GEMINI_API_KEY` and `FAL_KEY`). pHash also
penalises framing — the canonical is a square head-and-shoulders portrait, these
are 16:9 full-body scenes.

**Open action:** re-grade with `--judge openai` once a working key exists. Raw
results: `visual/hero-identity-grade.json`.

```bash
cd ~/Developer/zesttube && python3 scripts/yuzu_identity_grader.py \
  --canonical ~/.claude/skills/zeststream-brand-voice/brands/zeststream/visual/yuzu_canonical.jpg \
  --candidates <dir> --output <out.json> --judge openai --threshold 70
```

## Failure log (2026-08-12) — read before regenerating

1. **Halo.** This spec asked for "warm rim light so the character stays warm
   against a cool room" and the model rendered a literal glowing outline.
   The bible does want warm rim lighting; it does not want a halo.
2. **Wide anime eyes.** The generated face had large round eyes with heavy white
   sclera — an explicitly **banned** variant in the bible. Root cause: this spec
   was written by *looking at* `yuzu_canonical.jpg` and describing it, instead of
   reading the character bible that already defined the face. The bible was not
   discovered until Josh rejected the second attempt.
3. **Cool-blue palette.** This spec called for "cool greys and blues", which is
   in the bible's NEVER list. A data hall's natural palette fights the brand;
   the brand wins.

4. **Wrong canonical file (the root cause of all three).** Two images share the
   filename `yuzu_canonical.jpg`. I used the one under `~/Developer/...`
   (sha `b1a726c7…`); the graded canonical is under `~/.claude/skills/...`
   (sha `52fb1b09…`). They are different characters — the real one has **no
   eyebrows**. Every failed attempt drew heavy dark brows because the character
   bible's prose describes the wrong file.

**Lesson:** identity is a *hash*, not a filename and not a description. A grader,
a locked sha, a threshold, and 25 recorded grades all existed the whole time.
Verify the sha, then run the grader — do not eyeball it, and do not trust prose.

## Generation

`codex exec` on the Studio authenticates via ChatGPT (`~/.codex/auth.json`) and
can generate images. The `OPENAI_API_KEY`, `GEMINI_API_KEY` and `FAL_KEY` in
Infisical all fail auth as of 2026-08-12, so the multi-provider script in
zesttube cannot run.

Always pass `yuzu_canonical.jpg` as a character reference. Text-only prompting
is what allowed the face to drift.
