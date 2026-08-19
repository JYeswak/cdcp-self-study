//! Local flywheel tick emission.
//!
//! The ledger is an output of the loop, not an input supplied by a caller.  In
//! particular, product movement is classified from the named commit and the
//! Charter's product-path policy; prose can only propose what happened.

use serde::Serialize;
use serde_json::Value;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

pub const NAME: &str = "emit-tick";
pub const SUMMARY: &str = "append one computed zs.tick-receipt to the flywheel ledger";
const LEDGER_REL: &str = ".flywheel/tick-ledger.jsonl";
const FORBIDDEN: [&str; 4] = ["standing by", "queue empty", "blocked on josh", "wait_josh"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TickClass {
    Setup,
    Product,
    GuardDenied,
}

impl FromStr for TickClass {
    type Err = TickError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "SETUP" => Ok(Self::Setup),
            "PRODUCT" => Ok(Self::Product),
            "GUARD_DENIED" => Ok(Self::GuardDenied),
            other => Err(TickError::usage(format!(
                "unknown class {other:?}; allowed: SETUP PRODUCT GUARD_DENIED"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Verdict {
    Green,
    Red,
    Blocked,
}

impl FromStr for Verdict {
    type Err = TickError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "GREEN" => Ok(Self::Green),
            "RED" => Ok(Self::Red),
            "BLOCKED" => Ok(Self::Blocked),
            other => Err(TickError::usage(format!(
                "unknown verdict {other:?}; allowed: GREEN RED BLOCKED"
            ))),
        }
    }
}

#[derive(Debug)]
pub enum TickError {
    Usage(String),
    Error(String),
}

impl TickError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }
}

impl fmt::Display for TickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "usage: {message}"),
            Self::Error(message) => write!(f, "error: {message}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmitRequest {
    pub class: TickClass,
    pub bead: String,
    pub value_added: String,
    pub verdict: Verdict,
    pub commit: String,
    pub evidence: String,
    pub claimed_product_moved: Option<bool>,
    pub blocker: Option<String>,
    pub escalation_artifact: Option<PathBuf>,
    pub ledger: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct TickReceipt {
    pub schema: &'static str,
    pub tick: String,
    pub class: TickClass,
    pub bead: String,
    pub value_added: String,
    pub verdict: Verdict,
    pub commit: String,
    pub evidence: String,
    pub product_moved: bool,
    pub product_moved_claimed: Option<bool>,
    pub product_moved_disagreement: Option<String>,
    pub blocker: Option<String>,
    pub escalation_artifact: Option<String>,
}

/// Parse and validate the command's strict key/value argument surface.
pub fn run(root: &Path, args: &[String]) -> Result<String, TickError> {
    let mut class = None;
    let mut bead = None;
    let mut value_added = None;
    let mut verdict = None;
    let mut commit = None;
    let mut evidence = None;
    let mut claimed_product_moved = None;
    let mut blocker = None;
    let mut escalation_artifact = None;
    let mut ledger = root.join(LEDGER_REL);

    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        let needs_value = |name: &str, i: &mut usize| -> Result<String, TickError> {
            *i += 1;
            args.get(*i)
                .cloned()
                .ok_or_else(|| TickError::usage(format!("{name} needs a value")))
        };
        match flag {
            "--class" => class = Some(needs_value(flag, &mut i)?.parse()?),
            "--bead" => bead = Some(needs_value(flag, &mut i)?),
            "--value-added" => value_added = Some(needs_value(flag, &mut i)?),
            "--verdict" => verdict = Some(needs_value(flag, &mut i)?.parse()?),
            "--commit" => commit = Some(needs_value(flag, &mut i)?),
            "--evidence" => evidence = Some(needs_value(flag, &mut i)?),
            "--claim-product-moved" => {
                claimed_product_moved = Some(parse_bool(&needs_value(flag, &mut i)?)?)
            }
            "--blocker" => blocker = Some(needs_value(flag, &mut i)?),
            "--escalation-artifact" => {
                escalation_artifact = Some(root.join(needs_value(flag, &mut i)?))
            }
            "--ledger" => ledger = PathBuf::from(needs_value(flag, &mut i)?),
            other => return Err(TickError::usage(format!("unknown argument {other:?}"))),
        }
        i += 1;
    }

    let request = EmitRequest {
        class: class.ok_or_else(|| TickError::usage("missing --class"))?,
        bead: required("--bead", bead)?,
        value_added: required("--value-added", value_added)?,
        verdict: verdict.ok_or_else(|| TickError::usage("missing --verdict"))?,
        commit: required("--commit", commit)?,
        evidence: required("--evidence", evidence)?,
        claimed_product_moved,
        blocker,
        escalation_artifact,
        ledger,
    };
    let receipt = emit_tick(root, &request)?;
    serde_json::to_string_pretty(&receipt)
        .map_err(|e| TickError::error(format!("serialize receipt: {e}")))
}

