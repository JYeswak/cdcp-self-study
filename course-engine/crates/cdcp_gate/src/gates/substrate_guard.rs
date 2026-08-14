//! substrate-guard — S0 of the Rust migration (bd-substrate-rust-migration-jhd.1).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises a floor. It enforces exactly one property: **no unreasoned
//! non-Rust source file enters the tree.** Concretely — a `.py` or `.sh` file
//! tracked or staged under `scripts/`, `crates/`, or the engine root must have a
//! row in `registries/substrate_allowlist.toml` carrying a non-empty `reason`, a
//! bead id, and an `expires` date that has not passed.
//!
//! # WHAT THIS GATE CANNOT DO
//!
//! It cannot decide whether a stated `reason` is honest. It cannot tell a real
//! migration bead from a plausible-looking id. It cannot tell an achievable
//! `expires` from a date chosen to be far away. It reads none of the scripts it
//! permits, so it says nothing about what they do. An author who wants a script
//! in this tree can still get one in by writing a sentence — the change is that
//! the sentence is now dated, attributed, reviewable in a diff, and it rots.
//!
//! The floor moves from *silence* to *a signed, expiring exemption*. That is the
//! whole of the claim; there is no stronger one available from a path-and-registry
//! check, and this header will not pretend otherwise.
//!
//! # WHY IT IS RUST
//!
//! The guard that bans shell is not itself shell. `hooks/pre-commit` is a shim
//! whose entire body is one `exec` of this binary; it holds no decision logic.
//!
//! # THE ALLOWLIST IS THE WORKLIST
//!
//! Every row is a debt. `expires` is what stops a temporary exemption from being
//! permanent by another name. Row count is the migration progress metric and its
//! target is zero.

use crate::date::{self, Ymd};
use crate::registry::{GateCtx, GateError};
use crate::vcs;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

pub const NAME: &str = "substrate-guard";
pub const SUMMARY: &str =
    "no .py/.sh may enter scripts//crates//root without a reasoned, dated, bead-linked allowlist row";

/// Where the registry lives, relative to the engine root.
pub const REGISTRY_PATH: &str = "registries/substrate_allowlist.toml";

/// Extensions the registry may WIDEN but may never narrow below.
pub const FLOOR_EXTENSIONS: &[&str] = &["py", "sh"];
/// Directories the registry may ADD to but may never drop.
pub const FLOOR_ROOTS: &[&str] = &["scripts", "crates"];
/// A reason shorter than this is not a reason.
pub const MIN_REASON_LEN: usize = 24;

const KNOWN_FLAGS: &[&str] = &["--staged", "--verify-wired", "--quiet"];

