# Hero image spec — Operator Yuzu in the data hall

> **The Yuzu Character Bible is the authority, not this file.**
> `~/Developer/zeststream-brand-voice/visual/character-bible.md`
> (status: CANONICAL, locked session 15, 2026-04-20)
>
> This file specifies only the **repo-specific scene**. Character form, face,
> palette, and rendering style are governed by the bible. Where the two ever
> disagree, the bible wins — do not re-derive the character by looking at an
> image, which is exactly how the 2026-08-12 drift happened (see Failure log).

- **Output:** `visual/hero.jpg` (16:9, ≥1600 px wide)
- **Character anchor:** `~/Developer/zeststream-brand-voice/visual/yuzu_canonical.jpg`
  — pass as `--cref` / IP-Adapter / edit-chain reference, never text-only
- **Phase:** **PEEL** (discovery · research · audit · learn) — this repo is a
  study tool, so per the bible's scene pattern it is a PEEL-phase image

## What the bible locks (summarised — read it, don't trust this summary)

| Aspect | Lock |
|---|---|
| Eyes | **Medium-sized** emerald-green, **small** white highlights, slight laugh lines. Senior operator, not fresh apprentice |
| Smile | Warm, subtle, quietly confident |
| Brows | Gentle, often slightly raised asymmetrically |
| Head | Yuzu citrus, slightly bumpy, yellow-green, subsurface scattering; ~40% of figure height; **one** leaf on a short stem |
| Wardrobe | Cream henley rolled to elbows **under** a natural canvas apron with tool pockets |
| Signature prop | Wood-handled clipboard with **visible receipt / score pages** |
| Render | 3D Pixar / DreamWorks CG · subsurface scattering · soft global illumination · **warm rim lighting** · shallow DoF |
| Palette | `#CEE741` peel · `#6B8E23` leaf · `#F5F0E1` cream · `#1A1B1F` ink · `#E8A94B` amber · `#5B7553` sage |

**Bible's banned face variants** (any of these ⇒ off-canon, regenerate):
chibi open smile with teeth · blushing pink cheeks · **wide anime sparkle-eyes**
· stern or angry · eyes closed / winking.

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

**Lesson:** find the canonical spec before authoring a derivative one. A
character bible existed the whole time.

## Generation

`codex exec` on the Studio authenticates via ChatGPT (`~/.codex/auth.json`) and
can generate images. The `OPENAI_API_KEY`, `GEMINI_API_KEY` and `FAL_KEY` in
Infisical all fail auth as of 2026-08-12, so the multi-provider script in
zesttube cannot run.

Always pass `yuzu_canonical.jpg` as a character reference. Text-only prompting
is what allowed the face to drift.
