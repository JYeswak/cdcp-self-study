//! Anti-vacuous path-guard contract for extracted gate owners.
//!
//! The gate-shrink ratchet moves assertion bodies out of `cdcp_gate`; this
//! scanner follows the bodies into their product crates. The crate globs have
//! nonzero file floors, and each zero-floor dispatcher is paired with an owner
//! here. Future extraction must move the owner, its measured guard floor, and
//! the dispatcher/owner row together.

use std::path::{Path, PathBuf};

const PRODUCT_SOURCE_ROOTS: &[(&str, usize)] = &[
    ("crates/cdcp_bank/src", 10),
    ("crates/cdcp_registry_check/src", 5),
    ("crates/cdcp_learn/src", 10),
];

const DISPATCHER_OWNERS: &[(&str, &str)] = &[
    (
        "crates/cdcp_gate/src/gates/answer_key_skew.rs",
        "crates/cdcp_bank/src/answer_key_skew.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/construction_faults.rs",
        "crates/cdcp_bank/src/construction_faults.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/emit_tick.rs",
        "crates/cdcp_bank/src/tick_emitter.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/grounding_wave.rs",
        "crates/cdcp_bank/src/grounding_wave.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/key_contradiction.rs",
        "crates/cdcp_bank/src/key_contradiction.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/near_duplicate_items.rs",
        "crates/cdcp_bank/src/near_duplicate.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/required_tests.rs",
        "crates/cdcp_bank/src/required_tests.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/validate_grounding.rs",
        "crates/cdcp_bank/src/validate_grounding.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_bank.rs",
        "crates/cdcp_bank/src/verify_bank.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_content_lock.rs",
        "crates/cdcp_bank/src/verify_content_lock.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_coverage.rs",
        "crates/cdcp_bank/src/verify_coverage.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_objectives.rs",
        "crates/cdcp_learn/src/objectives.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_orphans.rs",
        "crates/cdcp_bank/src/orphans.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_knowledge_paths.rs",
        "crates/cdcp_learn/src/knowledge_paths.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/doc_facts.rs",
        "crates/cdcp_registry_check/src/doc_facts.rs",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_doc_consistency.rs",
        "crates/cdcp_registry_check/src/verify_doc_consistency.rs",
    ),
];

#[derive(Debug, Default)]
pub struct ScanReport {
    pub total: usize,
    pub reasoned: usize,
    pub flagged: Vec<String>,
    pub floors: Vec<String>,
    pub policy_errors: Vec<String>,
}

pub fn scan(root: &Path) -> ScanReport {
    let targets = scan_targets(root);
    let mut report = ScanReport::default();
    for (path, rel, python) in targets {
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{rel} unreadable: {e} — scan failure"));
        let sites = guard_sites(&src, python);
        let floor = min_sites(&rel);
        if sites.len() < floor {
            report.floors.push(format!(
                "{rel}: found {} path guards, floor is {floor} — extracted guard vanished",
                sites.len()
            ));
        }
        let mut file_reasoned = 0;
        for site in &sites {
            report.total += 1;
            match classify(site) {
                Verdict::RecordsError => {}
                Verdict::Reasoned => {
                    report.reasoned += 1;
                    file_reasoned += 1;
                }
                Verdict::Unreasoned => {
                    report
                        .flagged
                        .push(format!("{rel}:{}: {}", site.line, site.head));
                }
            }
        }
        let max = max_reasoned(&rel, sites.len());
        if file_reasoned > max {
            report.policy_errors.push(format!(
                "{rel}: {file_reasoned} of {} guards are ABSENT-OK (cap {max})",
                sites.len()
            ));
        }
    }
    report
}