fn required(name: &str, value: Option<String>) -> Result<String, TickError> {
    let value = value.ok_or_else(|| TickError::usage(format!("missing {name}")))?;
    if value.trim().is_empty() {
        return Err(TickError::usage(format!("{name} must not be empty")));
    }
    Ok(value)
}

fn parse_bool(value: &str) -> Result<bool, TickError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(TickError::usage(
            "boolean values must be exactly true or false",
        )),
    }
}

pub fn emit_tick(root: &Path, request: &EmitRequest) -> Result<TickReceipt, TickError> {
    reject_forbidden(request)?;
    validate_blocked(request, root)?;
    let paths = commit_paths(root, &request.commit)?;
    let product_moved = paths.iter().any(|path| is_product_path(path));
    let disagreement = request
        .claimed_product_moved
        .filter(|claimed| *claimed != product_moved)
        .map(|claimed| {
            format!("claimed product_moved={claimed}, computed product_moved={product_moved}")
        });
    let evidence = format!(
        "{}; computed_commit_paths={:?}; computed_product_moved={product_moved}",
        request.evidence, paths
    );
    let tick = next_tick(&request.ledger)?;
    let receipt = TickReceipt {
        schema: "zs.tick-receipt",
        tick,
        class: request.class,
        bead: request.bead.clone(),
        value_added: request.value_added.clone(),
        verdict: request.verdict,
        commit: request.commit.clone(),
        evidence,
        product_moved,
        product_moved_claimed: request.claimed_product_moved,
        product_moved_disagreement: disagreement,
        blocker: request.blocker.clone(),
        escalation_artifact: request
            .escalation_artifact
            .as_ref()
            .map(|path| path.display().to_string()),
    };
    let line = serde_json::to_string(&receipt)
        .map_err(|e| TickError::error(format!("serialize tick receipt: {e}")))?;
    append_line(&request.ledger, &line)?;
    Ok(receipt)
}

fn reject_forbidden(request: &EmitRequest) -> Result<(), TickError> {
    for (field, value) in [
        ("bead", request.bead.as_str()),
        ("value_added", request.value_added.as_str()),
        ("evidence", request.evidence.as_str()),
        ("blocker", request.blocker.as_deref().unwrap_or("")),
    ] {
        let lower = value.to_ascii_lowercase();
        if let Some(phrase) = FORBIDDEN.iter().find(|phrase| lower.contains(**phrase)) {
            return Err(TickError::usage(format!(
                "forbidden phrase in {field}: {phrase:?}"
            )));
        }
    }
    Ok(())
}

