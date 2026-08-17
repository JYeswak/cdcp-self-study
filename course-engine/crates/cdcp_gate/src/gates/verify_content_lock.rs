//! verify-content-lock — Rust port of `scripts/verify_content_lock.py`
//! (bd-substrate-rust-migration-jhd.8).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises one floor: **the content under the locked roots is exactly
//! the content `content.lock` pins, in both directions.** Concretely it goes RED
//! when any of these is true —
//!
//!   1. `content.lock` is absent (nothing is pinned at all);
//!   2. `schema_version` is not `1` (the pin was written by a shape this reader
//!      does not understand);
//!   3. `bank_hash` is absent or empty;
//!   4. the live bank digest differs from the pinned `bank_hash`, cannot be
//!      obtained, or the bank-hash subprocess exits 0 saying nothing / hangs
//!      past its budget (bd-hw3 — those two used to escape as a traceback);
//!   5. `[knowledge]` or `[modules]` is empty — a lock that pins nothing must
//!      not report like a lock whose pins all held (L4 anti-vacuous);
//!   6. a pinned path no longer resolves to a file;
//!   7. a pinned path's sha256 differs from the recorded digest;
//!   8. a file exists under a locked root and NO row pins it. This is the
//!      tree-side walk (bd-z3v). Without it the gate's coverage was defined by
//!      the artifact the gate is checking, so deleting a row deleted the check
//!      and the removal was indistinguishable from a pass;
//!   9. a locked root is missing, or matches zero files — a root that checked
//!      nothing must not report like a root whose every file matched.
//!
//! The locked roots are `knowledge/*.toml`, `web/content/modules/*.md`, and
//! `../modules/*.md`: exactly the roots `cdcp content-lock` writes
//! rows from. They are named on stdout on the GREEN path with their file counts,
//! so a reader of a green verdict is told what the verdict does not mean.
//!
//! # WHAT THIS GATE CANNOT DECIDE
//!
//! It says nothing about any path outside those three globs. `bank/items` is
//! covered only transitively through `bank_hash`; the `knowledge/`
//! subdirectories (`corpus`, `citations`, `graph`, `schema`), non-`.md` files
//! under the module roots, `scripts/`, `registries/`, and the rest of `web/` are
//! not walked at all. It cannot decide that a pinned digest is the *right*
//! digest: a lock regenerated over corrupted content is internally consistent
//! and reports green. It cannot decide that the content is correct, current,
//! accurate, or well written — only that it is byte-for-byte what someone
//! pinned. It does not look inside subdirectories of a locked root, so it cannot
//! see a file smuggled one level down. And when it falls back to
//! `goldens/bank_hash.txt` for the live bank digest (see `live_bank_hash`) it is
//! comparing a pin against another pin, which is weaker than comparing a pin
//! against a freshly computed digest — that fallback is inherited from the
//! oracle, not introduced here.
//!
//! The floor moves from *every row that exists in the lock still matches* to
//! *the set of pinned rows and the set of files under the locked roots are the
//! same set, and every digest in it holds*. That is the whole claim.
//!
//! # EXTRACT-THEN-DELETE (bd-retire-oracle-on-behaviour-change-gna0)
//!
//! `scripts/verify_content_lock.py` and `tests/diff_verify_content_lock.rs`
//! are deleted. check.sh never invoked the .py. The gate is this module;
//! known-bad is the tests below plus `CDCP_CONTENT_LOCK_SELFTEST=1`. A
//! behaviour change lands here only.
//!
//! The port still:
//!
//!   * writes its failure report to **stderr and exits 1** instead of routing
//!     through `GateError` — the dispatcher's `report()` uses a different prefix
//!     and maps to exit 2/4, which the oracle never produces. Same knowing,
//!     single-file deviation `verify_orphans` records; `crate::exit`'s
//!     VIOLATION code is deliberately not used on the RED path here.
//!   * carries hand-written emulations of `repr()`, `str()`, truthiness,
//!     `str.strip()`, `str.splitlines()`, character slicing, and sha256, rather
//!     than the idiomatic Rust nearest-neighbour.
//!   * spells the truncation marker as U+2026 HORIZONTAL ELLIPSIS (bytes
//!     `e2 80 a6`), which is what the oracle's source actually contains — not
//!     three ASCII dots.
//!
//! Ordering is not a fragility here: the oracle iterates `sorted(mapping.items())`,
//! so both sides walk the rows in Python code-point order, which for `String`
//! keys is identical to Rust's `Ord` (UTF-8 byte order). Nothing in this gate
//! reads a directory, so `read_dir` order never enters the picture.
//!
//! ## Behaviours of the oracle this port does NOT reproduce byte for byte
//!
//! Each of these makes CPython raise an uncaught exception and print a traceback
//! (exit 1). A traceback embeds interpreter version, file paths, and line
//! numbers; it is not reproducible from Rust, so this port reports an ERROR
//! (exit 4) instead — never a pass — and the divergence is recorded here rather
//! than hidden:
//!
//!   * `content.lock` is not valid TOML (`tomllib.TOMLDecodeError`);
//!   * `content.lock` or `goldens/bank_hash.txt` is not valid UTF-8
//!     (`UnicodeDecodeError`);
//!   * a pinned file exists but cannot be read (`PermissionError`).
//!
//! Two of the former entries on that list are gone as of bd-hw3: an empty
//! bank-hash output and a bank-hash timeout are now typed errors on BOTH sides
//! (`bank-hash exited 0 with no output …`, `bank-hash timed out …`), so the two
//! implementations agree byte for byte on them instead of one crashing.
//!
//! Three narrower gaps remain: `repr()` of a float is emulated without CPython's
//! exponent thresholds (`1e17` prints as `100000000000000000` here);
//! `repr()`/`str()` of a TOML datetime prints its TOML spelling rather than
//! `datetime.date(...)`; and `CDCP_BANK_HASH_TIMEOUT_S` is parsed with Rust's
//! `f64::from_str` rather than CPython's `float()`, which differ on exotic
//! spellings (`1_000`) and on a non-UTF-8 environment value. The first two are
//! reachable only from a hand-edited `schema_version` or digest; the third only
//! from a deliberately malformed env var. A locked-root filename that is not
//! valid UTF-8 is RED on both sides but renders differently (CPython
//! surrogateescape vs U+FFFD), which is a divergence in the *text* of a RED, not
//! in the verdict.
//!
//! Finally, the dispatcher rejects unknown arguments with USAGE (exit 3) while
//! the oracle ignores `sys.argv` entirely. `check.sh` passes no arguments, so
//! the differential surface is unaffected; the crate-wide rule that a typo must
//! not read as a pass wins here.

use crate::registry::{GateCtx, GateError};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use toml::Value;

pub const NAME: &str = "verify-content-lock";
pub const SUMMARY: &str =
    "L7 content.lock: pinned bank_hash and knowledge/module digests still match the tree";

