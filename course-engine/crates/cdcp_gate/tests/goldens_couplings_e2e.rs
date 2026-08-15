//! End-to-end legs for the `goldens-couplings` gate (bd-hardening-b-ledgers-gvm.2).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! Three things are held here and nothing more:
//!
//!   1. **Known-bad.** Each way a golden can be uncoupled from what it was
//!      frozen against — a struct field added, a builder's key set changed, a
//!      source region edited, a symbol renamed away, a source file deleted, a
//!      locked dependency moved, a version left unbumped after a pin moved, a
//!      golden re-frozen with nothing re-affirmed, a golden that did not
//!      re-affirm a surface that moved, a path constant drifting from the row
//!      that pins it, a blank field, an empty `depends_on`, and an artifact
//!      appearing under `goldens/` with no row — is planted and asserted to
//!      reach a non-zero exit naming both sides.
//!   2. **Known-GOOD.** A well-formed ledger passes, and — the leg that decides
//!      whether this gate is usable — a LEGITIMATELY updated golden passes: move
//!      the source, re-pin, re-derive the version, re-freeze the artifact, and
//!      re-affirm, and the gate goes green. An attack-only suite ships an
//!      over-strict gate, and over-strict gates get routed around instead of
//!      fixed.
//!   3. **The live ledger.** `registries/goldens-couplings.toml` is schema
//!      clean and its outstanding findings are exactly `KNOWN_DEBTS`, with a
//!      reason each, so a finding cannot appear silently and one that is paid
//!      off fails this test until it is struck from the list.
//!
//! # WHAT THIS SUITE CANNOT DECIDE
//!
//! It cannot decide that a golden is correct, that a justification is honest,
//! or that anyone re-read anything before re-dating a row. It runs the gate
//! binary, so it says nothing about whether `scripts/check.sh` calls it;
//! BUILT != WIRED is settled by the check.sh step, not by a test.

use cdcp_gate::gates::goldens_couplings as gc;
use cdcp_gate::gates::verify_content_lock::{sha256_file, sha256_hex_bytes};
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");

/// Exit codes, mirrored from `cdcp_gate::exit` so a change there shows up here.
const OK: i32 = 0;
const VIOLATION: i32 = 2;
const USAGE: i32 = 3;
const ERROR: i32 = 4;

/// The findings the LIVE ledger carries, with the reason each is outstanding.
///
/// EVERY ONE OF THESE IS A FINDING ABOUT THE REPO, not a defect in the registry,
/// and together they are the reason this gate was not vacuous on its first run.
///
/// The ledger is seeded from the COMMITTED tree, because that is the last state
/// in which these artifacts and the source above them agreed.
/// bd-hardening-c-status-hzs.2 (C2) is in flight against
/// `crates/cdcp_bank/src/lib.rs`: `objective_ids`, `citation_ids` and `tags`
/// join `BankItem`, those three plus `status` join `hash_payload`, and
/// `compute_bank_hash` gains an empty-bank guard — so `bank_hash` is being
/// redefined under all seven frozen artifacts, and all seven were re-frozen on
/// disk in the same hour (measured 2026-08-14 15:44). That is PLAN §B2's defect
/// happening: nothing in the tree recorded that a surface had moved, and the
/// only step that would eventually have noticed is a byte `cmp` five hundred
/// lines into check.sh, which names a file and never a surface.
///
/// The working tree does not agree with itself while C2 lands — `goldens check`
/// exits non-zero, `verify-content-lock` reports bank_hash drift, and
/// `export-web --seed 42` reproduces none of the three committed packs — so the
/// VALUES in these findings churn. Their IDS do not, which is what this list
/// tracks.
///
/// PAID OFF 2026-08-14, and the list is empty because the WALK was done — not
/// because the rows were struck to get to green.
///
/// C2 and C3 landed; the four-command block in `goldens/PROVENANCE.md` ran in
/// order; both bank surfaces were re-pinned from source; both versions were
/// re-derived by the gate rather than invented; all seven `frozen` digests were
/// re-taken; every `depends_on` was re-affirmed. Two rows had their MEANING
/// changed and were rewritten rather than re-dated — `bank.hash-payload` most of
/// all, because `status` entering the payload inverts PROVENANCE §"Bank drift":
/// a selection-only edit used to leave bank_hash alone and now moves it.
///
/// WHAT AN EMPTY LIST DOES AND DOES NOT MEAN. It means no surface is currently
/// known to have moved under a frozen artifact. It does NOT mean the couplings
/// are correct — this gate cannot decide that a golden holds the right answer,
/// that a justification is honest, or that anyone re-read anything before
/// re-affirming. bd-6ycw is outstanding and WILL turn this list red again when
/// the hash domain moves to v2; that is the mechanism working, not a regression.
///
/// A NEW id here is a NEW finding about the repo: file it, then add it with a
/// reason. Striking a row without the walk above is the defect this file exists
/// to prevent, performed on the file that prevents it.
const KNOWN_DEBTS: &[(&str, &str)] = &[];

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn run_gate(root: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(BIN)
        .current_dir(root)
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .expect("run cdcp_gate");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), s)
}

