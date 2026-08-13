# Drill Cards — CDCP Self-Study

40 flash-style prompts for rapid recall. Cover the full syllabus (M1–M14). Use **Front** first; only then flip to **Back**.

**Modes**
- **Exam mode:** 5 seconds think → answer out loud → check Back. Miss = restudy module.
- **Interview mode:** Answer in 30–45 seconds with a trade-off or example, not just a definition.

---

### Card 1
**Front:** What is ATS?  
**Back:** Automatic Transfer Switch — electromechanical source selector (e.g. utility ↔ generator), typically slower break-before-make transfer (cycles–seconds).

### Card 2
**Front:** What is STS?  
**Back:** Static Transfer Switch — solid-state transfer between two live AC sources, typically sub-cycle to few-ms class when sources are in tolerance.

### Card 3
**Front:** ATS vs STS — one-line contrast?  
**Back:** ATS answers “utility died, start genset and switch.” STS answers “two live sources, nearly seamless transfer.”

### Card 4
**Front:** What is UPS?  
**Back:** Uninterruptible Power Supply — bridges utility/generator gaps and conditions power for critical loads using stored energy (batteries/flywheel/etc.).

### Card 5
**Front:** What is double-conversion / online UPS?  
**Back:** VFI-class path where load is normally supplied via rectifier→inverter (AC–DC–AC), isolating many utility disturbances.

### Card 6
**Front:** What is N vs N+1 vs 2N?  
**Back:** **N** = capacity for full design load. **N+1** = N plus one spare unit. **2N** = two full independent paths each able to carry the load.

### Card 7
**Front:** What are A-B feeds?  
**Back:** Two independent power paths to dual-corded IT so loss of one path should not drop the equipment.

### Card 8
**Front:** What is a PDU?  
**Back:** Power Distribution Unit — room/row transformer-PDU, busway tap, or rack strip (rPDU); resolve level from context.

### Card 9
**Front:** What is busbar / busway?  
**Back:** Rigid bar or enclosed bus duct distributing power along a run with flexible tap-offs, often overhead in white space.

### Card 10
**Front:** Draw (words) the utility-to-rack power path.  
**Back:** Utility/gen → main switchgear/transfer (ATS/STS) → transformer as needed → UPS → downstream distribution/PDU/busway → rack PDU A/B → IT PSUs.

### Card 11
**Front:** What is PUE?  
**Back:** Power Usage Effectiveness = total facility energy ÷ IT equipment energy. Closer to 1.0 means less overhead.

### Card 12
**Front:** What is WUE?  
**Back:** Water Usage Effectiveness — site water use relative to IT energy; tracks cooling-water intensity.

### Card 13
**Front:** What is white space vs grey space?  
**Back:** **White space** = IT floor (racks/aisles). **Grey space** = MEP plant (UPS, gens, chillers, switchgear).

### Card 14
**Front:** What is CRAC vs CRAH?  
**Back:** **CRAC** ≈ DX/refrigerant computer-room AC. **CRAH** ≈ chilled-water computer-room air handler (central chiller plant).

### Card 15
**Front:** What is containment?  
**Back:** Physical separation of hot and cold air (CAC or HAC) to stop mixing and improve cooling capacity/efficiency.

### Card 16
**Front:** Hot aisle vs cold aisle?  
**Back:** **Cold aisle** faces intakes (supply). **Hot aisle** faces exhausts (return/capture).

### Card 17
**Front:** Why blanking panels?  
**Back:** Block unused U-space so cold air doesn’t bypass IT and short-circuit to the hot side.

### Card 18
**Front:** Sensible vs latent heat in a DC?  
**Back:** IT load is mostly **sensible** (temperature rise). **Latent** is moisture phase-change — people, outdoor air, humidify/dehumidify.

### Card 19
**Front:** What is a CDU?  
**Back:** Coolant Distribution Unit — pumps/exchanges/controls liquid-cooling loops between facility and rack/chip loops.

### Card 20
**Front:** What is ASHRAE TC 9.9 about (interview level)?  
**Back:** Thermal guidelines — recommended/allowable temp/humidity envelopes for IT equipment classes.

