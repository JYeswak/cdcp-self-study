# Q21 — m06 recall-to-apply conversion pilot

Date: 2026-08-20

This is a human-review draft only. No `bank/items` file or learner-pack artifact was changed.
The sample is 12 of the 68 m06 items classified as convertible in the Q20 inventory. The
proposed choices below are drafts for review, not approved item content.

## 1. Pack-freshness gate proof

The gate was exercised against the live tree and an isolated synthetic git repository. The
synthetic repository had one bank commit newer than the three pack files; it was not used to
alter the live tree.

| leg | setup | exit | observed output/result |
|---|---|---:|---|
| known-good | current tree, freshly regenerated pack | 0 | `pack_freshness: bank_files=957 pack_files=3 bank_commit=00406504c56a6a350476837e1735840ee7d93082 bank_epoch=1787251959 pack_commit=54f5916f0167a956f68b8533b857244c7027bd99 pack_epoch=1787253979`; `PASS (freshness only; content correctness remains covered by pack/golden checks)` |
| known-bad | scratch pack commit older than a later scratch bank commit | 2 | `FAIL: learner pack is stale: bank a9026695cd713a3edc3b6c0c5ec08ca5c4507ecd (1787184180) is newer than pack 4b6bd91458dcd5389a575d69c8edea650bee6ac6 (1787184120)`; `FAIL: 1 violation(s)` |
| absent pack | scratch `web/data/keys_seed42.json` moved aside | 4 | `ERROR: missing required learner pack web/data/keys_seed42.json: No such file or directory (os error 2)` |
| empty bank | scratch `bank/items/a.toml` moved aside | 4 | `ERROR: zero bank/items TOML files is an ERROR` |

The gate therefore has a green leg, a named stale-drift red leg, and fail-closed
anti-vacuous legs. It detects freshness, not correctness: a regenerated pack with the wrong
contents can still pass if its recorded freshness relation is current.

## 2. Evaluation rubric

Each draft is judged on three separate questions:

1. Does the proposed scenario leave exactly one defensible answer?
2. Are the distractors plausible to a practitioner who knows the domain, rather than cartoons?
3. Is this a situation a data-centre professional actually encounters, rather than a fact with a
   decorative scenario?

`SHIP CANDIDATE` means it is promising but still needs human ratification. `HOLD` means the
scenario or source is not sufficient to approve it. `WORSE — DO NOT SHIP` is the required
should-fail case.

## 3. Twelve m06 drafts

### m06-q013 — critical power path

Current recall item: “Which sequence best matches a typical critical power path?” The keyed
proposition is `Utility/generator → transfer → UPS → distribution/PDU → rack`.

Proposed apply version: “During a design review, an engineer traces the normal critical-power
path from the incoming source to a rack. Which sequence should the one-line drawing show?”

Proposed choices:

- `Utility/generator → transfer → UPS → distribution/PDU → rack` **(key)**
- `Utility/generator → rack → UPS → transfer → distribution/PDU`
- `CRAH → UPS → rack → transfer → generator`
- `BMS → DCIM → EMS → fuel system → rack`

Grounding: Public syllabus heading “Power distribution / busbar trunking”; ISO/IEC 22237-3:2021,
<https://www.iso.org/standard/78551.html?browse=tc>.

Assessment: one defensible answer; the reversed electrical order is a plausible novice error,
while the other two confuse supporting systems with the electrical path; a one-line design
review is a routine commissioning/design activity. **SHIP CANDIDATE**, subject to replacing
the two weak systems-mix distractors if the reviewer finds them cartoonish.

### m06-q042 — generator-start ride-through

Current recall item: “Which component typically bridges the multi-second gap while a standby
generator starts and transfers?” The key is UPS energy storage.

Proposed apply version: “A generator takes several seconds to start and reach the transfer
condition, but the IT load must not see an interruption. Which function supplies the load during
that interval?”

Proposed choices:

- `UPS energy storage (batteries or equivalent ride-through)` **(key)**
- `A mechanical ATS that only switches once a source is available`
- `An STS that transfers between two sources that are both already live`
- `The generator starting battery, which cranks the engine`

