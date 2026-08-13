# Hero image spec — Professor Yuzu in the data hall

Repo-specific derivative of the canonical ZestStream Yuzu, in the pattern
established by `zeststream-cast-gaps/visual/hero.jpg` (same character, placed in
the repo's domain, evidence motif preserved).

- **Output:** `visual/hero.jpg` (16:9, ≥1600 px wide)
- **Canonical reference:** `~/Developer/zeststream-brand-voice/visual/yuzu_canonical.jpg`
- **Prior derivative:** `~/Developer/zeststream-cast-gaps/visual/hero.jpg`

## Character continuity (non-negotiable — this is the brand anchor)

Yuzu must remain recognisably the same character across every repo:

| Element | Spec |
|---|---|
| Head | Large textured yuzu citrus, yellow-green with a pebbled rind, warm and slightly uneven — not a smooth lemon |
| Stem + leaf | Short brown stem, **one** glossy green leaf angled to the character's left |
| Eyes | Large, round, bright green irises, dark pupils, soft catchlights |
| Brows | Dark, expressive, slightly raised — curious rather than stern |
| Expression | Warm closed-mouth smile. Confident, not smug. Never goofy |
| Body | Humanoid, yellow-green skin, short proportions relative to the head |
| Wardrobe | Cream long-sleeve henley under a canvas/tan work apron with leather straps and tool pockets |
| Prop | Wooden clipboard, always. It is the evidence motif |

## Scene — the data hall

Yuzu stands in a **cold aisle** between two rows of server racks, in the posture
of someone about to explain something, mid-gesture toward the racks.

- **Setting:** modern data hall. Perforated raised-floor tiles underfoot, one
  tile lifted a few inches to show the plenum beneath. Racks with cable
  management arms and neat structured cabling — believable, not a
  chrome-and-neon fantasy.
- **Lighting:** cool white overhead task lighting from the aisle, warm rim light
  on Yuzu so the character stays warm against a cool room. Shallow depth of
  field; racks fall off softly into the background.
- **Palette:** cool greys and blues in the room; Yuzu's yellow-green and the tan
  apron are the only warm notes. This contrast is the point of the image.
- **Clipboard:** the checklist reads as a **site inspection sheet** — legible
  headers, ticks in the margin. Suggest, do not spell out, a hot-aisle /
  cold-aisle containment check. Same handwritten-but-tidy feel as the RECEIPTS
  clipboard in the canonical image.
- **Optional background detail:** a whiteboard at the end of the aisle with a
  simple power-chain sketch (utility → UPS → PDU → rack). Must read as a
  sketch, not as an accurate schematic — it should not assert a claim.

## Style

3D-rendered character in the canonical Pixar-adjacent style — subsurface warmth
in the rind, soft shadows, cinematic but not glossy. The *room* should feel
photographic; the *character* stays stylised. Same treatment as the zscast hero,
where a rendered Yuzu sits in a believable room.

## Avoid

- Blue-neon "cyber" data centre clichés, glowing floors, holograms
- Hard hat or hi-vis vest — the apron is the character; safety costume breaks it
- Any readable brand, logo, or vendor name on racks or hardware
- **Any text asserting certification.** No "CDCP", no "Certified", no diploma,
  no mortarboard. The repo's entire honesty posture is that it does not certify
  anyone; the hero image must not undercut it in a single frame
- Faces of real people, or any other character in frame

## Why this scene

The canonical Yuzu is a craftsman holding receipts — *the work, plus the
evidence for it*. This repo is a study tool whose whole thesis is that a claim
is only worth what its evidence proves. Professor Yuzu in a cold aisle with an
inspection sheet is the same idea in the repo's domain: someone who walks the
floor and writes down what they actually observed.

## Generation

The 4-provider candidate generator lives at
`~/Developer/zesttube/scripts/gen_zf001_hero_candidates.py` (openai
gpt-image-1, google imagen4-ultra, and two others; 2 aspect ratios each).

**Running it costs API spend, so it is Josh's call, not an agent's.** Pick with
rationale, then commit the chosen frame to `visual/hero.jpg` and record the
provider + prompt hash next to it.