// ── fixture ────────────────────────────────────────────────────────────────

/// The demo source every pin in the fixture reads out of.
const DEMO_SRC: &str = r#"//! demo crate
pub const GOLDEN_PATH: &str = "goldens/bank_hash.txt";
pub const DEMO_VERSION: u32 = 1;

pub struct Shape {
    pub alpha: u32,
    #[serde(default)]
    pub beta: String,
}

pub fn build() -> Map {
    let mut m = Map::new();
    m.insert("alpha".into(), 1);
    json!({ "beta": 2 })
}
"#;

const DEMO_LOCK: &str = r#"
[[package]]
name = "demo_app"
version = "0.1.0"
dependencies = ["rng 0.8.7"]

[[package]]
name = "rng"
version = "0.8.7"

[[package]]
name = "rng"
version = "0.9.5"
"#;

/// The four artifacts the gate compiles in as REQUIRED, plus the prose file
/// discovery must skip.
const GOLDEN_FILES: &[(&str, &str)] = &[
    ("goldens/bank_hash.txt", "e82817572a82d13f\n"),
    ("goldens/fixtures/mock40_seed42.json", "{\"seed\":42}\n"),
    ("goldens/mock40_seed42_all_correct.sha256", "7bb20d74\n"),
    ("goldens/mock40_seed42_all_wrong.sha256", "deb1de3b\n"),
];

const LONG: &str =
    "a justification long enough that a reviewer has something concrete to disagree with here";

