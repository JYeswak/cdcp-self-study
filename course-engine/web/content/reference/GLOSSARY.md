# CDCP Self-Study Glossary

Crisp definitions for interview-ready data-centre facilities language. Terms track this program’s 14 modules (mission-critical site → auxiliary systems). Not an official EPI dictionary.

**How to use:** Skim once after Module 1–2; drill with [`../practice/DRILL-CARDS.md`](drill.html) before the practice exam.

---

## A–C

| Term | Definition |
|---|---|
| **A-B feeds** | Two independent power paths (A side and B side) to dual-corded IT equipment so loss of one path does not drop the load. |
| **AHJ** | Authority Having Jurisdiction — the local body (fire marshal, electrical inspector, building department) whose code interpretation prevails on site. |
| **ATS** | Automatic Transfer Switch — electromechanical switch that selects between two sources (e.g. utility ↔ generator), typically on a break-before-make cycle measured in cycles to seconds. |
| **Availability** | Fraction of time a service or system is up and usable; often expressed as “nines” (e.g. 99.9%). Distinct from reliability (how often failures occur). |
| **ASHRAE (TC 9.9)** | Thermal guidelines for data processing environments — recommended and allowable temperature/humidity envelopes for IT equipment classes. |
| **BESS** | Battery Energy Storage System — large-scale battery plant used for ride-through, peak shaving, grid services, or extended UPS-class support (broader than traditional UPS strings alone). |
| **Blanking panel** | Solid filler in unused rack U-space that blocks cold air from bypassing equipment and mixing into the hot aisle. |
| **BMS** | Building Management System — monitors and controls facility plant (HVAC, chillers, generators status points, dampers, often some electrical telemetry). |
| **Bonding** | Intentional low-impedance connection of conductive parts so they stay at the same potential under fault; pairs with grounding/earthing. |
| **Busbar / busway** | Rigid conductive bar or enclosed bus duct distributing power along a run (row/overhead) instead of discrete long cable pulls to every load. |
| **Bypass (UPS)** | Path that feeds load around the UPS inverter (static or maintenance bypass) for transfer, failure, or service. |
| **CAC** | Cold Aisle Containment — physical barriers that keep cold supply air in the cold aisle and reduce mixing with hot exhaust. |
| **CDU** | Coolant Distribution Unit — pumps, heat exchange, and control for liquid-cooling loops (e.g. facility water ↔ secondary loop to racks/chips). |
| **CHW** | Chilled Water — plant water loop cooled by chillers and delivered to CRAH coils or other heat exchangers. |
| **Cold aisle** | Aisle facing equipment air intakes, supplied with cold air (underfloor, overhead, or in-row). |
| **Colocation (colo)** | Multi-tenant facility where the provider supplies building, power, cooling, and security; the customer owns/operates IT in cages or suites. |
| **Comfort cooling** | HVAC designed for people (wide bands, more latent capacity, occupied-hours focus) — wrong primary tool for dense IT white space. |
| **Concurrent maintainability** | Design property: any single capacity component can be taken offline for maintenance without dropping the critical load. |
| **Containment** | Physical separation of hot and cold air streams (cold-aisle or hot-aisle) to improve cooling efficiency and capacity. |
| **CRAC** | Computer Room Air Conditioner — precision cooling unit typically using direct-expansion (DX) refrigerant with a compressor in the cooling chain. |
| **CRAH** | Computer Room Air Handler — precision air handler usually on chilled water; fans and coil only (chiller is central plant). |
| **Critical load** | IT (and sometimes life-safety) load that must stay powered through utility events within the design autonomy window. |
| **Cross-connect** | Structured connection between customer and carrier/provider circuits, often in an MMR or meet-me space. |

## D–G