/// Engine-root-relative lock file. Mirrors the oracle's `LOCK_PATH`.
pub const LOCK_REL: &str = "content.lock";
/// Fallback source for the live bank digest. Mirrors `GOLDEN_BANK_HASH`.
pub const GOLDEN_REL: &str = "goldens/bank_hash.txt";
/// The bank directory handed to `cdcp bank-hash`.
pub const BANK_ARG: &str = "bank/items";
/// U+2026, the marker the oracle's source really contains.
const ELLIPSIS: char = '\u{2026}';
/// Env switch for the oracle's optional mutate-selftest.
pub const SELFTEST_ENV: &str = "CDCP_CONTENT_LOCK_SELFTEST";
/// The bank-hash subprocess budget, in seconds. Mirrors the oracle's `timeout=`.
pub const BANK_HASH_TIMEOUT_S: f64 = 300.0;
/// Env var that may SHORTEN that budget. It cannot turn a RED into a pass; an
/// unparseable or non-positive value is an error, never a silent default.
pub const TIMEOUT_ENV: &str = "CDCP_BANK_HASH_TIMEOUT_S";

/// One root the tree-side walk enumerates.
pub struct LockedRoot {
    /// Which lock section a file found here must be pinned in.
    pub section: &'static str,
    /// How the root is spelled in the receipt and in RED messages.
    pub label: &'static str,
    /// `true` = under the engine root; `false` = under the engine root's parent.
    pub under_engine: bool,
    /// Slash-separated directory, relative to whichever base `under_engine` picks.
    pub rel_dir: &'static str,
    /// Filename suffix the glob `*<suffix>` accepts.
    pub suffix: &'static str,
}

/// EXACTLY the roots `cdcp content-lock` enumerates when it writes the
/// lock. Keeping the two lists identical is what makes "present in the tree but
/// absent from the lock" decidable rather than an opinion: a freshly regenerated
/// lock is green here by construction, so any divergence is a real unpinned file.
pub const LOCKED_ROOTS: [LockedRoot; 3] = [
    LockedRoot {
        section: "knowledge",
        label: "knowledge/*.toml",
        under_engine: true,
        rel_dir: "knowledge",
        suffix: ".toml",
    },
    LockedRoot {
        section: "modules",
        label: "web/content/modules/*.md",
        under_engine: true,
        rel_dir: "web/content/modules",
        suffix: ".md",
    },
    LockedRoot {
        section: "modules",
        label: "../modules/*.md",
        under_engine: false,
        rel_dir: "modules",
        suffix: ".md",
    },
];

/// Everything one invocation writes plus the status it leaves with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

/// The result of `verify()`: either the oracle's error list (possibly empty,
/// which is GREEN), or a state the oracle reaches only by raising.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Errors(Vec<String>),
    Undecidable(String),
}

// ── entry point ────────────────────────────────────────────────────────────

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&[])?;

    // The oracle resolves its own location (`Path(__file__).resolve()`), so its
    // ROOT is symlink-free and the `missing content.lock at <path>` message
    // prints a real path. Do the same to the engine root.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());

    let selftest = std::env::var(SELFTEST_ENV).unwrap_or_default() == "1";
    let outcome = if selftest {
        selftest_mutate(&root)
    } else {
        evaluate(&root)
    };

    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();
    eprint!("{}", outcome.stderr);
    let _ = std::io::stderr().flush();

    if outcome.code != 0 {
        // See the module header: the oracle exits 1 with this exact stderr, and
        // this port's acceptance bar is byte-identical output. Routing through
        // `GateError` would rewrite the text and change the code.
        std::process::exit(outcome.code);
    }
    Ok(())
}

/// The oracle's `main()` minus the selftest branch.
pub fn evaluate(root: &Path) -> Outcome {
    let lock = root.join(LOCK_REL);

    let errors = match verify(root, &lock) {
        Verdict::Undecidable(m) => return undecidable(m),
        Verdict::Errors(e) => e,
    };

    if !errors.is_empty() {
        let mut s = String::from("verify_content_lock: FAIL\n");
        for e in &errors {
            s.push_str(&format!("  - {e}\n"));
        }
        s.push_str("Regenerate (human review): cdcp content-lock\n");
        return Outcome {
            stdout: String::new(),
            stderr: s,
            code: 1,
        };
    }

    // Count pins for receipt — the oracle re-reads and re-parses the lock here.
    let data = match read_table(&lock) {
        Ok(d) => d,
        Err(m) => return undecidable(m),
    };
    let nk = py_len(or_empty(data.get("knowledge")));
    let nm = py_len(or_empty(data.get("modules")));
    let bh_full = match data.get("bank_hash") {
        Some(v) if py_truthy(v) => py_str(v),
        _ => String::new(),
    };
    let bh = py_slice(&bh_full, 12);
    let counts: Vec<String> = root_counts(root)
        .into_iter()
        .map(|(label, n)| format!("{label}={n}"))
        .collect();
    let mut stdout =
        format!("verify_content_lock: PASS bank_hash={bh}{ELLIPSIS} knowledge={nk} modules={nm}\n");
    // A green verdict must say what it ranges over, or the reader supplies their
    // own optimistic scope. These are the roots that were walked file by file.
    stdout.push_str(&format!(
        "verify_content_lock: covered roots (every file found under these is pinned and matched): {}\n",
        counts.join(" ")
    ));
    stdout.push_str(
        "verify_content_lock: NOT covered: anything outside those roots \u{2014} bank/items only \
         through bank_hash, the knowledge/ subdirectories (corpus, citations, graph, schema), \
         non-.md files under the module roots, scripts/, registries/, and the rest of web/\n",
    );
    Outcome {
        stdout,
        stderr: String::new(),
        code: 0,
    }
}

fn undecidable(m: String) -> Outcome {
    Outcome {
        stdout: String::new(),
        stderr: format!("verify_content_lock: ERROR: {m}\n"),
        code: crate::exit::ERROR as i32,
    }
}

// ── the oracle's verify() ──────────────────────────────────────────────────