fn scan_targets(root: &Path) -> Vec<(PathBuf, String, bool)> {
    let mut out = Vec::new();
    let gates = root.join("crates/cdcp_gate/src/gates");
    let mut gate_files = 0;
    for entry in std::fs::read_dir(&gates)
        .unwrap_or_else(|e| panic!("gates dir unreadable: {e} — empty scan is ERROR"))
        .flatten()
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        out.push((
            path,
            format!(
                "crates/cdcp_gate/src/gates/{}",
                entry.file_name().to_string_lossy()
            ),
            false,
        ));
        gate_files += 1;
    }
    assert!(
        gate_files >= 10,
        "gates glob found {gate_files}; scan is ERROR"
    );

    for &(rel_dir, floor) in PRODUCT_SOURCE_ROOTS {
        let dir = root.join(rel_dir);
        let mut files = 0;
        let mut owners = 0;
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{rel_dir} unreadable: {e} — scan is ERROR"))
            .flatten()
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let rel = format!("{rel_dir}/{}", entry.file_name().to_string_lossy());
            files += 1;
            if DISPATCHER_OWNERS.iter().any(|(_, owner)| *owner == rel) {
                out.push((path, rel, false));
                owners += 1;
            }
        }
        assert!(
            files >= floor,
            "{rel_dir} glob found {files}, floor is {floor}"
        );
        assert!(owners > 0, "{rel_dir} has no paired assertion owner");
    }

    let scripts = root.join("scripts");
    let mut oracles = 0;
    for entry in std::fs::read_dir(&scripts)
        .unwrap_or_else(|e| panic!("scripts/ unreadable: {e} — scan is ERROR"))
        .flatten()
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if (name.starts_with("verify_") || name.starts_with("validate_")) && name.ends_with(".py") {
            out.push((entry.path(), format!("scripts/{name}"), true));
            oracles += 1;
        }
    }
    // Six oracles (`verify_orphans.py`, `validate_grounding.py`,
    // `verify_doc_consistency.py`, `verify_injection_count.py`,
    // `verify_coverage.py`, and `verify_objectives.py`) were deliberately
    // retired into Rust in the current migration slices. One remains; zero is
    // still an ERROR, so this cannot silently become vacuous.
    assert!(oracles >= 1, "oracle glob found {oracles}; scan is ERROR");
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

#[cfg(test)]
fn product_root_floor(rel: &str) -> Option<usize> {
    PRODUCT_SOURCE_ROOTS.iter().find_map(|(dir, floor)| {
        rel.strip_prefix(dir)
            .filter(|suffix| suffix.starts_with('/'))
            .map(|_| *floor)
    })
}

fn min_sites(rel: &str) -> usize {
    match rel {
        "crates/cdcp_bank/src/answer_key_skew.rs"
        | "crates/cdcp_bank/src/construction_faults.rs"
        | "crates/cdcp_bank/src/key_contradiction.rs"
        | "crates/cdcp_bank/src/required_tests.rs" => 0,
        "crates/cdcp_bank/src/grounding_wave.rs"
        | "crates/cdcp_bank/src/near_duplicate.rs"
        | "crates/cdcp_bank/src/tick_emitter.rs" => 1,
        "crates/cdcp_bank/src/orphans.rs" => 2,
        "crates/cdcp_bank/src/validate_grounding.rs" => 7,
        "crates/cdcp_bank/src/verify_bank.rs" => 5,
        "crates/cdcp_bank/src/verify_content_lock.rs" => 9,
        "crates/cdcp_bank/src/verify_coverage.rs" => 5,
        "crates/cdcp_registry_check/src/doc_facts.rs" => 2,
        "crates/cdcp_registry_check/src/verify_doc_consistency.rs" => 3,
        "crates/cdcp_learn/src/knowledge_paths.rs" => 3,
        "crates/cdcp_learn/src/objectives.rs" => 8,
        "scripts/verify_orphans.py" | "scripts/verify_paraphrase_pairs.py" => 2,
        "scripts/verify_knowledge_paths.py" => 3,
        "scripts/verify_bank.py" | "scripts/validate_grounding.py" => 5,
        "scripts/verify_doc_consistency.py" => 1,
        _ => match rel.rsplit('/').next().unwrap_or(rel) {
            "answer_key_skew.rs"
            | "construction_faults.rs"
            | "emit_tick.rs"
            | "grounding_wave.rs"
            | "key_contradiction.rs"
            | "mod.rs"
            | "install_hooks.rs"
            | "near_duplicate_items.rs"
            | "required_tests.rs"
            | "validate_grounding.rs"
            | "verify_orphans.rs"
            | "verify_bank.rs"
            | "verify_knowledge_paths.rs"
            | "verify_objectives.rs"
            | "verify_content_lock.rs"
            | "verify_coverage.rs"
            | "doc_facts.rs"
            | "verify_doc_consistency.rs" => 0,
            "capability_maturity.rs"
            | "goldens_couplings.rs"
            | "verify_injection_count.rs"
            | "verify_step_count.rs"
            | "verify_doc_consistency.py" => 1,
            "verify_orphans.py" | "verify_paraphrase_pairs.py" => 2,
            "verify_knowledge_paths.py" => 3,
            "substrate_guard.rs" => 2,
            "verify_bank.py" | "validate_grounding.py" => 5,
            _ => 0,
        },
    }
}

