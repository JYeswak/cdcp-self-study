//! End-to-end legs for the `doc-facts` gate (bd-hardening-b-ledgers-gvm.3).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! Four things are held here and nothing more:
//!
//!   1. **Known-bad.** Every way a present-tense claim about code can be stale
//!      or unbacked — a polarity the tree contradicts, a marker naming no row, a
//!      marker with no polarity or an unreadable one, a triggered document with
//!      no marker at all, a probe whose file or symbol is gone, a registry that
//!      shrank below its floor, a dead probe kind, an uncited row, a dead
//!      trigger, an exclusion with no reason, a dead exclusion, a top-level
//!      prefix exclusion, and an exclusion shadowing the un-excludable doc
//!      surface — is planted and asserted to reach a non-zero exit naming the
//!      file, the line where one exists, and both sides of the disagreement.
//!   2. **Known-GOOD.** A true claim passes. Rewriting the sentence around a
//!      marker passes. Adding a paragraph that mentions no trigger passes.
//!      Editing code the probes do not ask about passes. An attack-only suite
//!      ships an over-strict gate, and an over-strict gate gets routed around
//!      instead of fixed — which is a slower death than no gate.
//!   3. **Anti-vacuous.** Zero rows, zero docs, zero marker sites, zero negative
//!      assertions, and a probe that could not be evaluated are each an ERROR
//!      with its own exit code, not a pass. On the live tree the scan is
//!      asserted to have reached a floor of documents, so a gate that lost a
//!      directory cannot report like one that read it.
//!   4. **The live corpus.** `registries/doc-facts.toml` is schema clean, every
//!      probe resolves, and every marker agrees with the tree. Outstanding
//!      findings are enumerated in `KNOWN_DEBTS` with a reason each, so a debt
//!      cannot appear silently and a paid-off debt fails this test until it is
//!      struck from the list.
//!
//! # WHAT THIS SUITE CANNOT DECIDE
//!
//! It cannot decide that a sentence means what its marker asserts — the marker
//! binds a polarity to a line, and no test here reads English. It cannot decide
//! that a probe asks the question its `question` field states. It runs the gate
//! binary, so it says nothing about whether `scripts/check.sh` calls it: BUILT
//! != WIRED is settled by the check.sh step, never by a test.

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");

/// Exit codes, mirrored from `cdcp_gate::exit` so a change there shows up here.
const OK: i32 = 0;
const VIOLATION: i32 = 2;
const USAGE: i32 = 3;
const ERROR: i32 = 4;

/// Findings the LIVE corpus carries, with the reason each is outstanding.
///
/// EMPTY as of 2026-08-14. The emptiness is a real state, not a vacuous one: it
/// asserts that every marker's polarity and every probe's answer agree today,
/// across 228 scanned markdown files and 16 marker sites. Authoring this gate
/// produced one finding and it was fixed rather than recorded — `docs/PLAN-A-TO-Z.md`
/// §C6 still read "`min_modules` is accepted and ignored (`_min_modules`)" while
/// the parameter had begun reaching `sample_item_ids` and a short pool had become
/// a `ModuleShortfall` error. The prose was corrected; the underlying gap (the
/// SELECTED items are still unchecked) is a bead, not a silenced row.
///
/// A NEW finding here is a finding about the repo. File it, then add it with a
/// reason — never narrow a detector to clear it.
const KNOWN_DEBTS: &[(&str, &str)] = &[];

/// The floor on documents the live scan must reach.
///
/// 120, and the number is deliberate. The corpus holds 228 markdown files today
/// and `course-engine/beads_compliance_audit/` alone holds 120 of them; the
/// engine's own `docs/` holds 24. A run reporting fewer than 120 has lost a
/// whole directory — the usual causes being a wrong `--root` and a walk that
/// stopped climbing to the corpus. A doc gate that scanned a tenth of the corpus
/// prints exactly the same `ok:` line as one that scanned all of it, so the
/// count is asserted rather than trusted.
const MIN_LIVE_DOCS: usize = 120;

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

