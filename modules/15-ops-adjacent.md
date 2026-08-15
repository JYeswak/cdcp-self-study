# Operations Practices (Ops-Adjacent)

## Scope note — read this first

The curriculum map is **15 modules**. Modules 01–14 are the **public EPI CDCP facility headings**.
This file is **Module 15 — 2.1 Operational Considerations**, the ops-adjacent Learn surface. It is
**not** one of the 14 public facility domains. Exam weight is unknown and stays unknown — do not
invent a percentage.

Module 15 covers the operations practices a working data centre runs on after handover: the public
**2.1 headings** (service catalog; Service Level Management; organizational structure;
training-program requirements; safety roles; security matrix; maintenance-agreement content; floor
management; monitoring; document-management steps; vendor management) plus the procedure craft
(MOP / SOP / EOP / SWMS) that makes those headings executable at 03:00.

**Unlearn, stated once so it cannot be missed.** Module 01 Q3 / the map objective treated power,
cooling, and human error as peer buckets — **that is the sentence this module retires.** Facility
unavailability is power-path led; cooling is usually a cascade; people and process are
**contributing factors, plural**. The rest of this file is the mechanism. Do not replace the
cartoon with an unverifiable majority percentage.

Two consequences, stated plainly:

- **Exam weight is unknown.** `knowledge/domains.toml` marks this domain `exam_weight_unknown = true`
  and it stays marked. Nothing here should be read as "this many questions on the real exam."
- **The assessed pool includes it.** The item bank holds 39 items under `15-ops-adjacent`, and the
  mock assembler samples every approved module. Before this module existed, a learner could be
  scored on operations material the course never taught. That was a fairness defect; this file is
  the fix. The alternative — retiring the 39 items — was rejected because operations discipline is
  the single most interview-probed body of knowledge for a facilities-hybrid seat.

Everything below is grounded in **freely redistributable primary sources**: US federal regulation
(29 CFR, 10 CFR), US government publications (DOE, NASA, NIST, FEMA), UK HSE guidance, and public
post-incident reports. Where a source is paywalled or read-only, it is **named and not quoted** —
and the gap is stated rather than filled from a secondary summary.

---

## Learning objectives

By the end of this module you can:

1. Distinguish **MOP / SOP / EOP / SWMS** and — more importantly — state a procedure's **level of
   use**, which is the attribute that actually decides behaviour at 03:00.
2. Write and critique a procedure **step**: imperative verb, precise equipment designator, warnings
   and cautions placed correctly, hold points, and explicit stop conditions.
3. Explain what **as-built documentation** is for, why documentation that diverges from reality is
   more dangerous than missing documentation, and what a **labelling** scheme must carry.
4. Interpret **MTBF, MTTR and availability as in Module 01**, and explain why MTTR is usually the
   cheaper lever.
5. Specify a **maintenance contract / SLA** in measurable, CMMS-auditable terms rather than
   response-time theatre — and place it as an **underpinning contract** under Service Level
   Management, not as the customer SLA itself.
6. Name **floor management** as work control on the live floor (not only cleaning), and design a
   white-space cleaning programme that does not itself become an incident.
7. Apply **operational security and safety practices** — escorting, permit to work, contractor
   control, hand-back — as a system rather than a courtesy, with a **safety-roles inventory** and a
   **vendor lifecycle** (select, score, underpinning contract, performance).
8. Classify an incident on defensible axes, run **incident command**, and write a **blameless
   postmortem** that survives both a learning review and a contractual one.
9. Name the **human-factors** mechanisms — error precursors, turnover, three-way communication,
   self-checking, fatigue, normalisation of deviance — that decide whether any of the above holds.
10. State correctly **which OSHA regime governs which hazard** on a data-centre campus, a boundary
    most of the industry gets wrong.
11. Interrogate a domain statistic: who measured it, and can you read the measurement.
12. Map the public **2.1 headings** this file owns: **service catalog**; **Service Level
    Management** (SLA vs OLA vs underpinning contract); **data-centre organizational structure**;
    **training-program** requirements; **security matrix** (role × zone × privilege).
13. Walk a **document-management** lifecycle — create, review, approve, issue, supersede, archive —
    and say why currency is the property to gate.
14. Walk four **2026 EOPs** a Fluidstack-style floor actually runs: isolate a leaking CDU without
    killing the pod; Li-ion / BESS fire (not Class A); load-shed when thermal ride-through is
    seconds; respond to grid curtailment / BTM islanding. Plant lives in Modules 06 / 09 / 12; this
    file owns the procedure.
15. Name **Cx / ASHRAE Guideline 0** and **ISO/IEC 30134** as vocabulary and point at **Module 02**
    — do not fake a commissioning syllabus here.

---

## Why it matters (ops/design/TPM interview angle)

Design decides what a facility *can* do. Operations decides what it *does* do on a Tuesday at 03:00
with one technician, a vendor on the phone, and a procedure written by someone who has never stood
in that room.

**Ops angle.** Nearly every long outage has an operations sentence in it: a procedure that had never
been dry-run, a label that did not match the drawing, an alarm nobody had defined a response for, a
shift handover that happened in a corridor. The plant rarely fails alone.

**Design angle.** Several operations properties are *designed*, not managed. Whether an electrician
can lawfully isolate a UPS without dropping the floor is a one-line diagram question settled months
before anyone in operations is hired. Whether the alarm count is 400 or 40,000 is settled by the
integrator's point schedule. Operations inherits these and cannot renegotiate them.

**TPM / hybrid interview angle.** Interviewers probe operations because it is where candidates either
narrate real experience or recite a vendor deck. "Walk me through your last MOP" and "what did your
postmortem action items look like" are the two highest-signal questions in the domain. In 2026 they
also ask the 2.1 control set — "what is in the catalog, and which OLA makes that SLA true?" — and
the four EOPs below. This module gives you the vocabulary *and* the primary sources, which is what
separates an answer from an opinion.

---

## Core concepts

### 2.1 Operational Considerations — the public heading set

The public 2.1 list is a **control set**, not a facilities add-on and not a percentage of any exam.
This subsection teaches the headings that were missing or thin. Procedure craft, isolation, and
human factors stay in the sections that already own them.

#### Service catalog

A **service catalog** is the owned, published list of what this site will actually deliver. It is
not the sales brochure, not the CMMS asset list, and not a slide of logos.

Each entry carries, at minimum: the **service name**; what "done" looks like; **who the customer
is**; **who the provider is**; the SLO / SLA it points at; the **OLA or underpinning contract** that
makes that SLO possible; and **how you request it**.

Typical data-centre catalog entries, spoken as SKUs not slogans:

- kW landing at a named density (air hall vs liquid-ready hall)
- cross-connect / meet-me
- remote hands and after-hours escort
- white-space access windows
- liquid-loop landing (CDU / manifold / QDC — hardware families in **Module 09**)
- generator-test or maintenance window as a *service* the tenant consumes

**A catalog without owners is a brochure.** A catalog that promises 40 kW liquid when the OLA with
facilities is still a 10 kW air hall is how the **org-split outage** in Module 01 starts — two green
objects, no owner of the path. The catalog is the place that sentence is supposed to die.

Do not invent an exam-weight for this heading. It is taught because a working floor runs on it.

#### Service Level Management — SLA vs OLA vs underpinning contract

**Service Level Management (SLM)** is the discipline of defining, measuring, reviewing, and
correcting service levels. A vendor PDF titled "SLA" is one instrument inside that discipline, not
the discipline.

Three instruments, three counterparties:

| Instrument | Counterparty | What it commits | Failure if missing |
|---|---|---|---|
| **SLA** — Service Level Agreement | The *customer* (tenant, internal LOB, GPU customer) | Availability, restore, credits, **whose clock** | A promise nobody inside the building is staffed to keep |
| **OLA** — Operational Level Agreement | An *internal* team (facilities ↔ NOC ↔ security ↔ floor) | The hand-offs that make the SLA possible — who pages, who isolates, who escorts | The SLA looks green and the path has no owner |
| **UC** — underpinning contract | A *vendor* | Response **and** restore, parts, qualifications, proof | Four-hour customer restore sitting on next-business-day parts |

