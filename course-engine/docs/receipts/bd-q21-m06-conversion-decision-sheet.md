# Q21 — m06 conversion decision sheet

Consumer: a human domain reviewer deciding whether to approve six m06 recall-to-apply conversions.
Feature gated: human ratification before any conversion reaches `bank/items` or the learner pack.
Observed defect: the Q20 inventory found 534 recall items (56% of the bank), so decorative
scenario rewrites must be separated from conversions that add a real operational decision.
Deletion condition: delete this sheet after each listed item has a recorded human yes/no
disposition in the ratification record.

## m06-q013

Current stem: Which sequence best matches a typical critical power path?

Current correct: B

Current choices:

- A. Rack → UPS → Utility → Generator
- B. Utility/generator → transfer → UPS → distribution/PDU → rack
- C. CRAH → UPS → Transformer → Lighting only
- D. BMS → DCIM → EMS → fuel

Proposed stem: During a design review, an engineer traces the normal critical-power path from the incoming source to a rack. Which sequence should the one-line drawing show?

Proposed correct: A

Proposed choices:

- A. Utility/generator → transfer → UPS → distribution/PDU → rack
- B. Utility/generator → rack → UPS → transfer → distribution/PDU
- C. CRAH → UPS → rack → transfer → generator
- D. BMS → DCIM → EMS → fuel system → rack

Source line: Public syllabus heading “Power distribution / busbar trunking”; ISO/IEC 22237-3:2021 — https://www.iso.org/standard/78551.html?browse=tc

New knowledge required: map the incoming-source, transfer, UPS, distribution, and rack functions onto a one-line design-review decision rather than only recognize the memorized sequence.

## m06-q042

Current stem: Which component typically bridges the multi-second gap while a standby generator starts and transfers?

Current correct: C

Current choices:

- A. CRAH unit humidity tray
- B. Fire pre-action compressor only
- C. UPS energy storage (batteries or equivalent ride-through)
- D. Overhead cable tray grounding

Proposed stem: A generator takes several seconds to start and reach the transfer condition, but the IT load must not see an interruption. Which function supplies the load during that interval?

Proposed correct: A

Proposed choices:

- A. UPS energy storage (batteries or equivalent ride-through)
- B. A mechanical ATS that only switches once a source is available
- C. An STS that transfers between two sources that are both already live
- D. The generator starting battery, which cranks the engine

Source line: Public syllabus heading “UPS systems”; IEC 62040-3:2021 — https://webstore.iec.ch/en/publication/60140

New knowledge required: distinguish the UPS function that carries the IT load through generator start from ATS source switching, STS transfer between live sources, and the generator’s engine-start battery.

## m06-q047

Current stem: Compared with a mechanical ATS, an STS is chosen primarily for:

Current correct: B

Current choices:

- A. Cheaper fuel contracts
- B. Fast transfer between live AC sources
- C. Slower weekly manual exercise, not a transfer function
- D. DC-only plants without AC

Proposed stem: Two in-tolerance AC sources are available and the load specification requires a sub-cycle transfer when the preferred source fails. Which transfer technology fits that requirement?

Proposed correct: A

Proposed choices:

- A. A static transfer switch (STS)
- B. A mechanical ATS selected for its ordinary source-transfer function
- C. A generator paralleling controller, which synchronizes generation rather than serving as the load-transfer device
- D. A manual maintenance bypass, which requires an operator action

Source line: Public syllabus heading “ATS and STS”; IEC 62310-3:2008 — https://webstore.iec.ch/en/publication/6803

New knowledge required: select the transfer technology from the explicit sub-cycle requirement and the availability of two live AC sources, while separating transfer from generation control and manual bypass.

## m06-q054

Current stem: Concurrent maintainability as a design intent means:

Current correct: C

Current choices:

- A. Maintenance is forbidden even for planned service
- B. Software is patched, not electrical capacity
- C. Can maintain one capacity component without interruption
- D. Fuel deliveries require no planning

Proposed stem: A maintenance procedure removes one power-capacity component from service while ICT load remains online. Which claim is the design team actually demonstrating?

Proposed correct: A

Proposed choices:

- A. One capacity component can be serviced without interrupting the ICT load
- B. The component is redundant only after the ICT load has been shut down
- C. The software maintenance window can proceed without changing electrical capacity
- D. Fuel deliveries can be scheduled without a maintenance operating procedure

Source line: Public syllabus heading “Power redundancy levels and techniques”; ANSI/TIA-942-C — https://tiaonline.org/standard/tia-942/

New knowledge required: infer concurrent maintainability from an online power-capacity maintenance event, rather than repeat its definition without deciding what the procedure demonstrates.

## m06-q072

Current stem: Transformer losses show up operationally as:

Current correct: D

Current choices:

- A. Zero heat in electrical rooms
- B. Negative kW that reduces cooling load
- C. Only network latency
- D. Heat in facility thermal budget

Proposed stem: An electrical design review is closing the room heat-rejection budget. How should the transformer’s measured load and no-load losses be treated?

Proposed correct: A

Proposed choices:

- A. Add them as heat that the facility cooling design must reject
- B. Subtract them from the IT heat load because the loss is upstream of the racks
- C. Count only network latency because transformer loss is an electrical-quality issue
- D. Ignore them once the transformer has been assigned an electrical rating

Source line: Public syllabus heading “Transformers”; IEC 60076-19-1:2023 — https://webstore.iec.ch/en/publication/59982

New knowledge required: apply both measured no-load and load losses as a thermal-budget input in a room heat-rejection review, not merely recall that transformer losses become heat.

## m06-q094

Current stem: The relationship kW = kVA × power factor means:

Current correct: C

Current choices:

- A. kVA and kW are always identical with no exceptions
- B. PF is always zero in data centres
- C. Real power = apparent power × PF
- D. Generators rate only in lux, not electrical power

Proposed stem: A generator is rated 100 kVA and the connected load is assessed at a 0.8 power factor. What real-power capacity does that rating represent at that power factor?

Proposed correct: A

Proposed choices:

- A. 80 kW
- B. 100 kW
- C. 125 kW
- D. 0.8 kW

Source line: Public syllabus heading “Power sizing”; IEC 60050 IEV 131-11-46 — https://www.electropedia.org/iev/iev.nsf/display?ievref=131-11-46&openform=

New knowledge required: calculate a real generator capacity with units from a kVA nameplate and a stated power factor, rather than only recite the kW–kVA relationship.
