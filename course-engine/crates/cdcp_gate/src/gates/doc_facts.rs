//! doc-facts — B3 of milestone B (bd-hardening-b-ledgers-gvm.3).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises exactly one floor: **a present-tense claim about code, made
//! in prose, carries a yes/no answer that the tree recomputes.**
//!
//! Concretely. `registries/doc-facts.toml` registers rows, each holding a
//! QUESTION about this tree and a PROBE that answers it — does a named Rust
//! item's body mention a name; does a file contain a literal; is a named
//! function defined; is a value in a named TOML array. Prose asserts the answer
//! inline at the point a reader reads it, as `[[fact:<id>=yes]]` or
//! `[[fact:<id>=no]]`. The gate walks every markdown file it can find under the
//! corpus root, evaluates each row's probe once, and compares the computed
//! answer to the polarity written at every marker site. A disagreement is RED at
//! that site's `file:line`, with the prose's answer, the tree's answer, and the
//! probe that produced it all printed.
//!
//! Two obligations, deliberately separate:
//!
//! * **resolve** — every marker anywhere must name a registered row and carry a
//!   readable polarity, and that polarity must match the tree. No exclusion
//!   releases this. You cannot silence a lying marker by excluding its file.
//! * **cite** — any non-excluded markdown file whose text contains a row's
//!   `trigger` must carry a marker for that row. This is the anti-omission leg,
//!   and the file set it ranges over is DISCOVERED BY WALKING THE TREE, never
//!   supplied by the registry: a gate scoped by the author it exists to check is
//!   scoped by the wrong party.
//!
//! # WHAT THIS GATE CANNOT DECIDE — read this before quoting it
//!
//! **It cannot decide whether an English sentence is true.** Nothing here parses
//! prose. It decides whether the BOOLEAN an author attached to a sentence still
//! matches the boolean the tree computes. A marker reading `=yes` next to a
//! sentence that says the opposite of yes passes this gate and always will. The
//! author is still writing the claim; what changed is that the claim now has one
//! machine-checkable component, and that component is the one that rotted in
//! every instance we measured.
//!
//! **It cannot decide that the probe asks the question `question` states.**
//! `question` is prose in a registry, and a registry is not a truth source about
//! itself. A row whose probe has drifted from its question is a green light with
//! a citation — worse than no row. That is what the review procedure at the head
//! of the registry is for, and it is a human procedure.
//!
//! **It cannot decide that the marker sits on the sentence it describes.** The
//! marker binds a polarity to a FILE and a LINE, nothing finer. Moving a marker
//! one paragraph away from its sentence is invisible here.
//!
//! **The probes are textual, and are only as sharp as their needles.** A needle
//! inside a comment or a string literal counts as present; `file_contains` says
//! nothing about whether the line is reachable, compiled, or on a `cfg` that is
//! off. `symbol_body_contains` finds an item by keyword-and-name and brace-
//! matches its body with a scanner that understands strings, char literals and
//! comments but not macro expansion, `include!`, or `#[cfg]`-gated duplicates —
//! it takes the FIRST item of that name and says so. `fn_defined` reads a name
//! and never a body, exactly as B1's ledger does, so `fn known_bad_x() {}`
//! resolves like a real one.
//!
//! **It covers the claim classes somebody registered.** A present-tense claim
//! about code with no row is invisible. The `trigger` leg narrows that hole —
//! once a row exists, every file that mentions its subject is obliged — but the
//! hole is real and it is where the next instance of this failure will come
//! from. §"What still passes" in the bead and the report enumerates the live
//! ones rather than implying there are none.
//!
//! **It says nothing about milestone tables or maturity cells.** The CHARTER §5
//! `L2/L3/L5 · YES · wired` class is owned by `capability-maturity` (B1), which
//! quotes the cell verbatim and refuses a published "wired" over an unevidenced
//! level. Two gates arguing over one cell is worse than one gate owning it, so
//! `CHARTER.md` carries no rows here.
//!
//! The floor moves from *a sentence about code is whatever somebody last typed*
//! to *a sentence about code carries a yes/no the build recomputes, at every
//! site, and disagreeing is a build failure that names both sides*. That is the
//! whole of it.
//!
//! # ANTI-VACUOUS (L4)
//!
//! A doc gate that scans nothing reports exactly like one that scanned
//! everything, so every way of scanning nothing is an ERROR and never a pass:
//!
//! * zero registered rows, or fewer than [`MIN_FACTS`];
//! * zero markdown files discovered, or more than [`MAX_MARKDOWN_FILES`] (a
//!   scan that escaped its root is not a thorough scan, it is a wrong one);
//! * zero marker sites, or fewer than [`MIN_MARKER_SITES`];
//! * a row cited by zero marker sites — a row nobody quotes checks nothing;
//! * a row whose `trigger` matches zero scanned files — the subject vanished
//!   from the corpus, or the walk lost the directory it lived in. This is the
//!   floor that needs no magic number: it is derived entirely from the tree;
//! * a compiled-in probe kind exercised by zero rows — dead probe code reports
//!   like exercised probe code;
//! * fewer than [`MIN_NEGATIVE_SITES`] sites asserting `no`;
//! * an exclusion with an empty reason, one matching no scanned file, or one
//!   that shadows a file inside [`NEVER_EXCLUDABLE`];
//! * **a probe that could not be evaluated** — an unreadable path, a symbol that
//!   is not there, a missing TOML key. "Could not verify" is an ERROR, never
//!   `false`. Were it `false`, deleting a file would turn every "omits" claim in
//!   the corpus true.

#![forbid(unsafe_code)]

use crate::registry::{GateCtx, GateError};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const NAME: &str = "doc-facts";
pub const SUMMARY: &str =
    "present-tense prose claims about code carry a yes/no the tree recomputes";

/// Where the rows live, relative to the engine root.
pub const REGISTRY_PATH: &str = "registries/doc-facts.toml";

/// Marker delimiters. Deliberately shaped like claims-lint's `[[claim:...]]` so
/// a reader meets one convention, and deliberately a DIFFERENT keyword so the
/// two never resolve against each other's registry.
pub const MARKER_OPEN: &str = "[[fact:";
pub const MARKER_CLOSE: &str = "]]";

/// The probe kinds compiled in. The registry may not add one; each must be
/// exercised by at least one live row, so a kind cannot rot unnoticed.
pub const PROBE_KINDS: &[&str] = &[
    "file_contains",
    "fn_defined",
    "symbol_body_contains",
    "toml_array_contains",
];