Read them as a stack. The customer SLA is only as true as the OLAs and UCs underneath it. "Four-hour
onsite response" in a maintenance agreement is a **UC term**, not a customer SLA — the later
maintenance-contract section is that UC, written so a CMMS can audit it.

The SLM failure mode that interviews catch: an SLA with **no OLA and no UC**. Restore-in-four-hours
to the tenant, a NOC that pages facilities "when we can," and a vendor whose clock starts when the
part ships, is three documents that cannot add up.

Scope of the clock is **as in Module 01**: per service vs per site vs per component; planned vs
unplanned; whose clock. SLM does not invent a new nines table.

#### Data-centre organizational structure

Module 01 names four seats that show up on every incident bridge — **facilities, IT, finance/real
estate, TPM** — and how an org-split outage starts. This file owns the **2.1 organizational-structure
heading**: how a site is actually staffed so those seats have a path.

A speakable site org (titles vary; the *functions* do not):

| Function | Typically owns | Must not be the only owner of |
|---|---|---|
| **Critical facilities / site engineering** | Grey-space MEP, plant vendors, many colo-provider obligations | The customer SLA (that is SLM) |
| **NOC / operations** | Monitoring, first response, the bridge, often DCIM | Isolation authority on switchgear (Subpart S) |
| **Physical security** | Zones, badges, escorts, SOC | Life-safety egress (safety / AHJ) |
| **EHS / safety** | PTW programme, LOTO competence, the Safety Officer role | Uptime bonus (conflict of interest) |
| **Floor / white-space management** | Who is on the live floor, simultaneous work, heat/power/liquid budget of the change | The plant one-line |
| **Vendor management** | Select, score, UC, performance | Escort-as-a-substitute-for-a-contract |
| **Capacity / planning** | Catalog SKUs vs remaining kW, ports, liquid landing | A promise the plant cannot keep |

**Reporting lines are a control.** If the Safety Officer reports to the person whose bonus is
uptime, the independent veto in incident command is fiction. If floor management reports only to
IT, the MOP that sheds a CDU loop will not have a facilities signature.

Do not grow this back into Module 01's four-seat lesson. Name the functions, name the split, keep
the 2.1 heading here.

#### Training-program requirements

A training programme is not "we sent people to a vendor class." It is a **role-based control** with
records.

Minimum contents, spoken as requirements not aspirations:

1. **Role × competence matrix** — which role needs which qualification (electrical qualified person,
   PTW issuer, CDU isolator, fire warden, NOC alarm-owner).
2. **Initial qualification** before unsupervised work.
3. **Recurrent interval** with a named owner — not "when we remember."
4. **Procedure-change training** when a MOP / SOP / EOP is revised (already required by the
   procedure programme below).
5. **Drills** — the dry run is the acceptance test; an EOP that has never been walked is an untested
   artefact.
6. **Competence records** that survive an audit and a night-shift handover.

**DOE-HDBK-1028** still governs the honesty of the programme: "No amount of counseling, training, or
motivation can alter a person's fallibility." The programme exists to manage **error-likely
situations**, not to produce infallible people. Training that does not include the procedure's
**level of use** (use-each-time vs reference) trains the wrong behaviour.

#### Security matrix — role × zone × privilege

**Module 13** owns the physical layers (perimeter → building → suite → cage/rack) and
authentication vs authorization. This file owns the **security matrix**: a table, not a badge
printer.

- **Rows** are roles — employee, vendor-with-escort, cleaner, fire department, customer tech, NOC
  operator.
- **Columns** are zones — lobby, grey space, white space, battery / UPS room, BESS yard, NOC,
  loading dock.
- **Cells** are privileges — unescorted, escorted-only, no-access, time-window, two-person rule.

Privilege is more than "door opens." It includes photography, removable media, network port, EPO,
and **isolation authority**. A badge that opens every zone is not a matrix; it is a single
failure that looks like convenience.

Least privilege is the rule. The matrix is reviewed when a role, a zone, or a privilege changes —
the same currency rule as as-builts.

### Procedure discipline — MOP, SOP, EOP, SWMS

The four-letter taxonomy is trade vernacular with **no public normative definition** — the closest
authority is paywalled. Use it, but know that the grounded part is elsewhere.

| Term | Working meaning | Typical trigger |
|---|---|---|
| **SOP** — Standard Operating Procedure | Recurring, expected activity | Routine rounds, monthly generator exercise |
| **MOP** — Method of Procedure | One planned change to a live system, with a defined start and end | Breaker maintenance, UPS bypass, firmware update |
| **EOP** — Emergency Operating Procedure | Response to an unplanned condition, executed under time pressure | Utility loss, chilled-water loss, fire alarm |
| **SWMS** — Safe Work Method Statement | Task-level safety controls for hazardous work | Hot work, confined space, work at height |
| **Runbook** | Reusable, repeatedly executed reference for a known scenario | Server room evacuation, failover rehearsal |

A **runbook differs from a one-off MOP** by reuse: the runbook is maintained, versioned, and expected
to be executed many times; the MOP is authored for one specific change and closed out afterwards.

**The attribute nobody teaches, and the one that matters.** DOE O 422.1 (US Government, public)
requires a technical-procedure programme to define "specified and defined procedure use requirements,
i.e., reader-worker method, reference use only, use-each-time, and emergency response."

- **Use-each-time / continuous use** — the paper is open, the finger is on the step.
- **Reference use** — read it before, then work from memory.

A procedure that does not declare its level of use **is not a procedure, it is a document**. Confusing
the two is exactly how a competent engineer skips a step and can still truthfully say they followed
the procedure.

The same source enumerates the rest of a defensible programme: a development process; consistent
format and terms (prerequisites, warnings, cautions, notes, **hold points**); a change process that
distinguishes pen-and-ink changes from full revisions; training on changed procedures; approval
authority; initial-issue and **periodic review and testing**; and availability of the latest revision
at the point of use.

#### Step grammar

**DOE-STD-1029-92** is a public-domain writer's guide and transfers directly to a MOP:

- Requirements documents use "shall". **Procedures use imperatives.** `Open`, `Rack out`, `Verify` —
  action verb, direct object, supportive information, singular present tense.
- "Identify equipment precisely as it is in the facility." If the breaker label reads `MSB-2A-CB07`,
  the step says `MSB-2A-CB07`, not "the utility breaker".
- Avoid acronyms in action steps. Limit nesting to **two levels**.
- **Warnings** (harm to people) and **cautions** (harm to equipment) go immediately before, and on the
  same page as, the affected step; are complete on one page; are **statements, not commands**; carry
  no embedded action; cover one topic each; and state the hazardous condition, the consequence of
  ignoring it, and any critical time consideration. Do not set them in all capitals — they become
  harder to read.

The standard supplies its own justification: "An industry study of significant events attributed
one-fourth of all human performance events to a failure to provide proper warnings and cautions."

#### Review, dry run, hold points, abort

Three rules, all from public sources:

1. **The author cannot be the sole validator.** DOE-HDBK-1028 Vol 2 is explicit that "procedures
   sometimes contain hidden flaws" and that following one without question does not guarantee safety.
2. **The procedure defines its own stop conditions.** The handbook's mandatory STOP triggers: the step
   cannot be performed as written; injury or damage will result; use will produce an incorrect or
   unsafe configuration; the procedure is technically incorrect; **unexpected results are achieved
   after performing a step**; it conflicts with another procedure; it is otherwise unsafe. That fifth
   trigger — *unexpected result is a stop condition in its own right* — is the cheapest abort
   criterion in existence. Put it in every MOP.