### Card 21
**Front:** What is BMS?  
**Back:** Building Management System — monitors/controls facility plant (HVAC, chillers, many electrical status points).

### Card 22
**Front:** What is DCIM?  
**Back:** Data Centre Infrastructure Management — inventory, power, environment, capacity, assets spanning IT and facilities views.

### Card 23
**Front:** BMS vs DCIM in one sentence each?  
**Back:** BMS runs the building plant. DCIM manages DC capacity/assets/environment from an infrastructure-ops lens. Overlap exists; roles differ by site.

### Card 24
**Front:** What is EMS (and the ambiguity)?  
**Back:** May mean **Energy** Management System or **Environmental** Monitoring System — always clarify on that site.

### Card 25
**Front:** What is a MOP?  
**Back:** Method of Procedure — reviewed step-by-step plan for risky work: steps, owners, verification, backout.

### Card 26
**Front:** What is IST?  
**Back:** Integrated Systems Testing — commissioning-style tests proving power, cooling, and controls work together under failover scenarios.

### Card 27
**Front:** What is concurrent maintainability?  
**Back:** Ability to take any single capacity component offline for maintenance without dropping the critical load.

### Card 28
**Front:** What is EMF and a common source?  
**Back:** Electromagnetic fields; strong sources include transformers, busbars, UPS/plant rooms, elevators, RF.

### Card 29
**Front:** What is one RU?  
**Back:** Rack Unit = 1.75 in (44.45 mm) vertical space; 42U cabinets are common.

### Card 30
**Front:** What is TIA-942 (conceptually)?  
**Back:** ANSI/TIA data-centre infrastructure standard family (pathways, cabling, redundancy language / Rated concepts) — not identical to Uptime Tier.

### Card 31
**Front:** What is an AHJ?  
**Back:** Authority Having Jurisdiction — local enforcer of code (fire, electrical, building). Code beats voluntary standards when they conflict.

### Card 32
**Front:** Why might a new high-density hall skip raised floor?  
**Back:** Slab/hard floor with overhead busway, row/liquid cooling, and cable trays can fit modern density better; underfloor air has limits.

### Card 33
**Front:** What is a mantrap?  
**Back:** Interlocked dual-door entry preventing a continuous open path from public to secure space.

### Card 34
**Front:** Clean agent vs wet pipe in white space (concept)?  
**Back:** Clean agent aims to suppress without water damage profile; wet systems have different cost/effectiveness/trade-offs — design and AHJ decide.

### Card 35
**Front:** What is aspirating / VESDA-class detection for?  
**Back:** Very early smoke detection by continuous air sampling — warning before conventional detectors.

### Card 36
**Front:** What is an MMR?  
**Back:** Meet-Me Room — carrier/customer interconnection and cross-connect space.

### Card 37
**Front:** Generator vs UPS roles?  
**Back:** UPS = short-duration bridge + conditioning. Generator = long-duration energy after start/transfer and fuel logistics.

### Card 38
**Front:** What is thermography used for in power rooms?  
**Back:** IR scanning finds hot electrical connections/components before failure under load.

### Card 39
**Front:** RTO vs RPO?  
**Back:** **RTO** = how fast service must return. **RPO** = how much data loss (time) is acceptable.

### Card 40
**Front:** Walk a white-space tour in 5 bullets.  
**Back:** (1) Power path A-B to racks (2) Cooling path / containment (3) Cabling pathways (4) Fire detection/suppression zones (5) Security layers + what BMS/DCIM watches.

---

## Drill sets (optional rotation)

| Set | Cards | Focus |
|---|---|---|
| Power core | 1–12, 37–38 | Path, redundancy, UPS, efficiency |
| Thermal | 13–20 | Cooling vocabulary |
| Ops & life safety | 21–27, 33–35 | Process, monitoring, fire, security |
| Integration | 28–32, 36, 39–40 | Standards, site, tour narrative |

**Spaced repetition tip:** Missed cards → again same day → next day → Day 14 full shuffle.

---

*Educational flash prompts. Not official exam content.*