/// A corpus shaped like the real one: a root holding `CHARTER.md`, with the
/// engine in a subdirectory. This also exercises the walk's climb — the gate is
/// pointed at the engine and must scan the corpus.
struct Corpus {
    _dir: tempfile::TempDir,
    corpus: PathBuf,
    engine: PathBuf,
}

/// The probed source. Everything the fixture's probes ask about lives here.
const SRC: &str = r#"
use rand::rngs::StdRng;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item { pub id: String }

pub fn alpha(seed: u64) -> StdRng {
    let _tag = "needle_one";
    StdRng::seed_from_u64(seed)
}

#[test]
fn parity_test_fn() { assert!(true); }
"#;

const CARGO: &str = "[workspace]\nmembers = [\"crates/a\"]\nexclude = [\"fuzz\"]\n";

/// `(fact id, probe toml, trigger word, the polarity the clean tree yields)`.
const FACTS: &[(&str, &str, &str, &str)] = &[
    (
        "f-alpha-has-needle",
        r#"{ kind = "symbol_body_contains", path = "src/lib.rs", symbol = "alpha", needle = "needle_one" }"#,
        "alpha-subject",
        "yes",
    ),
    (
        "f-alpha-has-other",
        r#"{ kind = "symbol_body_contains", path = "src/lib.rs", symbol = "alpha", needle = "needle_absent" }"#,
        "beta-subject",
        "no",
    ),
    (
        "f-parity-test",
        r#"{ kind = "fn_defined", path = "src/lib.rs", symbol = "parity_test_fn" }"#,
        "parity-subject",
        "yes",
    ),
    (
        "f-denies-unknown",
        r##"{ kind = "file_contains", path = "src/lib.rs", needle = "#[serde(deny_unknown_fields)]" }"##,
        "deny-subject",
        "yes",
    ),
    (
        "f-uses-rng",
        r#"{ kind = "file_contains", path = "src/lib.rs", needle = "StdRng::seed_from_u64" }"#,
        "rng-subject",
        "yes",
    ),
    (
        "f-fuzz-member",
        r#"{ kind = "toml_array_contains", path = "Cargo.toml", key = "workspace.members", value = "fuzz" }"#,
        "fuzz-subject",
        "no",
    ),
];

fn registry(excludes: &str) -> String {
    let mut s = String::from("schema_version = 1\n");
    for (id, probe, trigger, _) in FACTS {
        s.push_str(&format!(
            "\n[[fact]]\nid = \"{id}\"\nquestion = \"does this tree really do what the sentence about {trigger} says?\"\nprobe = {probe}\ntrigger = \"{trigger}\"\n"
        ));
    }
    s.push_str(excludes);
    s
}

/// A document asserting every fact at its clean-tree polarity. Twelve sites over
/// two documents clears the compiled-in marker floor.
fn doc(title: &str) -> String {
    let mut s = format!("# {title}\n\n");
    for (id, _, trigger, pol) in FACTS {
        s.push_str(&format!(
            "Prose about {trigger} and what the code does today [[fact:{id}={pol}]].\n\n"
        ));
    }
    s
}

impl Corpus {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let corpus = dir.path().canonicalize().expect("canonicalize");
        let engine = corpus.join("course-engine");
        let c = Corpus {
            _dir: dir,
            corpus,
            engine,
        };
        c.write_corpus("CHARTER.md", "# charter\n");
        c.write("registries/claims.toml", "schema_version = 1\n");
        c.write("src/lib.rs", SRC);
        c.write("Cargo.toml", CARGO);
        c.write("docs/one.md", &doc("one"));
        c.write("docs/two.md", &doc("two"));
        c.write(
            "docs/plain.md",
            "# plain\n\nNo subject words at all here.\n",
        );
        c.set_registry(&registry(""));
        c
    }

    fn write(&self, rel: &str, body: &str) {
        let p = self.engine.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
    }

    fn write_corpus(&self, rel: &str, body: &str) {
        let p = self.corpus.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
    }

    fn remove(&self, rel: &str) {
        std::fs::remove_file(self.engine.join(rel)).unwrap();
    }

    fn set_registry(&self, body: &str) {
        self.write("registries/doc-facts.toml", body);
    }

    fn gate(&self, args: &[&str]) -> (i32, String) {
        run_gate(&self.engine, args)
    }
}