/// Port of `verify(lock_path)`. Returns the error list in the oracle's order.
pub fn verify(root: &Path, lock_path: &Path) -> Verdict {
    let mut errors: Vec<String> = Vec::new();

    if !lock_path.is_file() {
        return Verdict::Errors(vec![format!(
            "missing content.lock at {}",
            lock_path.display()
        )]);
    }

    let data = match read_table(lock_path) {
        Ok(d) => d,
        Err(m) => return Verdict::Undecidable(m),
    };

    let schema = data.get("schema_version");
    if !py_eq_one(schema) {
        errors.push(format!(
            "unsupported schema_version={} (want 1)",
            py_repr_opt(schema)
        ));
    }

    // `if not pinned_bank or not isinstance(pinned_bank, str)`
    let pinned_bank: Option<&str> = match data.get("bank_hash") {
        Some(Value::String(s)) if !s.is_empty() => Some(s.as_str()),
        _ => None,
    };
    match pinned_bank {
        None => errors.push("content.lock missing bank_hash".to_string()),
        Some(pinned) => match live_bank_hash(root) {
            Ok(live) => {
                if live != pinned {
                    errors.push(format!(
                        "bank_hash drift: lock={}{ELLIPSIS} live={}{ELLIPSIS}",
                        py_slice(pinned, 16),
                        py_slice(&live, 16)
                    ));
                }
            }
            Err(msg) => errors.push(msg),
        },
    }

    // `knowledge = data.get("knowledge") or {}` — None here stands for `{}`.
    let knowledge = or_empty(data.get("knowledge"));
    let modules = or_empty(data.get("modules"));
    if knowledge.is_none() {
        errors.push("content.lock [knowledge] empty (vacuous ERROR)".to_string());
    }
    if modules.is_none() {
        errors.push("content.lock [modules] empty (vacuous ERROR)".to_string());
    }

    // The rows each section pins, for the tree-side walk below. A section that
    // is absent, empty, or not a table pins NOTHING, which is what makes every
    // file under its root unpinned — the fail-closed direction.
    let mut pinned_knowledge: BTreeSet<String> = BTreeSet::new();
    let mut pinned_modules: BTreeSet<String> = BTreeSet::new();

    for (section, mapping) in [("knowledge", knowledge), ("modules", modules)] {
        let Some(value) = mapping else {
            // The substituted `{}` IS a dict; the oracle iterates it and finds
            // nothing. Not an error, not a skip of the type check.
            continue;
        };
        let Value::Table(table) = value else {
            errors.push(format!("[{section}] must be a table of path = hash"));
            continue;
        };
        let sink = if section == "knowledge" {
            &mut pinned_knowledge
        } else {
            &mut pinned_modules
        };
        sink.extend(table.keys().cloned());
        // `sorted(mapping.items())`: Python orders str keys by code point, which
        // for UTF-8 is byte order, which is Rust's `String: Ord`.
        let mut keys: Vec<&String> = table.keys().collect();
        keys.sort();
        for rel in keys {
            let expected = &table[rel];
            let path = resolve_pinned(root, rel);
            if !path.is_file() {
                errors.push(format!("[{section}] missing file: {rel}"));
                continue;
            }
            let actual = match sha256_file(&path) {
                Ok(h) => h,
                Err(e) => {
                    return Verdict::Undecidable(format!("cannot read {}: {e}", path.display()))
                }
            };
            let matches = matches!(expected, Value::String(s) if *s == actual);
            if !matches {
                errors.push(format!(
                    "[{section}] hash mismatch: {rel} lock={}{ELLIPSIS} live={}{ELLIPSIS}",
                    py_slice(&py_str(expected), 12),
                    py_slice(&actual, 12)
                ));
            }
        }
    }

    errors.extend(tree_side_errors(root, &pinned_knowledge, &pinned_modules));

    Verdict::Errors(errors)
}

// ── the tree-side walk (bd-z3v) ────────────────────────────────────────────

/// Absolute directory for a locked root.
fn root_dir(root: &Path, r: &LockedRoot) -> PathBuf {
    let mut p = if r.under_engine {
        root.to_path_buf()
    } else {
        root.parent().unwrap_or(root).to_path_buf()
    };
    for seg in r.rel_dir.split('/') {
        p.push(seg);
    }
    p
}

/// The spelling a `content.lock` row would use for a discovered file. Inverse of
/// [`resolve_pinned`]: engine-relative when the file is under the engine root,
/// else parent-relative, else absolute — and an absolute key matches no row, so
/// a symlink pointing out of the tree goes RED rather than quietly passing.
fn lock_key(root: &Path, path: &Path) -> String {
    let p = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let engine = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    if let Ok(rel) = p.strip_prefix(&engine) {
        return rel.to_string_lossy().into_owned();
    }
    let parent = engine.parent().unwrap_or(engine.as_path());
    if let Ok(rel) = p.strip_prefix(parent) {
        return rel.to_string_lossy().into_owned();
    }
    p.to_string_lossy().into_owned()
}

/// Files matching `*<suffix>` DIRECTLY under `dir` — the oracle uses
/// `Path.glob`, which does not recurse and does not skip dotfiles. `is_file()`
/// follows symlinks on both sides, so a symlink to a regular file is walked.
fn discover(dir: &Path, suffix: &str) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = Vec::new();
    for e in rd.flatten() {
        let p = e.path();
        let Some(name) = p.file_name() else {
            continue;
        };
        // Lossy on purpose: an undecodable name still has to be REPORTED, not
        // skipped. Skipping it would be the same vacuity this walk exists to end.
        if !name.to_string_lossy().ends_with(suffix) {
            continue;
        }
        // ABSENT-OK: walk type-filter; a non-file suffix match is not a locked
        // artifact (a locked root that matches zero files is ERROR above).
        if !p.is_file() {
            continue;
        }
        out.push(p);
    }
    out
}

/// `(label, file count)` per locked root, for the GREEN receipt.
fn root_counts(root: &Path) -> Vec<(&'static str, usize)> {
    LOCKED_ROOTS
        .iter()
        .map(|r| (r.label, discover(&root_dir(root, r), r.suffix).len()))
        .collect()
}

/// The walk the lock cannot narrow: every file under a locked root must be
/// pinned by a row, and a root that matched nothing is an ERROR rather than a
/// quiet pass.
fn tree_side_errors(
    root: &Path,
    pinned_knowledge: &BTreeSet<String>,
    pinned_modules: &BTreeSet<String>,
) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    for r in LOCKED_ROOTS.iter() {
        let dir = root_dir(root, r);
        let section = r.section;
        let label = r.label;
        if !dir.is_dir() {
            errors.push(format!(
                "[{section}] locked root is not a directory: {label} \
                 (nothing was checked there \u{2014} vacuous ERROR)"
            ));
            continue;
        }
        let mut found: Vec<String> = discover(&dir, r.suffix)
            .iter()
            .map(|p| lock_key(root, p))
            .collect();
        found.sort();
        if found.is_empty() {
            errors.push(format!(
                "[{section}] locked root matched zero files: {label} \
                 (nothing was checked there \u{2014} vacuous ERROR)"
            ));
            continue;
        }
        let pinned = if section == "knowledge" {
            pinned_knowledge
        } else {
            pinned_modules
        };
        for key in found {
            if !pinned.contains(&key) {
                errors.push(format!(
                    "[{section}] in the tree but not pinned in content.lock: {key}"
                ));
            }
        }
    }
    errors
}

/// Port of `resolve_pinned`: engine root first, then the parent corpus.
pub fn resolve_pinned(root: &Path, rel: &str) -> PathBuf {
    let p = Path::new(rel);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let cand = root.join(p);
    // ABSENT-OK: first-candidate miss falls through to the parent corpus;
    // this is a search, not a verdict.
    if cand.exists() {
        return cand.canonicalize().unwrap_or(cand);
    }
    let parent = root.parent().unwrap_or(root);
    let cand2 = parent.join(p);
    cand2.canonicalize().unwrap_or(cand2)
}

// ── live bank hash ─────────────────────────────────────────────────────────

/// Seconds this gate will wait on the bank-hash subprocess. Fail closed on
/// garbage: an env value that does not parse to a positive number is an error,
/// not a silent fall back to the default.
pub fn bank_hash_timeout() -> Result<f64, String> {
    let raw = std::env::var(TIMEOUT_ENV).unwrap_or_default();
    if raw.is_empty() {
        return Ok(BANK_HASH_TIMEOUT_S);
    }
    match raw.parse::<f64>() {
        Ok(v) if v > 0.0 => Ok(v),
        _ => Err(format!(
            "invalid {TIMEOUT_ENV} (want a positive number of seconds)"
        )),
    }
}

/// `Duration::from_secs_f64` panics on a non-finite or overflowing value, and
/// `inf` is a legal budget on the oracle's side, so clamp instead of panicking.
fn budget(secs: f64) -> Duration {
    if !secs.is_finite() || secs > 1e9 {
        Duration::from_secs(1_000_000_000)
    } else {
        Duration::from_secs_f64(secs)
    }
}