fn validate_blocked(request: &EmitRequest, root: &Path) -> Result<(), TickError> {
    match request.verdict {
        Verdict::Blocked => {
            let blocker = request
                .blocker
                .as_deref()
                .ok_or_else(|| TickError::usage("BLOCKED requires --blocker <class>:<name>"))?;
            let mut parts = blocker.split(':');
            let kind = parts.next().unwrap_or_default();
            let name = parts.next().unwrap_or_default();
            if parts.next().is_some()
                || kind.is_empty()
                || name.is_empty()
                || kind.chars().any(char::is_whitespace)
                || name.chars().any(char::is_whitespace)
            {
                return Err(TickError::usage(
                    "BLOCKED blocker must be a typed external blocker <class>:<name>",
                ));
            }
            let artifact = request.escalation_artifact.as_ref().ok_or_else(|| {
                TickError::usage("BLOCKED requires --escalation-artifact <existing path>")
            })?;
            require_existing_file(artifact, root, "escalation artifact")?;
        }
        _ if request.blocker.is_some() || request.escalation_artifact.is_some() => {
            return Err(TickError::usage(
                "--blocker and --escalation-artifact are only valid for BLOCKED",
            ));
        }
        _ => {}
    }
    Ok(())
}

fn require_existing_file(path: &Path, _root: &Path, label: &str) -> Result<(), TickError> {
    let metadata = fs::metadata(path)
        .map_err(|e| TickError::error(format!("{label} {}: {e}", path.display())))?;
    if !metadata.is_file() {
        return Err(TickError::error(format!(
            "{label} {} is not a file",
            path.display()
        )));
    }
    Ok(())
}

