//! Deterministic host for loop-skill research waves.
//!
//! The work list is compiled here — not discovered by an agent. Each probe
//! reads bytes from this repo (or a fixture tree) plus the standing franken-
//! harvest ledger. Product_moved is classified by `cdcp_bank::tick_emitter`;
//! this crate only *grades whether that choke is wired*.
//!
//! Fail closed: a missing harvest file is an error, never an empty grade.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const SKILL_ENFORCEMENT: &str = "loop-enforcement";
pub const SKILL_ENGINEERING: &str = "loop-engineering";
pub const FINDING_PHRASE: &str = "comment-only emit-tick choke";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Wiring {
    Wired,
    Unwired,
    Contradicted,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Probe {
    pub id: String,
    pub skill: String,
    pub path: String,
    pub claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeResult {
    pub id: String,
    pub skill: String,
    pub path: String,
    pub claim: String,
    pub wiring: Wiring,
    pub citation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillGrade {
    pub skill: String,
    pub finding: String,
    pub probes: Vec<ProbeResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorRef {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveReport {
    pub wave: u32,
    pub harvest_path: String,
    pub harvest_sha256: String,
    pub prior: Option<PriorRef>,
    pub grades: Vec<SkillGrade>,
}

struct Spec {
    id: &'static str,
    skill: &'static str,
    path: &'static str,
    claim: &'static str,
    wired: &'static [&'static str],
    contradicted: &'static [&'static str],
}

const SPECS: &[Spec] = &[
    Spec {
        id: "emit-tick-computes-product-moved",
        skill: SKILL_ENGINEERING,
        path: "crates/cdcp_bank/src/tick_emitter.rs",
        claim: "a tick counts only if the product moved — product_moved is COMPUTED from the commit, not claimed",
        wired: &[
            "computed_product_moved",
            "claimed_product_moved",
            "product_moved_disagreement",
        ],
        contradicted: &[],
    },
    Spec {
        id: "emit-tick-forbidden-phrases",
        skill: SKILL_ENFORCEMENT,
        path: "crates/cdcp_bank/src/tick_emitter.rs",
        claim: "ticks pass emit_tick / tick_guard; fabricated standing-by prose is rejected",
        wired: &["standing by", "queue empty", "blocked on josh", "wait_josh"],
        contradicted: &[],
    },
    Spec {
        id: "check-sh-comment-only-choke",
        skill: SKILL_ENFORCEMENT,
        path: "scripts/check.sh",
        claim: "every loop tick goes through the emit_tick choke-point as a live gate",
        wired: &[],
        contradicted: &["not a verdict-producing gate"],
    },
    Spec {
        id: "pre-commit-is-substrate-guard",
        skill: SKILL_ENFORCEMENT,
        path: "hooks/pre-commit",
        claim: "the installed hook is the tick choke-point (emit_tick / tick_guard)",
        wired: &["substrate-guard --staged"],
        contradicted: &[],
    },
    Spec {
        id: "charter-rule-zero",
        skill: SKILL_ENGINEERING,
        path: ".flywheel/CHARTER.md",
        claim: "a tick counts only if it changed the product (CHARTER value_bar)",
        wired: &["PRODUCT MOVED", "value_bar"],
        contradicted: &[],
    },
    Spec {
        id: "tick-ledger-exists",
        skill: SKILL_ENGINEERING,
        path: ".flywheel/tick-ledger.jsonl",
        claim: "the flywheel ledger is the durable tick audit trail",
        wired: &["zs.tick-receipt"],
        contradicted: &[],
    },
    Spec {
        id: "harvest-ledger-is-standing",
        skill: SKILL_ENGINEERING,
        path: "__HARVEST__",
        claim: "mine the standing franken-harvest ledger, not GitHub",
        wired: &["FRANKEN HARVEST", "Mirror:"],
        contradicted: &[],
    },
    Spec {
        id: "gate-wrapper-emit-tick",
        skill: SKILL_ENFORCEMENT,
        path: "crates/cdcp_gate/src/gates/emit_tick.rs",
        claim: "cdcp_gate emit-tick is the live writer of the ledger",
        wired: &["tick_emitter"],
        contradicted: &[],
    },
];

/// Compiled work list. Agents do not get to invent the probe set.
pub fn work_list() -> Vec<Probe> {
    SPECS
        .iter()
        .map(|spec| Probe {
            id: spec.id.to_string(),
            skill: spec.skill.to_string(),
            path: spec.path.to_string(),
            claim: spec.claim.to_string(),
        })
        .collect()
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if bytes.is_empty() {
        return Err(format!("empty file is ERROR, not a grade: {}", path.display()));
    }
    Ok(hex::encode(Sha256::digest(&bytes)))
}

pub fn load_prior(path: &Path) -> Result<WaveReport, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read prior {}: {e}", path.display()))?;
    if text.trim().is_empty() {
        return Err(format!("prior artifact is empty: {}", path.display()));
    }
    serde_json::from_str(&text).map_err(|e| format!("parse prior {}: {e}", path.display()))
}

fn read_probe_text(root: &Path, harvest: &Path, rel: &str) -> Result<String, String> {
    let path = if rel == "__HARVEST__" {
        harvest.to_path_buf()
    } else {
        root.join(rel)
    };
    fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))
}