Grounding: Public syllabus heading “UPS systems”; IEC 62040-3:2021,
<https://webstore.iec.ch/en/publication/60140>.

Assessment: one defensible answer; the ATS/STS distinction and generator starting battery are
plausible confusions, and the start/transfer interval is a real operations concern. **SHIP
CANDIDATE** after a human confirms the proposed STS wording matches the site architecture.

### m06-q047 — STS versus mechanical ATS

Current recall item: “Compared with a mechanical ATS, an STS is chosen primarily for:” The key
is fast transfer between live AC sources.

Proposed apply version: “Two in-tolerance AC sources are available and the load specification
requires a sub-cycle transfer when the preferred source fails. Which transfer technology fits
that requirement?”

Proposed choices:

- `A static transfer switch (STS)` **(key)**
- `A mechanical ATS selected for its ordinary source-transfer function`
- `A generator paralleling controller, which synchronizes generation rather than serving as the load-transfer device`
- `A manual maintenance bypass, which requires an operator action`

Grounding: Public syllabus heading “ATS and STS”; IEC 62310-3:2008,
<https://webstore.iec.ch/en/publication/6803>.

Assessment: one defensible answer; mechanical ATS, paralleling control, and manual bypass are
real adjacent concepts a practitioner could confuse, and specifying sub-cycle transfer is a
real design requirement. **SHIP CANDIDATE**.

### m06-q052 — module-level N+1

Current item: “A UPS plant needs 800 kW and one module may be unavailable for maintenance.
Which arrangement meets N+1 at that layer?” The current key already states “enough modules for
the 800 kW load plus one spare module.”

Proposed apply version: “During a planned UPS-module maintenance window, the site load remains
800 kW. Which nameplate plan preserves module-level N+1 while that module is unavailable?”

Proposed choices:

- `Install enough modules to carry 800 kW plus one spare module` **(key)**
- `Install exactly enough modules for 800 kW and take the module offline`
- `Call the arrangement N+1 because two utility services feed the building`
- `Add a second site but leave the local UPS layer without a spare`

Grounding: Public syllabus heading “Power redundancy levels and techniques”; ISO/IEC 22237-3:2021,
<https://www.iso.org/standard/78551.html?browse=tc>.

Assessment: one defensible answer and the maintenance window is realistic, but this is only a
near-paraphrase of the current scenario. It does not add meaningful judgement, and two options
shift to other redundancy layers. **HOLD — do not ship as a conversion; retain the current
item unless a reviewer identifies a genuinely different operational decision.**

### m06-q054 — concurrent maintainability

Current recall item: “Concurrent maintainability as a design intent means:” The key is that one
capacity component can be maintained without interruption.

Proposed apply version: “A maintenance procedure removes one power-capacity component from
service while ICT load remains online. Which claim is the design team actually demonstrating?”

Proposed choices:

- `One capacity component can be serviced without interrupting the ICT load` **(key)**
- `The component is redundant only after the ICT load has been shut down`
- `The software maintenance window can proceed without changing electrical capacity`
- `Fuel deliveries can be scheduled without a maintenance operating procedure`

Grounding: Public syllabus heading “Power redundancy levels and techniques”; ANSI/TIA-942-C,
<https://tiaonline.org/standard/tia-942/> and TIA ratings definitions,
<https://tiaonline.org/products-and-services/tia942certification/tia-942-certifications-ratings/>.

Assessment: one defensible answer; the shutdown-only and software/electrical substitutions are
plausible category errors, and an online maintenance procedure is a real facility activity.
**SHIP CANDIDATE**.

### m06-q072 — transformer loss in the thermal budget

Current recall item: “Transformer losses show up operationally as:” The key is heat in the
facility thermal budget.

Proposed apply version: “An electrical design review is closing the room heat-rejection
budget. How should the transformer’s measured load and no-load losses be treated?”

Proposed choices:

- `Add them as heat that the facility cooling design must reject` **(key)**
- `Subtract them from the IT heat load because the loss is upstream of the racks`
- `Count only network latency because transformer loss is an electrical-quality issue`
- `Ignore them once the transformer has been assigned an electrical rating`

Grounding: Public syllabus heading “Transformers”; IEC 60076-19-1:2023,
<https://webstore.iec.ch/en/publication/59982>.

Assessment: the key is unique and the upstream-loss misconception is realistic; the latency and
ignore options are weak unless rewritten. The budget review is a real design task. **SHIP
CANDIDATE only after replacing the two weak distractors with grounded thermal/electrical
confusions.**

### m06-q086 — VRLA trade-off

Current recall item: “VRLA batteries in UPS rooms are commonly chosen historically for:” The key
is sealed/recombinant trade-offs versus flooded cells.

Proposed apply version: “A facility is comparing a valve-regulated lead-acid bank with a flooded
lead-acid bank for a stationary UPS installation. Which consideration is the relevant reason
the VRLA option has historically been selected?”

Proposed choices:

- `Its sealed/recombinant maintenance, space, and lifecycle trade-offs versus flooded cells` **(key)**
- `It requires no monitoring or inspection for its entire service life`
- `It has unlimited cycle life regardless of chemistry or duty`
- `It performs best only at freezing temperatures`

Grounding: Public syllabus heading “Batteries”; IEC 60896-22:2004,
<https://webstore.iec.ch/en/publication/3851>.

Assessment: the scenario is real and the key remains the intended comparison, but the cited
official catalog only establishes the stationary VRLA/UPS scope; it does not expose the exact
maintenance, space, and lifecycle trade-off. The three distractors are still cartoons. **WORSE
THAN THE ORIGINAL — DO NOT SHIP.** Dressing an unsupported trade-off in a procurement scenario
adds reading cost without improving the evidence or the distractors.

### m06-q094 — sizing from power factor

Current recall item: “The relationship kW = kVA × power factor means:” The key is real power =
apparent power × PF.

Proposed apply version: “A generator is rated 100 kVA and the connected load is assessed at a
0.8 power factor. What real-power capacity does that rating represent at that power factor?”

Proposed choices:

- `80 kW` **(key)**
- `100 kW`
- `125 kW`
- `0.8 kW`

Grounding: Public syllabus heading “Power sizing”; IEC 60050 IEV 131-11-46,
<https://www.electropedia.org/iev/iev.nsf/display?ievref=131-11-46&openform=>.

Assessment: one answer; 100 and 125 are plausible formula/ratio errors, while 0.8 confuses a
dimensionless factor with power. Nameplate sizing is a real engineering task, and the source
grounds the relationship used. **SHIP CANDIDATE**, subject to a reviewer confirming that the
arithmetic is appropriate for this syllabus level.

### m06-q097 — thermography program

Current recall item: “A good IR thermography program includes:” The key is scanning under load and
trending anomalies.

Proposed apply version: “A thermography review finds a hotspot during a representative loaded
scan. Which follow-up makes the observation useful for maintenance planning?”

Proposed choices:

- `Compare it with prior load-context images and trend the anomaly` **(key)**
- `Keep one image and discard the historical series after closing the ticket`
- `Repeat only with the gear de-energized, regardless of the operating condition`
- `Wait for an outage before collecting any comparable baseline`

Grounding: Public syllabus heading “Thermographic scanning”; NFPA 70B 2026,
<https://link.nfpa.org/all-publications/70B/2026>. The item’s receipt says the official preview
does not expose the exact thermography-program proposition.

Assessment: one answer and a real maintenance workflow, but the official source is
catalogue-only for this exact claim. **HOLD pending a public source that supports the program
practice; do not treat the blocked catalogue as proof.**

### m06-q108 — wet stacking

Current recall item: “Wet stacking risk is associated with diesel generators that:” The key is
running at light load without loaded exercise.

Proposed apply version: “A diesel standby set repeatedly passes start checks but is operated at
very light load during tests. Which follow-up addresses the condition associated with wet
stacking?”