fn exclude(path: &str, reason: &str) -> String {
    format!("\n[[exclude]]\npath = {path:?}\nreason = {reason:?}\n")
}

const LONG_REASON: &str =
    "a reason long enough that a reviewer can disagree with it rather than skim past";

// ── 1. known-GOOD ─────────────────────────────────────────────────────────

#[test]
fn good_a_true_claim_passes() {
    let c = Corpus::new();
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, OK, "{out}");
    assert!(out.contains("sites=12"), "{out}");
    assert!(out.contains("negative_sites=4"), "{out}");
}

#[test]
fn good_rewriting_the_sentence_around_a_marker_is_not_churn() {
    let c = Corpus::new();
    let reworded = doc("one")
        .replace(
            "Prose about",
            "Substantially rewritten narrative discussing",
        )
        .replace("what the code does today", "the behaviour as it stands");
    c.write("docs/one.md", &reworded);
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(
        code, OK,
        "an ordinary prose edit must not force churn:\n{out}"
    );
}

#[test]
fn good_a_new_paragraph_with_no_trigger_is_not_churn() {
    let c = Corpus::new();
    c.write(
        "docs/three.md",
        "# three\n\nA whole new document about scheduling and pedagogy.\nIt mentions no probe subject at all.\n",
    );
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, OK, "{out}");
}

#[test]
fn good_editing_code_the_probes_do_not_ask_about_is_not_churn() {
    let c = Corpus::new();
    c.write(
        "src/lib.rs",
        &format!("{SRC}\npub fn unrelated() {{ let _ = 1; }}\n"),
    );
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, OK, "{out}");
}

#[test]
fn good_an_excluded_file_may_mention_a_subject_without_a_marker() {
    let c = Corpus::new();
    c.write("notes/scratch.md", "alpha-subject discussed informally\n");
    c.set_registry(&registry(&exclude(
        "course-engine/notes/scratch.md",
        LONG_REASON,
    )));
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, OK, "{out}");
    assert!(out.contains("excluded_trigger_hits=1"), "{out}");
}

#[test]
fn good_a_directory_prefix_exclusion_covers_its_tree() {
    let c = Corpus::new();
    c.write("audit/2026-01-01/a.md", "alpha-subject as it was then\n");
    c.write("audit/2026-02-02/b.md", "rng-subject as it was then\n");
    c.set_registry(&registry(&exclude("course-engine/audit/", LONG_REASON)));
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, OK, "{out}");
}

// ── 2. known-bad: the polarity leg ────────────────────────────────────────

#[test]
fn bad_a_polarity_the_tree_contradicts_is_red_naming_file_line_and_both_sides() {
    let c = Corpus::new();
    // The C2 shape exactly: prose says the field is omitted; the code covers it.
    c.write(
        "docs/one.md",
        &doc("one").replace(
            "[[fact:f-alpha-has-needle=yes]]",
            "[[fact:f-alpha-has-needle=no]]",
        ),
    );
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("docs/one.md:3"), "must name file:line:\n{out}");
    assert!(out.contains("prose asserts NO"), "{out}");
    assert!(out.contains("the tree says YES"), "{out}");
    assert!(
        out.contains("symbol_body_contains"),
        "the probe must be printed:\n{out}"
    );
    assert!(out.contains("does not decide which"), "{out}");
}

#[test]
fn bad_the_inverse_polarity_error_is_red_too() {
    let c = Corpus::new();
    // Prose claims a capability the code does not have.
    c.write(
        "docs/one.md",
        &doc("one").replace("[[fact:f-fuzz-member=no]]", "[[fact:f-fuzz-member=yes]]"),
    );
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("prose asserts YES"), "{out}");
    assert!(out.contains("the tree says NO"), "{out}");
}

#[test]
fn bad_a_code_change_turns_the_stale_document_red_by_itself() {
    let c = Corpus::new();
    // Nobody touched the docs. The code stopped seeding from StdRng.
    c.write(
        "src/lib.rs",
        &SRC.replace("StdRng::seed_from_u64", "ChaCha20Rng::seed_from_u64"),
    );
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("f-uses-rng"), "{out}");
    assert!(out.contains("docs/one.md"), "{out}");
    assert!(
        out.contains("docs/two.md"),
        "every stale site is named, not just the first:\n{out}"
    );
}

