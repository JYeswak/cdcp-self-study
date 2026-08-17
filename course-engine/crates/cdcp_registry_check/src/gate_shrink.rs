//! GATE-SHRINK RATCHET — bd-engine-not-gate-ar39.1
//!
//! `cdcp_gate` total `.rs` lines may fall and may never rise.
//!
//! WHY THIS LIVES HERE, NOT IN `cdcp_gate`: a measuring instrument that added
//! lines to the crate it measures would move the ceiling on arrival. The
//! checker is `cdcp_registry_check`; the measured tree is `crates/cdcp_gate`.
//!
//! RAISING `ceiling_lines` in `registries/gate_shrink.toml` is weakening a
//! gate and is ESCALATION-ONLY. Lowering it after an extraction is autonomous.
//!
//! DELETE THIS MODULE, its tests, the check.sh mention, and
//! `registries/gate_shrink.toml` when BOTH are true:
//!   1. `cdcp_gate` line count < `delete_when_lines_below` (15_000)
//!   2. no `product_gate_files` remain under `crates/cdcp_gate/src/gates/`
//!
//! The check FAILS once both hold, so the instrument cannot outlive its job
//! (FRANKEN-EXTRACT Doctrine #0 applied to the thing enforcing Doctrine #0).

#![forbid(unsafe_code)]

use super::CheckError;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const REGISTRY_REL: &str = "registries/gate_shrink.toml";
pub const GATE_CRATE_REL: &str = "crates/cdcp_gate";

#[derive(Debug, Deserialize)]
struct Registry {
    ceiling_lines: usize,
    delete_when_lines_below: usize,
    min_rs_files: usize,
    #[serde(default)]
    product_crates: Vec<String>,
    #[serde(default)]
    product_gate_files: Vec<String>,
    #[serde(default, rename = "file")]
    files: Vec<FileRow>,
}

#[derive(Debug, Deserialize)]
struct FileRow {
    path: String,
    lines: usize,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ShrinkReport {
    pub gate_lines: usize,
    pub gate_files: usize,
    pub ceiling: usize,
    pub product_lines: usize,
    pub grade_lines: usize,
}

fn bytecount_newlines(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == b'\n').count()
}

fn count_rs_files(dir: &Path) -> Result<BTreeMap<String, usize>, CheckError> {
    let mut out = BTreeMap::new();
    if !dir.is_dir() {
        return Err(CheckError::msg(format!(
            "gate-shrink: {dir:?} is not a directory — a scan that found no crate is an ERROR, not a pass"
        )));
    }
    walk_rs(dir, dir, &mut out)?;
    Ok(out)
}