fn contains_all(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().all(|n| haystack.contains(n))
}

pub fn run_probes(root: &Path, harvest: &Path) -> Result<Vec<ProbeResult>, String> {
    if !harvest.is_file() {
        return Err(format!(
            "harvest ledger missing (fail closed): {}",
            harvest.display()
        ));
    }
    let harvest_bytes = fs::read(harvest)
        .map_err(|e| format!("read harvest {}: {e}", harvest.display()))?;
    if harvest_bytes.is_empty() {
        return Err(format!(
            "harvest ledger empty (fail closed): {}",
            harvest.display()
        ));
    }

    let mut out = Vec::with_capacity(SPECS.len());
    for spec in SPECS {
        let text = match read_probe_text(root, harvest, spec.path) {
            Ok(text) => text,
            Err(err) => {
                out.push(ProbeResult {
                    id: spec.id.to_string(),
                    skill: spec.skill.to_string(),
                    path: spec.path.to_string(),
                    claim: spec.claim.to_string(),
                    wiring: Wiring::Missing,
                    citation: err,
                });
                continue;
            }
        };
        let wiring = if !spec.contradicted.is_empty() && contains_all(&text, spec.contradicted)
        {
            Wiring::Contradicted
        } else if spec.path == "hooks/pre-commit" {
            if text.contains("substrate-guard") && !text.contains("emit-tick") {
                Wiring::Unwired
            } else if contains_all(&text, spec.wired) {
                Wiring::Wired
            } else {
                Wiring::Unwired
            }
        } else if !spec.wired.is_empty() && contains_all(&text, spec.wired) {
            Wiring::Wired
        } else {
            Wiring::Unwired
        };
        let citation = match wiring {
            Wiring::Contradicted => spec
                .contradicted
                .first()
                .copied()
                .unwrap_or("contradicted")
                .to_string(),
            Wiring::Wired => spec.wired.first().copied().unwrap_or("wired").to_string(),
            Wiring::Unwired if spec.path == "hooks/pre-commit" => {
                "substrate-guard --staged (no emit-tick)".to_string()
            }
            Wiring::Unwired => format!("{} present but claim needles missing", spec.path),
            Wiring::Missing => "file missing".to_string(),
        };
        out.push(ProbeResult {
            id: spec.id.to_string(),
            skill: spec.skill.to_string(),
            path: spec.path.to_string(),
            claim: spec.claim.to_string(),
            wiring,
            citation,
        });
    }
    Ok(out)
}