3. **The dry run is the acceptance test, and a backout is not optional.** Cloudflare's public PDX-04
   postmortem states the failure in one sentence: "we had never tested fully taking the entire PDX-04
   facility offline. As a result, we had missed the importance of some of these dependencies." A MOP
   whose preconditions have never existed in reality is an untested artefact.

**Practical rule for the proposal and for the interview:** procedures are a **pre-handover deliverable
with an operator signature**. The vendor writes the equipment sequence; the person who will hold the
shift walks it down and signs that it is executable *with the labels actually on the gear*. Budget
the walkdown in shifts, not hours.

### 2026 EOPs a live floor actually runs

Procedure craft above is necessary and not sufficient. A Fluidstack-style hall in 2026 runs
scenarios the classic utility-loss / chilled-water-loss list does not name. **This file owns the
procedure.** The plant lives elsewhere: **Module 06** (power path, BESS ≠ UPS batteries, islanding),
**Module 09** (liquid families, CDU, thermal plant), **Module 12** (NFPA 855 / UL 9540A playbook),
**Module 14** (CDU-loop points and seconds-scale rate-of-rise), **Module 01** (the minutes→seconds
clock). Do not steal those lessons here.

Each EOP below is a **drill**, not a site-specific MOP. Every one still needs a declared **level of
use**, a **hold point**, and the abort sentence: *if an unexpected result occurs, stop.*

#### EOP 1 — Isolate a leaking CDU without killing the pod

The secondary loop is its own domain (**Module 14** owns the points; **Module 09** owns RDHx / D2C /
immersion / CDU as hardware). The operator's job is isolation of *this* loop, not a hall dump.

Speakable sequence:

1. **Confirm the domain.** Secondary-loop leak (CDU skid, QDC, hose tray, cold-plate / RDHx drip)
   is not a raised-floor CHW rope event. Read the point name on the label.
2. **Hold point — identify THIS loop's isolation valves** from the as-built and the label
   (`CDU-2B-ISO-S` / `CDU-2B-ISO-R`), not "the CHW header."
3. **Hold point — remaining capacity.** If the pod still has N+1 CDU, isolating one unit must leave
   the pod served. If remaining capacity cannot hold the load, go to the **pre-authorized shed
   list** for *that loop* — not the whole pod, not the hall.
4. **Isolate** the leaking CDU / loop. Verify flow and pressure on the remaining units. Verify
   inlet or cold-plate rate-of-rise is not still climbing.
5. **Abort** if the wrong valve moved, if remaining CDUs do not pick up, or if any unexpected
   result occurs.

Killing the pod is the failure mode this EOP exists to prevent: closing the primary header, tripping
every rack on a shared isolation, or treating a secondary puddle as a reason to dump the hall.

#### EOP 2 — Li-ion / BESS fire (not Class A)

**Not a Class A trash-can fire.** Chemistry changes the playbook. **Module 12** owns NFPA 855,
UL 9540A, LSFT, off-gas / deflagration, room-vs-yard, and the water-on-Li-ion controversy.
**Module 06** owns BESS ≠ UPS batteries and the electrical path. This file owns what the operator
*does*.

Speakable sequence:

1. **Treat the first signal as off-gas / thermal**, not as smoke-over-cardboard. Do not stay to
   "dump clean agent and see."
2. **Evacuate** per the written site EOP. People first. Life safety is not a restore metric.
3. **Room vs yard.** Indoor UPS / battery room is a people-and-box problem (off-gas can deflagrate
   in a sealed room). Outdoor BESS yard is an exposure-and-setback problem (stop one container from
   taking the next). Follow the EOP that matches the *space*, not a generic fire SOP.
4. **Isolate electrical only if the EOP names a physical disconnect that leaves the rest of the
   hall served.** Putting a string into maintenance bypass from its **HMI is not isolation** — that
   keeper is unchanged. If the one-line has no isolation point that leaves the load served, lawful
   isolation is an outage; say so.
5. **Who talks to the fire department** is named (Incident Commander / PIO). Do not invent
   "always water" or a fire percentage. Confirm the adopted 855 edition and **what this site's EOP
   actually says**.

#### EOP 3 — Load-shed when thermal ride-through is seconds

As in **Module 01**: at 40–100 kW, UPS without cooling collapses thermal ride-through from minutes
to **seconds**. **Module 09** owns the plant. **Module 14** owns the seconds-scale inlet or
cold-plate rate-of-rise. This file owns the shed procedure.

Speakable sequence:

1. **The clock is rate-of-rise, not the UPS runtime sticker.** Do not wait for a static high-temp
   trip. Do not invent a seconds number as statute.
2. **A pre-authorized shed list exists before the event** — which jobs, which racks, which feed,
   whose authority. Writing that list during the rise is not an EOP.
3. **Authority to act at 03:00** without a callback. A named person. The cheaper MTTR lever below
   is useless if the shed waits for a director.
4. **Hold point — confirm which feed / which loop** you are about to shed. Three-way the
   designator. STAR on the breaker or the orchestration control that actually drops the load.
5. **Abort** if the unexpected feed moved, if remaining cooling is already restoring, or if any
   unexpected result occurs.

This is not a cooling-design lesson and not a W-class lesson. It is the operator action when the
IT load is still on UPS and the fans, pumps, or CDU are not.

#### EOP 4 — Grid curtailment / behind-the-meter islanding

**Module 03** owns the interconnect queue and BTM as a *site type*. **Module 06** owns the power
path, BESS ≠ UPS batteries, and microgrid / island language. This file owns the operations response
when the grid (or the BTM plant) tells the hall to change state.

Speakable sequence:

1. **Name the event.** A curtailment order is not a utility *failure* and not a planned maintenance
   window. It is a required change of load or of island state under the interconnection agreement.
2. **Hold point — confirm mode.** Grid-following vs **intentional island**. Do not island from an
   HMI that only opens a breaker. Protection, grounding, and what the standby node does **not**
   inherit are Module 06's path questions; the operator's hold point is "are we in the
   configuration the protection study assumed?"
3. **Load-shed vs generator / BESS dispatch** is a *designed sequence*, not improvisation. The
   pre-authorized shed list from EOP 3 is often the same list. The UC / OLA must say who may
   dispatch the BESS into a role it was not in five minutes ago.
4. **Who talks to the utility** is named. One voice. The PIO / IC rule still applies.
5. **Abort** if islanding would leave a backfeed path you have not verified, if the BESS is being
   asked to be a UPS it is not, or if any unexpected result occurs.

Do not steal queue studies, UPS internals, or a fake "the campus islands itself" slogan. If the
one-line cannot island and leave the load served, the honest EOP is **shed**, not island.

### Documentation

Documentation is the mechanism by which a facility remains operable by someone who did not build it.

**What the library must contain**

| Artefact | Answers | Failure if stale |
|---|---|---|
| **As-built one-line** | What feeds what, through which device | Wrong breaker opened; isolation believed that does not exist |
| **Rack elevations + U map** | What is in which rack, at which U | Hands-and-eyes sent to the wrong cabinet |
| **Circuit/cable schedule** | Which whip, from which panel, to which PDU port | Capacity decisions made on fiction |
| **Master equipment list + criticality** | What exists and how much it matters | PM effort spread evenly across unequal risk |
| **Settings/parameter record** | Protection and control settings *as set* | Drift from the coordination study goes unnoticed |
| **On-call roster + escalation path** | Who, how, and what happens if they do not answer | The first ten minutes are spent finding a person |
| **Post-incident record** | Timeline, contributing factors, owned actions | The same outage recurs with new staff |

**Document-management steps — the 2.1 heading.** An artefact library is not a document-management
system. The lifecycle, in order:

1. **Create** — named author, named purpose, named level of use if it is a procedure.
2. **Review** — a person who is **not** the sole author. Hidden flaws stay hidden when the writer
   validates their own work (DOE-HDBK-1028).