fn max_reasoned(rel: &str, sites: usize) -> usize {
    match rel.rsplit('/').next().unwrap_or(rel) {
        "capability_maturity.rs"
        | "goldens_couplings.rs"
        | "verify_injection_count.rs"
        | "verify_step_count.rs" => 1,
        "doc_facts.rs" | "verify_doc_consistency.rs" => 2,
        "substrate_guard.rs" | "validate_grounding.py" => 3,
        "validate_grounding.rs" => 5,
        _ if sites < 3 => sites,
        _ => sites.saturating_sub(1) / 2,
    }
}

struct Site {
    line: usize,
    head: String,
    chain: String,
    preamble: String,
}

enum Verdict {
    RecordsError,
    Reasoned,
    Unreasoned,
}

fn classify(site: &Site) -> Verdict {
    const RECORDS: [&str; 7] = [
        "errors.push(",
        "errors.append(",
        "errors.extend(",
        "return Err(",
        "Err(",
        "missing",
        "parse error",
    ];
    if RECORDS.iter().any(|needle| site.chain.contains(needle)) {
        Verdict::RecordsError
    } else if site.chain.contains("ABSENT-OK:") || site.preamble.contains("ABSENT-OK:") {
        Verdict::Reasoned
    } else {
        Verdict::Unreasoned
    }
}

fn guard_sites(src: &str, python: bool) -> Vec<Site> {
    const TESTS: [&str; 3] = ["is_file()", ".exists()", "is_dir()"];
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let head = raw.trim_start();
        let opens = if python {
            head.starts_with("if ") || head.starts_with("elif ")
        } else {
            head.starts_with("if ")
                || head.starts_with("} else if ")
                || head.starts_with("else if ")
        };
        if !opens || !TESTS.iter().any(|needle| head.contains(needle)) {
            continue;
        }
        let chain = if python {
            py_chain(&lines, i)
        } else {
            rs_chain(&lines, i)
        };
        out.push(Site {
            line: i + 1,
            head: head.to_string(),
            chain,
            preamble: lines[i.saturating_sub(12)..i].join("\n"),
        });
    }
    out
}

