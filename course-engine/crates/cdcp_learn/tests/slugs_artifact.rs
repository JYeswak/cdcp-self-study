//! Verdict suite for `cdcp_learn::slugs` (bd-we5a).
//!
//! # NEVER RUN THE BUILDER AGAINST THE LIVE TREE
//!
//! `build-learn-slugs` MUTATES a tracked file. Every case here builds a
//! TREE COPY in temp. The live case then asserts the produced bytes EQUAL
//! the tracked `web/data/module_learn_slugs.js`.
//!
//! WRITE-AFTER-VERDICT, asserted on every case: a run that exits non-zero
//! must leave no artifact, and a run that exits zero must leave one.
//!
//! ANTI-VACUOUS: a missing or empty registry is RED. A generator that
//! emits an empty map is ERROR. A registry module with no id (no slug to
//! emit) is RED, naming the module.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const SRC_REL: &str = "knowledge/domains.toml";
const ARTIFACT_REL: &str = "web/data/module_learn_slugs.js";

static COMPARED: AtomicUsize = AtomicUsize::new(0);
static ROUND: AtomicUsize = AtomicUsize::new(0);

fn engine_root() -> PathBuf {
    cdcp_learn::resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn write_file(path: &Path, body: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, body).unwrap();
}

fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src)
        .expect("read fixture template")
        .flatten()
    {
        let from = e.path();
        let to = dst.join(e.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

struct Fixture {
    dir: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Fixture {
        let f = Fixture {
            dir: tempfile::tempdir().unwrap(),
        };
        std::fs::create_dir_all(f.template()).unwrap();
        f
    }

    fn template(&self) -> PathBuf {
        self.dir.path().join("template")
    }

    fn seed_live_source(&self) {
        let live = engine_root().join(SRC_REL);
        assert!(
            live.is_file(),
            "the live domain registry is missing: {SRC_REL}"
        );
        let dst = self
            .template()
            .join(SRC_REL.replace('/', std::path::MAIN_SEPARATOR_STR));
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(&live, &dst).unwrap();
        assert!(
            std::fs::metadata(&dst).unwrap().len() > 0,
            "copied an empty registry — a vacuous fixture is an ERROR, not a pass"
        );
    }

    fn put_domains(&self, body: &str) {
        write_file(&self.template().join("knowledge/domains.toml"), body);
    }
}

struct Run {
    code: i32,
    stdout: Vec<u8>,
    artifact: Option<Vec<u8>>,
}

impl Run {
    fn out(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
}

fn artifact_of(root: &Path) -> Option<Vec<u8>> {
    std::fs::read(root.join(ARTIFACT_REL)).ok()
}

fn run_builder(label: &str, f: &Fixture) -> Run {
    let n = ROUND.fetch_add(1, Ordering::SeqCst);
    let root = f.dir.path().join(format!("round{n}"));
    copy_tree(&f.template(), &root);

    let outcome = cdcp_learn::slugs::write_slugs(&root).expect("write_slugs");
    let r = Run {
        code: outcome.code,
        stdout: outcome.stdout.into_bytes(),
        artifact: artifact_of(&root),
    };

    if r.code != 0 {
        assert!(
            !r.out().contains("PASS"),
            "[{label}] exited {} with a success token on stdout:\n{}",
            r.code,
            r.out()
        );
        assert!(
            r.artifact.is_none(),
            "[{label}] exited {} but left {ARTIFACT_REL} behind",
            r.code
        );
    } else {
        assert!(
            r.artifact.is_some(),
            "[{label}] exited 0 without writing {ARTIFACT_REL}"
        );
    }

    COMPARED.fetch_add(1, Ordering::SeqCst);
    r
}

/// A registry of `count` modules whose slugs are FORMATTED, not enumerated.
fn domains(count: i64) -> String {
    let mut s = String::from("schema_version = 1\n");
    for n in 1..=count {
        s.push_str(&format!("\n[[domain]]\nid = \"{n:02}-mod\"\norder = {n}\n"));
    }
    s
}

#[test]
fn live_inputs_are_green_and_reproduce_the_tracked_artifact() {
    let f = Fixture::new();
    f.seed_live_source();
    let rs = run_builder("live inputs", &f);

    assert_eq!(rs.code, 0, "live inputs must be GREEN: {}", rs.out());
    assert!(
        rs.out().starts_with("PASS: build_learn_slugs modules="),
        "{}",
        rs.out()
    );

    let tracked = std::fs::read(engine_root().join(ARTIFACT_REL))
        .expect("the tracked web/data/module_learn_slugs.js must exist");
    assert_eq!(
        rs.artifact.as_deref(),
        Some(tracked.as_slice()),
        "the compiler would rewrite the tracked {ARTIFACT_REL}; a builder that \
         does not reproduce its own committed output is not byte-exact"
    );
}

#[test]
fn a_missing_registry_is_an_error_and_writes_nothing() {
    let f = Fixture::new();
    let rs = run_builder("missing registry", &f);
    assert_ne!(rs.code, 0, "a missing registry must never be a pass");
    assert!(rs.out().contains("domain registry missing"), "{}", rs.out());
    assert!(
        rs.out().starts_with("FAIL: build_learn_slugs"),
        "{}",
        rs.out()
    );
    assert!(rs.artifact.is_none());
}

#[test]
fn an_empty_registry_is_an_error() {
    let f = Fixture::new();
    f.put_domains("schema_version = 1\n");
    let rs = run_builder("empty registry", &f);
    assert_ne!(
        rs.code,
        0,
        "zero modules must never be a pass: {}",
        rs.out()
    );
    assert!(
        rs.out().contains("MODULE_LEARN_SLUGS empty"),
        "a generator that emits an empty map is ERROR: {}",
        rs.out()
    );
    assert!(
        rs.out().starts_with("FAIL: build_learn_slugs"),
        "{}",
        rs.out()
    );
    assert!(!rs.out().contains("PASS"), "{}", rs.out());
    assert!(rs.artifact.is_none());
}

#[test]
fn an_unparseable_registry_is_an_error() {
    let f = Fixture::new();
    f.put_domains("this is not toml [[[");
    let rs = run_builder("unparseable registry", &f);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("domain registry parse error"),
        "{}",
        rs.out()
    );
    assert!(rs.artifact.is_none());
}

/// Known-bad: a registry module with no id has no slug to emit and must go
/// RED naming the module. Deleting this assertion is deleting the acceptance.
#[test]
fn a_registry_module_with_no_id_is_red_naming_it() {
    let f = Fixture::new();
    f.put_domains(
        "schema_version = 1\n\
         \n[[domain]]\nid = \"01-mod\"\norder = 1\n\
         \n[[domain]]\norder = 6\n",
    );
    let rs = run_builder("module with no id", &f);
    assert_ne!(
        rs.code,
        0,
        "a registry module with no emitted slug must be RED:\n{}",
        rs.out()
    );
    assert!(
        rs.out().contains("module 6:") && rs.out().contains("no slug to emit"),
        "the gap must name the module:\n{}",
        rs.out()
    );
    assert!(
        rs.out().starts_with("FAIL: build_learn_slugs"),
        "{}",
        rs.out()
    );
    assert!(!rs.out().contains("PASS"), "{}", rs.out());
    assert!(rs.artifact.is_none(), "a red compile writes nothing");
}

#[test]
fn a_module_with_no_order_is_red_naming_it() {
    let f = Fixture::new();
    f.put_domains(
        "schema_version = 1\n\
         \n[[domain]]\nid = \"01-mod\"\norder = 1\n\
         \n[[domain]]\nid = \"no-order-mod\"\n",
    );
    let rs = run_builder("module with no order", &f);
    assert_ne!(rs.code, 0, "{}", rs.out());
    assert!(
        rs.out().contains("no-order-mod") && rs.out().contains("no slug to emit"),
        "{}",
        rs.out()
    );
    assert!(rs.artifact.is_none());
}

#[test]
fn a_green_registry_emits_every_declared_slug() {
    let f = Fixture::new();
    f.put_domains(&domains(3));
    let rs = run_builder("three modules", &f);
    assert_eq!(rs.code, 0, "{}", rs.out());
    let body = String::from_utf8(rs.artifact.expect("artifact")).unwrap();
    let parsed = cdcp_learn::slugs::parse_module_learn_slugs(&body).unwrap();
    assert_eq!(parsed.len(), 3, "empty or short map: {body}");
    for n in 1..=3i64 {
        assert_eq!(
            parsed.get(&n).map(String::as_str),
            Some(format!("{n:02}-mod").as_str()),
            "declared module {n} missing from emitted map:\n{body}"
        );
    }
}

#[test]
fn the_suite_ran_something() {
    let before = COMPARED.load(Ordering::SeqCst);
    let f = Fixture::new();
    f.put_domains(&domains(2));
    run_builder("suite self-check", &f);
    assert!(
        COMPARED.load(Ordering::SeqCst) > before,
        "the slugs artifact suite ran nothing"
    );
}