fn grade_one(skill: &str, probes: &[ProbeResult]) -> SkillGrade {
    let mine: Vec<ProbeResult> = probes
        .iter()
        .filter(|p| p.skill == skill)
        .cloned()
        .collect();
    let finding = if skill == SKILL_ENFORCEMENT {
        let choke = mine.iter().find(|p| p.id == "check-sh-comment-only-choke");
        let hook = mine.iter().find(|p| p.id == "pre-commit-is-substrate-guard");
        match (choke.map(|p| p.wiring), hook.map(|p| p.wiring)) {
            (Some(Wiring::Contradicted), Some(Wiring::Unwired)) => {
                format!(
                    "{FINDING_PHRASE}: scripts/check.sh calls emit-tick 'not a verdict-producing gate'; hooks/pre-commit execs substrate-guard --staged only. emit_tick / tick_guard exist in crates/cdcp_bank/src/tick_emitter.rs but a commit can land with no ledger row. BUILT ≠ WIRED."
                )
            }
            _ => format!(
                "{FINDING_PHRASE} not confirmed on this tree; enforcement probes: {:?}",
                mine.iter().map(|p| (&p.id, p.wiring)).collect::<Vec<_>>()
            ),
        }
    } else {
        let computed = mine
            .iter()
            .find(|p| p.id == "emit-tick-computes-product-moved");
        let charter = mine.iter().find(|p| p.id == "charter-rule-zero");
        match (computed.map(|p| p.wiring), charter.map(|p| p.wiring)) {
            (Some(Wiring::Wired), Some(Wiring::Wired)) => {
                "Rule Zero is Charter-stated and product_moved is COMPUTED in tick_emitter.rs (product_moved_disagreement when claim ≠ compute). A tick still counts only if it changed the product; the classifier decides, the agent proposes. The choke that would refuse a no-tick commit is unwired — see loop-enforcement grade."
                    .to_string()
            }
            _ => format!(
                "loop-engineering probes incomplete: {:?}",
                mine.iter().map(|p| (&p.id, p.wiring)).collect::<Vec<_>>()
            ),
        }
    };
    SkillGrade {
        skill: skill.to_string(),
        finding,
        probes: mine,
    }
}

pub fn grade_skills(probes: &[ProbeResult]) -> Vec<SkillGrade> {
    vec![
        grade_one(SKILL_ENFORCEMENT, probes),
        grade_one(SKILL_ENGINEERING, probes),
    ]
}

pub fn assemble_wave(
    root: &Path,
    harvest: &Path,
    prior_path: Option<&Path>,
) -> Result<WaveReport, String> {
    if !harvest.is_file() {
        return Err(format!(
            "harvest ledger missing (fail closed): {}",
            harvest.display()
        ));
    }
    let harvest_sha256 = sha256_file(harvest)?;
    let prior = match prior_path {
        Some(path) => {
            let report = load_prior(path)?;
            if report.grades.is_empty() {
                return Err(format!(
                    "prior wave at {} has empty grades (fail closed)",
                    path.display()
                ));
            }
            Some(PriorRef {
                path: path.display().to_string(),
                sha256: sha256_file(path)?,
            })
        }
        None => None,
    };
    let probes = run_probes(root, harvest)?;
    let grades = grade_skills(&probes);
    if grades.len() != 2
        || grades.iter().any(|g| g.finding.trim().is_empty() || g.probes.is_empty())
    {
        return Err("wave produced an empty grade (fail closed)".to_string());
    }
    let wave = if prior.is_some() { 2 } else { 1 };
    let _ = root;
    Ok(WaveReport {
        wave,
        harvest_path: harvest.display().to_string(),
        harvest_sha256,
        prior,
        grades,
    })
}

pub fn render_stdout(report: &WaveReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("loop-wave wave={}\n", report.wave));
    out.push_str(&format!(
        "harvest_sha256={} harvest_path={}\n",
        report.harvest_sha256, report.harvest_path
    ));
    match &report.prior {
        Some(prior) => {
            out.push_str(&format!(
                "prior_artifact_path={} prior_artifact_sha256={}\n",
                prior.path, prior.sha256
            ));
            out.push_str("wave-2 consumed wave-1 artifact (not a blank-slate rerun)\n");
        }
        None => out.push_str("prior_artifact=none\n"),
    }
    for grade in &report.grades {
        out.push_str(&format!("\nGRADE skill={}\n", grade.skill));
        out.push_str(&format!("FINDING {}\n", grade.finding));
        for probe in &grade.probes {
            out.push_str(&format!(
                "PROBE id={} path={} wiring={:?} citation={}\n",
                probe.id, probe.path, probe.wiring, probe.citation
            ));
        }
    }
    out
}