| Term | Definition |
|---|---|
| **DCIM** | Data Centre Infrastructure Management — IT-centric inventory, power, environment, capacity, and often change/asset views spanning facilities and IT. |
| **Dew point** | Temperature at which moisture condenses from air; cold surfaces below dew point create water risk near power. |
| **Distributed redundant** | Redundancy topology where spare capacity is shared across multiple paths/systems rather than a simple dedicated spare or full dual plant. |
| **Double-conversion UPS** | Online UPS topology: AC→DC→AC continuously so load is isolated from most utility disturbances (VFI class). |
| **Dry-bulb temperature** | Ordinary air temperature measured by a dry thermometer (°C/°F). |
| **DX** | Direct Expansion — refrigerant expands in the cooling coil (typical of many CRAC systems). |
| **Economizer (free cooling)** | Using cool outdoor air or water conditions to reduce mechanical refrigeration runtime. |
| **Edge data centre** | Smaller facility near users/devices optimized for latency and local processing, often with thinner staffing and fewer redundant layers. |
| **Egress lighting** | Illumination of exit paths required for safe evacuation under emergency conditions. |
| **EMF** | Electro Magnetic Fields — electric and magnetic fields from power equipment, RF, elevators, etc., that can affect people or sensitive gear. |
| **EMS** | Energy (or Environmental) Management System — depending on context: energy metering/optimization platform, or environmental monitoring (temp/humidity/leaks); always clarify which “E” is meant on a given site. |
| **EN standards** | European Norms — European standards family used alongside or instead of TIA/ISO in EU contexts. |
| **Enterprise DC** | Facility owned/operated primarily for one organization’s own IT workloads. |
| **EPO** | Emergency Power Off — shutoff means (where code/design requires) that can de-energize designated equipment/areas in an emergency. |
| **ESD** | Electrostatic Discharge — static spark risk; controlled by humidity bands, flooring, bonding, and handling practices. |
| **Fault tolerance (concept)** | Ability to withstand a fault without interrupting the critical load (stronger than mere concurrent maintainability; language varies by rating scheme). |
| **Five-nines** | 99.999% availability ≈ ~5.26 minutes unplanned downtime per year if continuous operation is assumed. |
| **Generator (standby)** | On-site engine-generator that starts after utility loss to supply long-duration power; not instantaneous — UPS covers start/transfer gap. |
| **Grey space (gray space)** | MEP plant areas — UPS rooms, switchgear, generators, chillers, batteries — as opposed to IT white space. |
| **Grounding (earthing)** | Connection of the electrical system/equipment to earth reference for fault clearing and safety; practices are code- and region-specific. |

## H–N

| Term | Definition |
|---|---|
| **HAC** | Hot Aisle Containment — barriers that capture hot exhaust for return to cooling units, isolating it from cold supply. |
| **Hot aisle** | Aisle facing equipment exhausts where hot air is collected for return. |
| **HPC** | High-Performance Computing — dense compute that drives high kW/rack and often liquid cooling. |
| **Hot spot** | Localized overtemperature from airflow short-circuit, missing blanking, blocked exhaust, or overloaded cooling zone. |
| **Hyperscale** | Very large, highly automated facilities (cloud/web scale) optimized for density, cost, and fleet uniformity. |
| **In-row cooler** | Cooling unit placed in the rack row for short air path and higher local density support. |
| **Isolation transformer** | Transformer providing galvanic isolation between primary and secondary; can help noise/ground issues but is not a UPS substitute. |
| **IST** | Integrated Systems Testing — end-to-end commissioning tests proving power, cooling, controls, and failovers work together under load scenarios. |
| **kVA** | Kilovolt-ampere — apparent power; UPS and transformer nameplates often in kVA. |
| **kW** | Kilowatt — real power; IT load and heat are discussed in kW. |
| **Latent heat** | Heat tied to moisture phase change (humidify/dehumidify); secondary to sensible heat for pure IT loads. |
| **Line-interactive UPS** | UPS class that normally feeds from utility with voltage regulation and switches to battery/inverter on deeper disturbances (not full double-conversion). |
| **Load bank** | Resistive (or other) load used to test generators/UPS under real power without relying on live IT. |
| **MDA** | Main Distribution Area — primary structured-cabling cross-connect / core distribution space in TIA-942-style topology. |
| **HDA** | Horizontal Distribution Area — intermediate distribution serving a zone of cabinets (LAN/SAN aggregation and horizontal cross-connects). |
| **EDA** | Equipment Distribution Area — rack/cabinet zone where IT equipment and equipment outlets live. |
| **ZDA** | Zone Distribution Area — optional consolidation point in white space for flexible zone cabling. |
| **IP rating (IEC 60529)** | Ingress Protection code for enclosure resistance to dust/solids and water (e.g. IP54); not the same as Internet Protocol. |
| **Microgrid** | Local multi-source energy system (utility, gens, renewables, BESS) that can manage and sometimes island critical load under designed controls. |
| **MMR** | Meet-Me Room — space where carriers and customers interconnect fiber/copper cross-connects. |
| **Modular DC** | Prefabricated or block-deployable capacity units for faster, stepwise growth. |
| **MOP** | Method of Procedure — step-by-step, reviewed procedure for high-risk maintenance or change (who, what, backout, verification). |
| **N** | Baseline capacity required to support the full design critical load with no spare. |
| **N+1** | Capacity equal to N plus one spare unit/module so one can fail or be maintained without dropping load (at that layer). |
| **2N** | Two complete independent capacity paths, each able to carry the full load (full dual systems). |
| **2N+1** | Dual path (2N) plus additional spare capacity beyond dual; rare language — confirm site definition. |
| **NOC** | Network (or Network Operations) Centre — ops space monitoring connectivity and often broader service health. |