// ── registry schema ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct Allowlist {
    pub schema_version: u32,
    pub scan: ScanCfg,
    pub wiring: Wiring,
    #[serde(default)]
    pub allow: Vec<Row>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanCfg {
    pub roots: Vec<String>,
    pub extensions: Vec<String>,
    pub include_engine_root_files: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Wiring {
    /// `"pending"` — check.sh has not been wired yet; report but do not fail.
    /// `"wired"`   — check.sh must invoke this gate; its absence is RED.
    /// Anything else (including empty) is a schema ERROR. Blank is never permissive.
    pub status: String,
    pub check_sh: String,
    pub invocation: String,
    pub bead: String,
}

/// One exemption. Every field is load-bearing; `#[serde(default)]` exists so a
/// MISSING field lands here as an empty string and is reported as the schema
/// error it is, rather than as an opaque TOML parse failure.
#[derive(Debug, Clone, Deserialize)]
pub struct Row {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub migration_bead: String,
    #[serde(default)]
    pub expires: String,
}

// ── pure logic (unit-tested without git, without a filesystem) ─────────────

pub fn parse_allowlist(text: &str) -> Result<Allowlist, String> {
    let a: Allowlist =
        toml::from_str(text).map_err(|e| format!("parse {REGISTRY_PATH}: {e}"))?;
    if a.schema_version != 1 {
        return Err(format!(
            "{REGISTRY_PATH}: schema_version {} unsupported (expected 1)",
            a.schema_version
        ));
    }
    Ok(a)
}

/// The registry configures the scan, so the registry is itself an attack surface:
/// dropping `"py"` from `extensions` would disable the gate with a one-word diff.
/// The floor is compiled in and the registry may only widen it.
pub fn check_floor(scan: &ScanCfg) -> Vec<String> {
    let mut v = Vec::new();
    for ext in FLOOR_EXTENSIONS {
        if !scan.extensions.iter().any(|e| e == ext) {
            v.push(format!(
                "{REGISTRY_PATH}: [scan].extensions is missing the compiled-in floor {ext:?} — the registry may widen the scan, never narrow it"
            ));
        }
    }
    for r in FLOOR_ROOTS {
        if !scan.roots.iter().any(|e| e == r) {
            v.push(format!(
                "{REGISTRY_PATH}: [scan].roots is missing the compiled-in floor {r:?} — the registry may widen the scan, never narrow it"
            ));
        }
    }
    if !scan.include_engine_root_files {
        v.push(format!(
            "{REGISTRY_PATH}: [scan].include_engine_root_files = false narrows the compiled-in floor"
        ));
    }
    if scan.extensions.iter().any(|e| e.starts_with('.')) {
        v.push(format!(
            "{REGISTRY_PATH}: [scan].extensions must be bare (\"py\"), not dotted (\".py\")"
        ));
    }
    v
}

pub fn check_wiring_status(w: &Wiring) -> Vec<String> {
    let mut v = Vec::new();
    match w.status.trim() {
        "pending" | "wired" => {}
        "" => v.push(format!(
            "{REGISTRY_PATH}: [wiring].status is empty — blank is never permissive; use \"pending\" or \"wired\""
        )),
        other => v.push(format!(
            "{REGISTRY_PATH}: [wiring].status {other:?} is not \"pending\" or \"wired\""
        )),
    }
    if w.invocation.trim().is_empty() {
        v.push(format!("{REGISTRY_PATH}: [wiring].invocation is empty"));
    }
    if w.check_sh.trim().is_empty() {
        v.push(format!("{REGISTRY_PATH}: [wiring].check_sh is empty"));
    }
    if w.bead.trim().is_empty() {
        v.push(format!("{REGISTRY_PATH}: [wiring].bead is empty"));
    }
    v
}

/// Bead ids look like `bd-<slug>` / `cp-<slug>`, optionally dotted.
pub fn looks_like_bead_id(s: &str) -> bool {
    let s = s.trim();
    let Some(rest) = s.strip_prefix("bd-").or_else(|| s.strip_prefix("cp-")) else {
        return false;
    };
    !rest.is_empty()
        && rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Is this engine-root-relative path inside the scanned surface?
pub fn is_in_scope(path: &str, scan: &ScanCfg) -> bool {
    if path.is_empty() || path.starts_with('/') || path.contains("..") {
        return false;
    }
    match path.split_once('/') {
        // Engine-root file: no directory component.
        None => scan.include_engine_root_files,
        Some((head, _)) => scan.roots.iter().any(|r| r == head),
    }
}

pub fn has_scanned_extension(path: &str, scan: &ScanCfg) -> bool {
    let Some((_, ext)) = path.rsplit_once('.') else {
        return false;
    };
    !ext.contains('/') && scan.extensions.iter().any(|e| e == ext)
}

/// Schema validation of the rows themselves. `exists` answers "is there a file at
/// this path", injected so this stays a pure function under test.
pub fn validate_rows(
    rows: &[Row],
    scan: &ScanCfg,
    today: Ymd,
    exists: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut v = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();

    for (i, r) in rows.iter().enumerate() {
        let where_ = if r.path.trim().is_empty() {
            format!("[[allow]] #{}", i + 1)
        } else {
            format!("[[allow]] {}", r.path.trim())
        };

        if r.path.trim().is_empty() {
            v.push(format!("{where_}: empty `path`"));
            continue;
        }
        let path = r.path.trim();
        if path.starts_with('/') || path.contains("..") || path.contains('\\') {
            v.push(format!(
                "{where_}: `path` must be a normalised engine-root-relative path"
            ));
        }
        if !seen.insert(path) {
            v.push(format!("{where_}: duplicate `path` row"));
        }
        if !is_in_scope(path, scan) {
            v.push(format!(
                "{where_}: outside the scanned surface ({}, or an engine-root file) — an exemption for something the gate never scans is dead weight",
                scan.roots.join("/, ")
            ));
        }
        if !has_scanned_extension(path, scan) {
            v.push(format!(
                "{where_}: extension is not one this gate scans ({}) — delete the row",
                scan.extensions.join(", ")
            ));
        }

        // reason — the whole point of the row
        let reason = r.reason.trim();
        if reason.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `reason` — a blank reason is a SCHEMA ERROR, never permission"
            ));
        } else if reason.len() < MIN_REASON_LEN {
            v.push(format!(
                "{where_}: `reason` is {} chars; at least {MIN_REASON_LEN} are needed to say anything a reviewer can disagree with",
                reason.len()
            ));
        }

        // migration_bead — who owns the debt
        let bead = r.migration_bead.trim();
        if bead.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `migration_bead` — an exemption nobody owns is not tracked work"
            ));
        } else if !looks_like_bead_id(bead) {
            v.push(format!(
                "{where_}: `migration_bead` {bead:?} is not a bead id (bd-… / cp-…)"
            ));
        }

        // expires — what stops "temporary" from meaning "forever"
        let expires = r.expires.trim();
        if expires.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `expires` — an exemption that cannot expire is permanent by another name"
            ));
        } else {
            match date::parse_ymd(expires) {
                Err(e) => v.push(format!("{where_}: `expires` {e}")),
                Ok(d) if date::before(d, today) => v.push(format!(
                    "{where_}: EXPIRED on {expires} (today is {:04}-{:02}-{:02}) — port it to Rust under {bead}, or re-affirm the row with a new date and a reason that survives review",
                    today.0, today.1, today.2
                )),
                Ok(_) => {}
            }
        }

        // A row for a file that is gone is the migration's own litter.
        if !exists(path) {
            v.push(format!(
                "{where_}: no file at this path — if it was ported or deleted, delete the row (the allowlist is the worklist; it shrinks to zero)"
            ));
        }
    }
    v
}