pub fn write_report(dir: &Path, report: &WaveReport) -> Result<PathBuf, String> {
    fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    let json_path = dir.join(format!("wave-{}.json", report.wave));
    let md_path = dir.join(format!("wave-{}.md", report.wave));
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("serialize report: {e}"))?;
    if json.trim().is_empty() {
        return Err("serialized report empty".to_string());
    }
    fs::write(&json_path, json).map_err(|e| format!("write {}: {e}", json_path.display()))?;
    fs::write(&md_path, render_stdout(report))
        .map_err(|e| format!("write {}: {e}", md_path.display()))?;
    Ok(json_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn plant(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn live_tree() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn work_list_is_compiled_and_covers_both_skills() {
        let list = work_list();
        assert!(!list.is_empty(), "empty work list is ERROR");
        assert!(list.iter().any(|p| p.skill == SKILL_ENFORCEMENT));
        assert!(list.iter().any(|p| p.skill == SKILL_ENGINEERING));
        assert!(list.iter().any(|p| p.path.contains("tick_emitter.rs")));
        assert!(list.iter().any(|p| p.path == "scripts/check.sh"));
    }

    #[test]
    fn missing_harvest_is_error_not_empty_grade() {
        let tmp = TempDir::new().unwrap();
        let err = assemble_wave(tmp.path(), &tmp.path().join("no-harvest.md"), None).unwrap_err();
        assert!(err.contains("fail closed"), "{err}");
        assert!(!err.is_empty());
    }

    #[test]
    fn empty_harvest_is_error() {
        let tmp = TempDir::new().unwrap();
        let harvest = tmp.path().join("harvest.md");
        fs::write(&harvest, "").unwrap();
        let err = assemble_wave(tmp.path(), &harvest, None).unwrap_err();
        assert!(err.contains("empty"), "{err}");
    }

    #[test]
    fn fixture_tree_grades_comment_only_choke() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        plant(
            root,
            "crates/cdcp_bank/src/tick_emitter.rs",
            "computed_product_moved claimed_product_moved product_moved_disagreement standing by queue empty blocked on josh wait_josh",
        );
        plant(
            root,
            "scripts/check.sh",
            "# cdcp_gate emit-tick STAYS. It is an emitter, not a verdict-producing gate;\n",
        );
        plant(
            root,
            "hooks/pre-commit",
            "exec cargo run -p cdcp_gate -- substrate-guard --staged\n",
        );
        plant(
            root,
            ".flywheel/CHARTER.md",
            "value_bar: PRODUCT MOVED — a learner-visible surface changed\n",
        );
        plant(
            root,
            ".flywheel/tick-ledger.jsonl",
            "{\"schema\":\"zs.tick-receipt\"}\n",
        );
        plant(
            root,
            "crates/cdcp_gate/src/gates/emit_tick.rs",
            "emit-tick wraps tick_emitter\n",
        );
        let harvest = root.join("franken-harvest.md");
        fs::write(&harvest, "# FRANKEN HARVEST\n\n**Mirror:** /Volumes/ZestData/dicklesworthstone-mirror\n").unwrap();

        let report = assemble_wave(root, &harvest, None).unwrap();
        let enf = report
            .grades
            .iter()
            .find(|g| g.skill == SKILL_ENFORCEMENT)
            .unwrap();
        assert!(
            enf.finding.contains(FINDING_PHRASE),
            "finding={}",
            enf.finding
        );
        let check = enf
            .probes
            .iter()
            .find(|p| p.id == "check-sh-comment-only-choke")
            .unwrap();
        assert_eq!(check.wiring, Wiring::Contradicted);
        let hook = enf
            .probes
            .iter()
            .find(|p| p.id == "pre-commit-is-substrate-guard")
            .unwrap();
        assert_eq!(hook.wiring, Wiring::Unwired);
        let eng = report
            .grades
            .iter()
            .find(|g| g.skill == SKILL_ENGINEERING)
            .unwrap();
        assert!(eng.finding.contains("COMPUTED"), "{}", eng.finding);
        assert!(
            eng.finding.contains("A tick still counts only if it changed the product"),
            "{}",
            eng.finding
        );
    }

    #[test]
    fn known_bad_missing_needles_cannot_yield_wired_choke() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        plant(root, "crates/cdcp_bank/src/tick_emitter.rs", "not the emitter\n");
        plant(root, "scripts/check.sh", "echo ok\n");
        plant(root, "hooks/pre-commit", "exit 0\n");
        plant(root, ".flywheel/CHARTER.md", "hello\n");
        plant(root, ".flywheel/tick-ledger.jsonl", "{}\n");
        plant(root, "crates/cdcp_gate/src/gates/emit_tick.rs", "other\n");
        let harvest = root.join("franken-harvest.md");
        fs::write(&harvest, "# FRANKEN HARVEST\n\n**Mirror:** x\n").unwrap();
        let report = assemble_wave(root, &harvest, None).unwrap();
        let check = report
            .grades
            .iter()
            .flat_map(|g| &g.probes)
            .find(|p| p.id == "check-sh-comment-only-choke")
            .unwrap();
        assert_ne!(check.wiring, Wiring::Wired, "known-bad must not grade the choke wired");
    }

    #[test]
    fn wave_two_merge_requires_prior_and_records_hash() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        plant(
            root,
            "crates/cdcp_bank/src/tick_emitter.rs",
            "computed_product_moved claimed_product_moved product_moved_disagreement standing by queue empty blocked on josh wait_josh",
        );
        plant(
            root,
            "scripts/check.sh",
            "not a verdict-producing gate\n",
        );
        plant(
            root,
            "hooks/pre-commit",
            "substrate-guard --staged\n",
        );
        plant(root, ".flywheel/CHARTER.md", "value_bar PRODUCT MOVED\n");
        plant(root, ".flywheel/tick-ledger.jsonl", "zs.tick-receipt\n");
        plant(
            root,
            "crates/cdcp_gate/src/gates/emit_tick.rs",
            "tick_emitter wrap\n",
        );
        let harvest = root.join("franken-harvest.md");
        fs::write(&harvest, "# FRANKEN HARVEST\n\n**Mirror:** x\n").unwrap();
        let wave1 = assemble_wave(root, &harvest, None).unwrap();
        let prior_path = write_report(root, &wave1).unwrap();
        let wave2 = assemble_wave(root, &harvest, Some(&prior_path)).unwrap();
        assert_eq!(wave2.wave, 2);
        let prior = wave2.prior.as_ref().expect("wave 2 must cite wave 1");
        assert_eq!(prior.sha256, sha256_file(&prior_path).unwrap());
        let rendered = render_stdout(&wave2);
        assert!(rendered.contains("prior_artifact_sha256="));
        assert!(rendered.contains("not a blank-slate rerun"));
        assert!(rendered.contains(SKILL_ENFORCEMENT));
        assert!(rendered.contains(SKILL_ENGINEERING));
    }

    #[test]
    fn empty_prior_json_is_error() {
        let tmp = TempDir::new().unwrap();
        let prior = tmp.path().join("empty.json");
        fs::write(&prior, "   ").unwrap();
        let err = load_prior(&prior).unwrap_err();
        assert!(err.contains("empty"), "{err}");
    }

    #[test]
    fn live_engine_tree_produces_both_grades() {
        let root = live_tree();
        let harvest = PathBuf::from(std::env::var("HOME").unwrap())
            .join(".claude/references/franken-harvest.md");
        if !harvest.is_file() {
            panic!("harvest ledger must exist for the live-tree test: {}", harvest.display());
        }
        let report = assemble_wave(&root, &harvest, None).expect("live wave");
        assert_eq!(report.grades.len(), 2);
        for grade in &report.grades {
            assert!(!grade.finding.is_empty());
            assert!(!grade.probes.is_empty());
            assert!(grade.probes.iter().any(|p| Path::new(&p.path).is_relative() || p.path == "__HARVEST__" || p.path.contains('/')));
        }
        let rendered = render_stdout(&report);
        assert!(rendered.contains("GRADE skill=loop-enforcement"));
        assert!(rendered.contains("GRADE skill=loop-engineering"));
        assert!(rendered.contains("crates/cdcp_bank/src/tick_emitter.rs") || rendered.contains("scripts/check.sh"));
    }
}