struct Repo {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    /// A repo whose ledger is self-consistent: every pin resolves, every
    /// version is the derived one, every artifact matches its `frozen` digest.
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonicalize");
        let r = Repo { _dir: dir, root };
        r.write("crates/demo/src/lib.rs", DEMO_SRC);
        r.write("Cargo.lock", DEMO_LOCK);
        for (p, body) in GOLDEN_FILES {
            r.write(p, body);
        }
        r.write("goldens/PROVENANCE.md", "# prose, not a golden\n");
        r.rebuild_ledger();
        r
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn write(&self, rel: &str, body: &str) {
        let p = self.path(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
    }

    fn remove(&self, rel: &str) {
        std::fs::remove_file(self.path(rel)).unwrap();
    }

    fn set_ledger(&self, body: &str) {
        self.write(gc::REGISTRY_PATH, body);
    }

    fn read_ledger(&self) -> String {
        std::fs::read_to_string(self.path(gc::REGISTRY_PATH)).unwrap()
    }

    /// Regenerate a ledger that is honest about the CURRENT fixture: pins read
    /// from the source on disk, versions derived from those pins, `frozen`
    /// digests taken from the artifacts as they now stand. This is the
    /// mechanical form of "the author did the whole re-affirmation".
    fn rebuild_ledger(&self) {
        let src = std::fs::read_to_string(self.path("crates/demo/src/lib.rs")).unwrap();
        let region = {
            let (a, b) = gc::find_region(&src, "build").expect("build region");
            sha256_hex_bytes(gc::normalise_region(&src[a..b]).as_bytes())
        };
        let fields = {
            let (a, b) = gc::find_region(&src, "Shape").expect("Shape region");
            gc::extract_fields(&src[a..b])
        };
        let keys = {
            let (a, b) = gc::find_region(&src, "build").expect("build region");
            gc::extract_key_literals(&src[a..b])
        };
        let konst = {
            let (a, b) = gc::find_region(&src, "DEMO_VERSION").expect("const region");
            gc::extract_const_value(&src[a..b]).expect("const value")
        };
        let text = ledger(&LedgerSpec {
            konst,
            fields,
            keys,
            region,
            frozen: self.frozen_digests(),
            ..LedgerSpec::default()
        });
        self.set_ledger(&text);
    }

    fn frozen_digests(&self) -> Vec<String> {
        GOLDEN_FILES
            .iter()
            .map(|(p, _)| sha256_hex_bytes(&std::fs::read(self.path(p)).unwrap()))
            .collect()
    }

    fn gate(&self, args: &[&str]) -> (i32, String) {
        run_gate(&self.root, args)
    }

    /// Rewrite one line of the shipped ledger.
    fn patch_ledger(&self, from: &str, to: &str) {
        let text = self.read_ledger();
        assert!(
            text.contains(from),
            "fixture edit target {from:?} not present"
        );
        self.set_ledger(&text.replacen(from, to, 1));
    }
}

/// Everything the fixture ledger is parameterised by.
struct LedgerSpec {
    konst: String,
    fields: Vec<String>,
    keys: Vec<String>,
    region: String,
    frozen: Vec<String>,
    affirmed: String,
    lock_version: String,
    /// `None` means "quote whatever the surface derives".
    dep_override: Option<String>,
    /// `None` means "record whatever the pin block derives".
    version_override: Option<String>,
    path_const: String,
}

impl Default for LedgerSpec {
    fn default() -> Self {
        Self {
            konst: "1".into(),
            fields: vec!["alpha".into(), "beta".into()],
            keys: vec!["alpha".into(), "beta".into()],
            region: "0".repeat(64),
            frozen: vec!["0".repeat(64); GOLDEN_FILES.len()],
            affirmed: "2026-08-14".into(),
            lock_version: "0.8.7".into(),
            dep_override: None,
            version_override: None,
            path_const: "crates/demo/src/lib.rs::GOLDEN_PATH".into(),
        }
    }
}

fn toml_list(v: &[String]) -> String {
    v.iter()
        .map(|s| format!("{s:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Render a ledger, deriving each surface's `version` from its own pins the way
/// the gate does — so a fixture cannot pass by recording a version nobody could
/// have computed.
fn ledger(spec: &LedgerSpec) -> String {
    let mut body = format!(
        "schema_version = 1\n\n[policy]\naffirmation_days = 365\n\n\
         [[surface]]\nid = \"demo.shape\"\ntitle = \"demo shape\"\nversion = \"AUTO_SHAPE\"\n\
         justification = \"{LONG}\"\n\
         [[surface.pin]]\nkind = \"const\"\nfile = \"crates/demo/src/lib.rs\"\nsymbol = \"DEMO_VERSION\"\nexpect = [{:?}]\n\
         [[surface.pin]]\nkind = \"fields\"\nfile = \"crates/demo/src/lib.rs\"\nsymbol = \"Shape\"\nexpect = [{}]\n\
         [[surface.pin]]\nkind = \"keys\"\nfile = \"crates/demo/src/lib.rs\"\nsymbol = \"build\"\nexpect = [{}]\n\
         [[surface.pin]]\nkind = \"region\"\nfile = \"crates/demo/src/lib.rs\"\nsymbol = \"build\"\ndigest = \"{}\"\n\n\
         [[surface]]\nid = \"demo.prng\"\ntitle = \"demo prng\"\nversion = \"AUTO_PRNG\"\n\
         justification = \"{LONG}\"\n\
         [[surface.pin]]\nkind = \"lockdep\"\nfile = \"Cargo.lock\"\nsymbol = \"rng\"\nvia = \"demo_app\"\nexpect = [{:?}]\n",
        spec.konst,
        toml_list(&spec.fields),
        toml_list(&spec.keys),
        spec.region,
        spec.lock_version,
    );

    for (i, (path, _)) in GOLDEN_FILES.iter().enumerate() {
        let konst = if i == 0 && !spec.path_const.is_empty() {
            format!("const = {:?}\n", spec.path_const)
        } else {
            String::new()
        };
        body.push_str(&format!(
            "\n[[golden]]\nid = \"g{i}\"\nfile = {path:?}\n{konst}\
             frozen = \"{}\"\naffirmed = \"{}\"\njustification = \"{LONG}\"\n\
             depends_on = [\n  {{ surface = \"demo.shape\", version = \"DEP_SHAPE\" }},\n  {{ surface = \"demo.prng\", version = \"DEP_PRNG\" }},\n]\n",
            spec.frozen[i], spec.affirmed,
        ));
    }

    // Two passes: parse what we just wrote, derive each version, substitute.
    let parsed = gc::parse_ledger(&body).expect("fixture ledger parses");
    let shape = gc::derive_version(&parsed.surface[0]);
    let prng = gc::derive_version(&parsed.surface[1]);
    let recorded_shape = spec
        .version_override
        .clone()
        .unwrap_or_else(|| shape.clone());
    let dep_shape = spec.dep_override.clone().unwrap_or(recorded_shape.clone());
    body.replace("AUTO_SHAPE", &recorded_shape)
        .replace("AUTO_PRNG", &prng)
        .replace("DEP_SHAPE", &dep_shape)
        .replace("DEP_PRNG", &prng)
}

// ── 1. known-GOOD ─────────────────────────────────────────────────────────

#[test]
fn good_a_self_consistent_ledger_passes() {
    let r = Repo::new();
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, OK, "{out}");
    assert!(out.contains("surfaces=2"), "{out}");
    assert!(out.contains("pins=5"), "{out}");
    assert!(out.contains("goldens=4"), "{out}");
    assert!(out.contains("couplings=8"), "{out}");
    // PROVENANCE.md is prose, not a frozen artifact.
    assert!(out.contains("discovered=4"), "{out}");
}

/// THE leg that decides whether this gate is usable rather than merely strict:
/// a real semantics change, re-pinned and re-affirmed the way the header says,
/// must go green — otherwise the only way to ship is to delete the gate.
#[test]
fn good_a_legitimately_updated_golden_that_re_affirmed_passes() {
    let r = Repo::new();
    // 1. the surface moves
    r.write(
        "crates/demo/src/lib.rs",
        &DEMO_SRC.replace("pub beta: String,", "pub beta: String,\n    pub gamma: u8,"),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(
        code, VIOLATION,
        "the moved surface must be red first:\n{out}"
    );

    // 2. the artifact is re-frozen
    r.write("goldens/bank_hash.txt", "a_new_bank_hash\n");
    // 3. pins re-read, versions re-derived, digests re-taken, rows re-affirmed
    r.rebuild_ledger();
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, OK, "a fully re-affirmed re-freeze must pass:\n{out}");
}

#[test]
fn good_a_doc_comment_edit_does_not_force_a_re_freeze() {
    let r = Repo::new();
    r.write(
        "crates/demo/src/lib.rs",
        &DEMO_SRC.replace(
            "pub fn build() -> Map {",
            "// a fresh whole-line comment\npub fn build() -> Map {",
        ),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(
        code, OK,
        "a comment must not re-freeze every golden:\n{out}"
    );
}

// ── 2. known-bad: the surface moved ───────────────────────────────────────

#[test]
fn bad_a_field_added_to_a_pinned_struct_is_red_and_names_both_sides() {
    let r = Repo::new();
    r.write(
        "crates/demo/src/lib.rs",
        &DEMO_SRC.replace("pub beta: String,", "pub beta: String,\n    pub gamma: u8,"),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("demo.shape"), "{out}");
    assert!(out.contains("gamma"), "{out}");
    assert!(out.contains("MOVED"), "{out}");
}

#[test]
fn bad_a_changed_builder_key_is_red() {
    let r = Repo::new();
    r.write(
        "crates/demo/src/lib.rs",
        &DEMO_SRC.replace("\"alpha\".into()", "\"alpha_renamed\".into()"),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("alpha_renamed"), "{out}");
    assert!(out.contains("emitted key"), "{out}");
}

#[test]
fn bad_an_edited_region_is_red_even_when_the_shape_is_unchanged() {
    let r = Repo::new();
    r.write(
        "crates/demo/src/lib.rs",
        &DEMO_SRC.replace(
            "m.insert(\"alpha\".into(), 1);",
            "m.insert(\"alpha\".into(), 2);",
        ),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("normalises to sha256"), "{out}");
}

#[test]
fn bad_a_moved_const_is_red_and_quotes_both_values() {
    let r = Repo::new();
    r.write(
        "crates/demo/src/lib.rs",
        &DEMO_SRC.replace("DEMO_VERSION: u32 = 1;", "DEMO_VERSION: u32 = 2;"),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("\"2\""), "{out}");
    assert!(out.contains("\"1\""), "{out}");
}

#[test]
fn bad_a_renamed_symbol_is_red_and_says_the_pin_could_not_be_checked() {
    let r = Repo::new();
    r.write(
        "crates/demo/src/lib.rs",
        &DEMO_SRC.replace("pub struct Shape", "pub struct ShapeV2"),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("no longer declares \"Shape\""), "{out}");
    assert!(out.contains("must never read as checked"), "{out}");
}

#[test]
fn bad_a_deleted_source_file_is_red_not_a_skipped_pin() {
    let r = Repo::new();
    r.remove("crates/demo/src/lib.rs");
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("not a readable file"), "{out}");
}

#[test]
fn bad_a_moved_locked_dependency_is_red() {
    let r = Repo::new();
    r.write(
        "Cargo.lock",
        &DEMO_LOCK.replace("\"rng 0.8.7\"", "\"rng 0.9.5\""),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("0.9.5"), "{out}");
    assert!(out.contains("0.8.7"), "{out}");
}

#[test]
fn bad_an_ambiguous_locked_dependency_is_red_not_a_guess() {
    let r = Repo::new();
    r.write("Cargo.lock", &DEMO_LOCK.replace("\"rng 0.8.7\"", "\"rng\""));
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("not decidable here"), "{out}");
}

// ── 3. known-bad: the chain that makes a re-freeze deliberate ─────────────

#[test]
fn bad_a_repaired_pin_with_an_unbumped_version_is_red() {
    // The author fixed the pin and left the label alone — the exact move a
    // hand-written version number invites.
    let r = Repo::new();
    let stale = gc::derive_version(&gc::parse_ledger(&r.read_ledger()).unwrap().surface[0]);
    r.patch_ledger(
        "expect = [\"alpha\", \"beta\"]",
        "expect = [\"alpha\", \"beta_renamed\"]",
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains(&stale), "{out}");
    assert!(out.contains("derives"), "{out}");
}

#[test]
fn bad_a_golden_that_did_not_re_affirm_a_moved_surface_is_red() {
    let r = Repo::new();
    r.set_ledger(&ledger(&LedgerSpec {
        region: {
            let src = std::fs::read_to_string(r.path("crates/demo/src/lib.rs")).unwrap();
            let (a, b) = gc::find_region(&src, "build").unwrap();
            sha256_hex_bytes(gc::normalise_region(&src[a..b]).as_bytes())
        },
        frozen: r.frozen_digests(),
        dep_override: Some("v000000000000".into()),
        ..LedgerSpec::default()
    }));
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("was not re-affirmed"), "{out}");
    assert!(out.contains("v000000000000"), "{out}");
}

#[test]
fn bad_a_re_frozen_golden_is_red_even_when_every_surface_held() {
    // This is the UPDATE_GOLDENS hole in its purest form: the artifact changed,
    // the code did not, and every other check in the repo agrees with itself.
    let r = Repo::new();
    r.write("goldens/mock40_seed42_all_correct.sha256", "a_new_digest\n");
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("RE-FROZEN"), "{out}");
    assert!(out.contains("mock40_seed42_all_correct.sha256"), "{out}");
}