/// Scanned files with no row. `rows` is assumed already schema-checked.
pub fn unlisted(candidates: &[String], rows: &[Row], scan: &ScanCfg) -> Vec<String> {
    let listed: BTreeSet<&str> = rows.iter().map(|r| r.path.trim()).collect();
    let mut out = Vec::new();
    for c in candidates {
        if !is_in_scope(c, scan) || !has_scanned_extension(c, scan) {
            continue;
        }
        if !listed.contains(c.as_str()) {
            out.push(format!(
                "{c}: non-Rust file with no row in {REGISTRY_PATH}. Port it to Rust (see epic bd-substrate-rust-migration-jhd), or add a row with a real `reason`, a `migration_bead`, and an `expires` date"
            ));
        }
    }
    out
}

/// Does `scripts/check.sh` actually invoke this gate?
///
/// BUILT != WIRED. A gate no ordered chain calls is a file, not a gate. Matching
/// is on the binary+subcommand pair, not on the full command line, so the
/// orchestrator can wire it with whatever flags and `||` handler check.sh uses.
pub fn check_sh_wires_guard(text: &str) -> bool {
    text.lines()
        .map(|l| l.trim_start())
        .filter(|l| !l.starts_with('#'))
        // A banner that mentions the gate is not the gate running. `echo "==> …
        // substrate-guard"` and `ok "substrate floor"` are the two shapes
        // check.sh uses around every step, and neither is an invocation.
        .filter(|l| !l.starts_with("echo ") && !l.starts_with("ok "))
        .any(|l| l.contains("cdcp_gate") && l.contains(NAME))
}