#[test]
fn bad_a_marker_naming_no_registered_row_is_red() {
    let c = Corpus::new();
    c.write(
        "docs/one.md",
        &doc("one").replace("f-parity-test=yes", "f-invented-row=yes"),
    );
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("not a row in"), "{out}");
}

#[test]
fn bad_a_marker_with_no_polarity_or_an_unreadable_one_is_red() {
    // `sites` counts CHECKED assertions, so a malformed marker does not count as
    // one. The spare document keeps the corpus at the site floor, so the leg
    // under test is the malformed marker and not the floor it would otherwise
    // trip on the way past.
    for (broken, needle) in [
        ("[[fact:f-parity-test]]", "carries no `=yes`"),
        (
            "[[fact:f-parity-test=probably]]",
            "unreadable polarity is never a passing one",
        ),
    ] {
        let c = Corpus::new();
        c.write(
            "docs/one.md",
            &doc("one").replace("[[fact:f-parity-test=yes]]", broken),
        );
        c.write("docs/spare.md", "# spare\n\n[[fact:f-parity-test=yes]]\n");
        let (code, out) = c.gate(&["doc-facts"]);
        assert_eq!(code, VIOLATION, "{out}");
        assert!(out.contains(needle), "{out}");
        assert!(out.contains("docs/one.md"), "{out}");
    }
}

// ── 3. known-bad: the anti-omission leg ───────────────────────────────────

#[test]
fn bad_a_document_that_discusses_a_subject_without_a_marker_is_red() {
    let c = Corpus::new();
    c.write(
        "docs/new.md",
        "# new\n\nToday alpha-subject behaves the way it always has.\n",
    );
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("docs/new.md"), "{out}");
    assert!(out.contains("carries no"), "{out}");
    assert!(out.contains("exclude the file WITH A REASON"), "{out}");
}

#[test]
fn bad_a_marker_cannot_satisfy_its_own_trigger() {
    let c = Corpus::new();
    // The only occurrence of the trigger word is inside a marker for a DIFFERENT
    // row. Stripping markers before the trigger scan is what makes this red.
    c.write(
        "docs/new.md",
        "# new\n\nDiscussion of alpha-subject [[fact:f-parity-test=yes]].\n",
    );
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("f-alpha-has-needle"), "{out}");
}

// ── 4. known-bad: fail-closed when the probe cannot run ───────────────────

#[test]
fn bad_a_probe_whose_file_is_gone_is_an_error_not_a_pass() {
    let c = Corpus::new();
    c.remove("src/lib.rs");
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(
        code, ERROR,
        "a probe that could not run must not report as `no`:\n{out}"
    );
    assert!(out.contains("could not be evaluated"), "{out}");
}

#[test]
fn bad_a_probe_whose_symbol_is_gone_is_an_error_not_a_pass() {
    let c = Corpus::new();
    c.write(
        "src/lib.rs",
        &SRC.replace("pub fn alpha", "pub fn renamed_alpha"),
    );
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("declares no item named"), "{out}");
}

#[test]
fn bad_a_toml_probe_whose_key_is_gone_is_an_error_not_a_pass() {
    let c = Corpus::new();
    c.write("Cargo.toml", "[workspace]\nexclude = [\"fuzz\"]\n");
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("has no key"), "{out}");
}

// ── 5. known-bad: the registry is itself an attack surface ────────────────

#[test]
fn bad_an_exclusion_without_a_reason_is_a_schema_error() {
    let c = Corpus::new();
    c.write("notes/scratch.md", "alpha-subject informally\n");
    c.set_registry(&registry(
        "\n[[exclude]]\npath = \"course-engine/notes/scratch.md\"\nreason = \"\"\n",
    ));
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("SCHEMA ERROR"), "{out}");
}

#[test]
fn bad_a_dead_exclusion_is_an_error() {
    let c = Corpus::new();
    c.set_registry(&registry(&exclude(
        "course-engine/notes/never-existed.md",
        LONG_REASON,
    )));
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("matches no scanned markdown file"), "{out}");
}