/// Rust item keywords `symbol_body_contains` will look behind.
pub const ITEM_KEYWORDS: &[&str] = &["fn", "struct", "enum", "trait", "impl", "const", "static"];

/// Minimum registered rows.
///
/// SIX, and the number is deliberate: the four compiled-in probe kinds must each
/// be live (checked separately, and that check is the derived one), plus the two
/// `hash_payload` rows that are the motivating instances — the pair of documents
/// that disagreed about bank_hash coverage while the doc gate stayed green. The
/// tree carries seven, so the floor binds with one row of headroom. Lowering it
/// is weakening a detector.
pub const MIN_FACTS: usize = 6;

/// Minimum marker sites across the corpus.
///
/// TWELVE. Seven would be met by one marker per row, and a claim asserted in
/// exactly one place is a claim one edit removes — the cross-document
/// contradiction this gate exists for needs at least two documents to have an
/// opinion. The corpus carries sixteen today; twelve leaves room for one
/// document to be retired without a registry edit, and refuses the collapse of
/// every claim into a single file.
pub const MIN_MARKER_SITES: usize = 12;

/// Minimum sites asserting `no`.
///
/// ONE. Without a live negative the polarity mechanism has only ever been
/// exercised in the direction where adding a capability is safe. The measured
/// failure was the other direction: an "omits" that silently became a "covers".
pub const MIN_NEGATIVE_SITES: usize = 1;

/// Ceiling on discovered markdown files. A walk that escaped the corpus is not a
/// thorough scan, it is a wrong one, and it must not report as thoroughness.
pub const MAX_MARKDOWN_FILES: usize = 5_000;

/// Directory names never descended into. Dot-directories are skipped too (see
/// [`walk_markdown`]): `.beads/` alone holds a hundred history files that quote
/// prose verbatim, and a bead body quoting a doc is not that doc.
pub const SKIP_DIRS: &[&str] = &["target", "dist", "node_modules"];

/// The primary doc surface, which no `[[exclude]]` may shadow however good its
/// reason. Prefix exclusions are one line that silences a directory, and the
/// directory an author would most like silenced is the one the claims live in.
/// Checked against the paths the walk ACTUALLY produced, not against the
/// exclusion's spelling, so a cleverer prefix does not get around it.
pub const NEVER_EXCLUDABLE: &[&str] = &[
    "docs/",
    "course-engine/docs/",
    "README.md",
    "course-engine/README.md",
    "CHARTER.md",
];

/// Depth cap for the walk.
pub const MAX_DEPTH: usize = 12;

/// A `question` shorter than this says nothing a reviewer can check the probe against.
pub const MIN_QUESTION_LEN: usize = 30;

/// An exclusion reason shorter than this is a shrug, not a reason.
pub const MIN_REASON_LEN: usize = 40;

const KNOWN_FLAGS: &[&str] = &["--quiet"];

// ── registry schema ────────────────────────────────────────────────────────

/// Every field is `#[serde(default)]` so a MISSING field arrives empty and is
/// reported as the schema error it is, rather than as a TOML parse failure that
/// names a line and not a rule.
#[derive(Debug, Clone, Deserialize)]
pub struct Registry {
    pub schema_version: u32,
    #[serde(default)]
    pub fact: Vec<Fact>,
    #[serde(default)]
    pub exclude: Vec<Exclude>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Fact {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub probe: Probe,
    /// Substring; any non-excluded markdown file containing it must cite this row.
    #[serde(default)]
    pub trigger: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Probe {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub needle: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: String,
}

/// A file, or — when `path` ends in `/` — a directory prefix.
///
/// The prefix form exists for ONE observed class: dated snapshot directories
/// (`beads_compliance_audit/`), whose files carry a `**Date:**` header, record
/// the tree as it was, and arrive continuously. An exact-path list over a
/// growing directory breaks the build on every new snapshot, and a gate that
/// breaks on unrelated work gets routed around — a slower death than no gate.
/// The price is that one line can silence a directory, which is why
/// [`NEVER_EXCLUDABLE`] exists and why the reason string is mandatory.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Exclude {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub reason: String,
}

impl Exclude {
    pub fn is_prefix(&self) -> bool {
        self.path.trim().ends_with('/')
    }

