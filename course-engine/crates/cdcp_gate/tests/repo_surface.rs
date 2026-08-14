//! Assertions against THIS repo (not a fixture): the seeded allowlist must stay
//! honest, and the crate must not overclaim anywhere.

mod support;
use std::path::{Path, PathBuf};

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

#[test]
fn the_real_allowlist_is_schema_clean_and_unexpired() {
    let root = engine_root();
    let text = std::fs::read_to_string(root.join(cdcp_gate::gates::substrate_guard::REGISTRY_PATH))
        .expect("registries/substrate_allowlist.toml must exist");
    let al = cdcp_gate::gates::substrate_guard::parse_allowlist(&text).expect("parses");

    assert!(
        cdcp_gate::gates::substrate_guard::check_floor(&al.scan).is_empty(),
        "the shipped registry must not narrow the compiled-in floor"
    );
    assert!(
        cdcp_gate::gates::substrate_guard::check_wiring_status(&al.wiring).is_empty(),
        "the shipped [wiring] block must be well formed"
    );
    assert!(
        !al.allow.is_empty(),
        "an empty allowlist here would mean the seeding never happened"
    );

    let exists = |p: &str| root.join(p).exists();
    let problems = cdcp_gate::gates::substrate_guard::validate_rows(
        &al.allow,
        &al.scan,
        cdcp_gate::date::today(),
        &exists,
    );
    assert!(
        problems.is_empty(),
        "shipped allowlist has {} problem(s):\n  {}",
        problems.len(),
        problems.join("\n  ")
    );
}

#[test]
fn every_row_carries_a_bead_and_a_future_date() {
    let root = engine_root();
    let text = std::fs::read_to_string(root.join(cdcp_gate::gates::substrate_guard::REGISTRY_PATH))
        .unwrap();
    let al = cdcp_gate::gates::substrate_guard::parse_allowlist(&text).unwrap();
    for r in &al.allow {
        assert!(
            cdcp_gate::gates::substrate_guard::looks_like_bead_id(&r.migration_bead),
            "{}: migration_bead {:?}",
            r.path,
            r.migration_bead
        );
        let d = cdcp_gate::date::parse_ymd(r.expires.trim()).expect("expires parses");
        assert!(
            !cdcp_gate::date::before(d, cdcp_gate::date::today()),
            "{}: expired on {}",
            r.path,
            r.expires
        );
    }
}

/// The header's claim class is FLOOR-RAISE. Nothing in the shipped source,
/// registry, or hook may promise more than that.
#[test]
fn no_overclaiming_language_in_the_shipped_surface() {
    let root = engine_root();
    let mut files: Vec<PathBuf> = Vec::new();
    collect_rs(&root.join("crates/cdcp_gate/src"), &mut files);
    files.push(root.join("crates/cdcp_gate/build.rs"));
    files.push(root.join("registries/substrate_allowlist.toml"));
    files.push(root.join("hooks/pre-commit"));
    assert!(
        files.len() >= 8,
        "scanned {} files — a vacuous honesty scan is an ERROR",
        files.len()
    );

    for f in &files {
        let text = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        // Test modules quote the banned words on purpose; scan the shipped part.
        let shipped = text
            .split("#[cfg(test)]")
            .next()
            .unwrap_or("")
            .to_lowercase();
        for banned in ["guarantee", "proves", "makes impossible"] {
            assert!(
                !shipped.contains(banned),
                "{} overclaims with {banned:?}",
                f.display()
            );
        }
    }
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// The committed shim and the shim installed in this clone must agree.
/// BUILT != WIRED — a hook that exists only on one machine is not a gate.
#[test]
fn the_pre_commit_shim_is_installed_in_this_clone() {
    let root = engine_root();
    let (code, out) = support::run_gate(&root, &["install-hooks", "--check"]);
    assert_eq!(
        code, 0,
        "the committed hooks/pre-commit is not installed here:\n{out}"
    );
}