/// Port of `live_bank_hash()`.
///
/// Candidate order is the oracle's: the prebuilt `target/debug/cdcp` when it is
/// a file, then `cargo run -q -p cdcp_cli --locked`. A candidate that cannot be
/// spawned, exits non-zero, or whose last output line is not 64 lowercase hex
/// characters is skipped. If neither yields a digest, `goldens/bank_hash.txt`
/// is read verbatim (the oracle does not hex-validate that fallback, and
/// neither does this).
///
/// Two candidate outcomes are NOT skipped, because a broken hash oracle must not
/// be able to read as a pass (bd-hw3): a candidate that exits 0 having written
/// nothing, and a candidate that outlives its budget. Both end the whole lookup
/// with an error.
///
/// On error the string returned is the oracle's `RuntimeError` message, because
/// the caller appends `str(e)` to the error list.
pub fn live_bank_hash(root: &Path) -> Result<String, String> {
    let timeout = bank_hash_timeout()?;
    let mut candidates: Vec<Vec<String>> = Vec::new();
    let bin_path = root.join("target").join("debug").join("cdcp");
    // ABSENT-OK: candidate search; cargo run is always appended, so a missing
    // debug binary does not skip the hash.
    if bin_path.is_file() {
        candidates.push(vec![
            bin_path.to_string_lossy().into_owned(),
            "bank-hash".to_string(),
            "--bank".to_string(),
            BANK_ARG.to_string(),
        ]);
    }
    candidates.push(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "cdcp_cli",
            "--locked",
            "--",
            "bank-hash",
            "--bank",
            BANK_ARG,
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    );

    for cmd in &candidates {
        let raw = match capture_merged(cmd, root, timeout) {
            Capture::TimedOut => {
                return Err("bank-hash timed out (cannot obtain live bank_hash)".to_string())
            }
            Capture::Skip => continue,
            Capture::Output(d) => d,
        };
        let Some(hx) = last_stripped_line(&raw) else {
            return Err(
                "bank-hash exited 0 with no output (cannot obtain live bank_hash)".to_string(),
            );
        };
        // `len(hx) == 64 and all(c in "0123456789abcdef" for c in hx)`
        if hx.chars().count() == 64 && hx.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
            return Ok(hx);
        }
    }

    let golden = root.join(GOLDEN_REL);
    if golden.is_file() {
        match fs::read(&golden) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => Ok(py_strip(&s)),
                Err(_) => Err(format!("{} is not valid UTF-8", golden.display())),
            },
            Err(e) => Err(format!("cannot read {}: {e}", golden.display())),
        }
    } else {
        Err("cannot obtain live bank_hash".to_string())
    }
}

/// Create a fresh 0600 file under the temp dir that CANNOT be an attacker's
/// pre-planted symlink.
///
/// `File::create` (O_CREAT|O_TRUNC) FOLLOWS a symlink at the target path. In a
/// world-writable temp dir with a guessable name — pid plus a nanosecond stamp is
/// guessable — a local attacker plants a symlink and this process truncates and
/// writes whatever the symlink points at, with the invoking user's authority.
///
/// The fix is `create_new(true)` (O_CREAT|O_EXCL), which makes an existing path
/// a hard ERROR rather than something to follow. Name predictability then buys an
/// attacker only denial of service, never a write. `mode(0o600)` keeps the
/// contents unreadable by other local users. Retried a few times so an
/// unlucky-or-hostile collision does not fail the gate outright.
///
/// FLOOR-RAISE, and what this CANNOT decide: it does not make the temp directory
/// itself trustworthy. If `TMPDIR` points somewhere an attacker fully controls,
/// they can still deny service. It removes the symlink-following WRITE, not every
/// hazard of using a shared directory.
fn create_exclusive(stem: &str, ext: &str) -> Option<(fs::File, std::path::PathBuf)> {
    use std::os::unix::fs::OpenOptionsExt;
    let dir = std::env::temp_dir();
    for attempt in 0..8u32 {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = dir.join(format!(
            "{stem}_{}_{nonce}_{attempt}.{ext}",
            std::process::id()
        ));
        match fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(f) => return Some((f, path)),
            Err(_) => continue,
        }
    }
    None
}

/// What one candidate invocation produced.
enum Capture {
    /// Exited 0. The merged stdout+stderr bytes.
    Output(Vec<u8>),
    /// Spawn failure or non-zero exit — the oracle catches both and tries the
    /// next candidate.
    Skip,
    /// Outlived its budget. Not skippable: see `live_bank_hash`.
    TimedOut,
}

/// Run `cmd` with stdout and stderr pointed at ONE file description, which is
/// how `subprocess.check_output(..., stderr=subprocess.STDOUT)` merges the two
/// streams, and give up after `timeout_s` the way `timeout=` does.
fn capture_merged(cmd: &[String], cwd: &Path, timeout_s: f64) -> Capture {
    // The shared file description is deliberate — it is what reproduces Python's
    // stream INTERLEAVING. Capturing the two streams separately and concatenating
    // would diverge byte-for-byte whenever both write, which is exactly what the
    // differential test exists to catch. So the temp file stays; it is created
    // safely instead (see `create_exclusive`).
    let Some((file, sink)) = create_exclusive("cdcp_gate_verify_content_lock", "out") else {
        return Capture::Skip;
    };
    let Ok(dup) = file.try_clone() else {
        let _ = fs::remove_file(&sink);
        return Capture::Skip;
    };
    let spawned = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::from(file))
        .stderr(Stdio::from(dup))
        .spawn();
    let mut child = match spawned {
        Ok(c) => c,
        Err(_) => {
            let _ = fs::remove_file(&sink);
            return Capture::Skip;
        }
    };

    let deadline = Instant::now() + budget(timeout_s);
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Some(st),
            Ok(None) => {}
            Err(_) => break None,
        }
        if Instant::now() >= deadline {
            // `subprocess` kills the child before raising TimeoutExpired; leaving
            // it running would leak a process per invocation.
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_file(&sink);
            return Capture::TimedOut;
        }
        std::thread::sleep(Duration::from_millis(5));
    };

    let data = fs::read(&sink).unwrap_or_default();
    let _ = fs::remove_file(&sink);

    match status {
        Some(s) if s.success() => Capture::Output(data),
        _ => Capture::Skip,
    }
}

/// `out.strip().splitlines()[-1].strip()` with `text=True` universal newlines.
/// `None` stands for empty output, which is a typed error at the call site
/// rather than the oracle's former `IndexError`.
fn last_stripped_line(raw: &[u8]) -> Option<String> {
    let decoded = String::from_utf8_lossy(raw);
    let unified = decoded.replace("\r\n", "\n").replace('\r', "\n");
    let stripped = py_strip(&unified);
    let lines = py_splitlines(&stripped);
    lines.last().map(|l| py_strip(l))
}

// ── the oracle's selftest_mutate() ─────────────────────────────────────────