fn walk_rs(root: &Path, dir: &Path, out: &mut BTreeMap<String, usize>) -> Result<(), CheckError> {
    let rd = fs::read_dir(dir)
        .map_err(|e| CheckError::msg(format!("gate-shrink: read {}: {e}", dir.display())))?;
    for ent in rd {
        let ent = ent.map_err(|e| CheckError::msg(format!("gate-shrink: dirent: {e}")))?;
        let path = ent.path();
        let name = ent.file_name();
        if name == "target" {
            continue;
        }
        if path.is_dir() {
            walk_rs(root, &path, out)?;
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(&path)
            .map_err(|e| CheckError::msg(format!("gate-shrink: read {}: {e}", path.display())))?;
        // Same metric as `wc -l`: number of newline bytes, not splitlines().
        out.insert(rel, bytecount_newlines(&bytes));
    }
    Ok(())
}

fn product_lines(engine_root: &Path, crates: &[String]) -> (usize, usize) {
    let mut total = 0usize;
    let mut grade = 0usize;
    for name in crates {
        let dir = engine_root.join("crates").join(name);
        if !dir.is_dir() {
            continue;
        }
        if let Ok(map) = count_rs_files(&dir) {
            let n: usize = map.values().sum();
            total += n;
            if name == "cdcp_grade" {
                grade = n;
            }
        }
    }
    (total, grade)
}

/// Live check. Prints the ceiling and the product-Rust total on green.
pub fn check_gate_shrink(engine_root: &Path) -> Result<ShrinkReport, CheckError> {
    let reg_path = engine_root.join(REGISTRY_REL);
    if !reg_path.is_file() {
        return Err(CheckError::msg(format!(
            "gate-shrink: missing {REGISTRY_REL} — a ratchet with no pin is not a ratchet"
        )));
    }
    let text = fs::read_to_string(&reg_path)
        .map_err(|e| CheckError::msg(format!("gate-shrink: read {REGISTRY_REL}: {e}")))?;
    let reg: Registry = toml::from_str(&text)
        .map_err(|e| CheckError::msg(format!("gate-shrink: parse {REGISTRY_REL}: {e}")))?;

    if reg.ceiling_lines == 0 {
        return Err(CheckError::msg(
            "gate-shrink: ceiling_lines = 0 is vacuous — a ceiling of nothing is not a ceiling",
        ));
    }

    let gate_dir = engine_root.join(GATE_CRATE_REL);
    let live = count_rs_files(&gate_dir)?;
    if live.len() < reg.min_rs_files {
        return Err(CheckError::msg(format!(
            "gate-shrink: found {} .rs file(s) under {GATE_CRATE_REL} < min_rs_files={} — \
             a scan that found nothing (or almost nothing) is an ERROR, not a pass",
            live.len(),
            reg.min_rs_files
        )));
    }

    let gate_lines: usize = live.values().sum();
    let (product_lines, grade_lines) = product_lines(engine_root, &reg.product_crates);

    let product_gates_present: Vec<&str> = reg
        .product_gate_files
        .iter()
        .filter(|rel| live.contains_key(rel.as_str()))
        .map(|s| s.as_str())
        .collect();

    if gate_lines < reg.delete_when_lines_below && product_gates_present.is_empty() {
        return Err(CheckError::msg(format!(
            "gate-shrink: DELETE THIS RATCHET — cdcp_gate={gate_lines} < {} \
             and no product_gate_files remain under src/gates/. Doctrine #0: \
             the instrument has earned deletion. Remove registries/gate_shrink.toml \
             and crates/cdcp_registry_check/src/gate_shrink.rs.",
            reg.delete_when_lines_below
        )));
    }

    if gate_lines > reg.ceiling_lines {
        let baseline: BTreeMap<&str, usize> = reg
            .files
            .iter()
            .map(|f| (f.path.as_str(), f.lines))
            .collect();
        let mut deltas: Vec<String> = Vec::new();
        for (path, &n) in &live {
            let was = baseline.get(path.as_str()).copied().unwrap_or(0);
            if n > was {
                if was == 0 {
                    deltas.push(format!("  +{n} {path} (new)"));
                } else {
                    deltas.push(format!("  +{} {path} ({was} -> {n})", n - was));
                }
            }
        }
        for f in &reg.files {
            if !live.contains_key(&f.path) {
                deltas.push(format!("  -{} {} (removed)", f.lines, f.path));
            }
        }
        deltas.sort();
        let shown = if deltas.is_empty() {
            "  (no per-file baseline deltas; total still exceeded)".to_string()
        } else {
            deltas.join("\n")
        };
        return Err(CheckError::msg(format!(
            "gate-shrink: cdcp_gate {gate_lines} > ceiling {} — the crate GREW. \
             Raising ceiling_lines is weakening a gate (escalation-only). \
             Extract or delete; do not transcribe.\n{shown}",
            reg.ceiling_lines
        )));
    }

    let ratio = if product_lines == 0 {
        0.0
    } else {
        gate_lines as f64 / product_lines as f64
    };
    println!(
        "gate-shrink: ok: cdcp_gate={gate_lines}/{} files={} product={product_lines} \
         ratio={ratio:.1}x grade={grade_lines}",
        reg.ceiling_lines,
        live.len()
    );
    if gate_lines < reg.ceiling_lines {
        println!(
            "gate-shrink: RATCHET SLACK: measured {gate_lines} < ceiling {}. \
             Lower ceiling_lines in {REGISTRY_REL} to {gate_lines} so the number stays honest.",
            reg.ceiling_lines
        );
    }

    Ok(ShrinkReport {
        gate_lines,
        gate_files: live.len(),
        ceiling: reg.ceiling_lines,
        product_lines,
        grade_lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, rel: &str, body: &str) {
        let p = dir.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, body).unwrap();
    }

    fn pin(
        dir: &Path,
        ceiling: usize,
        min_files: usize,
        delete_below: usize,
        files: &[(&str, usize)],
    ) {
        let mut t = format!(
            "ceiling_lines = {ceiling}\n\
             delete_when_lines_below = {delete_below}\n\
             min_rs_files = {min_files}\n\
             product_crates = []\n\
             product_gate_files = [\"src/gates/build_units.rs\"]\n"
        );
        for (p, n) in files {
            t.push_str(&format!("\n[[file]]\npath = \"{p}\"\nlines = {n}\n"));
        }
        write(dir, REGISTRY_REL, &t);
    }

    #[test]
    fn missing_crate_is_an_error_not_a_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        pin(root, 100, 1, 0, &[]);
        let err = check_gate_shrink(root).unwrap_err();
        assert!(err.to_string().contains("not a directory"), "{err}");
    }

    #[test]
    fn zero_rs_files_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        pin(root, 100, 1, 0, &[]);
        fs::create_dir_all(root.join(GATE_CRATE_REL)).unwrap();
        let err = check_gate_shrink(root).unwrap_err();
        assert!(err.to_string().contains("min_rs_files"), "{err}");
    }

    #[test]
    fn over_ceiling_is_red_and_names_the_new_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, &format!("{GATE_CRATE_REL}/src/lib.rs"), "fn a() {}\n");
        pin(root, 1, 1, 0, &[("src/lib.rs", 1)]);
        write(
            root,
            &format!("{GATE_CRATE_REL}/src/gates/extra.rs"),
            "fn extra() {}\nfn more() {}\n",
        );
        let err = check_gate_shrink(root).unwrap_err();
        let s = err.to_string();
        assert!(s.contains("GREW"), "{s}");
        assert!(s.contains("extra.rs"), "{s}");
    }

    #[test]
    fn at_ceiling_is_green() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            &format!("{GATE_CRATE_REL}/src/lib.rs"),
            "fn a() {}\nfn b() {}\n",
        );
        pin(root, 2, 1, 0, &[("src/lib.rs", 2)]);
        let rep = check_gate_shrink(root).unwrap();
        assert_eq!(rep.gate_lines, 2);
        assert_eq!(rep.ceiling, 2);
    }

    #[test]
    fn deletion_condition_fails_so_the_instrument_cannot_linger() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // 2 lines, delete_when=15, no product gate file.
        write(
            root,
            &format!("{GATE_CRATE_REL}/src/lib.rs"),
            "fn a() {}\nfn b() {}\n",
        );
        pin(root, 100, 1, 15, &[("src/lib.rs", 2)]);
        let err = check_gate_shrink(root).unwrap_err();
        assert!(err.to_string().contains("DELETE THIS RATCHET"), "{err}");
    }

    #[test]
    fn deletion_condition_does_not_fire_while_a_product_gate_file_remains() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, &format!("{GATE_CRATE_REL}/src/lib.rs"), "fn a() {}\n");
        write(
            root,
            &format!("{GATE_CRATE_REL}/src/gates/build_units.rs"),
            "fn units() {}\n",
        );
        pin(
            root,
            100,
            1,
            15,
            &[("src/lib.rs", 1), ("src/gates/build_units.rs", 1)],
        );
        let rep = check_gate_shrink(root).unwrap();
        assert_eq!(rep.gate_lines, 2);
    }
}