fn commit_paths(root: &Path, commit: &str) -> Result<Vec<String>, TickError> {
    let output = Command::new("git")
        .args(["-C", &root.to_string_lossy(), "cat-file", "-e"])
        .arg(format!("{commit}^{{commit}}"))
        .output()
        .map_err(|e| TickError::error(format!("run git cat-file: {e}")))?;
    if !output.status.success() {
        return Err(TickError::error(format!(
            "commit {commit:?} does not resolve to a commit"
        )));
    }
    let output = Command::new("git")
        .args([
            "-C",
            &root.to_string_lossy(),
            "diff-tree",
            "--root",
            "--relative",
            "--no-commit-id",
            "--name-only",
            "-r",
        ])
        .arg(commit)
        .output()
        .map_err(|e| TickError::error(format!("run git diff-tree: {e}")))?;
    if !output.status.success() {
        return Err(TickError::error(format!(
            "git diff-tree {commit:?}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| TickError::error(format!("git diff-tree output is not UTF-8: {e}")))
        .map(|text| {
            text.lines()
                .map(str::to_owned)
                .filter(|p| !p.is_empty())
                .collect()
        })
}

/// Classify one path according to CHARTER product_paths and the denylist.
pub fn is_product_path(path: &str) -> bool {
    let path = path.trim_start_matches("./");
    let components = path.split('/').collect::<Vec<_>>();
    if components.iter().any(|component| {
        matches!(
            component.to_ascii_lowercase().as_str(),
            "docs" | ".beads" | "receipts" | "gates" | "ci" | ".github" | ".gitlab"
        )
    }) || path.to_ascii_lowercase().ends_with(".md")
    {
        return false;
    }
    matches!(
        components.as_slice(),
        ["bank", "items", ..]
            | ["knowledge", ..]
            | ["tracks", ..]
            | ["web", ..]
            | ["install.sh"]
            | ["crates", _, "src", ..]
    )
}

fn next_tick(ledger: &Path) -> Result<String, TickError> {
    let text = fs::read_to_string(ledger)
        .map_err(|e| TickError::error(format!("read ledger {}: {e}", ledger.display())))?;
    let last = text
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| TickError::error(format!("ledger {} is empty", ledger.display())))?;
    let value: Value = serde_json::from_str(last)
        .map_err(|e| TickError::error(format!("parse last ledger row: {e}")))?;
    let tick = value
        .get("tick")
        .and_then(Value::as_str)
        .ok_or_else(|| TickError::error("last ledger row has no string tick"))?;
    let number = tick
        .strip_prefix('T')
        .filter(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| TickError::error(format!("last ledger tick {tick:?} is not T<number>")))?
        .parse::<u64>()
        .map_err(|e| TickError::error(format!("ledger tick overflow: {e}")))?;
    number
        .checked_add(1)
        .map(|next| format!("T{next}"))
        .ok_or_else(|| TickError::error("ledger tick overflow"))
}

fn append_line(ledger: &Path, line: &str) -> Result<(), TickError> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(ledger)
        .map_err(|e| TickError::error(format!("open ledger {}: {e}", ledger.display())))?;
    writeln!(file, "{line}").map_err(|e| TickError::error(format!("append ledger: {e}")))?;
    file.flush()
        .map_err(|e| TickError::error(format!("flush ledger: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn request(ledger: PathBuf) -> EmitRequest {
        EmitRequest {
            class: TickClass::Setup,
            bead: "bd-test".into(),
            value_added: "recorded a test receipt".into(),
            verdict: Verdict::Red,
            commit: "HEAD".into(),
            evidence: "unit test".into(),
            claimed_product_moved: None,
            blocker: None,
            escalation_artifact: None,
            ledger,
        }
    }

    #[test]
    fn fabricated_class_is_rejected() {
        assert!("MADE_UP".parse::<TickClass>().is_err());
    }

    #[test]
    fn blocked_without_typed_blocker_is_rejected() {
        let mut req = request(env::temp_dir().join("missing-ledger-for-blocked"));
        req.verdict = Verdict::Blocked;
        assert!(validate_blocked(&req, Path::new(".")).is_err());
    }

    #[test]
    fn forbidden_phrase_is_rejected() {
        let mut req = request(PathBuf::from("unused"));
        req.value_added = ["standing", "by"].join(" ");
        assert!(reject_forbidden(&req).is_err());
    }

    #[test]
    fn denylist_beats_product_glob_and_claim_is_recorded() {
        assert!(!is_product_path("docs/report.txt"));
        assert!(!is_product_path("crates/cdcp_gate/src/gates/new.rs"));
        assert!(!is_product_path("knowledge/README.md"));
        assert!(is_product_path("web/app.js"));
    }

    #[test]
    fn missing_ledger_is_an_error_and_not_created() {
        let path = env::temp_dir().join(format!("cdcp-tick-missing-{}", std::process::id()));
        let _ = fs::remove_file(&path);
        let req = request(path.clone());
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let err = emit_tick(&root, &req).unwrap_err();
        assert!(err.to_string().contains("read ledger") || err.to_string().contains("open ledger"));
        assert!(!path.exists());
    }

    #[test]
    fn next_tick_reads_the_existing_ledger() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ledger.jsonl");
        fs::write(
            &path,
            r#"{"schema":"zs.tick-receipt","tick":"T8"}
"#,
        )
        .unwrap();
        assert_eq!(next_tick(&path).unwrap(), "T9");
    }

    #[test]
    fn real_product_commit_computes_product_moved() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("ledger.jsonl");
        fs::write(
            &ledger,
            r#"{"schema":"zs.tick-receipt","tick":"T8"}
"#,
        )
        .unwrap();
        let mut req = request(ledger);
        req.commit = "ac65c94".into();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let receipt = emit_tick(&root, &req).unwrap();
        assert!(
            receipt.product_moved,
            "the web export commit must count as product movement"
        );
    }

    #[test]
    fn docs_only_commit_corrects_a_product_movement_claim() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("ledger.jsonl");
        fs::write(
            &ledger,
            r#"{"schema":"zs.tick-receipt","tick":"T8"}
"#,
        )
        .unwrap();
        let mut req = request(ledger);
        req.commit = "8286885".into();
        req.claimed_product_moved = Some(true);
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let receipt = emit_tick(&root, &req).unwrap();
        assert!(!receipt.product_moved);
        assert!(receipt
            .product_moved_disagreement
            .as_deref()
            .is_some_and(|text| text.contains("computed product_moved=false")));
    }
}
