//! Assertions against THIS repo (not a fixture): the seeded allowlist must stay
//! honest, and the crate must not overclaim anywhere.
//!
//! The overclaim scan distinguishes a **claim** from a **denial**. A header
//! that says "this gate does not prove X" is the required ceiling paragraph;
//! a header that says "this gate proves X" is still RED. A substring match
//! that cannot tell those apart punishes the author for writing the ceiling
//! (bd-overclaim-scan-hits-disclaimers-xidi).

mod support;
use std::path::{Path, PathBuf};

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

/// Same three stems the scan has always used. Inflections that contain a
/// stem (`guarantees`, `guaranteed`) still match; a different word that
/// merely contains the letters (`improves` ⊃ `proves`) does not.
const OVERCLAIM_STEMS: &[&str] = &["guarantee", "proves", "makes impossible"];

/// Measured 2026-08-14 at `export_anki.rs:43` (now `cdcp_anki`). The author
/// stating the ceiling, quoting the banned stems. This is a denial.
const MEASURED_EXPORT_ANKI_DISCLAIMER: &str = concat!(
    "//! It therefore does not \"guarantee\", \"prove\", or \"make impossible\" anything\n",
    "//! about the learner s experience.\n",
);

/// Minimum files the live scan must open. An empty-set green is an ERROR.
const MIN_SCANNED_FILES: usize = 8;

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
/// registry, or hook may promise more than that. A denial of the banned
/// stems is not an overclaim.
#[test]
fn no_overclaiming_language_in_the_shipped_surface() {
    let root = engine_root();
    let files = honesty_scan_files(&root);
    assert!(
        files.len() >= MIN_SCANNED_FILES,
        "scanned {} files — a vacuous honesty scan is an ERROR",
        files.len()
    );

    let mut shipped_regions = 0usize;
    let mut problems = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        // Test modules quote the banned words on purpose; scan the shipped part.
        let shipped = shipped_region(&text);
        if shipped.trim().is_empty() {
            continue;
        }
        shipped_regions += 1;
        for hit in overclaim_claims(&shipped) {
            let rel = f.strip_prefix(&root).unwrap_or(f);
            problems.push(format!(
                "{}:{} {} in {:?}",
                rel.display(),
                hit.line,
                hit.stem,
                hit.snippet
            ));
        }
    }
    assert!(
        shipped_regions >= MIN_SCANNED_FILES,
        "scan read {shipped_regions} shipped region(s) — zero, or only test modules, is an ERROR"
    );
    assert!(
        problems.is_empty(),
        "overclaim(s) in the shipped surface:\n  {}",
        problems.join("\n  ")
    );
}

/// Known-good: the measured export_anki / cdcp_anki ceiling sentence.
#[test]
fn measured_export_anki_disclaimer_is_not_an_overclaim() {
    assert!(
        overclaim_claims(MEASURED_EXPORT_ANKI_DISCLAIMER).is_empty(),
        "measured disclaimer must stay GREEN: {:?}",
        overclaim_claims(MEASURED_EXPORT_ANKI_DISCLAIMER)
    );
}

/// The live successor of export_anki.rs must still write the ceiling in
/// the measured words. Rewording it to dodge the scan is the bug.
#[test]
fn live_cdcp_anki_disclaimer_is_green() {
    let path = engine_root().join("crates/cdcp_anki/src/lib.rs");
    assert!(
        path.is_file(),
        "cdcp_anki is the measured known-good; missing it makes this scan vacuous"
    );
    let text = std::fs::read_to_string(&path).expect("cdcp_anki lib.rs");
    let lower = text.to_ascii_lowercase();
    assert!(
        lower.contains("does not") && lower.contains("guarantee"),
        "cdcp_anki must still disclaim the banned stems; a quieter header is the workaround this bead retires"
    );
    let hits = overclaim_claims(&shipped_region(&text));
    assert!(hits.is_empty(), "cdcp_anki overclaim(s): {hits:?}");
}

#[test]
fn a_header_that_does_not_prove_x_is_not_an_overclaim() {
    let text = "//! this gate does not prove X\n//! this gate does not guarantee X\n";
    assert!(
        overclaim_claims(text).is_empty(),
        "negated stems are denials, not claims: {:?}",
        overclaim_claims(text)
    );
}

#[test]
fn quoted_banned_stems_are_denials() {
    let text = "//! the words are \"guarantee\", \"proves\", and \"makes impossible\".\n";
    assert!(
        overclaim_claims(text).is_empty(),
        "a quoted stem is a mention, not a claim: {:?}",
        overclaim_claims(text)
    );
}

/// Anti-vacuous: a real overclaim is still RED. Without this, a classifier
/// that always returns "denial" would certify the shipped surface green.
#[test]
fn this_gate_guarantees_x_is_an_overclaim() {
    let hits = overclaim_claims("//! this gate guarantees X\n");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].stem, "guarantee");
}

#[test]
fn this_gate_proves_x_is_an_overclaim() {
    let hits = overclaim_claims("//! this gate proves X\n");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].stem, "proves");
}

#[test]
fn this_gate_makes_impossible_x_is_an_overclaim() {
    let hits = overclaim_claims("//! this gate makes impossible any silent skip\n");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].stem, "makes impossible");
}

#[test]
fn a_prior_sentence_not_does_not_launder_a_later_claim() {
    let text = "//! This gate does not scan the bank. It guarantees the allowlist is empty.\n";
    let hits = overclaim_claims(text);
    assert_eq!(hits.len(), 1, "the second sentence is a claim: {hits:?}");
    assert_eq!(hits[0].stem, "guarantee");
}