#[test]
fn bad_a_top_level_prefix_exclusion_is_a_schema_error() {
    let c = Corpus::new();
    c.set_registry(&registry(&exclude("course-engine/", LONG_REASON)));
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("at least two segments deep"), "{out}");
}

#[test]
fn bad_an_exclusion_shadowing_the_doc_surface_is_an_error() {
    let c = Corpus::new();
    c.set_registry(&registry(&exclude(
        "course-engine/docs/one.md",
        LONG_REASON,
    )));
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(
        code, ERROR,
        "the doc surface must not be silenceable however good the reason:\n{out}"
    );
    assert!(out.contains("un-excludable doc surface"), "{out}");
}

#[test]
fn bad_a_prefix_exclusion_reaching_into_the_doc_surface_is_an_error() {
    let c = Corpus::new();
    c.set_registry(&registry(&exclude("course-engine/docs/", LONG_REASON)));
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("un-excludable doc surface"), "{out}");
}

#[test]
fn bad_a_shrunken_registry_is_a_schema_error() {
    let c = Corpus::new();
    let mut s = String::from("schema_version = 1\n");
    for (id, probe, trigger, _) in FACTS.iter().take(4) {
        s.push_str(&format!(
            "\n[[fact]]\nid = \"{id}\"\nquestion = \"does this tree really do what the sentence about {trigger} says?\"\nprobe = {probe}\ntrigger = \"{trigger}\"\n"
        ));
    }
    c.set_registry(&s);
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("the floor is 6"), "{out}");
}

#[test]
fn bad_a_probe_kind_no_row_exercises_is_a_schema_error() {
    let c = Corpus::new();
    // Six rows, but every one of them a file_contains.
    let mut s = String::from("schema_version = 1\n");
    for i in 0..6 {
        s.push_str(&format!(
            "\n[[fact]]\nid = \"f-row-{i}\"\nquestion = \"does this tree really do what the sentence about subject {i} says?\"\nprobe = {{ kind = \"file_contains\", path = \"src/lib.rs\", needle = \"StdRng\" }}\ntrigger = \"subject-{i}\"\n"
        ));
    }
    c.set_registry(&s);
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("exercised by zero rows"), "{out}");
}

// ── 6. anti-vacuous ───────────────────────────────────────────────────────

#[test]
fn anti_vacuous_zero_rows_is_an_error_not_a_pass() {
    let c = Corpus::new();
    c.set_registry("schema_version = 1\n");
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("zero [[fact]] rows"), "{out}");
}

#[test]
fn anti_vacuous_a_row_nobody_cites_is_an_error() {
    let c = Corpus::new();
    for d in ["docs/one.md", "docs/two.md"] {
        let stripped = doc("x").replace("[[fact:f-parity-test=yes]]", "");
        c.write(d, &stripped);
    }
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("cited by zero marker sites"), "{out}");
}

#[test]
fn anti_vacuous_a_trigger_matching_no_document_is_an_error() {
    let c = Corpus::new();
    let mut s = registry("");
    s = s.replace(
        "trigger = \"parity-subject\"",
        "trigger = \"subject-nobody-writes\"",
    );
    c.set_registry(&s);
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("matches zero of the"), "{out}");
}

#[test]
fn anti_vacuous_too_few_marker_sites_is_an_error() {
    let c = Corpus::new();
    c.write("docs/two.md", "# two\n\nnothing here\n");
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("the floor is 12"), "{out}");
}

#[test]
fn anti_vacuous_a_corpus_with_no_negative_assertion_is_an_error() {
    let c = Corpus::new();
    // Flip both negative rows' probes so every live assertion is `yes`, then
    // flip the prose to match. Nothing is false — and it is still an ERROR,
    // because the polarity leg would never have been exercised downward.
    let mut s = registry("");
    s = s
        .replace("needle = \"needle_absent\"", "needle = \"needle_one\"")
        .replace("value = \"fuzz\"", "value = \"crates/a\"");
    c.set_registry(&s);
    for d in ["docs/one.md", "docs/two.md"] {
        c.write(
            d,
            &doc("x")
                .replace("f-alpha-has-other=no", "f-alpha-has-other=yes")
                .replace("f-fuzz-member=no", "f-fuzz-member=yes"),
        );
    }
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("assert `no`"), "{out}");
}