## O–R

| Term | Definition |
|---|---|
| **Online UPS** | Colloquial for double-conversion/VFI UPS where the inverter normally supplies the load continuously. |
| **PDU** | Power Distribution Unit — room/row transformer-PDU, busway tap, or rack-mounted strip (rPDU); always resolve which level from context. |
| **Power factor** | Ratio of real power (kW) to apparent power (kVA); low PF means more current for the same kW. |
| **Precision cooling** | Continuous, high-sensible cooling designed for IT (CRAC/CRAH class), not comfort HVAC. |
| **PUE** | Power Usage Effectiveness — total facility energy ÷ IT equipment energy; 1.0 is ideal, higher means more overhead. |
| **Raised floor** | Access floor on pedestals creating underfloor plenum for cable and/or cold air supply. |
| **RDHx** | Rear-Door Heat Exchanger — liquid-cooled door on the rack rear that removes heat at the cabinet. |
| **Redundancy** | Extra capacity or alternate paths so failure or maintenance of one element does not drop the critical load. |
| **Reliability** | Likelihood a component/system performs without failure over a period; complementary to availability. |
| **RPO** | Recovery Point Objective — maximum acceptable data loss measured in time (how stale backups/replicas may be). |
| **RTO** | Recovery Time Objective — maximum acceptable time to restore a service after disruption. |
| **RU (U)** | Rack Unit — 1.75 in (44.45 mm) of vertical rack space; standard 42U cabinet is common. |
| **SCR** | Silicon Controlled Rectifier — solid-state device used in static switches (STS) and power electronics. |
| **Sensible heat** | Heat that changes dry-bulb temperature; dominant heat form from IT equipment. |
| **Single point of failure (SPOF)** | Component or path whose sole failure causes outage of the critical function. |
| **SLA** | Service Level Agreement — contractual uptime/response commitments between provider and customer. |
| **SRG (Signal Reference Grid)** | Equipotential bonding grid (often under/at raised floor) intended as a low-impedance reference to reduce noise/potential differences between equipment; not a substitute for protective earthing or a freeform DIY ground scheme. |
| **STS** | Static Transfer Switch — solid-state transfer between two live AC sources, typically sub-cycle to few-ms class for seamless switchover when sources are in tolerance. |
| **STES / STER** | Seasonal Thermal Energy Storage / Seasonal Thermal Energy Reservoir (or related storage concepts) — storing thermal energy across seasons to improve efficiency; appear in advanced efficiency discussions. |
| **Stringer** | Lateral member in a raised-floor system that braces pedestals and supports tiles. |
| **Suspended ceiling** | Overhead ceiling system that may hide cable, form return-air plenum, or manage aesthetics/smoke — trade-offs with access and cooling. |

## S–Z