Proposed choices:

- `Schedule an appropriately loaded exercise rather than testing only at light load` **(key)**
- `Run the same light-load test more frequently without changing the load`
- `Remove the exhaust system so deposits cannot accumulate in it`
- `Treat a generator-start battery test as a substitute for loaded engine exercise`

Grounding: Public syllabus heading “Generators”; NFPA 110 2025,
<https://link.nfpa.org/all-publications/110/2025>. The item’s receipt says the official preview
does not expose the exact wet-stacking proposition.

Assessment: one answer and a genuine generator-maintenance situation; the light-load repetition
and battery-test alternatives represent realistic maintenance confusions. **HOLD pending a
public source for the exact wet-stacking/load-exercise claim.**

### m06-q212 — load-bank testing

Current recall item: “During generator load-bank testing, the operational goal is mainly to:”
The key is exercising the set under real thermal and electrical load.

Proposed apply version: “A commissioning team is deciding whether a generator test has exercised
the engine meaningfully or merely proved that it starts. Which observation supports the former
conclusion?”

Proposed choices:

- `The set carried a meaningful thermal and electrical load for the planned test interval` **(key)**
- `The set started once and immediately returned to idle`
- `The team disabled ATS transfers for the duration of the test`
- `The UPS batteries were intentionally drained to zero`

Grounding: Public syllabus heading “Generators”; NFPA 110 2025,
<https://link.nfpa.org/all-publications/110/2025>. The item’s receipt says the official preview
does not expose the exact load-bank-test objective.

Assessment: one answer and a real commissioning decision; the start-only versus loaded-test
distinction is useful. **HOLD pending a public source for the exact objective; no PDF or paid
standard body was fetched.**

### m06-q255 — shared common point

Current item: the whiteboard scenario already asks whether utilities and a generator through one
ATS and one UPS are genuinely dual-utility/2N. The key is that it is a shared-SPOF figure, not
independent 2N.

Proposed apply version: “At commissioning, a diagram labels Utility A, Utility B, and a generator
as ‘dual utility’, but all three terminate at one ATS and one UPS before the rack A/B split. What
claim can the reviewer sign off?”

Proposed choices:

- `The diagram still has a shared ATS/UPS common point and does not demonstrate independent 2N` **(key)**
- `The two utility names make independence automatic`
- `Three incoming sources make the arrangement N+2`
- `Rack A/B PDUs remove the upstream common point`

Grounding: Public syllabus heading “Power redundancy levels and techniques”; ISO/IEC 22237-1:2021,
<https://www.iso.org/standard/78550.html?browse=tc>. The item’s receipt says the official
catalogue does not expose the exact one-ATS/one-UPS proposition.

Assessment: one answer and the commissioning review is realistic, but this is a near-paraphrase
of the current item, not a meaningful conversion. The source is also blocked for the exact
common-point claim. **HOLD — do not rewrite this item on the strength of this pilot.**

## 4. Pilot result and limits

Of the 12 drafts, 6 are promising conditional candidates (`q013`, `q042`, `q047`, `q054`,
`q094`, and provisionally `q072`), 4 are source-blocked holds (`q097`, `q108`, `q212`, and
`q255`), 1 is a near-paraphrase hold (`q052`), and 1 is the required worse-than-original draft
(`q086`). Even the promising group needs human review of distractor plausibility; “SHIP
CANDIDATE” is not approval.

The rejected `q086` draft is the should-fail result: closer reading showed that the official
catalogue URL establishes the equipment/application scope, not the full trade-off claim, while
the proposed scenario adds procurement realism without repairing the cartoon distractors. The
honest recall item is safer until a source and better distractors exist.

This pilot does not establish that all 396 convertible items are good conversion candidates. It
also does not ratify the proposed 45/35/20 recall/apply/analyse target. Convertibility is a
judgement proxy: it can miss a real operational decision hidden in a terse fact, and it can
mistake a plausible-sounding scenario for a situation a practitioner actually encounters.
Human subject-matter review is required before any item is changed.