#[test]
fn anti_vacuous_a_missing_registry_is_an_error_not_a_pass() {
    let c = Corpus::new();
    c.remove("registries/doc-facts.toml");
    let (code, out) = c.gate(&["doc-facts"]);
    assert_eq!(code, ERROR, "{out}");
}

#[test]
fn an_unknown_flag_is_usage_not_a_silent_pass() {
    let c = Corpus::new();
    let (code, out) = c.gate(&["doc-facts", "--quite"]);
    assert_eq!(code, USAGE, "{out}");
}

// ── 7. the live corpus ────────────────────────────────────────────────────

/// The `file:line` prefixes of the gate's findings on the live tree.
fn live_finding_sites(out: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for line in out.lines() {
        let Some(rest) = line.strip_prefix("doc-facts: FAIL: ") else {
            continue;
        };
        if rest.starts_with(char::is_numeric) {
            continue; // the "N violation(s)" tail
        }
        let site = rest.split(&[':', ' '][..]).next().unwrap_or("").to_string();
        if !site.is_empty() && !v.contains(&site) {
            v.push(site);
        }
    }
    v.sort();
    v
}

#[test]
fn the_live_corpus_agrees_with_the_tree_and_its_findings_are_the_named_debts() {
    let root = engine_root();
    let (code, out) = run_gate(&root, &["doc-facts"]);
    assert_ne!(
        code, ERROR,
        "the shipped registry must be schema clean and every probe must be evaluable:\n{out}"
    );
    assert_ne!(code, USAGE, "the gate was invoked wrongly:\n{out}");

    let found = live_finding_sites(&out);
    let mut expected: Vec<String> = KNOWN_DEBTS.iter().map(|(p, _)| (*p).into()).collect();
    expected.sort();
    assert_eq!(
        found, expected,
        "the live corpus's outstanding findings changed.\n\
         If a claim was FIXED, strike its row from KNOWN_DEBTS in this file.\n\
         If a NEW one appeared, it is a finding about the repo — file it, then add it \
         here with a reason. Never narrow a detector to clear it.\ngate output:\n{out}"
    );
    if expected.is_empty() {
        assert_eq!(code, OK, "{out}");
    } else {
        assert_eq!(code, VIOLATION, "{out}");
    }
}

#[test]
fn the_live_scan_reached_the_whole_corpus() {
    let root = engine_root();
    let (code, out) = run_gate(&root, &["doc-facts"]);
    assert_eq!(code, OK, "{out}");
    let docs: usize = out
        .split("docs=")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("the ok: line must report a document count:\n{out}"));
    assert!(
        docs >= MIN_LIVE_DOCS,
        "scanned {docs} markdown files, floor {MIN_LIVE_DOCS} — a run that lost a directory \
         prints the same ok: line as one that read all of it:\n{out}"
    );
}

#[test]
fn every_known_debt_carries_a_reason() {
    for (site, reason) in KNOWN_DEBTS {
        assert!(!site.is_empty(), "a debt with no site is not tracked");
        assert!(
            reason.len() >= 40,
            "{site}: a debt without a reason a reviewer can disagree with is not recorded, it is hidden"
        );
    }
}

#[test]
fn the_gate_is_registered_and_listed() {
    let root = engine_root();
    let (code, out) = run_gate(&root, &["list"]);
    assert_eq!(code, OK);
    assert!(out.contains("doc-facts"), "{out}");
}

/// Anti-vacuous from the other side: if the fixture stopped producing verdicts,
/// every leg above would pass while checking nothing.
#[test]
fn the_suite_reached_both_verdicts() {
    let c = Corpus::new();
    let (green, _) = c.gate(&["doc-facts"]);
    c.write(
        "docs/one.md",
        &doc("one").replace("[[fact:f-uses-rng=yes]]", "[[fact:f-uses-rng=no]]"),
    );
    let (red, _) = c.gate(&["doc-facts"]);
    assert_eq!(green, OK);
    assert_eq!(red, VIOLATION);
    assert_ne!(
        green, red,
        "the fixture produced one verdict for both trees"
    );
}