/// Port of `selftest_mutate()`, reached only via `CDCP_CONTENT_LOCK_SELFTEST=1`.
pub fn selftest_mutate(root: &Path) -> Outcome {
    let lock = root.join(LOCK_REL);
    if !lock.is_file() {
        return Outcome {
            stdout: String::new(),
            stderr: "FAIL: content.lock missing; cannot selftest\n".to_string(),
            code: 1,
        };
    }
    let text = match fs::read(&lock).map(String::from_utf8) {
        Ok(Ok(t)) => t,
        Ok(Err(_)) => return undecidable(format!("{} is not valid UTF-8", lock.display())),
        Err(e) => return undecidable(format!("cannot read {}: {e}", lock.display())),
    };

    let mut mutated = false;
    let mut new_lines: Vec<String> = Vec::new();
    for line in py_splitlines(&text) {
        let mut line = line.to_string();
        if line.starts_with("bank_hash = ") && !mutated && line.ends_with('"') {
            let body: Vec<char> = line.chars().collect();
            // `body = line[:-1]` then `body[-1]`
            let body = &body[..body.len() - 1];
            if let Some(&last) = body.last() {
                let flip = if last != '0' { '0' } else { '1' };
                let mut rebuilt: String = body[..body.len() - 1].iter().collect();
                rebuilt.push(flip);
                rebuilt.push('"');
                line = rebuilt;
                mutated = true;
            }
        }
        new_lines.push(line);
    }
    if !mutated {
        return Outcome {
            stdout: String::new(),
            stderr: "FAIL: selftest could not locate bank_hash line\n".to_string(),
            code: 1,
        };
    }

    let (mut handle, tmp) = match create_exclusive("cdcp_gate_content_selftest", "lock") {
        Some(pair) => pair,
        None => return undecidable("cannot create a private temp file".to_string()),
    };
    let body = new_lines.join("\n") + "\n";
    if let Err(e) = std::io::Write::write_all(&mut handle, body.as_bytes()) {
        let _ = fs::remove_file(&tmp);
        return undecidable(format!("cannot write {}: {e}", tmp.display()));
    }
    drop(handle);

    let verdict = verify(root, &tmp);
    let _ = fs::remove_file(&tmp);

    let errs = match verdict {
        Verdict::Undecidable(m) => return undecidable(m),
        Verdict::Errors(e) => e,
    };
    if errs.is_empty() {
        return Outcome {
            stdout: String::new(),
            stderr: "FAIL: expected RED on mutated bank_hash but verify was green\n".to_string(),
            code: 1,
        };
    }
    if !errs.iter().any(|e| e.contains("bank_hash drift")) {
        let mut s = String::from("FAIL: expected bank_hash drift signal; got:\n");
        for e in &errs {
            s.push_str(&format!("  - {e}\n"));
        }
        return Outcome {
            stdout: String::new(),
            stderr: s,
            code: 1,
        };
    }
    Outcome {
        stdout: "verify_content_lock: ok: mutate-selftest trips RED (bank_hash drift)\n"
            .to_string(),
        stderr: String::new(),
        code: 0,
    }
}

// ── TOML reading ───────────────────────────────────────────────────────────

fn read_table(path: &Path) -> Result<toml::Table, String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let text =
        String::from_utf8(bytes).map_err(|_| format!("{} is not valid UTF-8", path.display()))?;
    text.parse::<toml::Table>()
        .map_err(|e| format!("{} is not valid TOML: {e}", path.display()))
}

// ── Python semantics emulated ──────────────────────────────────────────────

/// `bool(value)` for the types `tomllib` can produce.
pub fn py_truthy(v: &Value) -> bool {
    match v {
        Value::String(s) => !s.is_empty(),
        Value::Integer(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Boolean(b) => *b,
        Value::Array(a) => !a.is_empty(),
        Value::Table(t) => !t.is_empty(),
        Value::Datetime(_) => true,
    }
}

/// `data.get(k) or {}` — `None` here stands for the substituted empty dict.
fn or_empty(v: Option<&Value>) -> Option<&Value> {
    match v {
        Some(x) if py_truthy(x) => Some(x),
        _ => None,
    }
}

/// `len(x)` for the receipt counters. `None` is the substituted `{}`.
fn py_len(v: Option<&Value>) -> usize {
    match v {
        None => 0,
        Some(Value::Table(t)) => t.len(),
        Some(Value::Array(a)) => a.len(),
        Some(Value::String(s)) => s.chars().count(),
        Some(_) => 0,
    }
}

/// `value == 1` under Python's numeric tower (`True == 1`, `1.0 == 1`).
pub fn py_eq_one(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Integer(i)) => *i == 1,
        Some(Value::Float(f)) => *f == 1.0,
        Some(Value::Boolean(b)) => *b,
        _ => false,
    }
}

/// `repr(x)`; `None` renders as Python's `None`.
pub fn py_repr_opt(v: Option<&Value>) -> String {
    match v {
        None => "None".to_string(),
        Some(x) => py_repr(x),
    }
}

