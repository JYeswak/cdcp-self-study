# CDCS calculation direction

This is an original study track for checking vendor offers. It is a direction
for CDCS calculations and one-line defect catches, not an EPI/EXIN credential,
exam reconstruction, or substitute for a licensed engineer, AHJ, manufacturer,
or commissioning authority.

## Calculation discipline

- Battery autonomy: make voltage, amp-hours, usable fraction, efficiency, load,
  temperature, aging, reserve, and the vendor discharge curve visible. The
  arithmetic item is a screen, not a runtime guarantee.
- Airflow and cooling: show the conversion factor and units. For a stated
  sensible model, `Q = 1.085 × CFM × ΔT` is only the stated model; pressure,
  density, altitude, coil conditions, and latent load still belong in the
  design review. A psychrometric check must expose humidity ratio or enthalpy,
  not only dry-bulb temperature.
- Fire gas: a volume-times-concentration screen is not agent mass, nozzle
  design, discharge timing, enclosure integrity, or hold-time proof. Treat
  those as separate vendor/AHJ evidence requests.
- EMF: a ratio between measured points is a measured ratio. It is not a
  universal inverse-distance law, exposure limit, or clearance approval.
- Fuel and generators: usable volume divided by stated consumption is a first
  pass. Storage classification, piping, leak detection, fuel quality, code,
  synchronization, phase sequence, load sharing, and protective relaying must
  be checked in the actual offer.

## Public-heading coverage

The bank covers each requested heading with an original calculation or a
one-line defect catch:

| Heading | Item IDs | Interview action |
| --- | --- | --- |
| Battery autonomy + parallel pitfalls | `cdcs-battery-autonomy`, `cdcs-ups-parallel` | Show the bounded runtime arithmetic, then challenge impedance, sharing, bypass, and protection evidence. |
| One-line reading | `cdcs-one-line` | Trace the conductors and shared switching points before accepting a redundancy label. |
| CFM/CMH + ΔT + psychrometric | `cdcs-airflow-cmh`, `cdcs-sensible-delta-t`, `cdcs-psychrometric` | Write the units or name the missing moisture/enthalpy evidence. |
| Fire gas quantity + hold time | `cdcs-fire-gas` | Separate the screening arithmetic from enclosure-integrity and hold-time evidence. |
| EMF attenuation/shielding | `cdcs-emf-attenuation` | Compute only the measured ratio and state what it cannot establish. |
| Fuel storage | `cdcs-fuel-autonomy` | Calculate usable runtime, then ask for storage, piping, detection, quality, and local-review evidence. |
| Generator paralleling | `cdcs-generator-paralleling` | Reject a common-bus closure lacking synchronization, phase sequence, sharing, and protection evidence. |

Clause-backed missing-module variants are also live:

| Building floor loading | `cdcs-building-floor-loading` | Reject average-only evidence when concentrated loads and the structural-engineer limit are missing. |
| Raised-floor installation | `cdcs-raised-floor-layout` | Catch equipment/cabling installation before the access-floor layout was determined. |
| Raised-floor load testing | `cdcs-raised-floor-load-testing` | Catch static-only evidence that omits dynamic and impact loading conditions. |
| Cooling altitude correction | `cdcs-cooling-altitude-adjustment` | Catch a sea-level sensible-air factor used at elevation without correction evidence. |
| Wireless-AP cabling | `cdcs-cabling-wireless-ap` | Catch one Cat 6A run where the public TIA-942-C summary states a two-run minimum for balanced twisted-pair AP cabling. |
| Cable-tray fill | `cdcs-cable-tray-fill` | Calculate 40% initial fill and catch that it exceeds the 25% initial maximum. |
| Mixed-use PUE | `cdcs-mpue-mixed-use` | Calculate same-boundary mPUE as 1.50 rather than using whole-building energy. |

## Interview response pattern

For every item, the learner should: (1) state the inputs and units, (2) write
the arithmetic or point to the single-line defect, (3) name the vendor,
authority, or design evidence still missing, and (4) stop short of a
credential, design approval, or runtime guarantee. A correct answer that skips
the evidence boundary is incomplete specialist practice.

## What the learner should do

For each item, write the units or name the single-line defect before looking at
the explanation. Keep the source citation and the rights row visible. A green
track receipt proves data integrity and gate wiring; it does not certify a
person, a facility, a design, or a vendor proposal.

## Next clause-backed variants

These four additions extend the 11-module ledger without converting an exposed
topic summary into an unsupported question. Each item has an exact public EPI
heading and a current public primary-source anchor in `item-attribution.toml`.

| Module | Item | Public anchor | Witness |
| --- | --- | --- | --- |
| M04 Advanced Raised Floor & Suspended Ceiling | `cdcs-raised-floor-load-testing` | ANSI/TIA-569-E:2019 §9.6.2 | Static-only record misses dynamic and impact evidence. |
| M07 Advanced Cooling | `cdcs-cooling-altitude-adjustment` | ASHRAE Handbook—Fundamentals 2025 Ch. 18 §4 | Sea-level factor at elevation lacks correction evidence. |
| M09 Scalable Network Cabling | `cdcs-cable-tray-fill` | ANSI/TIA-569-E:2019 §9.7.1.1 | 40% initial fill exceeds the 25% initial limit. |
| M11 Data Centre Efficiency | `cdcs-mpue-mixed-use` | ISO/IEC 30134-2:2026 Definition 3.1.7 | Same-boundary mPUE is 1.50, not whole-building 2.00. |

The public-source bar remains a stop condition: seismic, suspended-ceiling,
OCPD, harmonics, flywheel, liquid cooling, CFD, and other open variants stay
ledger gaps until a clause or formula can be cited. This track does not grant a
credential or certify a design.

## Thin-module depth slice

The one-item modules now have a second, distinct failure mode with a resolved
public heading and source anchor:

| Module | Item | Failure mode |
| --- | --- | --- |
| M01 Life Cycle | `cdcs-lifecycle-aging-horizon` | Five-year evidence is applied to a named 15-year planned lifetime. |
| M02 Standards/Ratings | `cdcs-rating-nameplate-scope` | A component nameplate is treated as a facility certification. |
| M03 Building | `cdcs-building-floor-units` | 125 lbf/ft² is incorrectly repeated as 125 kPa. |
| M06 EMF | `cdcs-emf-unit-conversion` | 1.2 mT and 8 G are divided before unit normalization. |
| M08 Fire | `cdcs-fire-ahj-evidence` | A gas screen is called approved without AHJ/occupancy and safety evidence. |
| M10 Contamination | `cdcs-contamination-size-boundary` | 0.05 µm evidence is used outside the ISO 14644-1 classification range. |

Parallel pitfalls remain represented by `cdcs-ups-parallel`, and altitude by
`cdcs-cooling-altitude-adjustment`; both are already in the cited live bank.