3. **Approve** — named authority, dated.
4. **Issue** — the controlled copy is the one at the point of use. A share-drive PDF is not issued.
5. **Supersede** — the previous revision is marked and **pulled**. Two live revs is the trusted-wrong
   case below.
6. **Archive** — retention with a retrieval path, not fourteen unmarked copies.

The change that altered the plant **issues** the new as-built as part of close-out. Annual catch-up
is not a step in this list.

**The dangerous case is not missing documentation — it is confidently wrong documentation.** A blank
drawing makes you go and look. A drawing that shows a disconnect where none exists makes you act.
This is why *currency* is the property to gate: as-builts, one-lines, elevations and cable schedules
are updated **as part of the change that altered them**, not in an annual catch-up.

**The handover seam.** The master equipment list, asset tags, nameplate data, settings files and
as-built one-line all exist at the end of commissioning. If transferring them into the CMMS is not a
**condition of handover**, the operations team spends its first year re-deriving them from cable
labels. That is a procurement clause with an operations acceptance signature, not a wish.

#### Commissioning vocabulary — point Module 02; do not fake a Cx syllabus

**ASHRAE Guideline 0** names the **commissioning process**. **Module 02** owns the only split this
course teaches: **design** vs **as-built** vs **Cx** (whether the installed plant was tested and
handed over as a process, not a signature on a brochure). Isolation — HMI bypass is not isolation —
stays in this file's electrical-safety section; do not fold Cx names into that paragraph.

Name the levels so you can hear them at handover: **factory** (FAT), **field / site** (SAT),
**functional**, **integrated systems test**, **seasonal**. Authors disagree on numbering. Do not
memorize L1–L5 as statute, and do not turn this file into a Cx course.

**ISO/IEC 30134** is the KPI series (**PUE**, **WUE**, **CUE**). Named here so a good PUE is not
mistaken for concurrent maintainability. **Module 02** owns the lattice; **Module 10** owns WUE
depth. A KPI is a meter, not a topology class.

### Labelling

Labelling is the physical layer of documentation, and it is what a procedure step points at.

A useful power label carries, at minimum: **circuit ID · source panel/breaker · destination**.
Network labels carry both ends. Campus-wide colour coding and standard templates matter because
they make the scheme **learnable by someone who has never been in this room** — a vendor engineer, a
mutual-aid team, a new hire on night shift.

What labelling actually buys, in order of value:

1. **Reduced human error during isolation, restoration, and incident response.** The wrong-breaker
   mistake is a labelling failure long before it is a discipline failure.
2. **Reduced mean time to identify** a wiring or connectivity fault during a change.
3. A working substrate for the **three-way communication** protocol below — `MSB-2A-CB07` can only be
   repeated back verbatim if it is written on the gear.

**Stale labels are worse than no labels**, for the same reason as stale drawings: they are trusted.
Relabelling is therefore part of the change, and label verification belongs in the MOP's prerequisite
section. In a facility with `PDU-1A-B` and `PDU-1A-D` on adjacent whips, this is not ceremony.

### Floor management (work control on the live floor)

**Floor management** is a named activity set: work control on a live plant, not a synonym for
cleaning. Cleaning is one control inside it. The 2.1 heading is the set.

| Activity | What it controls | Failure if treated as housekeeping |
|---|---|---|
| **Who is on the floor** | Accountability — badges, escorts, a board that matches reality | A vendor in a row nobody authorized |
| **What work is live** | PTW board, simultaneous activities, conflict check | Two MOPs on the same loop |
| **Heat / power / liquid budget of the change** | Will this MOP consume the remaining N+1? | Isolating a CDU "for a look" cooks the pod |
| **Access to white space** | Time windows, escort, photography, tools allowed | A cleaner with a badge that opens every zone |
| **Material movement** | Ladders, tiles, hoses, CDU parts, lift paths | A tile lift that is an unscheduled cooling change |
| **Housekeeping / cleaning** | Particulate, egress, combustibles — the subset below | The only "floor" conversation the site has |

A floor-management programme that cannot answer "who is in row 12, on which permit, and what
capacity did we spend" is not managing the floor.

### Cleaning

White-space cleaning is a genuine reliability control, and a genuine incident source. It is
**not** the whole of floor management.

**Why it matters.** Dust and debris foul filters and heat-exchanger surfaces, degrade airflow, settle
on boards, and — with the right particle chemistry and humidity — contribute to leakage paths and
corrosion. Under a raised floor, the plenum *is* the supply-air path: debris there is delivered
directly to equipment inlets. Particulate control in the white space is what the cleanroom standards
family addresses; the specific ISO particle-class documents are paywalled and are **named, not
quoted**, here.

**How it goes wrong.**

- **Conductive or inappropriate liquids** near live IT and electrical gear — a cleaning chemical is a
  fault path. Mixing chemicals can also produce a hazard of its own.
- **Consumer vacuums** that exhaust fine particulate straight back into the room. Use HEPA-filtered
  equipment specified for the environment.
- **Uncoordinated underfloor work** — lifting tiles changes the static pressure profile of a live
  plenum. Cleaning under the floor and inside perforated tiles must be **scheduled with cooling
  operations** and treated as a change, not as housekeeping.
- **Disturbing what should not be disturbed** — older raised-floor structures and plated components
  can shed zinc whiskers when scraped or flexed; aggressive cleaning is the disturbance.

**Housekeeping rounds** are a separate, cheaper control with a different purpose: clear egress paths,
unblocked extinguishers and panels, closed fire doors, no stored combustibles in the white space.
Those checks support **life safety and fire protection**, not thermal performance, and they belong on
a rounds sheet with a signature.

### MTBF / MTTR (as in Module 01)

Two numbers, one relationship, and a lever most teams ignore. The formula and the nines-scope
drill are **as in Module 01**. This file does not re-own them; it owns why **MTTR is the cheaper
lever** on a live floor.

- **MTBF — Mean Time Between Failures.** The average operating time between failures of a repairable
  population. It is a **population statistic**, not a promise about your unit, and it says nothing
  about *when* your specific serial number will fail.
- **MTTR — Mean Time To Repair / Restore.** The average time from failure to service restored.
  Definitions vary on whether detection and travel time are inside the clock — **pin it in the
  contract**, because that is where the disagreement gets expensive.

Steady-state availability:

```text
Availability = MTBF / (MTBF + MTTR)
```

Read it once and the strategic point falls out: **availability improves by raising MTBF or by
lowering MTTR**, and MTTR is almost always the cheaper, faster, more controllable lever. Raising MTBF
means buying better equipment or adding redundancy — capital, lead time, design change. Lowering MTTR
means labelling, documentation, spares, procedures, training, and escalation paths — most of which
this module is about.

Concretely, MTTR is reduced by:

- **On-site critical spares**, chosen by criticality ranking rather than by unit price. A four-hour
  onsite response with no stocked part still yields a multi-day outage — the vendor arrives and then
  waits for logistics.
- **Findability** — labels, elevations, circuit schedules — so diagnosis is not archaeology.
- **Pre-written EOPs** for the failure modes you can actually anticipate.
- **Authority** — a named person who may act at 03:00 without waiting for a callback.

The honest caveat to state in an interview: both are *averages over an assumed population and
operating context*. Quoting a nameplate MTBF as if it predicted your site is the classic misuse.

### Maintenance contracts / SLA

A maintenance strategy becomes real in two places: the **CMMS** and the **contract**.

**The four maintenance modes**, with the benchmark that makes the conversation honest. DOE/PNNL's
*O&M Best Practices Guide* (public domain) measures the average US facility at **over 55% reactive**,
31% preventive, 12% predictive; continually top-performing facilities run **under 10% reactive,
25–35% preventive, 45–55% predictive**. Same guide: preventive over reactive saves "as much as 12% to
18% on the average"; predictive over preventive a further "8% to 12%".