#[test]
fn bad_a_deleted_golden_is_red_rather_than_an_unchecked_pass() {
    let r = Repo::new();
    r.remove("goldens/bank_hash.txt");
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("is not pinned"), "{out}");
}

#[test]
fn bad_a_path_const_that_drifted_from_the_row_is_red() {
    let r = Repo::new();
    r.write(
        "crates/demo/src/lib.rs",
        &DEMO_SRC.replace(
            "GOLDEN_PATH: &str = \"goldens/bank_hash.txt\"",
            "GOLDEN_PATH: &str = \"goldens/renamed.txt\"",
        ),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("drifted apart"), "{out}");
    assert!(out.contains("goldens/renamed.txt"), "{out}");
}

#[test]
fn bad_an_expired_affirmation_is_red() {
    let r = Repo::new();
    r.patch_ledger("affirmed = \"2026-08-14\"", "affirmed = \"2020-01-01\"");
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("EXPIRED"), "{out}");
}

// ── 4. known-bad: schema, where blank is never permissive ─────────────────

#[test]
fn bad_a_blank_field_is_a_schema_error_never_permission() {
    for (from, to, needle) in [
        (
            "justification = \"a justification",
            "justification = \"\" #",
            "`justification`",
        ),
        ("kind = \"const\"", "kind = \"\"", "`kind`"),
        ("symbol = \"DEMO_VERSION\"", "symbol = \"\"", "`symbol`"),
        ("affirmed = \"2026-08-14\"", "affirmed = \"\"", "`affirmed`"),
        ("version = \"v", "version = \"\" # v", "`version`"),
    ] {
        let r = Repo::new();
        let text = r.read_ledger();
        let at = text.find(from).unwrap_or_else(|| panic!("{from:?} absent"));
        let end = text[at..].find('\n').unwrap() + at;
        r.set_ledger(&format!("{}{to}{}", &text[..at], &text[end..]));
        let (code, out) = r.gate(&["goldens-couplings"]);
        assert_eq!(code, ERROR, "{from:?} -> {out}");
        assert!(out.contains(needle), "{from:?} -> {out}");
    }
}