    pub fn matches(&self, rel: &str) -> bool {
        let p = self.path.trim();
        if self.is_prefix() {
            rel.starts_with(p)
        } else {
            rel == p
        }
    }
}

impl Probe {
    /// One line, printable next to a finding, so a reader can rerun it by hand.
    pub fn describe(&self) -> String {
        match self.kind.as_str() {
            "file_contains" => format!("file_contains {} needle {:?}", self.path, self.needle),
            "fn_defined" => format!("fn_defined {} fn {}", self.path, self.symbol),
            "symbol_body_contains" => format!(
                "symbol_body_contains {} item {} needle {:?}",
                self.path, self.symbol, self.needle
            ),
            "toml_array_contains" => format!(
                "toml_array_contains {} key {} value {:?}",
                self.path, self.key, self.value
            ),
            other => format!("{other} {}", self.path),
        }
    }
}

pub fn parse_registry(text: &str) -> Result<Registry, String> {
    let r: Registry = toml::from_str(text).map_err(|e| format!("parse {REGISTRY_PATH}: {e}"))?;
    if r.schema_version != 1 {
        return Err(format!(
            "{REGISTRY_PATH}: schema_version {} unsupported (expected 1)",
            r.schema_version
        ));
    }
    Ok(r)
}

/// Is this a normalised engine-root-relative path? Same rule `capability-maturity`
/// applies to its evidence refs — a ref that can climb out of the tree, or name
/// an absolute location, is refused rather than resolved.
pub fn is_clean_relative_path(p: &str) -> bool {
    let p = p.trim();
    if p.is_empty() || p.starts_with('/') || p.contains('\\') {
        return false;
    }
    !p.split('/').any(|c| c.is_empty() || c == "." || c == "..")
}

/// A fact id: lowercase ascii, digits and hyphens. No `=`, because `=` splits
/// the marker, and no whitespace, because a marker is one token.
pub fn is_fact_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A Rust identifier, as an item name must be.
pub fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Everything decidable without touching the filesystem. These are SCHEMA
/// errors: a registry that cannot be read as a set of questions exempts nothing.
pub fn schema_errors(r: &Registry) -> Vec<String> {
    let mut v = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut kinds_used: BTreeSet<&str> = BTreeSet::new();

    if r.fact.is_empty() {
        v.push(format!(
            "{REGISTRY_PATH}: zero [[fact]] rows — a registry with nothing in it reports exactly like one whose every claim held. ERROR, not a pass"
        ));
    } else if r.fact.len() < MIN_FACTS {
        v.push(format!(
            "{REGISTRY_PATH}: {} [[fact]] row(s); the floor is {MIN_FACTS} — one live row per compiled-in probe kind plus the two hash_payload rows that are this gate's motivating instances. Dropping below it removes a claim rather than fixing it",
            r.fact.len()
        ));
    }

    for (i, f) in r.fact.iter().enumerate() {
        let id = f.id.trim();
        let where_ = if id.is_empty() {
            format!("[[fact]] #{}", i + 1)
        } else {
            format!("[[fact]] {id}")
        };
        if id.is_empty() {
            v.push(format!("{where_}: missing or empty `id`"));
        } else if !is_fact_id(id) {
            v.push(format!(
                "{where_}: `id` must be lowercase ascii, digits and hyphens — a marker is one token and `=` splits it"
            ));
        } else if !seen.insert(id) {
            v.push(format!("{where_}: duplicate `id`"));
        }

        let q = f.question.trim();
        if q.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `question` — a probe with no question is a boolean nobody can check the probe against"
            ));
        } else if q.len() < MIN_QUESTION_LEN {
            v.push(format!(
                "{where_}: `question` is {} chars; at least {MIN_QUESTION_LEN} are needed to state what the probe is asking",
                q.len()
            ));
        }

        let trig = f.trigger.trim();
        if trig.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `trigger` — without it the row is opt-in, and omission is the default failure this leg exists to catch"
            ));
        } else if trig.contains(MARKER_OPEN) {
            v.push(format!(
                "{where_}: `trigger` contains the marker prefix, so a marker would satisfy its own obligation"
            ));
        }

        let p = &f.probe;
        let kind = p.kind.trim();
        if kind.is_empty() {
            v.push(format!("{where_}: missing or empty `probe.kind`"));
        } else if !PROBE_KINDS.contains(&kind) {
            v.push(format!(
                "{where_}: `probe.kind` {kind:?} is not one of: {}",
                PROBE_KINDS.join(", ")
            ));
        } else {
            kinds_used.insert(PROBE_KINDS.iter().find(|k| **k == kind).copied().unwrap());
        }
        if !is_clean_relative_path(&p.path) {
            v.push(format!(
                "{where_}: `probe.path` {:?} is not a normalised engine-root-relative path",
                p.path.trim()
            ));
        }
        match kind {
            "file_contains" => {
                if p.needle.is_empty() {
                    v.push(format!(
                        "{where_}: `file_contains` needs a non-empty `needle` — every file contains the empty string"
                    ));
                }
            }
            "fn_defined" => {
                if !is_ident(p.symbol.trim()) {
                    v.push(format!(
                        "{where_}: `fn_defined` needs `symbol` to be a function name"
                    ));
                }
            }
            "symbol_body_contains" => {
                if !is_ident(p.symbol.trim()) {
                    v.push(format!(
                        "{where_}: `symbol_body_contains` needs `symbol` to be an item name"
                    ));
                }
                if p.needle.is_empty() {
                    v.push(format!(
                        "{where_}: `symbol_body_contains` needs a non-empty `needle`"
                    ));
                }
            }
            "toml_array_contains" => {
                if p.key.trim().is_empty() {
                    v.push(format!(
                        "{where_}: `toml_array_contains` needs a dotted `key` naming an array"
                    ));
                }
                if p.value.is_empty() {
                    v.push(format!(
                        "{where_}: `toml_array_contains` needs a non-empty `value`"
                    ));
                }
            }
            _ => {}
        }
    }

    if !r.fact.is_empty() {
        for k in PROBE_KINDS {
            if !kinds_used.contains(k) {
                v.push(format!(
                    "{REGISTRY_PATH}: probe kind {k:?} is exercised by zero rows — dead probe code reports exactly like exercised probe code. Register a row or delete the kind"
                ));
            }
        }
    }

    let mut seen_ex: BTreeSet<&str> = BTreeSet::new();
    for (i, e) in r.exclude.iter().enumerate() {
        let path = e.path.trim();
        let where_ = if path.is_empty() {
            format!("[[exclude]] #{}", i + 1)
        } else {
            format!("[[exclude]] {path}")
        };
        if path.is_empty() {
            v.push(format!("{where_}: missing or empty `path`"));
        } else if !is_clean_relative_path(path.trim_end_matches('/')) {
            v.push(format!("{where_}: not a normalised corpus-relative path"));
        } else if e.is_prefix() && !path.trim_end_matches('/').contains('/') {
            v.push(format!(
                "{where_}: a top-level directory prefix would silence a whole tree from one line; name a path at least two segments deep"
            ));
        } else if !seen_ex.insert(path) {
            v.push(format!("{where_}: duplicate exclusion"));
        }
        let reason = e.reason.trim();
        if reason.is_empty() {
            v.push(format!(
                "{where_}: exclusion without a `reason` is a SCHEMA ERROR — an unexplained exclusion is indistinguishable from a claim being hidden"
            ));
        } else if reason.len() < MIN_REASON_LEN {
            v.push(format!(
                "{where_}: `reason` is {} chars; at least {MIN_REASON_LEN} are needed to say what is out of reach and why",
                reason.len()
            ));
        }
    }
    v
}

// ── markers ────────────────────────────────────────────────────────────────

/// One `[[fact:id=yes]]` occurrence: where it is, what it names, what it asserts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    pub line: usize,
    pub id: String,
    pub asserts: bool,
}