**Metrics to put in the agreement**, all auditable from the CMMS without trusting a narrative:

| Metric | Benchmark |
|---|---|
| Equipment availability | > 95% |
| Schedule compliance | > 90% |
| **Emergency maintenance percentage** | **< 10%** |
| Maintenance overtime percentage | < 5% |
| **PM completion percentage** | **> 90%** |
| PM budget / total maintenance cost | 15–18% |
| PdM budget / total maintenance cost | 10–12% |

The two bolded rows are the ones to make **measured obligations** rather than reporting lines.

**A defensible maintenance SLA defines, at minimum:** scope of covered equipment; response *and*
restoration targets with the clock definition; parts responsibility and stocking obligation;
qualification of the attending technician; escalation path and authority; reporting cadence and data
access; and the measured metrics above. "Four-hour response" alone is theatre — it specifies arrival,
not restoration.

From the SLM stack above, this vendor agreement is an **underpinning contract**. It is not the
customer SLA. If the tenant SLA promises four-hour restore and this UC promises next-business-day
parts, the catalog entry is fiction.

**The evidence that kills naive time-based PM.** The NASA RCM Guide (public) reproduces the
conditional-probability-of-failure work of Nowlan & Heap and its replications: "In many cases
scheduled overhaul increases the overall failure rate by introducing a high infant mortality rate into
an otherwise stable system," and across the three studies "random failures are between 77 and 92
percent of the total failures."

The uncomfortable data-centre consequence: **intrusive PM on complex, electronic-heavy plant is a
source of outages, not only a mitigation.** Racking a breaker out and back, re-torquing a bus joint,
swapping a control card — each injects a fresh infant-mortality event into a stable system, on
live-adjacent equipment, at night, usually by a contractor. The NASA guide names the escape route
directly, listing "systems which failure may be induced by incorrect preventive maintenance" as a
*predictive* application.

So, as a design rule: **rotating and wearing plant** (chillers, pumps, CRAH fans, generator engines,
belts, filters) earns time-based PM. **Static electrical plant and its controls** (UPS modules and
firmware, protection relays, ATS/STS logic, PDU boards, BMS/EPMS controllers) should move to
condition monitoring — thermography, partial discharge, battery impedance trending, power-quality
capture — with intervention triggered by measured degradation rather than the calendar.

