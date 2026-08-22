## Power & Redundancy Cheatsheet (One-Pager)

Print or keep side-by-side with Module 6 / 9 study. Interview-ready shorthand — not a design manual.

---

## 1. Critical power path (utility → rack)

```text
  UTILITY FEED(S)              GENERATOR(S) + FUEL
         \                         /
          \                       /
           v                     v
        [ Main switchgear / source selection ]
                    |
            ATS (genset / utility)
            and/or STS (live dual AC)
                    |
                    v
           [ Transformer(s) MV→LV as needed ]
                    |
                    v
           [ UPS + batteries / BESS ]
            |                 |
         inverter          bypass paths
                    |
                    v
     [ Downstream boards / PDUs / busway ]
                    |
          +---------+---------+
          |                   |
          v                   v
     [ Rack PDU A ]      [ Rack PDU B ]
          \                   /
           v                 v
          [ Dual-corded IT PSUs ]
```

**Grey space:** switchgear, transformers, gens, UPS, batteries.  
**White space:** busway/PDUs, rack PDUs, IT load.

**Remember:** Generator is **not** instant → UPS autonomy covers start + transfer. STS ≠ ATS.

---

## 2. Redundancy table

| Topology | Meaning | Ops implication | CFO one-liner |
|---|---|---|---|
| **N** | Just enough capacity for design load | Any single unit failure/maintenance can drop load | Cheapest; highest risk |
| **N+1** | Full load capacity **plus one** spare unit/module | One failure or maintenance at that layer tolerated | Spare tire for the plant layer |
| **2N** | Two **independent** full-capacity paths | Path A can be lost; Path B carries all (if truly independent) | Two complete systems |
| **2N+1** | Dual path **plus** extra spare (site-defined) | Rare wording — confirm meaning on *that* design | Dual + margin |
| **Distributed redundant** | Spare capacity shared across multiple systems/paths | Efficient use of plant; more complex ops | Shared spare, careful loading |

**Related concepts**
- **Concurrent maintainability:** take one capacity element out without dropping critical load.  
- **Fault tolerance (language varies by scheme):** withstand a fault without interruption — stronger claim than “has a spare.”  
- **SPOF:** any sole element whose failure takes out the service.  
- **A-B feeds:** dual paths to dual-corded gear; still fails if both paths share an upstream SPOF.

**TIA Rated vs Uptime Tier:** separate frameworks. Don’t treat “Tier III” and “Rated-3” as automatic synonyms in interviews.

---

## 3. UPS types (interview level)

| Type / class | Behavior | Typical use note |
|---|---|---|
| **Passive standby** (VFD-class idea) | Load on utility until failure; then transfer to inverter | Small/office gear; rare as sole enterprise DC plant |
| **Line-interactive** (VI-class idea) | Utility feed with regulation; battery on deeper events | Mid-tier; not full isolation |
| **Double-conversion online** (VFI) | AC→DC→AC continuously; high isolation from many disturbances | Dominant critical IT UPS architecture |
| **Parallel redundant** | Multiple modules share load (capacity and/or N+1) | Module failure without full plant loss |
| **Bypass** | Static and/or maintenance path around UPS | Service and failure modes — know it exists |

**Battery / BESS**
- **Autonomy:** minutes (design-specific) at stated kW.  
- **Role:** bridge to generator or orderly shutdown.  
- **BESS:** larger storage / grid flexibility narrative — not always the same as UPS strings.

**kW vs kVA:** IT heat ≈ kW; nameplates often kVA; **power factor** links them.

---

## 4. ATS vs STS (do not confuse)

| | **ATS** | **STS** |
|---|---|---|
| Mechanism | Electromechanical | Solid-state (SCR) |
| Speed | Cycles → seconds | Sub-cycle → few ms (vendor-specific) |
| Classic job | Utility ↔ generator | Two live AC sources, seamless transfer |
| Mental model | “Source select for facility” | “Fast preference switch for sensitive bus” |

---

## 5. Distribution quick hits

| Element | One-liner |
|---|---|
| **Transformer** | Voltage change (often MV→LV); isolation types exist |
| **PDU (room/row)** | Steps/distributes to white-space panels or bus |
| **Busway/busbar** | Flexible overhead/row distribution with tap-offs |
| **Rack PDU** | A and/or B strips; metering helps capacity ops |
| **Grounding/bonding** | Safety + fault path; region/code specific |
| **Thermography** | IR scan for hot joints before they fail |
| **Three-phase** | Facility backbone; single-phase more at small loads |

---

## 6. Cooling at a glance (twin of power)

```text
  IT LOAD (heat ≈ kW in)
        |
        +-- air --> CRAC (DX) or CRAH (CHW) / in-row / RDHx
        |              |
        |              v
        |         Plant (chiller / pumps) --> heat rejection outdoors
        |
        +-- liquid --> CDU --> chip/loop --> facility exchange --> outdoors
```

| Topic | Cheat line |
|---|---|
| **Sensible vs latent** | IT ≈ sensible (temperature); moisture = latent |
| **CRAC** | DX / refrigerant chain |
| **CRAH** | Chilled-water air handler |
| **Cold / hot aisle** | Intakes vs exhausts |
| **CAC / HAC** | Contain cold supply **or** hot return |
| **Blanking panels** | Stop bypass air |
| **Raised floor** | Often cold supply plenum — open tiles can starve racks |
| **ASHRAE** | Recommended/allowable envelopes (guidance) |
| **PUE** | Total energy / IT energy |
| **WUE** | Water / IT energy intensity |

**Failure intuition:** Dense racks overheat in **minutes** without airflow/liquid — cooling outage is an availability event, not “comfort.”

---

## 7. 60-second oral drill

1. Trace power: utility → ATS/STS → UPS → bus/PDU → A-B rack → IT.  
2. Say why UPS and generator are a pair.  
3. Define N+1 vs 2N.  
4. CRAC vs CRAH + why containment.  
5. Name one metric each: **PUE**, **WUE**.

---

## See also

- Module notes: [`../modules/06-power.md`](../modules/06-power.md), [`../modules/09-cooling.md`](../modules/09-cooling.md)  
- Glossary: [`GLOSSARY.md`](./GLOSSARY.md)  
- Practice: [`../practice/DRILL-CARDS.md`](../practice/DRILL-CARDS.md), [`../practice/PRACTICE-EXAM.md`](../practice/PRACTICE-EXAM.md)

*Educational cheatsheet. Not a substitute for licensed engineering or site single-line diagrams.*