/// Every marker in `text`, plus every malformed one as an error string. A marker
/// we cannot read is never silently dropped: an unreadable assertion must not
/// report the way a checked one does.
pub fn scan_markers(text: &str) -> (Vec<Marker>, Vec<String>) {
    let mut out = Vec::new();
    let mut bad = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let mut rest = line;
        while let Some(at) = rest.find(MARKER_OPEN) {
            let after = &rest[at + MARKER_OPEN.len()..];
            let Some(end) = after.find(MARKER_CLOSE) else {
                bad.push(format!(
                    "line {}: {MARKER_OPEN} with no closing {MARKER_CLOSE}",
                    n + 1
                ));
                break;
            };
            let body = &after[..end];
            rest = &after[end + MARKER_CLOSE.len()..];
            match body.split_once('=') {
                None => bad.push(format!(
                    "line {}: marker {MARKER_OPEN}{body}{MARKER_CLOSE} carries no `=yes` / `=no`. A citation without a polarity asserts nothing and cannot go stale",
                    n + 1
                )),
                Some((id, pol)) => {
                    let id = id.trim();
                    let asserts = match pol.trim() {
                        "yes" => Some(true),
                        "no" => Some(false),
                        _ => None,
                    };
                    match (is_fact_id(id), asserts) {
                        (false, _) => bad.push(format!(
                            "line {}: marker id {id:?} is not a fact id",
                            n + 1
                        )),
                        (_, None) => bad.push(format!(
                            "line {}: marker {MARKER_OPEN}{body}{MARKER_CLOSE} has polarity {:?}; only `yes` and `no` are readable, and an unreadable polarity is never a passing one",
                            n + 1,
                            pol.trim()
                        )),
                        (true, Some(a)) => out.push(Marker {
                            line: n + 1,
                            id: id.to_string(),
                            asserts: a,
                        }),
                    }
                }
            }
        }
    }
    (out, bad)
}

/// `text` with every well-formed marker removed, so a marker cannot create the
/// trigger obligation it then satisfies.
pub fn strip_markers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        let Some(at) = rest.find(MARKER_OPEN) else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..at]);
        let after = &rest[at + MARKER_OPEN.len()..];
        match after.find(MARKER_CLOSE) {
            Some(end) => rest = &after[end + MARKER_CLOSE.len()..],
            None => {
                out.push_str(after);
                return out;
            }
        }
    }
}

// ── source probing ─────────────────────────────────────────────────────────

/// Does `text` define a function called `name`? Whole-token match on `fn`, so
/// `defn` and `fn alphabet` do not count.
///
/// Deliberately a local copy of the rule `capability_maturity` applies, not a
/// call into it: six agents are editing this crate concurrently and a read-only
/// coupling to another gate's private helper is a build break waiting for a
/// rename. Twenty lines is the cheaper side of that trade.
pub fn defines_fn(text: &str, name: &str) -> bool {
    item_span(text, "fn", name).is_some()
}

/// Byte offset just past `kw name` when `text` declares that item, else `None`.
pub fn item_span(text: &str, kw: &str, name: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let pat = format!("{kw} ");
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(&pat) {
        let at = from + rel;
        from = at + pat.len();
        if at > 0 {
            let prev = bytes[at - 1] as char;
            if prev.is_ascii_alphanumeric() || prev == '_' {
                continue;
            }
        }
        let tail = &text[at + pat.len()..];
        let lead = tail.len() - tail.trim_start().len();
        let rest = &tail[lead..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        if &rest[..end] == name {
            return Some(at + pat.len() + lead + end);
        }
    }
    None
}

/// The brace-delimited body of the item declared at/after `from`.
///
/// A hand-rolled scanner, and the limits are the point: it understands `//`,
/// `/* */`, `"…"`, `r#"…"#` and `'x'`, and treats a `'` that is not a closed
/// char literal as a lifetime. It does not expand macros, follow `include!`, or
/// know that two `#[cfg]` arms declare the same name twice — it takes the FIRST
/// item of that name. An unbalanced body is an error, never an empty body: a
/// body we could not delimit must not read as a body with nothing in it.
pub fn brace_body(text: &str, from: usize) -> Result<&str, String> {
    let b = text.as_bytes();
    let mut i = from;
    let mut depth = 0usize;
    let mut start = None::<usize>;
    while i < b.len() {
        let c = b[i] as char;
        // comments
        if c == '/' && i + 1 < b.len() {
            match b[i + 1] as char {
                '/' => {
                    while i < b.len() && b[i] != b'\n' {
                        i += 1;
                    }
                    continue;
                }
                '*' => {
                    i += 2;
                    while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                        i += 1;
                    }
                    i = (i + 2).min(b.len());
                    continue;
                }
                _ => {}
            }
        }
        // raw strings
        if c == 'r' && i + 1 < b.len() && (b[i + 1] == b'#' || b[i + 1] == b'"') {
            let mut j = i + 1;
            let mut hashes = 0usize;
            while j < b.len() && b[j] == b'#' {
                hashes += 1;
                j += 1;
            }
            if j < b.len() && b[j] == b'"' {
                let close = format!("\"{}", "#".repeat(hashes));
                match text[j + 1..].find(&close) {
                    Some(rel) => {
                        i = j + 1 + rel + close.len();
                        continue;
                    }
                    None => return Err("unterminated raw string".into()),
                }
            }
        }
        // strings
        if c == '"' {
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == b'"' {
                    break;
                }
                i += 1;
            }
            if i >= b.len() {
                return Err("unterminated string literal".into());
            }
            i += 1;
            continue;
        }
        // char literal vs lifetime
        if c == '\'' {
            let closes = if i + 2 < b.len() && b[i + 1] == b'\\' {
                text[i + 2..].find('\'').map(|r| i + 2 + r)
            } else if i + 2 < b.len() && b[i + 2] == b'\'' {
                Some(i + 2)
            } else {
                None
            };
            match closes {
                Some(end) => {
                    i = end + 1;
                    continue;
                }
                // a lifetime — ordinary code
                None => {
                    i += 1;
                    continue;
                }
            }
        }
        if c == '{' {
            if start.is_none() {
                start = Some(i + 1);
            }
            depth += 1;
        } else if c == '}' {
            if depth == 0 {
                return Err("closing brace before the item body opened".into());
            }
            depth -= 1;
            if depth == 0 {
                let s = start.expect("depth>0 implies a start");
                return Ok(&text[s..i]);
            }
        }
        i += 1;
    }
    Err("item body is not brace-balanced within the file".into())
}

/// Walk a dotted key into a TOML value.
pub fn toml_get<'a>(v: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    let mut cur = v;
    for part in key.split('.') {
        cur = cur.as_table()?.get(part)?;
    }
    Some(cur)
}

// ── evaluation ─────────────────────────────────────────────────────────────

/// Everything the evaluation needs from outside, injected so the rules stay
/// testable without a filesystem.
pub struct World<'a> {
    /// File contents by engine-relative path, or `None` when nothing readable is there.
    pub read: &'a dyn Fn(&str) -> Option<String>,
}