**Where intervals come from, and why they conflict.** Three sources disagree by construction:
manufacturer recommendation (written to bound *warranty* exposure — conservative and time-based);
insurer or code expectation (periodicity-shaped, audit-driven); and observed condition (the only one
grounded in the item's failure distribution). This is not "sources vary" — the axis is **basis of
interval**. Record which basis you chose and why. An RCM analysis is the instrument that resolves it,
and it is a design-phase deliverable.

**Change control.** Every PM window is a change: it needs a MOP with prerequisites, hold points,
verification steps and **backout criteria**. A PM window scheduled without rollback criteria is a
change you cannot stop halfway — which is the definition of an uncontrolled change.

### Operational security and safety practices

Operations security is about **controlling who is in the room, what they may do, and what leaves with
them**. Escort and PTW are necessary. They are **not** the whole of safety roles, the security
matrix, or vendor management.

#### Safety roles inventory

These are **roles**, not job titles. One person can hold two; some must not.

| Role | Authority | Must not also be |
|---|---|---|
| **PTW issuer / authorizing person** | Issues the permit; sets the limits of the job | The sole acceptor of the same permit |
| **Performing authority / acceptor** | Accepts the job and the controls | The only reviewer of their own MOP |
| **Isolating authority** | Qualified person for the energy source — Subpart S on switchgear / UPS; 1910.147 on machines | Someone isolating from an HMI |
| **Safety Officer** (incident command) | Stops unsafe acts during the incident — veto *independent* of the restorer | The Incident Commander driving restoration |
| **Escort** | Supervises a visitor in a live zone | A substitute for a UC or a qualification |
| **Fire warden / evacuation lead** | People out, headcount, assembly | The person also fighting the Li-ion event |
| **Named 03:00 authority** | May act on the pre-authorized shed / isolate list without a callback | An unnamed "someone will call" |

A site that can list PPE and cannot name these roles has practices, not a safety organisation.

The **security matrix** (role × zone × privilege) is the table that makes "who is in the room"
auditable — taught under 2.1 above. **Module 13** owns the physical layers.

#### Vendor management as a lifecycle

Escort / PTW is how a vendor *enters*. It is not vendor management. The 2.1 heading is a
**lifecycle**:

1. **Select** — qualification, insurance, Subpart S / LOTO competence, evidence they have executed
   this class of MOP. A cheaper bidder who cannot isolate lawfully is not selected.
2. **Score** — past performance against the UC (restore, not attendance; spare-stock audits; close-out
   quality). Price is one input.
3. **Underpinning contract** — the UC in the SLM stack: response *and* restore, parts obligation,
   technician qualification, clock definition, proof the CMMS can audit.
4. **Performance** — review on a cadence, with the same metrics the UC named. A vendor who arrives
   in four hours and waits three days for a part has failed the UC regardless of the response
   stopwatch.

Hand-back (below) is the last step of a visit, not the last step of the lifecycle.

**Escorting and contractor control.** Escorting vendors in plant areas is not distrust; it is
supervision of work in a live critical environment — the escort knows which surfaces are energised,
which valve must not be touched, and who to call. UK HSE's **HSG250** gives five essential features of
a permit-to-work system: who may authorise which jobs and the limits of that authority; training in
issue, use and closure; monitoring and auditing; clear identification of hazardous work types; and
standardised identification of the task, risk assessment, permitted duration, simultaneous activity
and control measures.

The two objectives data centres routinely omit are the ones that fail at shift boundaries:

- a **formal handover procedure** when a permit spans more than one shift; and
- a **formal hand-back procedure** confirming the affected plant is safe and ready for reinstatement.

HSG250 also records, on evidence, that "imposing systems without consultation can lead to procedures
that do not reflect the needs of maintenance staff… Procedural violations are then more likely."
**Violation is partly a design output of the permit system**, not only a property of the violator.

29 CFR 1910.147(f)(2) adds the contractor clause: the on-site employer and the outside employer
"shall inform each other of their respective lockout or tagout procedures." On a
construction-adjacent campus this is among the most commonly skipped requirements in the regulation.

**Information security in the physical layer.** Photographs of rack layouts, published cabinet
elevations, and visible customer labelling are reconnaissance material — they map the target for both
physical and social-engineering approaches, and in a multi-tenant facility they leak one customer's
posture to another. Control photography, restrict published layouts, and treat the asset database as
sensitive.

**Working near live plant.** Personal protective equipment appropriate to the incident-energy
exposure, a qualified person for electrical work, two-person rules where the site requires them, and
a **pre-job briefing** every time (see below). The specific electrical-safety standards most cited in
the industry (NFPA 70E and 70B) are read-only/licensed documents; they are **named here and not
quoted**, and a site programme should be written against licensed copies.

### Monitoring, alarms, and the failure that hides itself

Three systems, three owners, three seams:

| | Governs | Typical owner | Failure signature |
|---|---|---|---|
| **BMS** | Mechanical plant, sequences of operation, setpoints | Facilities / controls contractor | Slow trends, setpoint drift, an override left in |
| **EPMS** | Electrical metering, breaker status, power quality | Electrical contractor / switchgear OEM | Event-driven, high resolution, poor cross-domain context |
| **DCIM** | Asset, capacity, connectivity, aggregated telemetry | IT / operations | A pretty aggregate over a stale feed |

Nobody owns the **integration**. It is normally a per-point gateway mapping built once by a
subcontractor, never regression-tested, and silently stale after the first firmware update. "We have
monitoring" usually means "three systems exist and one has a dashboard on a wall."

> **Stated gap:** the dominant BMS field protocol is ASHRAE Standard 135 (BACnet). It is a licensed
> document and is deliberately **not** summarised in this course. Protocol-level BMS content must come
> from a licensed copy. This module marks the hole rather than filling it from a vendor summary.

**Alarm rationalisation.** UK HSE's freely published *Better alarm handling* sheet gives the numbers,
attributed there to EEMUA 191: a long-term average alarm rate in normal operation of **no more than
one every ten minutes**, and **no more than ten displayed in the first ten minutes** after a major
upset; and a proportionate priority split of roughly **5% high / 15% medium / 80% low** across about
three priorities, with priority set by the consequence of the operator *failing to respond*.

The definitional rule that removes most data-centre alarm noise on contact: an alarm must **require
operator action**. "Process status indicators should not be designated as alarms." Anything with no
defined operator response is deleted or downgraded, not acknowledged forever.

The same sheet prices the work honestly: a first-pass review may cover perhaps 50 alarms per shift,
but "a thorough review and redesign may take more than 1 shift per alarm." For a campus with several
thousand configured points that is a **funded programme**, not a punch-list item. And note the metric
conflict: DCIM/BMS deployments are scoped and priced by *monitored point count*; alarm quality is
measured in *operator-actionable alarms per operator per hour*. These move in opposite directions and
the procurement metric wins by default.

**Monitoring the monitor.** The U.S.–Canada Power System Outage Task Force report on the 2003
blackout is public domain and is the best teaching case in the domain. The alarm processor failed
**silently**: "Neither FE's control room operators nor FE's IT EMS support personnel were aware of the
alarm failure." The report states the design rule it violates in one sentence: an alarm system "can
also be set up to alarm them if the alarm system itself fails to perform properly. FE's EMS did not
have such a notification system."

Three further lessons transfer exactly, because the failure is in the *information path*:

- **Failover propagated the fault.** The stalled alarm application "moved intact onto the backup while
  still stalled and ineffective," and the backup failed 13 minutes later. A redundant pair that
  replicates application state replicates the defect. Ask your vendor what the standby node does
  **not** inherit.
- **Process-green is not function-green.** After a reboot, diagnostics "verified that the computer and
  all expected processes were running," yet "the alarm system remained frozen and non-functional."
  The post-maintenance test for a monitoring system is *an operator confirming receipt of a test
  alarm*, not a service showing as running.
- **Absence of alarm was read as evidence of health.** Operators continued "in the belief that their
  system was satisfactory, lacking any alarms from their EMS to the contrary" — and were surprised by
  phone calls that contradicted their screens. Write the counter-rule into the EOP: **an external
  report that conflicts with your instrumentation is evidence about your instrumentation.**

### Incident management

**Severity, on three axes rather than one number.** NIST SP 800-61r2 (public) supplies:
**functional impact** (none / low / medium / high), **information impact**, and **recoverability
effort** (regular / supplemented / extended / not recoverable). For a facility, substitute **safety
impact** — arc-flash exposure, fuel or refrigerant release, life-safety impairment — for information
impact. Functional × safety × recoverability is a defensible SEV matrix derived from a public source.

NIST also states the rule most runbooks violate: "Incidents should not be handled on a first-come,
first-served basis as a result of resource limitations," and requires an explicit **non-response
path** — how long to wait for a response, and what to do if none comes.

**Incident command.** Take the form from **FEMA NIMS** (public domain), which is the ancestor of the
incident-command pattern the software industry later adopted: an **Incident Commander** with overall
authority; **Command Staff** (Public Information Officer, Safety Officer, Liaison Officer);
**General Staff** sections (Operations, Planning, Logistics, Finance/Administration); **Unified
Command** where more than one organisation holds jurisdiction — for a campus, the normal case
(operator, tenant, utility, OEM field service); a **manageable span of control**; an **Incident Action
Plan**; and named, drilled procedures for **establishing and transferring command**.

Two roles data-centre processes usually lack:

- The **Safety Officer**, who stops or prevents unsafe acts during the incident — an authority
  *independent* of the person driving restoration. At 03:00 the pressure to close a breaker is
  enormous, and the Incident Commander is the wrong person to also hold the veto.
- The **PIO**, under the rule that the IC approves release of incident information and everyone
  speaks with one voice. Cloudflare's PDX-04 postmortem shows the cost of its absence on the facility
  side: "Flexential did not inform Cloudflare that they had failed over to generator power" — the
  tenant learned from its own routers going offline.

**The blameless postmortem — the form.** Assembled from NIST §3.4.1 and the public Cloudflare
write-up:

1. **A timestamped timeline.** Facility incidents have three clocks — EPMS event log, BMS trend, and
   the operator's written log — and they will disagree. Reconciling them is the first act of the
   investigation.
2. **Impact in customer terms**, not component terms.
3. **Contributing factors, plural** — never "root cause" singular. PDX-04 lists a utility maintenance
   event, a ground fault, a protection scheme that shut down all generators, a battery bank that
   failed early, an untested facility-level dependency, and an overnight staffing shortfall. No one of
   them is sufficient.
4. **What went well**, explicitly.
5. **Detection and diagnosis latency measured separately from repair latency.**
6. **Action items with a named owner and a due date**, tracked in the same backlog as ordinary work.
7. **Blameless framing with a scope note.** Blameless means the analysis does not attribute cause to
   individual character. It does not mean the record omits who did what and when — a regulator, an
   insurer, or an SLA-credit process needs attribution. Run **two artefacts** from one investigation:
   the learning document and the contractual record. Pretending one document serves both is why
   postmortems get lawyered into uselessness.
8. **Timing** — within several days of the incident's end, while memory is intact.

*(Google's SRE chapters are the canonical modern write-up and are copyrighted: cite and link, do not
reproduce. Their own account credits the Incident Command System as ancestor, which is why teaching
the form from FEMA loses nothing.)*

### Human factors

**People are fallible.** DOE-HDBK-1028 Vol 1 states it as a principle: "Error is universal. No one is
immune regardless of age, experience, or educational level… No amount of counseling, training, or
motivation can alter a person's fallibility." And the second principle is the one that makes this an
engineering discipline: "**Error-likely situations are predictable, manageable, and preventable.**"

The trap that catches experienced technicians specifically: "an unsafe personal dependency exists when
an individual relies on his or her personal experience, proficiency, or qualifications to maintain
control… **Competence does not guarantee positive control.**"

**Error precursors** come in four categories — Task Demands, Individual Capabilities, Work
Environment, Human Nature — with named members including time pressure, unfamiliar or first-time
task, departure from routine, out-of-service instrumentation, hidden system response, **irreversible
actions**, complacency, and **fatigue**. Read that list against a typical out-of-hours MOP: seven of
them are structurally present in almost every one. **That is what a pre-job brief is for — a precursor
sweep, not a safety share.** Tag irreversible steps in the MOP explicitly.

**Pre-job briefing content** has a free specification in US law. 29 CFR 1910.269(c) requires the
employee in charge to brief before each job, covering "hazards associated with the job, work
procedures involved, special precautions, energy-source controls, and personal protective equipment
requirements," with a briefing before the first job of each day or shift for repetitive work and
additional briefings when significant changes occur. Even where that rule does not legally bind a
tenant-side data centre, adopting its five subjects verbatim costs nothing and makes the brief
non-arbitrary.

**Shift turnover** is both law and practice. 29 CFR 1910.147(f)(4) requires specific procedures during
shift or personnel changes "to ensure the continuity of lockout or tagout protection." DOE-HDBK-1028
Vol 2 supplies the content list, usable as a turnover template: job status, work completed and
remaining, equipment status and key parameters; schedule changes and parallel activities; objectives
in progress; **procedures being used and the last step completed**; problems, unusual conditions and
system line-ups; **critical steps, error-likely situations, countermeasures and contingencies**;
resource availability; key contacts.

The method matters as much as the list: face to face, three-way communication on critical items, the
on-coming person **independently** reviewing the log and status boards before assuming
responsibility, a **joint walkdown**, and an explicit transfer of responsibility. The listed at-risk
practice that decides all of it is structural: "turnovers not accommodated in the schedule — hurrying
through the process." **If the roster has zero paid overlap, the handover is unfunded and will be
skipped.**

**Three-way communication.** Sender states the message and names the receiver; receiver **repeats it
back**, paraphrasing intent but **restating equipment-related information exactly as spoken**; sender
confirms "that is correct" or corrects. Note the asymmetry: paraphrase proves understanding,
**verbatim repetition proves the designator**. The phonetic alphabet belongs here for the reason the
handbook gives plainly — "D" and "B" are confusable, "Delta" and "Bravo" are not.

**Self-checking and verification.** **STAR** — *Stop* (eliminate distractions), *Think* (verify the
action is appropriate for current equipment status; identify the expected result; consider a
contingency if an unexpected result occurs), *Act* (perform the correct action on the correct
component — without losing contact, **read and touch the component label and compare it with the
guiding document**), *Review* (verify the anticipated result; perform the contingency if it does not
occur). **Independent verification** — checking by a *separate qualified person* — is distinct from
concurrent verification (two people, same time, same action) and is what catches a shared mental model
before a load transfer.

**Fatigue.** There is **no US work-hour limit that binds data-centre operators.** The absence is the
risk. 10 CFR 26.205 shows what a defensible, auditable rule looks like — 16 work hours in any 24, 26
in any 48, 72 in any 7 days; a 10-hour break between successive work periods; a 34-hour break in any
9-day period; days off scaled to shift length; or a weekly average cap; plus training, documented
waivers, and annual audit. **Transfer honestly:** borrow the *control design* — numeric caps, minimum
breaks, waiver with recorded basis, audit — and do not claim equivalence of consequence class. A
campus is mission-critical; a nuclear licensee is safety-critical in a different sense.

**Normalisation of deviance.** The Columbia Accident Investigation Board (public domain) states the
mechanism best: evidence that the design was not performing as expected was "reinterpreted as
acceptable and non-deviant, which diminished perceptions of risk"; the first decision to accept rather
than eliminate a deviation "established a precedent"; and the engineering experience base "functioned
as an elastic waistband, expanding to hold larger deviations from the original design."

The data-centre translation needs no exaggeration:

- A UPS module that alarms and clears on its own, monthly, for a year.
- A generator that starts on the second crank "but it always has."
- A CRAH left in hand/manual after a service call and never returned to auto.
- A chilled-water valve overridden in commissioning and never released.
- Protection settings adjusted in the field and never reconciled with the coordination study.
- **A PM deferred four quarters in a row with an approval each time** — the approval is what converts
  a deviation into policy.

The counters are structural, not attitudinal: a **deferred-defect register with an ageing report
reviewed above the person who benefits from deferring**; a hard rule that repeat-clearing alarms are
investigated rather than acknowledged; and periodic reconciliation of *as-set* parameters against the
*as-designed* study.

### The electrical-safety boundary most of the industry gets wrong

This is the single highest-value correction in the module, and it is verbatim from US law.

**29 CFR 1910.147(a)(1)(ii)** — "This standard does not cover the following:"

- "(C) Installations under the exclusive control of electric utilities for the purpose of power
  generation, transmission and distribution…"
- "**(D) Exposure to electrical hazards from work on, near, or with conductors or equipment in
  electric-utilization installations, which is covered by subpart S of this part;**"

A data centre's distribution — switchgear, UPS, PDU, busway, RPP — **is** an electric-utilization
installation. So:

| Hazard | Regime |
|---|---|
| Electrical-hazard exposure on switchgear / UPS / PDU / busway | **Subpart S — 1910.331–.335, principally 1910.333** |
| Unexpected energisation, start-up, or stored-energy release in *machines and equipment* (chillers, pumps, towers, CRAH fans, dampers, generator prime movers, fuel systems) | **1910.147 (LOTO)** |
| The utility's own substation under its exclusive control | **1910.269** |

A campus with its own substation therefore runs **three overlapping regimes on one site**, and an
energy-control programme that cites only 1910.147 is mis-scoped. OSHA supplies a bridge: 1910.333(b)(2)
NOTE 2 deems LOTO procedures complying with 1910.147(c)–(f) to also comply with 1910.333(b)(2),
*provided* they address Subpart S hazards and incorporate the specified test and verification
requirements. One programme **can** cover both — only if written deliberately to do so.

**And the consequence that changes a design decision.** 1910.333 requires that live parts be
**deenergized before work**, with narrow exceptions; that "control circuit devices, such as push
buttons, selector switches, and interlocks, **may not be used as the sole means for deenergizing**"
and that interlocks may not substitute for lockout and tagging; and that a qualified person test and
verify absence of voltage, expressly including "inadvertently induced voltage or **unrelated voltage
backfeed**".

Read together: **putting a UPS into maintenance bypass from its HMI is not isolation.** Isolation
requires a physical disconnecting means, locked, stored energy released, absence of voltage verified
by test — including a backfeed check, which in a dual-corded A/B topology with downstream transfer
switches is not theoretical.

Therefore **concurrent maintainability is a legal isolation constraint, not just a topology label**:
if the one-line has no physical isolation point that leaves the load served, lawful maintenance
requires a load outage. That question — *can my electrician lawfully do this work without dropping the
floor?* — is decided in design, months before operations is staffed.

*(One genuinely contested reading, recorded rather than resolved: 1910.333(a)(1) NOTE 2 offers
"work on circuits that form an integral part of a continuous industrial process in a chemical plant"
as an infeasibility example. Whether an IT load qualifies is a scope question no public OSHA
interpretation settles. Do not build an operations model that depends on the permissive reading.)*

### Evidence hygiene — the habit that outlives this module

**Unlearn pointer, restated at the evidence gate.** Module 01 Q3 / the map objective treated power,
cooling, and human error as peer buckets — **that is the sentence this module retires.** Start at
the power path; treat cooling as a cascade; treat people and process as **contributing factors,
plural**. Do not replace the cartoon with a memorized majority percentage.

The most-repeated human-factors claim in this domain is that human error is involved in the large
majority of data-centre outages. It is attributed to a subscription industry report. Attempts to
obtain a primary source reach registration gateways with no figures and dead links; the report itself
sits behind a customer boundary.

The figure may well be correct. **It is not currently verifiable from anything freely readable**, and
repetition across many secondary sources is *circulation, not corroboration*.

The lesson generalises, and it is the most portable thing in this module:

1. Ask **who measured it**, on what population, in what year, with what definition.
2. Ask whether **you can read the measurement** — not a press release about it.
3. If not, teach and reason about the **mechanism** instead of the number. Everything in the human
   factors section above is grounded in freely quotable primary sources, and none of it needs that
   statistic.

An operator who can defend a decision and an operator who repeats a vendor deck are distinguished
almost entirely by this habit.

---

## Interview drills

1. *"Walk me through the last MOP you executed."* — Name the level of use, the prerequisites, one
   hold point, the abort criterion, and how you verified the end state. If you can say what would have
   made you stop, you sound like an operator.
2. *"Your availability target is slipping. What do you do first?"* — MTTR, not MTBF. Then say why:
   spares by criticality, labelling and findability, pre-written EOPs, and authority to act.
3. *"How would you know your monitoring is lying to you?"* — Heartbeat and staleness alarms; a
   post-maintenance functional test confirmed by an operator; and the rule that an external report
   contradicting your screens is evidence about your screens.
4. *"Who governs LOTO on our switchgear?"* — Subpart S for electrical-hazard exposure; 1910.147 for
   unexpected energisation of machines; 1910.269 for the utility's substation. Very few candidates
   know this.
5. *"What was in your last postmortem?"* — Contributing factors plural, detection latency separate
   from repair latency, owned action items with dates, and two artefacts if there was contractual
   exposure.
6. *"What is in the service catalog, and which OLA makes that SLA true?"* — Name a SKU, an owner, the
   customer SLA, the internal OLA, and the vendor UC. A brochure is not a catalog.
7. *"CDU is leaking on row 12 — what do you isolate?"* — This loop's valves, remaining N+1, pre-
   authorized shed of *that* loop if capacity is gone. Not the CHW header. Not the pod.
8. *"BESS / Li-ion alarm — Class A?"* — No. Evacuate. Room vs yard. Follow the site EOP and the
   adopted 855 edition. Do not dump clean agent and stay. Do not invent a fire %.
9. *"Power, cooling, and human error — three buckets?"* — That is the sentence this module retires.
   Power path leads; cooling is a cascade; people are contributing factors, plural.

---

## Self-check

1. **A procedure's "level of use" primarily determines:**
   a) Who signs the approval block
   b) Whether the document is open in hand at the step, or read beforehand and worked from memory
   c) The font size of warnings
   d) The retention period of the record

