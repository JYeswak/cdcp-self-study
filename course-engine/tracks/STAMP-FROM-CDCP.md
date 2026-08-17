# Track stamp extracted from CDCP

This is an extraction record, not a claim that the stamp is domain-agnostic.
The reference was read from the shipped CDCP tree before CDCS was authored.

## Measured reference instance

- `bank/items/`: 854 TOML files, 829 `approved`, 25 `retired` (the packet's
  846 is stale in this checkout); `bank/MANIFEST.toml` is the file-set pin.
- `registries/objectives.toml`: 35 product objectives; `knowledge/domains.toml`:
  15 domains; `knowledge/topics.toml`: 106 topic atoms.
- `goldens/`: fixed mock-40 fixture, all-correct digest, all-wrong digest, and
  bank hash. `web/`: hub, Learn, Drill, Mock, Reference, Results, and the
  generated seed-42 packs.
- `scripts/check.sh` wires the Rust bank/grade/golden/coverage gates, browser
  surface smoke, schedule smoke, and known-bad suites. The SRS schedule keys
  are learner item IDs, but its surrounding UI still assumes numeric modules.

## Extracted stamp contract

Each stamped track owns a `manifest.toml` with these required data fields:

1. identity (`id`, `title`, schema/stamp version);
2. a named approved-item floor and declared count;
3. bank, objective, citation-corpus, notes, learner-data, and golden paths;
4. per-item objective IDs, source IDs, original-source status, and an explicit
   assessment class (`calculation` or `one-line-defect`);
5. live-surface reachability (`learner_page`, hub link, map marker);
6. a track-specific honesty row and a track-specific rights row, both required
   and non-inheriting; citation-only sources carry their own rights fields;
7. forbidden-language data and integrity pins for item IDs and canonical bank
   content.

The reusable engine side is the Rust parser, required-field validation,
objective/source referential checks, canonical hash, CDCP-overlap hash/id check,
surface reachability check, and planted selftests. The track side is every
stem, answer, explanation, formula, objective, topic, public URL, count floor,
golden, banner, rights decision, page, and learner-data row.

## Genericity debt: what resisted the split

This list is intentionally unfinished and unpolished.

- `cdcp_bank::BankItem` hardcodes numeric `module`, A–D `ChoiceLetter`, the
  `cdcp-bank-v3` hash domain, CDCP field names, and CDCP status semantics.
- `cdcp_assemble` hardcodes approved-pool sampling, module stratification,
  `mock40`, seed 42 fixtures, and a 40-item form. A stamped non-exam track
  cannot use it without a schema/contract rewrite.
- `cdcp_grade::GradeReport` hardcodes `exam_id`, `seed`, `bank_hash`,
  `by_module`, weak modules, and the CDCP study-signal shape.
- Browser quiz, drill, results, mastery, and SRS code filter by numeric module;
  `module_learn_slugs.js`, `modules_index.json`, and the 14/15-module floor are
  CDCP/EPI-shaped.
- `cdcp_cli` defaults to `bank/items`, `web/data`, `goldens`, `mock40`, and
  seed-42 artifacts. Installed-tree tests require those exact files.
- `cdcp_learn` compiles `knowledge/domains.toml` and parent `modules/*.md`;
  generated module copies, unit paths, glossary links, and hub copy are not
  track-neutral.
- `scripts/check.sh` names CDCP gates, CDCP selftests, mock-40 receipts, and
  CDCP web artifacts. The stamp is wired through the existing Rust registry
  gate, but the legacy chain remains welded.
- Global claims and objectives include CDCP/EPI honesty and the 27/40 study
  signal. They cannot be inherited by a new track; the track row must repeat
  its own denial and signal semantics.
- The global corpus policy and content lock cover CDCP snapshots and standards
  evidence, while ASHRAE/fire/fuel rights are domain-specific. No safe default
  can be copied into another track.
- `cdcp_bank::mock40_module`, `cdcp_anki`'s “CDCP Study” deck, goldens-coupling
  rows, and installed-bundle checks all encode product-specific names.
- Existing CDCP items often leave `objective_ids` empty because CDCP objective
  coverage is registry/topic-oriented. The stamp therefore requires item-level
  objective linkage instead of pretending the reference already satisfies it.

CDCS is the first stamped learner surface. A green CDCS receipt proves this
data contract and its gates; it does not prove that the remaining welds are
domain-independent. That question belongs to the proposed non-data-centre
canary, not to this track's status.

## Canary proposal — not run

Use a five-item **municipal water-pump station** micro-domain. Its data would
carry a floor of 5 and five original items: pump hydraulic power, wet-well
volume/run time, a motor-starter one-line defect, disinfection contact-time
arithmetic, and a transfer/control defect. It must bring its own objectives,
public citation metadata, honesty banner, rights decisions, item IDs, golden,
learner page, and map/hub link. No CDCP, ASHRAE, EPI, or data-centre row may be
inherited.

The canary falsifies genericity if it needs numeric module fields, mock-40
assembly, CDCP study-signal semantics, a copied rights banner, or a new engine
branch. It is only a proposal: no canary track or claim of domain independence
is shipped here.