/// Answer a probe against the tree. `Err` is an ERROR, never `Ok(false)`: a
/// probe that could not run must not report the way a probe that ran and said
/// "no" does, or deleting a file would turn every "omits" claim in the corpus
/// true.
pub fn eval_probe(p: &Probe, w: &World<'_>) -> Result<bool, String> {
    let path = p.path.trim();
    match p.kind.trim() {
        "toml_array_contains" => {
            let Some(text) = (w.read)(path) else {
                return Err(format!("{path} is not readable in this tree"));
            };
            let v: toml::Value = toml::from_str(&text)
                .map_err(|e| format!("{path} is not readable as TOML: {e}"))?;
            let key = p.key.trim();
            let Some(node) = toml_get(&v, key) else {
                return Err(format!("{path} has no key {key:?}"));
            };
            let Some(arr) = node.as_array() else {
                return Err(format!("{path} key {key:?} is not an array"));
            };
            Ok(arr.iter().any(|e| {
                e.as_str().is_some_and(|s| {
                    s == p.value || s.trim_end_matches('/').rsplit('/').next() == Some(&p.value)
                })
            }))
        }
        kind => {
            let Some(text) = (w.read)(path) else {
                return Err(format!("{path} is not readable in this tree"));
            };
            match kind {
                "file_contains" => Ok(text.contains(&p.needle)),
                "fn_defined" => Ok(defines_fn(&text, p.symbol.trim())),
                "symbol_body_contains" => {
                    let sym = p.symbol.trim();
                    let mut span = None;
                    for kw in ITEM_KEYWORDS {
                        if let Some(s) = item_span(&text, kw, sym) {
                            span = Some(s);
                            break;
                        }
                    }
                    let Some(s) = span else {
                        return Err(format!(
                            "{path} declares no item named {sym:?} (looked behind: {})",
                            ITEM_KEYWORDS.join(", ")
                        ));
                    };
                    let body = brace_body(&text, s).map_err(|e| format!("{path} {sym}: {e}"))?;
                    Ok(body.contains(&p.needle))
                }
                other => Err(format!("probe kind {other:?} has no evaluator")),
            }
        }
    }
}

/// One markdown file as the gate sees it.
#[derive(Debug, Clone)]
pub struct Doc {
    /// Corpus-relative, forward slashes.
    pub rel: String,
    pub text: String,
}

/// What one pass observed.
#[derive(Debug, Default, Clone)]
pub struct Report {
    pub violations: Vec<String>,
    pub errors: Vec<String>,
    pub sites: usize,
    pub negative_sites: usize,
    pub probes_evaluated: usize,
    pub excluded_hits: usize,
}

/// The verdict pass. Assumes `schema_errors` ran clean.
pub fn evaluate(reg: &Registry, docs: &[Doc], w: &World<'_>) -> Report {
    let mut rep = Report::default();

    if docs.is_empty() {
        rep.errors.push(
            "zero markdown files discovered — a doc gate that scanned nothing reports exactly like one that scanned everything. ERROR, not a pass".to_string()
        );
        return rep;
    }
    if docs.len() > MAX_MARKDOWN_FILES {
        rep.errors.push(format!(
            "{} markdown files discovered, over the ceiling of {MAX_MARKDOWN_FILES} — a walk that escaped the corpus is not a thorough scan",
            docs.len()
        ));
        return rep;
    }

    let by_id: BTreeMap<&str, &Fact> = reg.fact.iter().map(|f| (f.id.trim(), f)).collect();
    let is_excluded = |rel: &str| reg.exclude.iter().any(|e| e.matches(rel));

    // An exclusion is measured against the paths the walk PRODUCED, never
    // against its own spelling: the primary doc surface stays in scope whatever
    // a prefix says.
    for d in docs {
        if !is_excluded(&d.rel) {
            continue;
        }
        if let Some(p) = NEVER_EXCLUDABLE.iter().find(|p| {
            if p.ends_with('/') {
                d.rel.starts_with(**p)
            } else {
                d.rel == **p
            }
        }) {
            rep.errors.push(format!(
                "[[exclude]] shadows {}, which is inside the un-excludable doc surface {p:?} — the directory an author would most like silenced is the one the claims live in. ERROR, not a pass",
                d.rel
            ));
        }
    }

    // Probe once per row; a probe that could not run is an ERROR and its rows'
    // sites are reported as unevaluated rather than as agreeing.
    let mut answers: BTreeMap<&str, bool> = BTreeMap::new();
    for f in &reg.fact {
        rep.probes_evaluated += 1;
        match eval_probe(&f.probe, w) {
            Ok(a) => {
                answers.insert(f.id.trim(), a);
            }
            Err(e) => rep.errors.push(format!(
                "[[fact]] {}: probe could not be evaluated: {e}. `{}` — an unevaluated probe is an ERROR, never a pass",
                f.id.trim(),
                f.probe.describe()
            )),
        }
    }

    // ── obligation 1: every marker resolves and agrees ──────────────────────
    let mut cited: BTreeMap<&str, usize> = by_id.keys().map(|k| (*k, 0usize)).collect();
    for d in docs {
        let (markers, bad) = scan_markers(&d.text);
        for b in bad {
            rep.violations.push(format!("{}:{b}", d.rel));
        }
        for m in markers {
            rep.sites += 1;
            if !m.asserts {
                rep.negative_sites += 1;
            }
            let Some(f) = by_id.get(m.id.as_str()) else {
                rep.violations.push(format!(
                    "{}:{}: marker names {:?}, which is not a row in {REGISTRY_PATH}",
                    d.rel, m.line, m.id
                ));
                continue;
            };
            if let Some(c) = cited.get_mut(m.id.as_str()) {
                *c += 1;
            }
            let Some(actual) = answers.get(m.id.as_str()) else {
                // the probe errored; already reported once, do not double-count
                continue;
            };
            if *actual != m.asserts {
                rep.violations.push(format!(
                    "{}:{}: prose asserts {} for {:?}; the tree says {}. question: {} · probe: {} · Fix the side that is wrong — this gate does not decide which",
                    d.rel,
                    m.line,
                    if m.asserts { "YES" } else { "NO" },
                    m.id,
                    if *actual { "YES" } else { "NO" },
                    f.question.trim(),
                    f.probe.describe(),
                ));
            }
        }
    }

    // ── obligation 2: triggered files cite ──────────────────────────────────
    let mut trigger_hits: BTreeMap<&str, usize> = by_id.keys().map(|k| (*k, 0usize)).collect();
    for d in docs {
        let stripped = strip_markers(&d.text);
        let (markers, _) = scan_markers(&d.text);
        let names: BTreeSet<&str> = markers.iter().map(|m| m.id.as_str()).collect();
        for f in &reg.fact {
            let id = f.id.trim();
            if !stripped.contains(f.trigger.trim()) {
                continue;
            }
            if let Some(h) = trigger_hits.get_mut(id) {
                *h += 1;
            }
            if is_excluded(&d.rel) {
                rep.excluded_hits += 1;
                continue;
            }
            if !names.contains(id) {
                rep.violations.push(format!(
                    "{}: mentions {:?} but carries no {MARKER_OPEN}{id}=yes|no{MARKER_CLOSE}. question: {} · A present-tense claim about code with no polarity cannot go stale. Add the marker, or exclude the file WITH A REASON in {REGISTRY_PATH}",
                    d.rel,
                    f.trigger.trim(),
                    f.question.trim(),
                ));
            }
        }
    }

    // ── anti-vacuous ────────────────────────────────────────────────────────
    for (id, n) in &cited {
        if *n == 0 {
            rep.errors.push(format!(
                "[[fact]] {id}: cited by zero marker sites — a row nobody quotes checks nothing, and a registry of unquoted rows reports exactly like one that is enforced. ERROR, not a pass"
            ));
        }
    }
    for (id, n) in &trigger_hits {
        if *n == 0 {
            let t = by_id.get(id).map(|f| f.trigger.trim()).unwrap_or("");
            rep.errors.push(format!(
                "[[fact]] {id}: trigger {t:?} matches zero of the {} scanned markdown files — either the subject left the corpus or the walk lost the directory it lived in. ERROR, not a pass",
                docs.len()
            ));
        }
    }
    for e in &reg.exclude {
        let p = e.path.trim();
        if !docs.iter().any(|d| e.matches(&d.rel)) {
            rep.errors.push(format!(
                "[[exclude]] {p}: matches no scanned markdown file — a dead exclusion is how a scan quietly loses a directory. Remove it, or fix the path"
            ));
        }
    }
    if rep.sites == 0 {
        rep.errors.push(format!(
            "zero {MARKER_OPEN}…{MARKER_CLOSE} marker sites across {} markdown files — ERROR, not a pass",
            docs.len()
        ));
    } else if rep.sites < MIN_MARKER_SITES {
        rep.errors.push(format!(
            "{} marker site(s); the floor is {MIN_MARKER_SITES}. One marker per row would meet a floor of {}, and a claim asserted in exactly one place is a claim one edit removes",
            rep.sites,
            reg.fact.len()
        ));
    }
    if rep.negative_sites < MIN_NEGATIVE_SITES {
        rep.errors.push(format!(
            "{} site(s) assert `no`; the floor is {MIN_NEGATIVE_SITES}. Without a live negative the polarity leg has only ever been exercised in the safe direction, and the measured failure was the other one — an \"omits\" that silently became a \"covers\"",
            rep.negative_sites
        ));
    }
    rep
}