2. **Clear labelling of power circuits, breakers, and cables primarily reduces:**
   a) The need for any documentation forever
   b) Human error during isolation, restoration, and incident response
   c) Chilled-water approach temperature
   d) Camera retention requirements

3. **As-built documentation that diverges from reality is dangerous because:**
   a) It wastes storage
   b) It is trusted and acted on — a wrong drawing produces a confident wrong action
   c) It slows down printing
   d) It voids the fire certificate

4. **Availability is improved not only by higher MTBF but also by:**
   a) Lower MTTR — faster detection, diagnosis, and restoration
   b) Longer PM intervals
   c) More alarms
   d) Higher nameplate power

5. **A maintenance SLA that promises four-hour onsite response but stocks no critical spares may still
   yield long outages because:**
   a) Response time is the same as restoration time
   b) Arrival starts the logistics clock rather than ending the outage
   c) MTBF rises automatically
   d) Vendors always carry every part

6. **Cleaning under raised floors and inside perforated tiles should be coordinated with cooling
   operations because:**
   a) Cleaners prefer daylight
   b) The plenum is a live supply-air path and lifting tiles alters the pressure profile
   c) It reduces fire loading
   d) It is required for camera coverage

7. **An alarm, as distinct from a status indicator, is defined by:**
   a) Its colour on the dashboard
   b) A required operator action with a defined response
   c) Being generated by the EPMS
   d) Having a red border