#[test]
fn bad_an_empty_coupling_list_is_a_schema_error() {
    let r = Repo::new();
    let text = r.read_ledger();
    let at = text.find("depends_on = [").unwrap();
    let end = text[at..].find("]\n").unwrap() + at + 2;
    r.set_ledger(&format!("{}depends_on = []\n{}", &text[..at], &text[end..]));
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("empty `depends_on`"), "{out}");
}

#[test]
fn bad_a_short_justification_is_a_schema_error() {
    let r = Repo::new();
    r.patch_ledger(LONG, "because");
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("disagree with"), "{out}");
}

#[test]
fn bad_a_frozen_field_that_is_not_a_digest_is_a_schema_error() {
    let r = Repo::new();
    let text = r.read_ledger();
    let at = text.find("frozen = \"").unwrap();
    let end = text[at..].find('\n').unwrap() + at;
    r.set_ledger(&format!("{}frozen = \"yes\"{}", &text[..at], &text[end..]));
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("no content pin"), "{out}");
}

#[test]
fn bad_a_dependency_on_a_surface_nobody_declares_is_a_schema_error() {
    let r = Repo::new();
    r.patch_ledger("surface = \"demo.shape\"", "surface = \"demo.ghost\"");
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("no [[surface]] row declares"), "{out}");
}