fn rs_chain(lines: &[&str], start: usize) -> String {
    let mut depth = 0i32;
    let mut seen = false;
    let mut out = String::new();
    for line in &lines[start..] {
        out.push_str(line);
        out.push('\n');
        for c in strip_str_lits(line).chars() {
            match c {
                '{' => {
                    depth += 1;
                    seen = true;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        if seen && depth <= 0 {
            break;
        }
    }
    out
}

fn strip_str_lits(line: &str) -> String {
    let mut out = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for c in line.chars() {
        if quoted {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
        } else if c == '"' {
            quoted = true;
        } else {
            out.push(c);
        }
    }
    out
}

fn py_chain(lines: &[&str], start: usize) -> String {
    let indent = |s: &str| s.len() - s.trim_start().len();
    let base = indent(lines[start]);
    let mut out = format!("{}\n", lines[start]);
    for line in &lines[start + 1..] {
        if line.trim().is_empty() {
            out.push('\n');
            continue;
        }
        let level = indent(line);
        let head = line.trim_start();
        if level > base
            || (level == base && (head.starts_with("else") || head.starts_with("elif ")))
        {
            out.push_str(line);
            out.push('\n');
        } else {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn engine_root() -> PathBuf {
        cdcp_root::walk_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    struct SourceTree {
        _dir: tempfile::TempDir,
        root: PathBuf,
    }

    impl SourceTree {
        fn new() -> Self {
            let source = engine_root();
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().canonicalize().unwrap();
            let mut copied = 0;
            for rel in [
                "crates/cdcp_gate/src/gates",
                "crates/cdcp_bank/src",
                "crates/cdcp_registry_check/src",
                "crates/cdcp_learn/src",
            ] {
                let from = source.join(rel);
                let to = root.join(rel);
                std::fs::create_dir_all(&to).unwrap();
                for entry in std::fs::read_dir(from).unwrap().flatten() {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                        std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
                        copied += 1;
                    }
                }
            }
            let to = root.join("scripts");
            std::fs::create_dir_all(&to).unwrap();
            for entry in std::fs::read_dir(source.join("scripts")).unwrap().flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if (name.starts_with("verify_") || name.starts_with("validate_"))
                    && name.ends_with(".py")
                {
                    std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
                    copied += 1;
                }
            }
            assert!(copied >= 60, "source fixture copied only {copied} files");
            Self { _dir: dir, root }
        }
    }

    #[test]
    fn dispatcher_floors_have_scanned_owners() {
        let root = engine_root();
        let rels: BTreeSet<String> = scan_targets(&root).into_iter().map(|(_, r, _)| r).collect();
        for (dispatcher, owner) in DISPATCHER_OWNERS {
            assert_eq!(min_sites(dispatcher), 0);
            assert!(rels.contains(*dispatcher));
            assert!(rels.contains(*owner));
            assert!(product_root_floor(owner).unwrap() > 0);
            let src = std::fs::read_to_string(root.join(owner)).unwrap();
            let sites = guard_sites(&src, false).len();
            assert!(sites >= min_sites(owner));
            if sites > 0 {
                assert!(min_sites(owner) > 0);
            }
        }
    }

    #[test]
    fn deleting_an_extracted_guard_is_named_by_owner() {
        for (rel, needle) in [
            (
                "crates/cdcp_bank/src/verify_bank.rs",
                "if !items_dir.is_dir() {",
            ),
            (
                "crates/cdcp_registry_check/src/doc_facts.rs",
                "if ft.is_dir() {",
            ),
        ] {
            let tree = SourceTree::new();
            let path = tree.root.join(rel);
            let before = std::fs::read_to_string(&path).unwrap();
            assert_eq!(before.matches(needle).count(), 1);
            std::fs::write(&path, before.replacen(needle, "// deleted guard", 1)).unwrap();
            let report = scan(&tree.root);
            assert!(report
                .floors
                .iter()
                .any(|f| f.starts_with(&format!("{rel}:"))));
        }
    }

    #[test]
    fn scanner_planted_guard_legs_are_not_vacuous() {
        let bad = "fn f() {\n if Path::new(&p).is_file() {\n  do_it();\n }\n}\n";
        assert!(matches!(
            classify(&guard_sites(bad, false)[0]),
            Verdict::Unreasoned
        ));
        let good = "fn f() {\n // ABSENT-OK: optional input.\n if Path::new(&p).is_file() { do_it(); }\n}\n";
        assert!(matches!(
            classify(&guard_sites(good, false)[0]),
            Verdict::Reasoned
        ));
        let error = "fn f() {\n if Path::new(&p).is_file() { do_it(); } else { errors.push(\"missing\"); }\n}\n";
        assert!(matches!(
            classify(&guard_sites(error, false)[0]),
            Verdict::RecordsError
        ));
    }
}
