# Practice Exam — CDCP Self-Study Capstone

**40 original multiple-choice questions** covering Modules 1–14. Same *count* as the publicly advertised EPI CDCP exam format; **not** official questions, not a dump, not affiliated with EPI/EXIN.

| | |
|---|---|
| **Questions** | 40 |
| **Suggested time** | 60 minutes |
| **Self-pass bar** | 27/40 (67.5%) as a study signal only |
| **Style** | Closed notes recommended first pass |
| **Afterward** | Check answer key; drill misses with modules + [`DRILL-CARDS.md`](./DRILL-CARDS.md) |

**Honesty:** Completing this does not certify you. Wrong answers are learning data, not failure theater.

---

## Questions

### 1. Mission-critical framing
A payment processor defines a site as mission-critical mainly because:
- A) All racks exceed 10 kW  
- B) Sustained outage would cause severe business, legal, or safety impact  
- C) The building has a raised floor  
- D) PUE is below 1.4  

### 2. Availability vs reliability
Which statement is most accurate?
- A) Availability and reliability are identical metrics  
- B) Reliability is about how often failures occur; availability includes uptime fraction considering repair  
- C) Five-nines always means dual utility feeds  
- D) Availability only applies to network links, not power  

### 3. Data centre types
In **retail colocation**, the customer typically:
- A) Owns the chillers and generators  
- B) Owns IT gear in cages/cabinets while the provider owns shared hall power/cooling  
- C) Must use only provider-owned servers  
- D) Cannot use dual-corded equipment  

### 4. Unavailability causes
When a hall outage is written up, human/process during change is treated as:
- A) A third peer root-cause bucket next to power and cooling  
- B) One of several contributing factors on a power, cooling, or network object — not a pie slice and not a memorized survey percentage  
- C) Using LED lighting  
- D) Over-documenting MOPs  

### 5. Standards landscape
Which is generally **enforceable as code** rather than a voluntary industry standard?
- A) A vendor white paper on airflow  
- B) Local electrical and fire codes enforced by the AHJ  
- C) An internal runbook preference  
- D) A blog post comparing PUE  

### 6. TIA-942 vs Uptime Tier
The most careful interview answer is:
- A) They are identical rating systems  
- B) TIA-942 Rated concepts and Uptime Institute Tier are separate frameworks; do not treat brand names as interchangeable  
- C) Only ISO rates data centres  
- D) ASHRAE issues Tier certificates  

### 7. Site selection
Which site factor most directly threatens long-term dual-utility feasibility?
- A) Paint color of the lobby  
- B) Single substation/path geography with no realistic second feed  
- C) Having too many conference rooms  
- D) Using 19-inch racks  

### 8. Supporting spaces
A white-space design can still fail operationally if the site lacks adequate:
- A) Generator yard, staging, NOC/security ops, and utility intake space  
- B) Only decorative suspended ceiling  
- C) Hot-aisle posters  
- D) Non-metered PDUs  

### 9. Raised floor purpose
A raised-floor plenum is often used to:
- A) Store diesel fuel  
- B) Deliver cold air and/or route cabling beneath the IT floor  
- C) Replace the need for UPS  
- D) Eliminate grounding  

### 10. Floor loading
Which loading concept matters when rolling a heavy UPS battery cart across tiles?
- A) Only uniform load rating  
- B) Rolling load (and concentrated load) ratings of the floor system  
- C) PUE only  
- D) Lux level  

### 11. Airflow and tiles
Removing floor tiles in a cold-aisle supply design without a plan often:
- A) Improves global pressure balance always  
- B) Can short-circuit supply air and starve distant racks  
- C) Increases battery autonomy  
- D) Disables the BMS  

### 12. Lighting
Emergency/egress lighting is primarily intended to:
- A) Reduce PUE  
- B) Support safe evacuation and life safety when normal power/light fails  
- C) Cool the hot aisle  
- D) Replace generator testing  

### 13. Power path order
Which sequence best matches a typical critical power path?
- A) Rack → UPS → Utility → Generator  
- B) Utility/generator → transfer → UPS → distribution/PDU → rack  
- C) CRAH → UPS → Transformer → Lighting only  
- D) BMS → DCIM → EMS → fuel  

### 14. ATS vs STS
An **STS** is preferred over a mechanical ATS when the design need is:
- A) Slow generator start only  
- B) Very fast transfer between two synchronized/toleranced live AC sources  
- C) Manual weekly switching with a wrench  
- D) Only DC bus switching  

### 15. Redundancy N+1
N+1 at a UPS module layer means:
- A) Exactly one UPS total  
- B) Capacity for full load plus one spare module so one can fail/maintain  
- C) Two full independent sites  
- D) No batteries required  