#[test]
fn bad_a_missing_required_golden_row_is_a_schema_error() {
    let r = Repo::new();
    let text = r.read_ledger();
    let at = text.find("[[golden]]\nid = \"g3\"").unwrap();
    r.set_ledger(&text[..at]);
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("mock40_seed42_all_wrong.sha256"), "{out}");
    assert!(out.contains("not a way to pass"), "{out}");
}

#[test]
fn bad_a_widened_affirmation_window_is_a_schema_error() {
    let r = Repo::new();
    r.patch_ledger("affirmation_days = 365", "affirmation_days = 3650");
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("never widen it"), "{out}");
}

#[test]
fn bad_a_region_pin_carrying_an_expect_list_is_a_schema_error() {
    // A field that is never read must not sit in the file looking load-bearing.
    let r = Repo::new();
    r.patch_ledger(
        "kind = \"region\"\nfile = \"crates/demo/src/lib.rs\"\nsymbol = \"build\"\ndigest",
        "kind = \"region\"\nfile = \"crates/demo/src/lib.rs\"\nsymbol = \"build\"\nexpect = [\"x\"]\ndigest",
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("looking load-bearing"), "{out}");
}

// ── 5. anti-vacuous ───────────────────────────────────────────────────────

#[test]
fn anti_vacuous_an_undeclared_artifact_under_goldens_is_red() {
    let r = Repo::new();
    r.write("goldens/fixtures/mock40_seed7.json", "{\"seed\":7}\n");
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("mock40_seed7.json"), "{out}");
    assert!(out.contains("no [[golden]] row declares it"), "{out}");
}

#[test]
fn anti_vacuous_zero_discovered_goldens_is_an_error_not_a_pass() {
    let r = Repo::new();
    for (p, _) in GOLDEN_FILES {
        r.remove(p);
    }
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("zero files"), "{out}");
}

#[test]
fn anti_vacuous_zero_rows_is_an_error_not_a_pass() {
    let r = Repo::new();
    r.set_ledger("schema_version = 1\n\n[policy]\naffirmation_days = 365\n");
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("zero [[surface]] rows"), "{out}");
    assert!(out.contains("zero [[golden]] rows"), "{out}");
}