8. **On a data-centre campus, electrical-hazard exposure while working on the UPS and switchgear is
   governed principally by:**
   a) 29 CFR 1910.147 alone
   b) OSHA Subpart S (1910.331–.335), because the installation is an electric-utilization installation
   c) The utility's own tariff
   d) The building code only

9. **A data-centre service catalog is primarily:**
   a) The sales brochure with logos
   b) The owned list of delivered services — SKU, owner, SLO pointer, and the OLA or UC underneath
   c) The CMMS asset register
   d) A percentage of exam questions

10. **An Operational Level Agreement (OLA) is:**
    a) The customer-facing availability credit schedule
    b) An internal team-to-team commitment that makes the SLA possible
    c) A vendor four-hour response clause
    d) A synonym for underpinning contract

11. **A security matrix is:**
    a) The camera count on the perimeter
    b) A table of role × zone × privilege — a badge that opens every zone is not a matrix
    c) The Module 13 layer diagram copied into ops
    d) The visitor sign-in book

12. **Isolating a leaking CDU without killing the pod means:**
    a) Closing the hall chilled-water header
    b) Isolating *this* loop after confirming remaining CDU capacity (or shedding the pre-authorized
       load on that loop) — not dumping the pod
    c) Acknowledging the raised-floor rope and waiting
    d) Putting the UPS into maintenance bypass from the HMI

### Answers

<details>
<summary>Click to reveal answers</summary>

1. **b** — Use-each-time vs reference use is the behavioural attribute; DOE O 422.1 requires it be
   specified.
2. **b** — Labels are the substrate for isolation and for verbatim designator read-back.
3. **b** — Trusted-but-wrong beats missing, in the bad sense. Currency is the property to gate.
4. **a** — `Availability = MTBF / (MTBF + MTTR)`; MTTR is usually the cheaper lever.
5. **b** — Response ≠ restoration. Specify parts stocking and the clock definition in the contract.
6. **b** — Underfloor cleaning is a change to a live air path, not housekeeping.
7. **b** — "Process status indicators should not be designated as alarms" (HSE, *Better alarm
   handling*).
8. **b** — 1910.147(a)(1)(ii)(D) expressly excludes this exposure and points to Subpart S.
9. **b** — Catalog is owned delivery, not a brochure and not an exam-weight.
10. **b** — SLA is customer; OLA is internal; UC is vendor. They are a stack, not synonyms.
11. **b** — Module 13 owns layers; this file owns the matrix.
12. **b** — This-loop isolation with an N+1 hold point. HMI bypass is not isolation.

</details>

---

## Further free resources

All freely readable and redistributable unless noted:

| Resource | Why it helps |
|---|---|
| **29 CFR 1910.147** (control of hazardous energy) | The LOTO programme requirements — and, in (a)(1)(ii)(D), the exclusion almost everyone misses |
| **29 CFR 1910.331–.335** (Subpart S, esp. 1910.333) | Deenergise-first, control devices are not isolation, backfeed verification |
| **29 CFR 1910.269(c)** | The best free specification for a pre-job briefing anywhere in US law |
| **10 CFR 26.205** | What a defensible, auditable work-hour rule looks like numerically |
| **DOE O 422.1** (Conduct of Operations) | The enumeration of a technical-procedure programme, including level of use |
| **DOE-STD-1029-92** (writer's guide) | Step grammar; warning and caution placement |
| **DOE-HDBK-1028-2009 Vols 1–2** (human performance) | STAR, three-way communication, turnover, pre-job brief, error precursors |
| **DOE/PNNL O&M Best Practices Guide R3.0** | Maintenance-mix benchmarks and the seven contract metrics |
| **NASA RCM Guide (2008)** | Conditional-probability-of-failure evidence; PM-induced infant mortality |
| **NIST SP 800-61r2** | Incident prioritisation on three axes; lessons-learned agenda |
| **FEMA NIMS (2017)** | Incident Commander, Safety Officer, Unified Command, span of control |
| **HSE HSG250** (permit to work) · **HSE CHIS6** (*Better alarm handling*) | Permit features and hand-back; alarm rate and priority targets |
| **U.S.–Canada Power System Outage Task Force Final Report (2004)** | The silent alarm-failure case, used precisely |
| **CAIB Report Vol I, Ch. 8 (2003)** | Normalisation of deviance, stated better than any secondary source |
| **Cloudflare PDX-04 postmortem (2023)** | A data-centre facility incident in the open, from the tenant's side |
| *Named, not quoted (licensed or paywalled):* NFPA 70E, NFPA 70B, ASHRAE 135/BACnet, ASHRAE Guideline 0 (Cx process — vocabulary here, syllabus in Module 02), ISO/IEC 30134 (PUE / WUE / CUE — lattice in Module 02), ANSI/ISA-18.2, EEMUA 191, ISO 14644 particle classes, Uptime Tier and M&O criteria | Obtain licensed copies before writing a site programme against them |

**Study tip:** take one real MOP — yours or a vendor's — and audit it against three things: does it
declare its level of use; are its warnings on the same page as their steps; and does it contain the
sentence *"if an unexpected result occurs, stop."* Most fail all three, and noticing that is the
skill.

---

*Module ID: `15-ops-adjacent` · Depth: standard · **2.1 Operational Considerations** on the 15-module
map — **exam weight unknown**, not one of the 14 public EPI CDCP facility domains. Part of free
CDCP-domain self-study (not official EPI®/CDCP® certification material).*