#[test]
fn improves_is_not_proves() {
    assert!(
        overclaim_claims("//! this port improves the report\n").is_empty(),
        "letter-overlap is not a stem hit"
    );
}

#[test]
fn cannot_decide_clause_is_a_denial() {
    let text = "//! It cannot decide that this proves the notes are complete.\n";
    assert!(
        overclaim_claims(text).is_empty(),
        "the sanctioned cannot-decide paragraph is a denial: {:?}",
        overclaim_claims(text)
    );
}

#[test]
fn wrapped_doc_comment_does_not_still_denies() {
    let text = "//! It therefore does not\n//! \"guarantee\" anything about the learner.\n";
    assert!(
        overclaim_claims(text).is_empty(),
        "negation must survive a //! wrap: {:?}",
        overclaim_claims(text)
    );
}

fn honesty_scan_files(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_rs(&root.join("crates/cdcp_gate/src"), &mut files);
    files.push(root.join("crates/cdcp_gate/build.rs"));
    files.push(root.join("registries/substrate_allowlist.toml"));
    files.push(root.join("hooks/pre-commit"));
    files.push(root.join("crates/cdcp_anki/src/lib.rs"));
    files
}

/// Shipped region: everything before the first `#[cfg(test)]` module.
fn shipped_region(text: &str) -> String {
    text.split("#[cfg(test)]").next().unwrap_or("").to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Overclaim {
    stem: &'static str,
    line: usize,
    snippet: String,
}

/// Occurrences of a banned stem that ASSERT rather than deny.
fn overclaim_claims(text: &str) -> Vec<Overclaim> {
    let lower = text.to_ascii_lowercase();
    let mut hits = Vec::new();
    for stem in OVERCLAIM_STEMS {
        let mut from = 0;
        while let Some(rel) = lower[from..].find(stem) {
            let at = from + rel;
            if stem_starts_here(&lower, at) && !is_denial(&lower, at, stem.len()) {
                let (line, snippet) = line_at(text, at);
                hits.push(Overclaim {
                    stem,
                    line,
                    snippet,
                });
            }
            from = at + stem.len();
        }
    }
    hits
}

/// `improves` is not `proves`. `guarantees` still starts at `guarantee`.
fn stem_starts_here(lower: &str, at: usize) -> bool {
    match lower[..at].chars().next_back() {
        None => true,
        Some(c) => !c.is_ascii_alphabetic(),
    }
}

fn is_denial(lower: &str, at: usize, stem_len: usize) -> bool {
    let after = at + stem_len;
    if wrapped_in_quotes(lower, at, after) {
        return true;
    }
    if following_negates(lower, after) {
        return true;
    }
    clause_denies(&preceding_clause(lower, at))
}

fn wrapped_in_quotes(s: &str, start: usize, end: usize) -> bool {
    let before = s[..start].chars().rev().find(|c| !c.is_whitespace());
    let after = s[end..].chars().find(|c| !c.is_whitespace());
    matches!((before, after), (Some(b), Some(a)) if is_quote(b) && is_quote(a))
}

fn is_quote(c: char) -> bool {
    matches!(c, '"' | '\'' | '`' | '“' | '”' | '‘' | '’')
}

/// "guarantees nothing" / "proves none" is a denial, not a promise.
fn following_negates(lower: &str, after: usize) -> bool {
    let rest = &lower[after..];
    let first = rest
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect::<String>();
    matches!(
        first.split_whitespace().next(),
        Some("nothing") | Some("none") | Some("nobody") | Some("no")
    )
}

fn preceding_clause(lower: &str, at: usize) -> String {
    let start = at.saturating_sub(96);
    let raw = &lower[start..at];
    let stripped = raw
        .replace("//!", " ")
        .replace("///", " ")
        .replace("//", " ");
    let bytes = stripped.as_bytes();
    let mut cut = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if matches!(b, b'.' | b';' | b'!' | b'?') {
            cut = i + 1;
        }
    }
    stripped[cut..].to_string()
}

fn clause_denies(clause: &str) -> bool {
    let toks = tokenize(clause);
    if toks.is_empty() {
        return false;
    }
    // Sanctioned ceiling paragraph: the stem is the thing the gate cannot
    // decide, not a promise the gate makes.
    if toks.windows(2).any(|w| {
        matches!(
            (w[0].as_str(), w[1].as_str()),
            ("cannot", "decide") | ("can't", "decide")
        )
    }) {
        return true;
    }
    const FILLER: &[&str] = &[
        "however",
        "therefore",
        "thus",
        "actually",
        "simply",
        "merely",
        "really",
        "just",
        "also",
        "still",
        "even",
    ];
    let mut end = toks.len();
    while end > 0 && FILLER.contains(&toks[end - 1].as_str()) {
        end -= 1;
    }
    if end == 0 {
        return false;
    }
    matches!(
        toks[end - 1].as_str(),
        "not"
            | "never"
            | "without"
            | "no"
            | "nor"
            | "neither"
            | "cannot"
            | "can't"
            | "don't"
            | "doesn't"
            | "didn't"
            | "won't"
            | "wouldn't"
            | "couldn't"
            | "shouldn't"
            | "mustn't"
    )
}

fn tokenize(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '\'' {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn line_at(text: &str, at: usize) -> (usize, String) {
    let at = at.min(text.len());
    let prefix = &text[..at];
    let line = prefix.bytes().filter(|&b| b == b'\n').count() + 1;
    let start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = text[at..].find('\n').map(|i| at + i).unwrap_or(text.len());
    (line, text[start..end].trim().to_string())
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