#[test]
fn anti_vacuous_a_surface_with_no_pins_is_an_error() {
    let r = Repo::new();
    let text = r.read_ledger();
    let at = text.find("[[surface.pin]]").unwrap();
    let end = text.find("\n[[surface]]\nid = \"demo.prng\"").unwrap();
    r.set_ledger(&format!("{}{}", &text[..at], &text[end..]));
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("empty `pin`"), "{out}");
}

#[test]
fn anti_vacuous_an_emptied_struct_never_compares_clean() {
    let r = Repo::new();
    r.write(
        "crates/demo/src/lib.rs",
        &DEMO_SRC.replace(
            "pub struct Shape {\n    pub alpha: u32,\n    #[serde(default)]\n    pub beta: String,\n}",
            "pub struct Shape {}",
        ),
    );
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("zero fields"), "{out}");
}

#[test]
fn anti_vacuous_a_missing_ledger_is_an_error_not_a_pass() {
    let r = Repo::new();
    r.remove(gc::REGISTRY_PATH);
    let (code, out) = r.gate(&["goldens-couplings"]);
    assert_eq!(code, ERROR, "{out}");
}

#[test]
fn an_unknown_flag_is_usage_not_a_silent_pass() {
    let r = Repo::new();
    let (code, out) = r.gate(&["goldens-couplings", "--quite"]);
    assert_eq!(code, USAGE, "{out}");
}

/// Anti-vacuous, from the other side: if the fixture stopped producing verdicts
/// this suite would pass while checking nothing.
#[test]
fn the_suite_reached_both_verdicts() {
    let r = Repo::new();
    let (green, _) = r.gate(&["goldens-couplings"]);
    r.write("goldens/bank_hash.txt", "moved\n");
    let (red, _) = r.gate(&["goldens-couplings"]);
    assert_eq!(green, OK);
    assert_eq!(red, VIOLATION);
    assert_ne!(
        green, red,
        "the fixture produced one verdict for both trees"
    );
}

// ── 6. the live ledger ────────────────────────────────────────────────────