### 16. Redundancy 2N
2N power architecture conceptually provides:
- A) A single path with oversized cables  
- B) Two complete independent capacity paths each able to carry the load  
- C) N capacity with no spares  
- D) Only generator redundancy without UPS  

### 17. Dual-corded equipment
A dual-corded server on A-B feeds is intended so that:
- A) Both cords must fail together to lose the server (assuming healthy PSUs and independent paths)  
- B) Power draw doubles for free cooling  
- C) STS is never needed anywhere  
- D) Grounding is unnecessary  

### 18. UPS topology
Double-conversion (online/VFI) UPS primarily:
- A) Passes utility straight through with no battery ever  
- B) Continuously regenerates AC via rectifier/inverter path for high isolation from many disturbances  
- C) Only works on single-phase sites  
- D) Replaces chillers  

### 19. Generator role
Standby generators typically:
- A) Provide instantaneous power with zero transfer gap  
- B) Provide long-duration power after start/transfer; UPS covers the gap  
- C) Eliminate the need for fuel contracts  
- D) Are only used for lighting  

### 20. Busbar vs cable
Overhead busway in white space is often chosen to:
- A) Increase EMF intentionally  
- B) Enable flexible tap-off power distribution along a row without full recabling  
- C) Replace fire detection  
- D) Cool chips directly  

### 21. PUE
PUE is defined as:
- A) IT energy ÷ total facility energy  
- B) Total facility energy ÷ IT equipment energy  
- C) Water liters ÷ staff headcount  
- D) Generator kW ÷ rack count  

### 22. WUE
WUE primarily tracks:
- A) Copper utilization  
- B) Site water use relative to IT energy  
- C) Wi-Fi efficiency  
- D) Window U-factor only  

### 23. EMF sources
A common strong magnetic-field source near IT is:
- A) Fiber patch cords  
- B) Large transformers, busbars, and UPS/plant electrical gear  
- C) Plastic blanking panels  
- D) LED exit signs only  

### 24. Racks
One RU equals:
- A) 1 meter  
- B) 1.75 inches (44.45 mm) of vertical mounting space  
- C) 19 kW  
- D) One PDU phase  

### 25. Blanking panels
Blanking unused rack U-space mainly:
- A) Increases random mixing for comfort  
- B) Prevents bypass airflow and improves containment effectiveness  
- C) Removes need for CRAC filters  
- D) Grounds the cabinet automatically  

### 26. CRAC vs CRAH
The crisp distinction is:
- A) CRAC = typically DX/refrigerant-based room unit; CRAH = typically chilled-water air handler  
- B) They are identical acronyms  
- C) CRAH always includes a diesel  
- D) CRAC only cools batteries  

### 27. Containment
Hot-aisle containment (HAC) focuses on:
- A) Capturing and isolating hot exhaust for return  
- B) Heating the cold aisle on purpose  
- C) Eliminating need for any fans  
- D) Storing water under floor  

### 28. Sensible vs latent
IT equipment heat is primarily:
- A) Latent  
- B) Sensible  
- C) Nuclear  
- D) Only radiative into space vacuum  

### 29. Liquid cooling
A CDU in liquid-cooled designs typically:
- A) Routes carrier fiber  
- B) Circulates/conditions coolant and exchanges heat between loops  
- C) Replaces access control  
- D) Measures only lux  

### 30. ASHRAE envelopes
ASHRAE thermal guidelines for IT are best described as:
- A) Laws that override all local codes always  
- B) Recommended/allowable environmental envelopes by equipment class  
- C) Only generator fuel specs  
- D) Cabling color codes  

### 31. Water supply
Process water for cooling towers/humidification matters because:
- A) It is unrelated to uptime  
- B) Quality, availability, and treatment failures can force cooling derates or shutdowns  
- C) It powers the UPS inverter  
- D) It replaces fire suppression always  

### 32. Network infrastructure
A meet-me room (MMR) is primarily for:
- A) Battery watering  
- B) Carrier and customer interconnection / cross-connects  
- C) Diesel polishing  
- D) CRAH filter storage only  

### 33. Structured cabling
Scalable network design in a DC emphasizes:
- A) Random patching with no pathways  
- B) Planned pathways, media, and topologies that support growth without chaos  
- C) Copper only forever  
- D) No labeling  

### 34. Fire detection
Aspirating / very early smoke detection is valued because it:
- A) Suppresses fire with water immediately  
- B) Can detect smoke at very low concentrations before conventional detectors  
- C) Replaces EPO  
- D) Cools the room  

### 35. Suppression choice
Clean-agent (gaseous) suppression is often discussed for white space because:
- A) It is always cheaper than any other option  
- B) It can extinguish without the water damage profile of wet systems (with design trade-offs)  
- C) It increases PUE deliberately  
- D) It eliminates detection needs  