/// `repr(x)` for the types `tomllib` can produce.
pub fn py_repr(v: &Value) -> String {
    match v {
        Value::String(s) => py_str_repr(s),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => py_float_repr(*f),
        Value::Boolean(b) => {
            if *b {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        Value::Array(a) => {
            let parts: Vec<String> = a.iter().map(py_repr).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Table(t) => {
            let parts: Vec<String> = t
                .iter()
                .map(|(k, val)| format!("{}: {}", py_str_repr(k), py_repr(val)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        // Documented gap: CPython would print `datetime.date(2020, 1, 1)`.
        Value::Datetime(d) => d.to_string(),
    }
}

/// `str(x)` — differs from `repr` only for `str` itself.
pub fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => py_repr(other),
    }
}

/// CPython's `repr()` quoting for `str`.
fn py_str_repr(s: &str) -> String {
    let has_single = s.contains('\'');
    let has_double = s.contains('"');
    let quote = if has_single && !has_double { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `repr()` of a float, minus CPython's exponent thresholds (see header).
fn py_float_repr(f: f64) -> String {
    if f.is_nan() {
        return "nan".to_string();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf" } else { "-inf" }.to_string();
    }
    let s = format!("{f}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

/// `s[:n]` — Python slices by character, not by byte.
pub fn py_slice(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// True for the code points `str.isspace()` accepts.
fn py_is_space(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')
}

/// `str.strip()` with no argument.
pub fn py_strip(s: &str) -> String {
    s.trim_matches(py_is_space).to_string()
}

/// `str.splitlines()` — CPython's full separator set, `\r\n` counted once.
pub fn py_splitlines(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let is_sep = matches!(
            c,
            '\n' | '\r'
                | '\u{b}'
                | '\u{c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if is_sep {
            out.push(std::mem::take(&mut cur));
            if c == '\r' && i + 1 < chars.len() && chars[i + 1] == '\n' {
                i += 1;
            }
        } else {
            cur.push(c);
        }
        i += 1;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

// ── sha256 ─────────────────────────────────────────────────────────────────
//
// `cdcp_gate` depends on `serde` and `toml` only. Adding `sha2` would mean
// editing `crates/cdcp_gate/Cargo.toml`, a file three sibling migration beads
// are also touching this hour; a self-contained implementation keeps this bead
// to the single new file the registration contract promises. It is pinned by
// the NIST vectors in this module's tests and, more usefully, by a test that
// recomputes every digest already recorded in the committed `content.lock` —
// digests produced by CPython's `hashlib`, i.e. an oracle this code did not
// write.

const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Streaming SHA-256, FIPS 180-4.
pub struct Sha256 {
    state: [u32; 8],
    block: [u8; 64],
    filled: usize,
    bits: u64,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            block: [0u8; 64],
            filled: 0,
            bits: 0,
        }
    }

    pub fn update(&mut self, mut data: &[u8]) {
        self.bits = self.bits.wrapping_add((data.len() as u64) * 8);
        if self.filled > 0 {
            let want = 64 - self.filled;
            let take = want.min(data.len());
            self.block[self.filled..self.filled + take].copy_from_slice(&data[..take]);
            self.filled += take;
            data = &data[take..];
            if self.filled == 64 {
                let b = self.block;
                self.compress(&b);
                self.filled = 0;
            }
        }
        while data.len() >= 64 {
            let mut b = [0u8; 64];
            b.copy_from_slice(&data[..64]);
            self.compress(&b);
            data = &data[64..];
        }
        if !data.is_empty() {
            self.block[..data.len()].copy_from_slice(data);
            self.filled = data.len();
        }
    }

    pub fn finalize(mut self) -> [u8; 32] {
        let bits = self.bits;
        self.update_raw_pad(bits);
        let mut out = [0u8; 32];
        for (i, w) in self.state.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&w.to_be_bytes());
        }
        out
    }

    fn update_raw_pad(&mut self, bits: u64) {
        // 0x80 then zeros then the 64-bit big-endian length.
        self.block[self.filled] = 0x80;
        self.filled += 1;
        if self.filled > 56 {
            for i in self.filled..64 {
                self.block[i] = 0;
            }
            let b = self.block;
            self.compress(&b);
            self.filled = 0;
        }
        for i in self.filled..56 {
            self.block[i] = 0;
        }
        self.block[56..64].copy_from_slice(&bits.to_be_bytes());
        let b = self.block;
        self.compress(&b);
        self.filled = 0;
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K256[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, delta) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(delta);
        }
    }
}

pub fn sha256_hex_bytes(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex_lower(&h.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    const D: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(D[(b >> 4) as usize] as char);
        s.push(D[(b & 0x0f) as usize] as char);
    }
    s
}

/// Port of `sha256_file`, chunked at 1 MiB like the oracle.
pub fn sha256_file(path: &Path) -> Result<String, std::io::Error> {
    let mut f = fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex_lower(&h.finalize()))
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── temp-file creation refuses a planted symlink ──────────────────────
    //
    // The property under test is O_EXCL, not name entropy. A guessable name is
    // survivable; a followed symlink is not. These assert the mechanism directly
    // rather than trying to win a race against ourselves.

    #[test]
    fn create_new_refuses_to_follow_a_planted_symlink() {
        use std::os::unix::fs::OpenOptionsExt;
        let td = tempfile::tempdir().unwrap();
        let victim = td.path().join("victim.txt");
        fs::write(&victim, b"ORIGINAL CONTENTS - MUST SURVIVE").unwrap();
        let planted = td.path().join("planted.out");
        std::os::unix::fs::symlink(&victim, &planted).unwrap();

        // This is exactly the call create_exclusive makes.
        let res = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&planted);
        assert!(
            res.is_err(),
            "create_new must REFUSE an existing path, symlink or not — following it \
             is the vulnerability"
        );
        assert_eq!(
            fs::read(&victim).unwrap(),
            b"ORIGINAL CONTENTS - MUST SURVIVE",
            "the symlink target must be untouched"
        );

        // Contrast: the old File::create would have truncated the victim. Proving
        // the negative here is what makes the fix legible to the next reader.
        let _ = fs::File::create(&planted).unwrap();
        assert_eq!(
            fs::read(&victim).unwrap(),
            b"",
            "File::create FOLLOWS the symlink and truncates — this is what was fixed"
        );
    }

    #[test]
    fn create_exclusive_yields_a_private_file_and_cleans_up() {
        use std::os::unix::fs::PermissionsExt;
        let (f, p) = create_exclusive("cdcp_gate_unit_probe", "tmp")
            .expect("create_exclusive must succeed in a sane temp dir");
        assert!(p.exists(), "the file must exist at the returned path");
        let mode = f.metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "temp contents must not be world/group readable"
        );
        assert!(
            !fs::symlink_metadata(&p).unwrap().file_type().is_symlink(),
            "the created path must be a real file, never a symlink"
        );
        drop(f);
        fs::remove_file(&p).unwrap();
    }

    // ── sha256 against published vectors ──────────────────────────────────

    #[test]
    fn sha256_nist_vectors() {
        assert_eq!(
            sha256_hex_bytes(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex_bytes(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex_bytes(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            sha256_hex_bytes(&[b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn sha256_is_block_boundary_correct() {
        // 55/56/57/63/64/65 bytes exercise every padding branch.
        let expect = [
            (
                55usize,
                "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
            ),
            (
                56,
                "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
            ),
            (
                57,
                "f13b2d724659eb3bf47f2dd6af1accc87b81f09f59f2b75e5c0bed6589dfe8c6",
            ),
            (
                63,
                "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
            ),
            (
                64,
                "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
            ),
            (
                65,
                "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
            ),
            (
                119,
                "31eba51c313a5c08226adf18d4a359cfdfd8d2e816b13f4af952f7ea6584dcfb",
            ),
            (
                120,
                "2f3d335432c70b580af0e8e1b3674a7c020d683aa5f73aaaedfdc55af904c21c",
            ),
            (
                127,
                "c57e9278af78fa3cab38667bef4ce29d783787a2f731d4e12200270f0c32320a",
            ),
            (
                128,
                "6836cf13bac400e9105071cd6af47084dfacad4e5e302c94bfed24e013afb73e",
            ),
        ];
        for (n, want) in expect {
            assert_eq!(sha256_hex_bytes(&vec![b'a'; n]), want, "len {n}");
        }
    }

    #[test]
    fn sha256_streaming_matches_one_shot() {
        let data: Vec<u8> = (0u32..5000).map(|i| (i % 251) as u8).collect();
        let one = sha256_hex_bytes(&data);
        let mut h = Sha256::new();
        for chunk in data.chunks(7) {
            h.update(chunk);
        }
        assert_eq!(hex_lower(&h.finalize()), one);
    }

    // ── Python emulations ─────────────────────────────────────────────────

    #[test]
    fn truthiness_matches_python() {
        let t = |s: &str| -> Value { s.parse::<toml::Table>().unwrap()["x"].clone() };
        assert!(!py_truthy(&t("x = \"\"")));
        assert!(py_truthy(&t("x = \"a\"")));
        assert!(!py_truthy(&t("x = 0")));
        assert!(py_truthy(&t("x = 1")));
        assert!(!py_truthy(&t("x = 0.0")));
        assert!(!py_truthy(&t("x = false")));
        assert!(py_truthy(&t("x = true")));
        assert!(!py_truthy(&t("x = []")));
        assert!(py_truthy(&t("x = [1]")));
        assert!(!py_truthy(&t("x = {}")));
        assert!(py_truthy(&t("x = {a = 1}")));
    }

    #[test]
    fn schema_version_uses_pythons_numeric_tower() {
        let t = |s: &str| -> Value { s.parse::<toml::Table>().unwrap()["x"].clone() };
        assert!(py_eq_one(Some(&t("x = 1"))));
        // In Python `True == 1` and `1.0 == 1`, so both slip past `schema != 1`.
        assert!(py_eq_one(Some(&t("x = true"))));
        assert!(py_eq_one(Some(&t("x = 1.0"))));
        assert!(!py_eq_one(Some(&t("x = 2"))));
        assert!(!py_eq_one(Some(&t("x = \"1\""))));
        assert!(!py_eq_one(None));
    }

    #[test]
    fn repr_matches_cpython_quoting_rules() {
        assert_eq!(py_repr_opt(None), "None");
        let t = |s: &str| -> Value { s.parse::<toml::Table>().unwrap()["x"].clone() };
        assert_eq!(py_repr(&t("x = 2")), "2");
        assert_eq!(py_repr(&t("x = true")), "True");
        assert_eq!(py_repr(&t("x = false")), "False");
        assert_eq!(py_repr(&t("x = \"1\"")), "'1'");
        assert_eq!(py_repr(&t("x = 2.5")), "2.5");
        assert_eq!(py_repr(&t("x = 2.0")), "2.0");
        assert_eq!(py_repr(&t("x = [1, \"a\"]")), "[1, 'a']");
        assert_eq!(py_str_repr("it's"), "\"it's\"");
        assert_eq!(py_str_repr("say \"hi\""), "'say \"hi\"'");
        assert_eq!(py_str_repr("both ' and \""), "'both \\' and \"'");
        assert_eq!(py_str_repr("a\\b"), "'a\\\\b'");
        assert_eq!(py_str_repr("a\nb\tc"), "'a\\nb\\tc'");
        assert_eq!(py_str_repr("\u{7}"), "'\\x07'");
    }

    #[test]
    fn slicing_counts_characters_not_bytes() {
        assert_eq!(py_slice("abcdef", 3), "abc");
        assert_eq!(py_slice("abc", 12), "abc");
        assert_eq!(py_slice("é\u{2026}xyz", 2), "é\u{2026}");
    }

    #[test]
    fn splitlines_uses_the_full_cpython_separator_set() {
        assert_eq!(py_splitlines("a\nb"), vec!["a", "b"]);
        assert_eq!(py_splitlines("a\r\nb"), vec!["a", "b"]);
        assert_eq!(py_splitlines("a\rb"), vec!["a", "b"]);
        assert_eq!(py_splitlines("a\u{b}b"), vec!["a", "b"]);
        assert_eq!(py_splitlines("a\u{2028}b"), vec!["a", "b"]);
        assert_eq!(py_splitlines("a\n"), vec!["a"]);
        assert!(py_splitlines("").is_empty());
    }

    #[test]
    fn strip_matches_python_whitespace() {
        assert_eq!(py_strip("  a b \n\t"), "a b");
        assert_eq!(py_strip("\u{1c}x\u{85}"), "x");
        assert_eq!(py_strip("   "), "");
    }

    #[test]
    fn the_truncation_marker_is_u2026_not_three_dots() {
        assert_eq!(ELLIPSIS.to_string().as_bytes(), &[0xe2, 0x80, 0xa6]);
    }

    // ── the shipped lock is the oracle for the digest code ────────────────

    /// Every hash in the committed `content.lock` was produced by CPython's
    /// `hashlib.sha256`. Recomputing them here checks this module's digest and
    /// its path resolution against an artifact this code did not write.
    #[test]
    fn recomputes_every_digest_in_the_committed_lock() {
        let root = crate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("engine root")
            .canonicalize()
            .expect("canonical root");
        let table = read_table(&root.join(LOCK_REL)).expect("content.lock parses");
        let mut checked = 0usize;
        for section in ["knowledge", "modules"] {
            let Some(Value::Table(t)) = table.get(section) else {
                panic!("[{section}] missing from content.lock");
            };
            for (rel, expected) in t {
                let p = resolve_pinned(&root, rel);
                if !p.is_file() {
                    // A pinned file genuinely missing is the gate's own RED, not
                    // this test's business; the differential suite covers it.
                    continue;
                }
                let got = sha256_file(&p).expect("hash");
                assert_eq!(
                    Value::String(got.clone()),
                    *expected,
                    "{section}/{rel} recomputed to {got}"
                );
                checked += 1;
            }
        }
        assert!(
            checked >= 30,
            "only {checked} pinned digests recomputed — a vacuous digest check is an ERROR"
        );
    }

    #[test]
    fn the_live_tree_is_green_and_the_receipt_names_the_counts() {
        let root = crate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("engine root")
            .canonicalize()
            .expect("canonical root");
        let out = evaluate(&root);
        assert_eq!(out.code, 0, "stderr: {}", out.stderr);
        assert!(out.stderr.is_empty(), "{}", out.stderr);
        assert!(
            out.stdout
                .starts_with("verify_content_lock: PASS bank_hash="),
            "{}",
            out.stdout
        );
        assert!(out.stdout.contains('\u{2026}'), "{}", out.stdout);
        // A green verdict that does not say what it ranges over invites the
        // reader to supply their own scope. Pin the coverage receipt.
        assert!(
            out.stdout.contains(
                "verify_content_lock: covered roots (every file found under these is pinned \
                 and matched): knowledge/*.toml=9 web/content/modules/*.md=16 ../modules/*.md=15\n"
            ),
            "{}",
            out.stdout
        );
        assert!(
            out.stdout
                .contains("verify_content_lock: NOT covered: anything outside those roots"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn a_missing_lock_is_red_and_names_the_path() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().canonicalize().unwrap();
        let out = evaluate(&root);
        assert_eq!(out.code, 1);
        assert!(out.stdout.is_empty());
        assert!(
            out.stderr.contains(&format!(
                "missing content.lock at {}",
                root.join(LOCK_REL).display()
            )),
            "{}",
            out.stderr
        );
    }

    #[test]
    fn an_empty_lock_is_red_on_all_four_counts_never_a_pass() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().canonicalize().unwrap();
        fs::write(root.join(LOCK_REL), "").unwrap();
        let out = evaluate(&root);
        assert_eq!(out.code, 1, "{}", out.stdout);
        for want in [
            "unsupported schema_version=None (want 1)",
            "content.lock missing bank_hash",
            "content.lock [knowledge] empty (vacuous ERROR)",
            "content.lock [modules] empty (vacuous ERROR)",
        ] {
            assert!(
                out.stderr.contains(want),
                "missing {want:?} in:\n{}",
                out.stderr
            );
        }
    }

    // ── the tree-side walk (bd-z3v) ───────────────────────────────────────

    /// Build `<tmp>/engine` with all three locked roots populated, plus the
    /// matching pinned key sets. Returns (tempdir, engine root, keys).
    fn walkable_root() -> (
        tempfile::TempDir,
        PathBuf,
        BTreeSet<String>,
        BTreeSet<String>,
    ) {
        let td = tempfile::tempdir().unwrap();
        let base = td.path().canonicalize().unwrap();
        let engine = base.join("engine");
        fs::create_dir_all(engine.join("knowledge")).unwrap();
        fs::create_dir_all(engine.join("web/content/modules")).unwrap();
        fs::create_dir_all(base.join("modules")).unwrap();
        fs::write(engine.join("knowledge/a.toml"), "x = 1\n").unwrap();
        fs::write(engine.join("web/content/modules/01.md"), "# a\n").unwrap();
        fs::write(base.join("modules/01.md"), "# b\n").unwrap();
        // Files the roots must NOT pick up: wrong suffix, and one level down.
        fs::write(engine.join("knowledge/notes.txt"), "ignored\n").unwrap();
        fs::create_dir_all(engine.join("knowledge/corpus")).unwrap();
        fs::write(engine.join("knowledge/corpus/deep.toml"), "y = 1\n").unwrap();

        let k: BTreeSet<String> = ["knowledge/a.toml".to_string()].into();
        let m: BTreeSet<String> = [
            "web/content/modules/01.md".to_string(),
            "modules/01.md".to_string(),
        ]
        .into();
        (td, engine, k, m)
    }

    #[test]
    fn a_fully_pinned_tree_produces_no_tree_side_errors() {
        let (_td, engine, k, m) = walkable_root();
        assert!(
            tree_side_errors(&engine, &k, &m).is_empty(),
            "{:?}",
            tree_side_errors(&engine, &k, &m)
        );
    }

    #[test]
    fn a_file_with_no_row_behind_it_is_named() {
        let (_td, engine, k, m) = walkable_root();
        fs::write(engine.join("knowledge/smuggled.toml"), "z = 1\n").unwrap();
        fs::write(engine.parent().unwrap().join("modules/99.md"), "# no\n").unwrap();
        let errs = tree_side_errors(&engine, &k, &m);
        assert_eq!(
            errs,
            vec![
                "[knowledge] in the tree but not pinned in content.lock: knowledge/smuggled.toml"
                    .to_string(),
                "[modules] in the tree but not pinned in content.lock: modules/99.md".to_string(),
            ]
        );
    }

    /// The bd-z3v shape: the lock is the only thing that changed, and the file
    /// it stopped pinning is still sitting there.
    #[test]
    fn deleting_a_row_leaves_its_file_unpinned() {
        let (_td, engine, _k, m) = walkable_root();
        let none: BTreeSet<String> = BTreeSet::new();
        let errs = tree_side_errors(&engine, &none, &m);
        assert_eq!(
            errs,
            vec![
                "[knowledge] in the tree but not pinned in content.lock: knowledge/a.toml"
                    .to_string()
            ],
            "a row that stops existing must not make its file stop being checked"
        );
    }

    #[test]
    fn a_root_that_matched_nothing_is_an_error_not_a_pass() {
        let (_td, engine, k, m) = walkable_root();
        fs::remove_file(engine.join("knowledge/a.toml")).unwrap();
        let errs = tree_side_errors(&engine, &k, &m);
        assert_eq!(
            errs,
            vec![
                "[knowledge] locked root matched zero files: knowledge/*.toml \
                  (nothing was checked there \u{2014} vacuous ERROR)"
                    .to_string()
            ]
        );
    }

    #[test]
    fn a_root_that_is_absent_is_an_error_not_a_pass() {
        let (_td, engine, k, m) = walkable_root();
        fs::remove_file(engine.join("knowledge/a.toml")).unwrap();
        fs::remove_file(engine.join("knowledge/notes.txt")).unwrap();
        fs::remove_file(engine.join("knowledge/corpus/deep.toml")).unwrap();
        fs::remove_dir(engine.join("knowledge/corpus")).unwrap();
        fs::remove_dir(engine.join("knowledge")).unwrap();
        let errs = tree_side_errors(&engine, &k, &m);
        assert_eq!(
            errs,
            vec![
                "[knowledge] locked root is not a directory: knowledge/*.toml \
                  (nothing was checked there \u{2014} vacuous ERROR)"
                    .to_string()
            ]
        );
    }

    /// The walk is one level deep and suffix-exact, matching `Path.glob`.
    #[test]
    fn the_walk_does_not_recurse_and_does_not_widen_the_suffix() {
        let (_td, engine, ..) = walkable_root();
        let found: Vec<String> = discover(&engine.join("knowledge"), ".toml")
            .iter()
            .map(|p| lock_key(&engine, p))
            .collect();
        assert_eq!(found, vec!["knowledge/a.toml".to_string()]);
    }

    /// `lock_key` must invert `resolve_pinned`, or "in the tree but not pinned"
    /// would fire on files that ARE pinned.
    #[test]
    fn lock_key_inverts_resolve_pinned() {
        let (_td, engine, k, m) = walkable_root();
        for key in k.iter().chain(m.iter()) {
            let p = resolve_pinned(&engine, key);
            assert!(p.is_file(), "{key} did not resolve");
            assert_eq!(&lock_key(&engine, &p), key, "round trip failed for {key}");
        }
    }

    #[test]
    fn counts_are_reported_per_root() {
        let (_td, engine, ..) = walkable_root();
        assert_eq!(
            root_counts(&engine),
            vec![
                ("knowledge/*.toml", 1),
                ("web/content/modules/*.md", 1),
                ("../modules/*.md", 1),
            ]
        );
    }

    // ── the bank-hash budget (bd-hw3) ─────────────────────────────────────

    #[test]
    fn budget_never_panics_on_a_hostile_value() {
        assert_eq!(budget(1.5), Duration::from_millis(1500));
        assert_eq!(budget(f64::INFINITY), Duration::from_secs(1_000_000_000));
        assert_eq!(budget(f64::NAN), Duration::from_secs(1_000_000_000));
        assert_eq!(budget(1e300), Duration::from_secs(1_000_000_000));
    }

    #[test]
    fn the_default_budget_is_five_minutes() {
        // EXTRACT-THEN-DELETE: the Python timeout literal is gone. This
        // constant is the budget.
        assert_eq!(BANK_HASH_TIMEOUT_S, 300.0);
    }

    /// Known-bad that replaces the retired differential
    /// `tampered_knowledge_hash_is_red_in_both`.
    #[test]
    fn tampered_knowledge_hash_is_red() {
        let (_td, engine, ..) = walkable_root();
        let modules_a = sha256_hex_bytes(b"# a\n");
        let modules_b = sha256_hex_bytes(b"# b\n");
        let bh = "aa".repeat(32);
        fs::create_dir_all(engine.join("goldens")).unwrap();
        fs::write(engine.join("goldens/bank_hash.txt"), &bh).unwrap();
        // Stop `live_bank_hash` walking out of the fixture and `cargo run`-ing
        // the real workspace (slow, and a different digest).
        fs::write(engine.join("Cargo.toml"), "# not a package\n").unwrap();
        fs::write(
            engine.join(LOCK_REL),
            format!(
                "schema_version = 1\nbank_hash = \"{bh}\"\n\n[knowledge]\n\
                 \"knowledge/a.toml\" = \"0000000000000000000000000000000000000000000000000000000000000000\"\n\n\
                 [modules]\n\
                 \"web/content/modules/01.md\" = \"{modules_a}\"\n\
                 \"modules/01.md\" = \"{modules_b}\"\n"
            ),
        )
        .unwrap();
        let out = evaluate(&engine);
        assert_eq!(out.code, 1, "tampered pin must be RED:\n{}", out.stderr);
        assert!(
            out.stderr.contains("hash mismatch: knowledge/a.toml"),
            "tampered pin must be named:\n{}",
            out.stderr
        );
        assert!(out.stderr.contains("lock=000000000000"), "{}", out.stderr);
    }

    #[test]
    fn malformed_toml_is_error_not_pass() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().canonicalize().unwrap();
        fs::write(root.join(LOCK_REL), "this is not toml = = =\n").unwrap();
        let out = evaluate(&root);
        assert_eq!(out.code, crate::exit::ERROR as i32);
        assert!(out.stdout.is_empty());
    }
}