/// Surface and golden ids named by the gate's findings on the live tree.
fn live_finding_ids(out: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for line in out.lines() {
        for marker in ["[[surface]] ", "[[golden]] "] {
            let Some(rest) = line.split(marker).nth(1) else {
                continue;
            };
            let id = rest
                .split([':', ' '])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !id.is_empty() && !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids.sort();
    ids
}

#[test]
fn the_live_ledger_is_schema_clean_and_its_findings_are_the_named_debts() {
    let root = engine_root();
    let (code, out) = run_gate(&root, &["goldens-couplings"]);
    assert_ne!(
        code, ERROR,
        "the shipped ledger must be schema clean and every pin must be evaluable:\n{out}"
    );
    assert_ne!(code, USAGE, "the gate was invoked wrongly:\n{out}");

    let found = live_finding_ids(&out);
    let mut expected: Vec<String> = KNOWN_DEBTS.iter().map(|(id, _)| (*id).into()).collect();
    expected.sort();
    assert_eq!(
        found, expected,
        "the live ledger's outstanding findings changed.\n\
         If a debt was PAID OFF — the surface re-pinned, the goldens re-frozen and re-affirmed — \
         strike its row from KNOWN_DEBTS in this file.\n\
         If a NEW one appeared, it is a finding about the repo: a surface moved under a frozen \
         artifact. File it, then add it here with a reason.\ngate output:\n{out}"
    );
    if expected.is_empty() {
        assert_eq!(code, OK, "{out}");
    } else {
        assert_eq!(code, VIOLATION, "{out}");
    }
}

/// The findings a shipped ledger may never carry.
///
/// The split matters. A moved surface or a re-frozen artifact is a finding about
/// the TREE, and `KNOWN_DEBTS` above is where those are recorded with a reason.
/// The needles below mean something else: the REGISTRY cannot be evaluated —
/// it points at a file that is gone, a symbol nobody declares, an artifact that
/// cannot be read, a path constant that no longer agrees with it, a lock edge it
/// cannot resolve, or an affirmation nobody has renewed inside the window. None
/// of those is ever an accepted debt, because each one means a row is reporting
/// without checking anything.
#[test]
fn the_live_ledger_has_no_dangling_pin_and_no_unevaluable_row() {
    let root = engine_root();
    let (_, out) = run_gate(&root, &["goldens-couplings"]);
    for needle in [
        "not a readable file",
        "no longer declares",
        "could not be read",
        "EXPIRED",
        "in the future",
        "drifted apart",
        "no [[golden]] row declares it",
        "not decidable here",
    ] {
        assert!(
            !out.contains(needle),
            "the shipped ledger carries a {needle:?} finding, which is never an accepted debt:\n{out}"
        );
    }
}

#[test]
fn every_known_debt_carries_a_reason() {
    for (id, reason) in KNOWN_DEBTS {
        assert!(!id.is_empty(), "a debt with no row id is not tracked");
        assert!(
            reason.len() >= 60,
            "{id}: a debt without a reason a reviewer can disagree with is not recorded, it is hidden"
        );
    }
}

/// Every artifact `scripts/check.sh` pins — by byte-cmp OR by presence — must
/// have a row here. A coupling registry that covers seven of the eight frozen
/// files in the tree would report exactly like one that covers all of them.
/// Deleting the wasm row fails this test; the gate itself is silent because
/// discovery still only sweeps `goldens/` (parent residual, not this bead).
#[test]
fn the_live_ledger_covers_every_artifact_check_sh_pins() {
    let root = engine_root();
    let text = std::fs::read_to_string(root.join(gc::REGISTRY_PATH)).expect("read ledger");
    let l = gc::parse_ledger(&text).expect("live ledger parses");
    for want in [
        "goldens/fixtures/mock40_seed42.json",
        "goldens/mock40_seed42_all_correct.sha256",
        "goldens/mock40_seed42_all_wrong.sha256",
        "goldens/bank_hash.txt",
        "web/data/mock40_seed42.json",
        "web/data/keys_seed42.json",
        "web/data/bank_items_seed42.json",
        "web/assets/wasm/cdcp_wasm.wasm",
    ] {
        assert!(
            l.golden.iter().any(|g| g.file.trim() == want),
            "{want} is pinned by scripts/check.sh and has no [[golden]] row"
        );
    }
    assert!(
        l.golden.len() >= 8 && l.surface.len() >= 8,
        "the live ledger shrank: {} golden(s), {} surface(s)",
        l.golden.len(),
        l.surface.len()
    );
}

/// Planted: flip one nibble of the live wasm `frozen` field and evaluate
/// against the real tree. Same path `goldens-couplings` runs. Complements
/// `bad_a_re_frozen_golden_is_red_even_when_every_surface_held`, which
/// plants a fixture file rather than the eighth golden.
#[test]
fn flipping_the_wasm_frozen_nibble_is_red_on_the_live_ledger() {
    let root = engine_root();
    const WASM: &str = "web/assets/wasm/cdcp_wasm.wasm";
    let text = std::fs::read_to_string(root.join(gc::REGISTRY_PATH)).expect("read ledger");
    let mut l = gc::parse_ledger(&text).expect("live ledger parses");
    let g = l
        .golden
        .iter_mut()
        .find(|g| g.file.trim() == WASM)
        .expect("wasm must be an 8th [[golden]] row");
    let actual = sha256_file(&root.join(WASM)).expect("hash wasm");
    // Pin is the COMMITTED bytes. A dirty rebuild in the worktree is
    // already RE-FROZEN — do not require live==frozen or the test
    // launders an uncommitted wasm.
    if g.frozen.trim() != actual {
        eprintln!(
            "note: live {WASM} sha256={actual} != pinned {}; dirty rebuild, tripwire already hot",
            g.frozen.trim()
        );
    }
    let mut nibbles = g.frozen.clone().into_bytes();
    nibbles[0] = if nibbles[0] == b'0' { b'1' } else { b'0' };
    g.frozen = String::from_utf8(nibbles).expect("hex stays utf-8");

    let read = |rel: &str| std::fs::read_to_string(root.join(rel)).ok();
    let digest = |rel: &str| sha256_file(&root.join(rel)).map_err(|e| e.to_string());
    let w = gc::World {
        read: &read,
        digest: &digest,
        discovered: gc::discover(&root),
    };
    let rep = gc::evaluate(&l, cdcp_gate::date::today(), &w);
    assert!(
        rep.violations
            .iter()
            .any(|v| v.contains("RE-FROZEN") && v.contains(WASM)),
        "nibble flip must turn goldens-couplings RED naming the wasm:\n{:?}",
        rep.violations
    );
}

#[test]
fn the_gate_is_registered_and_listed() {
    let root = engine_root();
    let (code, out) = run_gate(&root, &["list"]);
    assert_eq!(code, OK);
    assert!(out.contains("goldens-couplings"), "{out}");
}