### 36. Physical security
A mantrap is designed to:
- A) Cool people  
- B) Enforce controlled, interlocked entry so two doors are not open as a free path  
- C) Store copper spools  
- D) Bypass badge logs  

### 37. Safety
An MOP (Method of Procedure) is used mainly to:
- A) Market the colo  
- B) Execute high-risk changes with steps, roles, verification, and backout  
- C) Calculate PUE only  
- D) Replace training forever  

### 38. BMS vs DCIM
A practical distinction:
- A) BMS focuses on building plant control/monitoring; DCIM focuses on data-centre IT/facilities inventory, power, environment, and capacity views  
- B) They are always the same product with one logo  
- C) DCIM only controls diesel fuel valves  
- D) BMS replaces all network monitoring  

### 39. Auxiliary monitoring
Underfloor leak detection is installed primarily to:
- A) Measure EMF  
- B) Alarm on water near electrical/IT spaces before major damage  
- C) Increase humidity always  
- D) Unlock cages  

### 40. Integration / IST
Integrated Systems Testing (IST) is valuable because it:
- A) Only tests a single breaker nameplate  
- B) Validates power, cooling, and controls failovers as a combined system under planned scenarios  
- C) Replaces daily backups of VMs  
- D) Is optional marketing text with no technical meaning  

---

## Answer key

| # | Ans | Short explanation |
|---|-----|---|
| 1 | **B** | Mission-critical is defined by impact of loss, not a single tech feature. |
| 2 | **B** | Reliability ≈ failure frequency; availability includes downtime/repair effects. |
| 3 | **B** | Retail colo: customer IT in shared hall infrastructure. |
| 4 | **B** | People and process contribute; they are not a peer root-cause bucket. Cite surveys as surveys. Refuse a fake percentage. |
| 5 | **B** | AHJ-enforced codes beat voluntary guides when they conflict. |
| 6 | **B** | Separate frameworks; do not conflate Tier vs TIA Rated branding. |
| 7 | **B** | True dual utility needs independent upstream paths, not just two cables on one feed. |
| 8 | **A** | Supporting facilities are part of operability, not cosmetics. |
| 9 | **B** | Plenum for air and/or cable is the classic purpose. |
| 10 | **B** | Rolling/concentrated loads govern cart/equipment moves. |
| 11 | **B** | Openings dump pressure and can create hot spots elsewhere. |
| 12 | **B** | Life safety egress, not efficiency marketing. |
| 13 | **B** | Utility/gen → transfer → UPS → distribution → rack is the teaching path. |
| 14 | **B** | STS = fast solid-state transfer between live sources. |
| 15 | **B** | N capacity + one spare unit/module at that layer. |
| 16 | **B** | Full dual independent capacity paths. |
| 17 | **A** | Independent A-B paths protect dual-PSU gear if both sides are healthy. |
| 18 | **B** | AC-DC-AC continuous conversion / isolation behavior. |
| 19 | **B** | Long-duration after start; UPS bridges seconds-class gap. |
| 20 | **B** | Flexible distribution along the row. |
| 21 | **B** | Total / IT; lower is better overhead. |
| 22 | **B** | Water intensity metric paired with IT energy. |
| 23 | **B** | High-current plant gear is the usual concern. |
| 24 | **B** | Standard rack unit height. |
| 25 | **B** | Stops bypass; essential with containment. |
| 26 | **A** | DX vs CHW handler is the interview shorthand. |
| 27 | **A** | Contain and return hot exhaust. |
| 28 | **B** | Servers make dry heat (sensible). |
| 29 | **B** | Coolant distribution and heat exchange control. |
| 30 | **B** | Guidance envelopes, not a substitute for code. |
| 31 | **B** | No good water → cooling risk. |
| 32 | **B** | Interconnection hub. |
| 33 | **B** | Pathways + plan enable growth. |
| 34 | **B** | Early warning via continuous sampling. |
| 35 | **B** | Non-water agent trade-off space (design-specific). |
| 36 | **B** | Interlocked entry control. |
| 37 | **B** | Controlled execution of risky work. |
| 38 | **A** | Plant control vs DC infrastructure management emphasis. |
| 39 | **B** | Water + electricity risk detection. |
| 40 | **B** | System-level proving of coupled failovers. |

### Score interpretation (study only)

| Score | Signal |
|---|---|
| 36–40 | Strong; teach weak modules out loud once more |
| 27–35 | At self-pass bar; drill misses + power/cooling cheatsheet |
| 20–26 | Re-read M6, M9, M12–M14; redo drill cards |
| ≤19 | Restart from curriculum map; do not rush exam mode |

---

*Original educational questions. Not EPI/EXIN exam content.*