| Term | Definition |
|---|---|
| **Three-phase power** | AC system with three phase conductors (plus neutrals/grounds as designed); backbone of facility distribution for efficiency and motor/UPS loads. |
| **Tier (Uptime)** | Commercial Uptime Institute rating system (I–IV) describing topology and fault/maintainability attributes — not the same as TIA-942 Rated levels. |
| **TIA-942** | ANSI/TIA standard family for data centre infrastructure (cabling, pathways, redundancy concepts, Rated-1…4 style language in current editions). |
| **Thermography (IR scan)** | Infrared imaging of electrical connections and equipment to find hot connections before failure. |
| **Transformer** | Magnetic device transferring energy between circuits, usually changing voltage (e.g. MV→LV). |
| **UPS** | Uninterruptible Power Supply — bridges utility/generator gaps and conditions power for critical loads using batteries, flywheels, or other stored energy. |
| **Utility feed** | Electrical service from the grid at the site boundary; dual feeds improve source diversity when truly independent. |
| **VFI** | Voltage and Frequency Independent — UPS classification (IEC) for double-conversion online behavior. |
| **VI / VFD** | Voltage Independent / Voltage and Frequency Dependent — other UPS classification bands (line-interactive vs passive standby concepts). |
| **Wet stacking** | Diesel generator condition from chronic light-load running where unburned fuel/soot accumulates — why load-bank testing matters. |
| **White space** | IT equipment floor area (racks, aisles, network gear) as opposed to grey/MEP space. |
| **WUE** | Water Usage Effectiveness — annual site water use ÷ IT energy (liters/kWh class metric); tracks cooling water impact. |
| **19-inch rack** | EIA-standard rack width for mounting flanges; equipment width convention, not external cabinet width. |

---

## Extra high-frequency interview terms

| Term | Definition |
|---|---|
| **Autonomy (battery)** | Designed runtime of UPS batteries at a stated load before generators must assume the load or load must shed. |
| **Bus duct** | Enclosed busway system for power distribution (see busbar/busway). |
| **Cage** | Locked mesh enclosure for a colo customer’s cabinets within a shared hall. |
| **Change freeze** | Period when non-essential changes are blocked (e.g. peak business events). |
| **Chiller** | Plant machine that produces chilled water by rejecting heat to condenser water or air. |
| **Commissioning (Cx)** | Structured verification that installed systems meet design intent; IST is a deep form of this. |
| **Diversity (electrical)** | Assumption that not all loads peak simultaneously; used in sizing — dangerous if misapplied to guaranteed IT peaks. |
| **Dual-corded** | IT device with two power supplies intended for A and B feeds. |
| **Fail-closed / fail-open** | Control behavior on loss of power/signal (e.g. fire dampers, door locks) — must match safety design. |
| **Fuel oil system** | Storage, transfer, polishing, and delivery of diesel (or other fuel) to generators. |
| **Hot work** | Welding/cutting/spark-producing work under permit due to fire risk. |
| **Humidification** | Adding moisture to air to hold RH targets and reduce ESD risk when required. |
| **Inverter (UPS)** | Stage that converts DC (battery/rectifier bus) to AC for the load. |
| **Leak detection** | Sensors/cables that alarm on water under floors, at CRAC pans, or along pipes. |
| **Maintenance bypass** | Manual path to power load while UPS is fully isolated for service. |
| **Mantrap** | Interlocked two-door entry preventing open path from public to secure space. |
| **MEP** | Mechanical, Electrical, Plumbing — core facility engineering disciplines. |
| **Parallel UPS** | Multiple UPS modules sharing load for capacity and/or redundancy (N+1 paralleling common). |
| **Plenum** | Air space used as part of HVAC path (underfloor supply, ceiling return). |
| **Rack PDU (rPDU)** | Power strip/metered unit inside the rack fed from A and/or B. |
| **Rectifier** | UPS stage converting AC to DC to charge batteries and feed the DC bus/inverter. |
| **Remote hands** | Colo provider tech labor on customer gear under ticket. |
| **Row-based cooling** | Cooling architecture centered on in-row units rather than perimeter CRAC/CRAH only. |
| **Seismic bracing** | Structural restraint of racks, trays, and plant for earthquake loads where required. |
| **Single-phase power** | Two-wire (+ neutral) AC common at smaller loads/outlets; less common as sole plant backbone. |
| **Static switch** | Solid-state transfer device (core of STS; also UPS internal static bypass). |
| **String (battery)** | Series/parallel set of battery cells/blocks treated as one UPS energy source. |
| **Suppression (fire)** | Agent systems (water, clean agent gas, etc.) that extinguish or control fire after detection. |
| **Transfer time** | Duration of power interruption or switchover during ATS/STS/UPS events. |
| **VESDA / aspirating detection** | Very early smoke detection by sampling air through pipes — common high-sensitivity option. |
| **Wholesale colo** | Large private suite/pod model with more customer control than retail cages. |

---

**Count:** 100+ defined terms. Prefer this glossary’s wording in drills; for code-critical work, defer to AHJ and licensed engineers.

*Educational reconstruction. Not EPI official terminology.*
