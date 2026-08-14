# Citation index — Google SRE Book & SRE Workbook

```
authority:   Google / O'Reilly Media
titles:      Site Reliability Engineering (2017) · The Site Reliability Workbook (2018)
licence:     CC BY-NC-ND 4.0   ← verified from the ToC pages, 2026-08-14
url:         https://sre.google/sre-book/table-of-contents/
             https://sre.google/workbook/table-of-contents/
access:      free to read online
verified_at: 2026-08-14
```

## What CC BY-NC-ND 4.0 actually permits — earlier guidance CORRECTED

Prior guidance in this repo said "cite and link, never vendor". That was **wrong on one axis**
and **missing a constraint on another**.

| | Permitted? |
|---|---|
| Verbatim redistribution, attributed, noncommercial | **YES** — BY-NC-ND explicitly allows copying and redistributing in any medium or format |
| Chapter/section citation | yes, always |
| Writing our own explanation of the *ideas* | yes — facts and concepts are not copyrightable, only expression |
| **Adapted / abridged / remixed version** | **NO** — the ND clause forbids distributing modified material |
| Commercial use | **NO** — NC clause |

**The counter-intuitive consequence:** an "excerpted SRE study guide" is *less* permitted than a
full verbatim copy. Excerpting feels safer than wholesale copying; under ND it is not.

**Practical posture:** we do not need to vendor — the books are online and stable. Cite
chapter-precisely, teach the concepts in our own words, never ship an abridged derivative.
Keep the repo noncommercial or the NC clause bites independently.

---

## Citation targets for module 15 (`ops-adjacent`)

Module 15 holds **39 bank items with no Learn surface at all** — learners are assessed on
ops material the course never teaches. These chapters are the external scaffold for that
missing curriculum.

| Our need | SRE Book | SRE Workbook |
|---|---|---|
| Incident command, roles, escalation | **Ch 14** Managing Incidents | **Ch 9** Incident Response |
| Emergency response under uncertainty | **Ch 13** Emergency Response | — |
| Blameless postmortem form | **Ch 15** Postmortem Culture | **Ch 10** Postmortem Culture |
| On-call structure, handover, fatigue | **Ch 11** Being On-Call | **Ch 8** On-Call |
| Diagnostic method under time pressure | **Ch 12** Effective Troubleshooting | — |
| Alerting that does not flood | **Ch 10** Practical Alerting · **Ch 6** Monitoring | **Ch 4** Monitoring · **Ch 5** Alerting on SLOs |
| Failure propagation | **Ch 22** Addressing Cascading Failures | **Ch 17** Identifying and Recovering from Overload |
| Toil vs engineering | **Ch 5** Eliminating Toil | **Ch 6** Eliminating Toil |
| Objective-setting / availability targets | **Ch 4** Service Level Objectives | **Ch 2** Implementing SLOs · **Ch 3** SLO Case Studies |
| Risk posture | **Ch 3** Embracing Risk | — |
| Change management | **Ch 8** Release Engineering | **Ch 16** Canarying Releases |

### Two chapters worth singling out

**SRE Book Ch 33 — "Lessons Learned from Other Industries."** Directly relevant to a claim this
project already got wrong: review round 3 rejected framing data-centre operations as equivalent
to nuclear or aviation ("mission-critical, not safety-critical"). This chapter is Google's own
treatment of exactly that cross-industry comparison and is the right citation for teaching the
comparison **honestly** rather than by borrowed prestige.

**SRE Book Ch 4 / Workbook Ch 2–3 — SLOs.** Pairs with citation record UT-01: the circulated
Tier availability percentages are absent from the Uptime standard. SLO chapters supply the
defensible way to reason about availability targets — as an engineering commitment with an
error budget, not as a number attached to a Tier label.

---

## Boundary

`Standard`/`Tech` nodes referencing these carry
`ground_contact: "read online, CC BY-NC-ND — cited by chapter, no text reproduced"`.
Chapter numbers and titles are bibliographic facts and are recorded here deliberately; nothing
from chapter bodies is stored in this repo.