// ── the walk ───────────────────────────────────────────────────────────────

/// The root the walk ranges over: the corpus root when the engine sits inside
/// one (the parent holding `CHARTER.md`), else the engine root itself.
///
/// Conditional on purpose, and NOT on a configuration value: an unconditional
/// `root.parent()` walk would, in a tempdir fixture, walk the system temp
/// directory. The condition is a property of the tree, so no author declares it.
pub fn corpus_root(engine_root: &Path) -> PathBuf {
    match engine_root.parent() {
        Some(p) if p.join("CHARTER.md").is_file() => p.to_path_buf(),
        _ => engine_root.to_path_buf(),
    }
}

/// Every markdown file under `root`, corpus-relative, sorted. Dot-directories
/// and [`SKIP_DIRS`] are not descended into.
pub fn walk_markdown(root: &Path) -> Result<Vec<Doc>, String> {
    let mut out = Vec::new();
    walk_into(root, root, 0, &mut out)?;
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(out)
}

fn walk_into(root: &Path, dir: &Path, depth: usize, out: &mut Vec<Doc>) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("dir entry under {}: {e}", dir.display()))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let ft = entry
            .file_type()
            .map_err(|e| format!("file type {}: {e}", path.display()))?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            walk_into(root, &path, depth + 1, out)?;
            continue;
        }
        if !name.to_ascii_lowercase().ends_with(".md") {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map_err(|e| format!("relativise {}: {e}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        // Unreadable is an ERROR, never a skip: a file that was never read must
        // not report the way one that agreed does.
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {rel}: {e} — an unreadable doc is never a passing doc"))?;
        out.push(Doc { rel, text });
    }
    Ok(())
}

// ── the gate ───────────────────────────────────────────────────────────────

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(KNOWN_FLAGS)?;
    let quiet = ctx.has_flag("--quiet");
    let root: &Path = &ctx.root;

    let reg_path = root.join(REGISTRY_PATH);
    let text = std::fs::read_to_string(&reg_path)
        .map_err(|e| GateError::error(format!("read {}: {e}", reg_path.display())))?;
    let reg = parse_registry(&text).map_err(GateError::error)?;

    let schema = schema_errors(&reg);
    if !schema.is_empty() {
        return Err(GateError::Error(format!(
            "{} schema error(s) in {REGISTRY_PATH}: {}",
            schema.len(),
            schema.join(" | ")
        )));
    }

    let corpus = corpus_root(root);
    let docs = walk_markdown(&corpus).map_err(GateError::error)?;

    // Probe paths are ENGINE-relative; markdown paths are CORPUS-relative. Both
    // resolutions are explicit so neither can silently borrow the other's root.
    let read = |rel: &str| -> Option<String> {
        let cand = root.join(rel);
        if cand.is_file() {
            return std::fs::read_to_string(cand).ok();
        }
        std::fs::read_to_string(corpus.join(rel)).ok()
    };
    let world = World { read: &read };

    let rep = evaluate(&reg, &docs, &world);

    if !rep.errors.is_empty() {
        return Err(GateError::Error(rep.errors.join(" | ")));
    }
    if !rep.violations.is_empty() {
        return Err(GateError::Violation(rep.violations));
    }

    if !quiet {
        println!(
            "{NAME}: ok: facts={} sites={} negative_sites={} docs={} probes={} excluded_trigger_hits={} corpus={}",
            reg.fact.len(),
            rep.sites,
            rep.negative_sites,
            docs.len(),
            rep.probes_evaluated,
            rep.excluded_hits,
            corpus.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(id: &str, kind: &str, trigger: &str) -> Fact {
        Fact {
            id: id.into(),
            question: "does this tree do the thing the sentence says it does?".into(),
            probe: Probe {
                kind: kind.into(),
                path: "a.rs".into(),
                symbol: "alpha".into(),
                needle: "needle".into(),
                key: "workspace.members".into(),
                value: "fuzz".into(),
            },
            trigger: trigger.into(),
        }
    }

    fn full_registry() -> Registry {
        Registry {
            schema_version: 1,
            fact: vec![
                fact("f-one", "file_contains", "t-one"),
                fact("f-two", "fn_defined", "t-two"),
                fact("f-three", "symbol_body_contains", "t-three"),
                fact("f-four", "toml_array_contains", "t-four"),
                fact("f-five", "file_contains", "t-five"),
                fact("f-six", "file_contains", "t-six"),
            ],
            exclude: vec![],
        }
    }

    #[test]
    fn markers_carry_a_polarity_and_a_bad_one_is_never_dropped() {
        let (ok, bad) = scan_markers("text [[fact:a-b=yes]] more\nand [[fact:c-d=no]]\n");
        assert_eq!(ok.len(), 2);
        assert_eq!(ok[0].line, 1);
        assert!(ok[0].asserts);
        assert_eq!(ok[1].line, 2);
        assert!(!ok[1].asserts);
        assert!(bad.is_empty());

        let (ok, bad) = scan_markers("[[fact:a-b]]\n[[fact:a-b=maybe]]\n[[fact:A_B=yes]]\n");
        assert!(ok.is_empty());
        assert_eq!(bad.len(), 3, "{bad:?}");
        assert!(bad[0].contains("no `=yes`"), "{bad:?}");
        assert!(bad[1].contains("polarity"), "{bad:?}");
    }

    #[test]
    fn a_marker_cannot_satisfy_its_own_trigger() {
        let text = "see [[fact:hash-thing=yes]] and nothing else\n";
        assert!(!strip_markers(text).contains("hash-thing"));
        assert!(strip_markers(text).contains("see  and nothing else"));
    }

    #[test]
    fn item_lookup_needs_a_whole_token() {
        let src = "pub fn alpha(x: u8) -> u8 { x }\nfn alphabet() {}\n";
        assert!(defines_fn(src, "alpha"));
        assert!(defines_fn(src, "alphabet"));
        assert!(!defines_fn(src, "alph"));
        assert!(!defines_fn("defn alpha() {}", "alpha"));
        assert!(!defines_fn("// mentions alpha", "alpha"));
    }

    #[test]
    fn body_extraction_stops_at_the_matching_brace() {
        let src = "fn a() { let x = 1; if x > 0 { needle } }\nfn b() { other }\n";
        let s = item_span(src, "fn", "a").unwrap();
        let body = brace_body(src, s).unwrap();
        assert!(body.contains("needle"));
        assert!(!body.contains("other"));
    }

    #[test]
    fn body_extraction_ignores_braces_in_strings_comments_and_chars() {
        let src = r#"fn a() { let s = "}"; let c = '}'; /* } */ // }
 let t = needle; }
fn b() { other }
"#;
        let s = item_span(src, "fn", "a").unwrap();
        let body = brace_body(src, s).unwrap();
        assert!(body.contains("needle"), "{body:?}");
        assert!(!body.contains("other"), "{body:?}");
    }

    #[test]
    fn a_lifetime_is_not_a_char_literal() {
        let src = "fn a<'x>(v: &'x str) { let q = needle; }\nfn b() { other }\n";
        let s = item_span(src, "fn", "a").unwrap();
        let body = brace_body(src, s).unwrap();
        assert!(body.contains("needle"), "{body:?}");
        assert!(!body.contains("other"), "{body:?}");
    }

    #[test]
    fn an_unbalanced_body_is_an_error_not_an_empty_body() {
        assert!(brace_body("fn a() { oops", 7).is_err());
    }

    #[test]
    fn a_probe_that_cannot_run_is_an_error_never_false() {
        let missing = |_: &str| None;
        let w = World { read: &missing };
        let p = Probe {
            kind: "file_contains".into(),
            path: "gone.rs".into(),
            needle: "x".into(),
            ..Probe::default()
        };
        assert!(eval_probe(&p, &w).is_err());

        let present = |_: &str| Some("fn beta() { }\n".to_string());
        let w = World { read: &present };
        let p = Probe {
            kind: "symbol_body_contains".into(),
            path: "a.rs".into(),
            symbol: "alpha".into(),
            needle: "x".into(),
            ..Probe::default()
        };
        let e = eval_probe(&p, &w).unwrap_err();
        assert!(e.contains("declares no item"), "{e}");
    }

    #[test]
    fn toml_array_probe_reads_the_named_key() {
        let text = "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\nexclude = [\"fuzz\"]\n";
        let read = move |_: &str| Some(text.to_string());
        let w = World { read: &read };
        let mut p = Probe {
            kind: "toml_array_contains".into(),
            path: "Cargo.toml".into(),
            key: "workspace.members".into(),
            value: "fuzz".into(),
            ..Probe::default()
        };
        assert!(!eval_probe(&p, &w).unwrap(), "exclude is not members");
        p.value = "a".into();
        assert!(eval_probe(&p, &w).unwrap(), "last path segment matches");
        p.key = "workspace.nope".into();
        assert!(eval_probe(&p, &w).is_err(), "a missing key is an ERROR");
    }

    #[test]
    fn an_exclusion_without_a_reason_is_a_schema_error() {
        let mut r = full_registry();
        r.exclude.push(Exclude {
            path: "docs/x.md".into(),
            reason: String::new(),
        });
        let errs = schema_errors(&r);
        assert!(errs.iter().any(|e| e.contains("SCHEMA ERROR")), "{errs:?}");
    }

    #[test]
    fn a_dead_probe_kind_is_a_schema_error() {
        let mut r = full_registry();
        r.fact.retain(|f| f.probe.kind != "toml_array_contains");
        r.fact.push(fact("f-seven", "file_contains", "t-seven"));
        let errs = schema_errors(&r);
        assert!(
            errs.iter()
                .any(|e| e.contains("toml_array_contains") && e.contains("zero rows")),
            "{errs:?}"
        );
    }

    #[test]
    fn too_few_rows_is_a_schema_error() {
        let mut r = full_registry();
        r.fact.truncate(4);
        let errs = schema_errors(&r);
        assert!(errs.iter().any(|e| e.contains("the floor is")), "{errs:?}");
    }

    #[test]
    fn a_polarity_that_disagrees_with_the_tree_is_a_violation_naming_both_sides() {
        let mut r = full_registry();
        r.fact[0].trigger = "subject".into();
        let read = |_: &str| Some("subject needle here".to_string());
        let w = World { read: &read };
        let docs = vec![Doc {
            rel: "docs/a.md".into(),
            text: "the subject omits it [[fact:f-one=no]]\n".into(),
        }];
        let rep = evaluate(&r, &docs, &w);
        let joined = rep.violations.join(" | ");
        assert!(joined.contains("docs/a.md:1"), "{joined}");
        assert!(joined.contains("asserts NO"), "{joined}");
        assert!(joined.contains("the tree says YES"), "{joined}");
        assert!(joined.contains("file_contains"), "{joined}");
    }

    #[test]
    fn a_triggered_file_with_no_marker_is_a_violation_and_an_excluded_one_is_not() {
        let mut r = full_registry();
        r.fact[0].trigger = "subject".into();
        let read = |_: &str| Some("subject needle here".to_string());
        let w = World { read: &read };
        let docs = vec![Doc {
            rel: "docs/a.md".into(),
            text: "prose about the subject with no marker\n".into(),
        }];
        let rep = evaluate(&r, &docs, &w);
        assert!(
            rep.violations
                .iter()
                .any(|v| v.contains("carries no") && v.contains("docs/a.md")),
            "{:?}",
            rep.violations
        );

        r.exclude.push(Exclude {
            path: "docs/a.md".into(),
            reason: "a reason long enough to say what is out of reach and why it is".into(),
        });
        let rep = evaluate(&r, &docs, &w);
        assert!(
            !rep.violations.iter().any(|v| v.contains("carries no")),
            "{:?}",
            rep.violations
        );
    }

    #[test]
    fn an_excluded_file_is_still_polarity_checked() {
        let mut r = full_registry();
        r.fact[0].trigger = "subject".into();
        r.exclude.push(Exclude {
            path: "docs/a.md".into(),
            reason: "a reason long enough to say what is out of reach and why it is".into(),
        });
        let read = |_: &str| Some("subject needle here".to_string());
        let w = World { read: &read };
        let docs = vec![Doc {
            rel: "docs/a.md".into(),
            text: "subject [[fact:f-one=no]]\n".into(),
        }];
        let rep = evaluate(&r, &docs, &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("asserts NO")),
            "an exclusion must never release a marker from being checked: {:?}",
            rep.violations
        );
    }

    #[test]
    fn zero_docs_and_a_flood_of_docs_are_both_errors() {
        let r = full_registry();
        let read = |_: &str| Some("x".to_string());
        let w = World { read: &read };
        assert!(evaluate(&r, &[], &w)
            .errors
            .iter()
            .any(|e| e.contains("zero markdown files")));

        let flood: Vec<Doc> = (0..MAX_MARKDOWN_FILES + 1)
            .map(|i| Doc {
                rel: format!("d{i}.md"),
                text: String::new(),
            })
            .collect();
        assert!(evaluate(&r, &flood, &w)
            .errors
            .iter()
            .any(|e| e.contains("over the ceiling")));
    }

    #[test]
    fn an_uncited_row_and_a_dead_trigger_are_errors() {
        let mut r = full_registry();
        r.fact[0].trigger = "subject".into();
        let read = |_: &str| Some("needle".to_string());
        let w = World { read: &read };
        let docs = vec![Doc {
            rel: "docs/a.md".into(),
            text: "nothing here\n".into(),
        }];
        let rep = evaluate(&r, &docs, &w);
        assert!(
            rep.errors
                .iter()
                .any(|e| e.contains("cited by zero marker sites")),
            "{:?}",
            rep.errors
        );
        assert!(
            rep.errors.iter().any(|e| e.contains("matches zero of the")),
            "{:?}",
            rep.errors
        );
    }

    #[test]
    fn a_dead_exclusion_is_an_error() {
        let mut r = full_registry();
        r.exclude.push(Exclude {
            path: "docs/never.md".into(),
            reason: "a reason long enough to say what is out of reach and why it is".into(),
        });
        let read = |_: &str| Some("needle".to_string());
        let w = World { read: &read };
        let docs = vec![Doc {
            rel: "docs/a.md".into(),
            text: String::new(),
        }];
        assert!(evaluate(&r, &docs, &w)
            .errors
            .iter()
            .any(|e| e.contains("matches no scanned markdown file")));
    }

    #[test]
    fn a_corpus_with_no_negative_assertion_is_an_error() {
        let mut r = full_registry();
        for (i, f) in r.fact.iter_mut().enumerate() {
            f.trigger = format!("subject{i}");
        }
        let read = |_: &str| Some("needle".to_string());
        let w = World { read: &read };
        let mut text = String::new();
        for (i, f) in r.fact.iter().enumerate() {
            for _ in 0..3 {
                text.push_str(&format!("subject{i} [[fact:{}=yes]]\n", f.id));
            }
        }
        let docs = vec![Doc {
            rel: "docs/a.md".into(),
            text,
        }];
        let rep = evaluate(&r, &docs, &w);
        assert!(
            rep.errors.iter().any(|e| e.contains("assert `no`")),
            "{:?}",
            rep.errors
        );
    }

    #[test]
    fn corpus_root_climbs_only_when_the_parent_looks_like_one() {
        let td = tempfile::tempdir().unwrap();
        let corpus = td.path().join("corpus");
        let engine = corpus.join("course-engine");
        std::fs::create_dir_all(&engine).unwrap();
        assert_eq!(corpus_root(&engine), engine, "no CHARTER.md yet");
        std::fs::write(corpus.join("CHARTER.md"), "# c\n").unwrap();
        assert_eq!(corpus_root(&engine), corpus);
    }

    #[test]
    fn the_walk_skips_dot_dirs_and_build_dirs() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        for rel in [
            "a.md",
            "docs/b.md",
            ".beads/c.md",
            "target/d.md",
            "node_modules/e.md",
            "docs/f.txt",
        ] {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, "x").unwrap();
        }
        let docs = walk_markdown(root).unwrap();
        let names: Vec<&str> = docs.iter().map(|d| d.rel.as_str()).collect();
        assert_eq!(names, vec!["a.md", "docs/b.md"], "{names:?}");
    }
}
