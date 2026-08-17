# SOURCES — Luna

**Date:** 2026-08-17 (America/Denver)  
**Use:** public technical sources opened for the Luna research pass. Each entry states what it confirms or kills. No exam dumps, paywalled EPI body text, or proprietary course material was used.

## UPS autonomy, parallel systems, and one-lines

| Public source | Confirms / kills |
|---|---|
| [Schneider Electric — Design considerations for UPS systems in data centers](https://blog.se.com/datacenter/2020/07/20/design-considerations-deployment-ups-systems-in-data-centers/) | Confirms that battery runtime is a business/risk decision, not a universal constant; generator start and transfer assumptions matter, and high-density thermal response can be much faster than generator response. Kills “15 minutes is always enough.” |
| [Schneider Electric — Electrical distribution equipment in data center environments](https://download.schneider-electric.com/files?p_Doc_Ref=SPD_VAVR-8W4MEX_EN&p_File_Name=VAVR-8W4MEX_R2_EN.pdf) | Confirms the input switchboard → UPS → output/bypass → distribution one-line and that bypass/maintenance breakers can isolate the UPS. Its “about 15 minutes” example is explicitly a configuration/load example, not a course constant. |
| [Schneider Electric — Easy UPS parallel system overview](https://productinfo.se.com/easyups3m/viewer?docidentity=SystemOverview-D51A76AE&extension=xml&lang=en&manualidentity=TechnicalSpecificationsEasyUPS3M601-D5104BCE) | Confirms unit input/output, static bypass, maintenance bypass, battery, and system-isolation functions. Also confirms that unequal bypass-path impedance can create uneven sharing, overload, and damage. Kills “same UPS rating means same load.” |
| [Eaton — Parallel UPS systems white paper](https://www.eaton.com/content/dam/eaton/products/backup-power-ups-surge-it-power-distribution/backup-power-ups/power-xpert-9395/eaton-ups-parallel-whitepaper-wp153026en.pdf) | Confirms synchronization, load sharing, control/communication common modes, and the need to identify and isolate a failed module. Kills “parallel UPS is only an N+1 capacity label.” |
| [Schneider Electric — Smart-UPS VT user guide](https://iportal2.schneider-electric.com/Contents/docs/UPS-AHIE-83NAUF_R0_EN.PDF) | Confirms that runtime is predicted at present load and that the UPS exposes battery amp-hours, temperature, load, and parallel status. Kills autonomy questions based only on nominal voltage or nameplate capacity. |

## Cooling, psychrometrics, and units

| Public source | Confirms / kills |
|---|---|
| [Trane — TRACE 3D+ load design calculations](https://trace3dplus.stage.help.trane.com/load_design_calculations_.html) | Confirms sensible-load reasoning such as `1.075 × CFM × ΔT`, while noting that density and specific heat vary with conditions; latent load requires humidity/psychrometric reasoning. Kills treating the factor as context-free law. |
| [ASHRAE Handbook — Psychrometrics](https://handbook.ashrae.org/Handbooks/F21/SI/F21_Ch01/F21_Ch01_si.aspx) | Confirms the psychrometric chart as a state/property tool involving enthalpy and humidity ratio, with pressure dependence and sensible/total relationships. Kills “CFM and dry-bulb delta alone solve every cooling problem.” |
| [ASHRAE Handbook — Data centers](https://handbook.ashrae.org/Handbooks/A19/IP/a19_ch20/a19_ch20_ip.aspx) | Confirms data-center thermal and moisture control as design limits involving dew point and equipment/environmental constraints. Kills a simplistic RH-only treatment. |
| [ASHRAE Handbook — Nonresidential load calculations](https://handbook.ashrae.org/Handbooks/F25/SI/F25_Ch18/f25_ch18_si.aspx) | Confirms airflow at a specified psychrometric state, sensible heat, and latent heat from humidity-ratio change; standard factors are approximations requiring condition/elevation awareness. |
| [NIST — SI conversion factors](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b9) | Confirms `1 cfm = 4.719474×10⁻⁴ m³/s`, or approximately `1.699 m³/h` (CMH). Kills unit-conversion handwaving in a CFM/CMH item. |

## Fire agents, quantity, and retention

| Public source | Confirms / kills |
|---|---|
| [ANSUL FM-200 engineering specification](https://docs.johnsoncontrols.com/specialhazards/api/khub/documents/VIa4_ngrGZnmlNH9R~0DYg/content) | Confirms that design concentration varies by hazard, engineered hydraulic calculations are submitted, room sealing is part of the design, and this public specification calls for a minimum 10-minute hold period or trained-response period. Kills a universal agent mass or “gas discharge ends the problem” item. |
| [ANSUL iFLOW inert-gas system](https://www.ansul.com/gaseous-suppression-agent-releasing/inert-gas-fire-suppression-systems/iflow_300_bar_system_fsp/iflow-300-bar-system) | Confirms the inert-gas mechanism: reduce oxygen below combustion while maintaining a designed occupiable condition. Kills treating inert gas and halocarbon systems as interchangeable mechanisms. |
| [OSHA — Fixed extinguishing systems](https://www.osha.gov/etools/evacuation-plans-procedures/emergency-standards/fixed-extinguishing) | Confirms employee protection and maintaining designed gaseous concentration until the fire is controlled. Kills “the agent quantity is the whole calculation” and “occupants can remain through discharge.” |

## EMF attenuation and shielding

| Public source | Confirms / kills |
|---|---|
| [NIOSH — Health Hazard Evaluation 94-0300](https://www.cdc.gov/niosh/hhe/reports/pdfs/1994-0300-2528.pdf) | Confirms that power-frequency magnetic fields are difficult to shield with common materials and that distance and exposure time are practical controls; measured fields were tied to nearby switchboards. Kills “ordinary aluminium sheet solves LF magnetic B-field exposure.” |
| [NIEHS — Electric and magnetic fields](https://www.niehs.nih.gov/health/topics/agents/emf) | Confirms rapid field reduction with distance and distinguishes electric-field shielding from the harder magnetic-field problem. Kills a single universal attenuation number without source geometry and distance. |

## Fuel storage and generator paralleling

| Public source | Confirms / kills |
|---|---|
| [Cummins — Paralleling application manual](https://www.cummins.com/sites/default/files/2024-08/t016-Paralleling-application-manual.pdf) | Confirms isolated-bus requirements for synchronization, load sharing, protection, and manual backup; utility-parallel requirements for kW/kVAR control, reverse power/VAR protection, master synchronization, and cold-start site testing. Kills “generators in parallel just need matching voltage.” |
| [Cummins — Generator set controls](https://www.cummins.com/en-na/generators/generator-set-controls) | Confirms synchronization, load sharing, protection, metering, monitoring, and first-start arbitration as control-system responsibilities. Kills a breaker-only view of paralleling. |
| [Cummins — Diesel fuel storage and transfer](https://www.cummins.com/en-na/engines/diesel-fuel-storage-and-transfer) | Confirms contamination paths through tanks, hoses, pumps, filters, seals, and maintenance, plus periodic tank cleaning and OEM intervals. Kills “stored gallons equal usable availability.” |
| [Caterpillar — When and how to design parallel generators](https://www.cat.com/en_US/by-industry/electric-power/Articles/ep-news/ep-news-when-and-how-to-design-parallel-generators.html) | Confirms day-tank/return-piping geometry, gravity-return preference, pump/check-valve consequences, and fuel polishing for large stored volumes. Kills “fuel piping is passive plumbing.” |
| [EPA — Federal UST requirements for emergency generator systems](https://www.epa.gov/ust/federal-ust-requirements-emergency-power-generator-ust-systems) | Confirms that generator UST design includes spill/overfill prevention, corrosion protection, release detection, inspection, training, response, and closure. Kills a universal tank rule detached from location and jurisdiction. |
| [EPA — UST technical compendium](https://www.epa.gov/ust/underground-storage-tank-technical-compendium-about-2015-ust-regulation) | Confirms that a buried main tank plus above-ground day tank is a common architecture and that applicability depends on the underground fraction and piping. Kills “day tank means the UST rules do not matter.” |

## Source discipline

These sources establish mechanisms, variables, and failure modes—not universal design values for every facility. Any future numeric item should name its assumptions, units, boundary, source class, and vendor/AHJ dependency. The source pass does not certify a learner or reproduce proprietary EPI exam content.