// ── wiring the pure logic to the tree ──────────────────────────────────────

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(KNOWN_FLAGS)?;
    let quiet = ctx.has_flag("--quiet");
    let root: &Path = &ctx.root;

    let reg_path = root.join(REGISTRY_PATH);
    let text = std::fs::read_to_string(&reg_path)
        .map_err(|e| GateError::error(format!("read {}: {e}", reg_path.display())))?;
    let al = parse_allowlist(&text).map_err(GateError::error)?;

    let mut schema = check_floor(&al.scan);
    schema.extend(check_wiring_status(&al.wiring));
    if !schema.is_empty() {
        return Err(GateError::Error(schema.join(" | ")));
    }

    if !vcs::is_repo(root) {
        return Err(GateError::error(format!(
            "{} is not inside a git working tree; this gate scans git's view of the tree",
            root.display()
        )));
    }

    let tracked = vcs::tracked_files(root).map_err(GateError::error)?;

    // ── anti-vacuous ────────────────────────────────────────────────────────
    // Zero files scanned is an ERROR. A never-scanned tree reports exactly like a
    // clean one; that is how a gate becomes decoration.
    if tracked.is_empty() {
        return Err(GateError::error(
            "scanned 0 files — a vacuous scan is an ERROR, not a pass",
        ));
    }
    let in_scope: Vec<&String> = tracked
        .iter()
        .filter(|p| is_in_scope(p, &al.scan))
        .collect();
    if in_scope.is_empty() {
        return Err(GateError::error(format!(
            "0 files in scope under {:?} (+ engine-root files) out of {} tracked — the scan roots resolve to nothing; ERROR, not a pass",
            al.scan.roots,
            tracked.len()
        )));
    }

    let today = date::today();
    let exists = |p: &str| root.join(p).exists();
    let mut violations = Vec::new();

    let schema_errs = validate_rows(&al.allow, &al.scan, today, &exists);
    if !schema_errs.is_empty() {
        // Schema errors are ERROR-class: the registry could not be honestly read
        // as a set of exemptions, so no file is exempt on its strength.
        return Err(GateError::Error(format!(
            "{} schema error(s) in {REGISTRY_PATH}: {}",
            schema_errs.len(),
            schema_errs.join(" | ")
        )));
    }

    // Presence leg: everything git tracks right now.
    violations.extend(unlisted(&tracked, &al.allow, &al.scan));

    // Staged leg: what THIS commit would add. Adds nothing new when check.sh runs
    // it, and is the whole gate when the pre-commit hook runs it.
    let mut staged_count = 0usize;
    if ctx.has_flag("--staged") {
        let staged = vcs::staged_additions(root).map_err(GateError::error)?;
        staged_count = staged.len();
        for s in unlisted(&staged, &al.allow, &al.scan) {
            let msg = format!("staged for commit — {s}");
            if !violations.iter().any(|v: &String| v.ends_with(&s)) {
                violations.push(msg);
            }
        }
    }

    // Wiring leg: BUILT != WIRED.
    let check_sh = root.join(al.wiring.check_sh.trim());
    let wired = std::fs::read_to_string(&check_sh)
        .map(|t| check_sh_wires_guard(&t))
        .unwrap_or(false);
    let hard_wiring = al.wiring.status.trim() == "wired" || ctx.has_flag("--verify-wired");
    if !wired {
        let msg = format!(
            "{} does not invoke `cdcp_gate {NAME}` — BUILT != WIRED. Add: {} ({})",
            al.wiring.check_sh.trim(),
            al.wiring.invocation.trim(),
            al.wiring.bead.trim()
        );
        if hard_wiring {
            violations.push(msg);
        } else {
            eprintln!("{NAME}: PENDING WIRING: {msg}");
        }
    }

    if !violations.is_empty() {
        return Err(GateError::Violation(violations));
    }

    if !quiet {
        let listed = al.allow.len();
        println!(
            "{NAME}: ok: scanned={} in_scope={} staged_adds={} exemptions={} wired={}",
            tracked.len(),
            in_scope.len(),
            staged_count,
            listed,
            if wired { "yes" } else { "PENDING" }
        );
        println!(
            "{NAME}: floor-raise only: a row records that a reason was WRITTEN, not that it is true. {listed} exemption(s) outstanding; target is 0."
        );
    }
    Ok(())
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn scan() -> ScanCfg {
        ScanCfg {
            roots: vec!["scripts".into(), "crates".into()],
            extensions: vec!["py".into(), "sh".into()],
            include_engine_root_files: true,
        }
    }

    fn row(path: &str) -> Row {
        Row {
            path: path.into(),
            reason: "Grandfathered check.sh gate; port tracked by the migration epic".into(),
            migration_bead: "bd-substrate-rust-migration-jhd.7".into(),
            expires: "2099-01-01".into(),
        }
    }

    fn always() -> impl Fn(&str) -> bool {
        |_: &str| true
    }

    const TODAY: Ymd = (2026, 8, 13);

    // ── the assertion this gate exists for ────────────────────────────────
    #[test]
    fn unlisted_py_is_red() {
        let v = unlisted(&["scripts/foo.py".to_string()], &[], &scan());
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("scripts/foo.py"), "must name the file: {v:?}");
    }

    #[test]
    fn unlisted_sh_is_red() {
        let v = unlisted(&["scripts/foo.sh".to_string()], &[], &scan());
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("scripts/foo.sh"));
    }

    #[test]
    fn unlisted_at_engine_root_is_red() {
        let v = unlisted(&["stray.sh".to_string()], &[], &scan());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn unlisted_under_crates_is_red() {
        let v = unlisted(&["crates/cdcp_core/gen.py".to_string()], &[], &scan());
        assert_eq!(v.len(), 1);
    }

    // ── known-good: the leg that keeps this gate from being routed around ──
    #[test]
    fn allowlisted_file_passes() {
        let v = unlisted(
            &["scripts/verify_bank.py".to_string()],
            &[row("scripts/verify_bank.py")],
            &scan(),
        );
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn rust_files_pass_anywhere() {
        let v = unlisted(
            &[
                "crates/cdcp_gate/src/main.rs".to_string(),
                "scripts/whatever.rs".to_string(),
                "build.rs".to_string(),
            ],
            &[],
            &scan(),
        );
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn files_outside_the_scanned_surface_pass() {
        // tests/ and docs/ are not in scope; the gate is a floor, not a dragnet.
        let v = unlisted(
            &["tests/voice-slop.sh".to_string(), "docs/x.py".to_string()],
            &[],
            &scan(),
        );
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn extensionless_and_other_extensions_pass() {
        let v = unlisted(
            &[
                "scripts/README".to_string(),
                "scripts/smoke_srs.mjs".to_string(),
                "scripts/_module_page_template.html".to_string(),
            ],
            &[],
            &scan(),
        );
        assert!(v.is_empty(), "{v:?}");
    }

    // ── known-bad: schema ────────────────────────────────────────────────
    #[test]
    fn empty_reason_is_a_schema_error_not_permission() {
        let mut r = row("scripts/a.py");
        r.reason = String::new();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(
            v.iter().any(|m| m.contains("empty `reason`")),
            "blank must never be permissive: {v:?}"
        );
    }

    #[test]
    fn whitespace_reason_is_a_schema_error() {
        let mut r = row("scripts/a.py");
        r.reason = "   \t ".into();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(v.iter().any(|m| m.contains("empty `reason`")), "{v:?}");
    }

    #[test]
    fn missing_reason_field_lands_as_a_schema_error() {
        let text = r#"
schema_version = 1
[scan]
roots = ["scripts", "crates"]
extensions = ["py", "sh"]
include_engine_root_files = true
[wiring]
status = "pending"
check_sh = "scripts/check.sh"
invocation = "cargo run -q -p cdcp_gate -- substrate-guard"
bead = "bd-substrate-rust-migration-jhd.1"
[[allow]]
path = "scripts/a.py"
migration_bead = "bd-x"
expires = "2099-01-01"
"#;
        let al = parse_allowlist(text).expect("parses; the field is missing, not malformed");
        let v = validate_rows(&al.allow, &al.scan, TODAY, &always());
        assert!(v.iter().any(|m| m.contains("`reason`")), "{v:?}");
    }

    #[test]
    fn token_reason_is_rejected() {
        let mut r = row("scripts/a.py");
        r.reason = "temp".into();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(v.iter().any(|m| m.contains("chars")), "{v:?}");
    }

    #[test]
    fn backdated_expires_is_red() {
        let mut r = row("scripts/a.py");
        r.expires = "2026-08-12".into();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(v.iter().any(|m| m.contains("EXPIRED")), "{v:?}");
    }

    #[test]
    fn expires_today_still_passes() {
        let mut r = row("scripts/a.py");
        r.expires = "2026-08-13".into();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn missing_or_unparseable_expires_is_red() {
        for bad in ["", "never", "soon", "2026-13-01"] {
            let mut r = row("scripts/a.py");
            r.expires = bad.into();
            let v = validate_rows(&[r], &scan(), TODAY, &always());
            assert!(!v.is_empty(), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn missing_or_bogus_migration_bead_is_red() {
        for bad in ["", "  ", "TODO", "issue-12"] {
            let mut r = row("scripts/a.py");
            r.migration_bead = bad.into();
            let v = validate_rows(&[r], &scan(), TODAY, &always());
            assert!(
                v.iter().any(|m| m.contains("migration_bead")),
                "{bad:?} -> {v:?}"
            );
        }
    }

    #[test]
    fn duplicate_rows_are_red() {
        let v = validate_rows(
            &[row("scripts/a.py"), row("scripts/a.py")],
            &scan(),
            TODAY,
            &always(),
        );
        assert!(v.iter().any(|m| m.contains("duplicate")), "{v:?}");
    }

    #[test]
    fn stale_row_for_a_deleted_file_is_red() {
        let v = validate_rows(&[row("scripts/gone.py")], &scan(), TODAY, &|_| false);
        assert!(v.iter().any(|m| m.contains("no file at this path")), "{v:?}");
    }

    #[test]
    fn row_outside_scope_is_red() {
        let v = validate_rows(&[row("docs/a.py")], &scan(), TODAY, &always());
        assert!(v.iter().any(|m| m.contains("outside the scanned surface")), "{v:?}");
    }

    #[test]
    fn good_row_is_clean() {
        let v = validate_rows(&[row("scripts/verify_bank.py")], &scan(), TODAY, &always());
        assert!(v.is_empty(), "{v:?}");
    }

    // ── known-bad: registry weakening ────────────────────────────────────
    #[test]
    fn registry_cannot_narrow_the_extension_floor() {
        let mut s = scan();
        s.extensions = vec!["sh".into()];
        let v = check_floor(&s);
        assert!(v.iter().any(|m| m.contains("\"py\"")), "{v:?}");
    }

    #[test]
    fn registry_cannot_drop_a_scan_root() {
        let mut s = scan();
        s.roots = vec!["scripts".into()];
        assert!(!check_floor(&s).is_empty());
    }

    #[test]
    fn registry_cannot_turn_off_engine_root_scanning() {
        let mut s = scan();
        s.include_engine_root_files = false;
        assert!(!check_floor(&s).is_empty());
    }

    #[test]
    fn registry_may_widen_the_floor() {
        let mut s = scan();
        s.extensions.push("mjs".into());
        s.roots.push("web".into());
        assert!(check_floor(&s).is_empty());
    }

    // ── known-bad: wiring status ─────────────────────────────────────────
    #[test]
    fn blank_wiring_status_is_a_schema_error() {
        let w = Wiring {
            status: "".into(),
            check_sh: "scripts/check.sh".into(),
            invocation: "x".into(),
            bead: "bd-x".into(),
        };
        assert!(check_wiring_status(&w)
            .iter()
            .any(|m| m.contains("never permissive")));
    }

    #[test]
    fn unknown_wiring_status_is_a_schema_error() {
        let w = Wiring {
            status: "skip".into(),
            check_sh: "scripts/check.sh".into(),
            invocation: "x".into(),
            bead: "bd-x".into(),
        };
        assert!(!check_wiring_status(&w).is_empty());
    }

    #[test]
    fn detects_the_check_sh_step() {
        assert!(check_sh_wires_guard(
            "echo hi\ncargo run -q -p cdcp_gate -- substrate-guard || fail \"substrate\"\n"
        ));
        assert!(!check_sh_wires_guard("echo hi\ncargo test --workspace\n"));
        assert!(
            !check_sh_wires_guard("# cargo run -p cdcp_gate -- substrate-guard\n"),
            "a commented-out step is not a wired step"
        );
        assert!(
            !check_sh_wires_guard("echo \"==> cdcp_gate substrate-guard (S0)\"\n"),
            "a banner that mentions the gate is not the gate running"
        );
        assert!(
            !check_sh_wires_guard("ok \"cdcp_gate substrate-guard floor\"\n"),
            "an ok receipt is not the gate running"
        );
        assert!(
            check_sh_wires_guard(
                "echo \"==> cdcp_gate substrate-guard\"\ncargo run -q -p cdcp_gate -- substrate-guard || fail \"x\"\nok \"substrate floor\"\n"
            ),
            "the real three-line step must be recognised"
        );
    }

    #[test]
    fn scope_predicate() {
        let s = scan();
        assert!(is_in_scope("scripts/a.py", &s));
        assert!(is_in_scope("crates/x/y/a.sh", &s));
        assert!(is_in_scope("a.sh", &s));
        assert!(!is_in_scope("docs/a.py", &s));
        assert!(!is_in_scope("/etc/a.sh", &s));
        assert!(!is_in_scope("../a.sh", &s));
    }

    #[test]
    fn bead_id_shape() {
        assert!(looks_like_bead_id("bd-substrate-rust-migration-jhd.7"));
        assert!(looks_like_bead_id("cp-123"));
        assert!(!looks_like_bead_id("bd-"));
        assert!(!looks_like_bead_id("xx-1"));
        assert!(!looks_like_bead_id(""));
    }

    // ── the header's own honesty ─────────────────────────────────────────
    #[test]
    fn header_states_a_floor_raise_and_overclaims_nothing() {
        let src = include_str!("substrate_guard.rs");
        let header: String = src
            .lines()
            .take_while(|l| l.starts_with("//!"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(header.contains("FLOOR-RAISE"), "header must state the claim class");
        assert!(
            header.contains("CANNOT"),
            "header must state what the gate cannot decide"
        );
        for banned in ["guarantee", "proves", "makes impossible", "impossible"] {
            assert!(
                !header.to_lowercase().contains(banned),
                "header overclaims with {banned:?}"
            );
        }
    }
}
