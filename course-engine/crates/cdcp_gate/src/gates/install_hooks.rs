//! install-hooks — copy the committed `hooks/` shims into this clone's git dir.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! A hook that exists on one machine is not a gate. This subcommand makes the
//! committed shim the installed shim, and `--check` reports when the two have
//! drifted. It does not stop anyone from deleting `.git/hooks/pre-commit`
//! afterwards, and `git commit --no-verify` skips hooks entirely — those are
//! outside what any hook installer can reach. What it removes is the failure mode
//! where the shim was never installed at all, which is the common one.
//!
//! The installer is Rust for the same reason the guard is: a shell installer for
//! a shell-banning gate is the joke this migration exists to stop telling.

use crate::registry::{GateCtx, GateError};
use crate::vcs;
use std::fs;
use std::path::Path;

pub const NAME: &str = "install-hooks";
pub const SUMMARY: &str = "install (or --check) the committed hooks/ shims into this clone's git dir";

const KNOWN_FLAGS: &[&str] = &["--check", "--force", "--quiet"];

/// Shims this gate manages: (source under `hooks/`, git hook name).
pub const MANAGED: &[(&str, &str)] = &[("hooks/pre-commit", "pre-commit")];

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Installed,
    Missing,
    Drifted,
}

pub fn state_of(target: &Path, want: &str) -> State {
    match fs::read_to_string(target) {
        Err(_) => State::Missing,
        Ok(have) if have == want => State::Installed,
        Ok(_) => State::Drifted,
    }
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(KNOWN_FLAGS)?;
    let check_only = ctx.has_flag("--check");
    let force = ctx.has_flag("--force");
    let quiet = ctx.has_flag("--quiet");
    let root = &ctx.root;

    if !vcs::is_repo(root) {
        return Err(GateError::error(format!(
            "{} is not inside a git working tree",
            root.display()
        )));
    }
    let hooks_dir = vcs::common_dir(root).map_err(GateError::error)?.join("hooks");

    if MANAGED.is_empty() {
        return Err(GateError::error(
            "zero managed hooks — an installer that installs nothing is an ERROR, not a pass",
        ));
    }

    let mut problems = Vec::new();
    let mut installed = 0usize;

    for (src_rel, hook_name) in MANAGED {
        let src = root.join(src_rel);
        let want = fs::read_to_string(&src)
            .map_err(|e| GateError::error(format!("read {}: {e}", src.display())))?;
        if want.trim().is_empty() {
            return Err(GateError::error(format!(
                "{src_rel} is empty — an empty shim installs as a no-op that reports like a pass"
            )));
        }
        let target = hooks_dir.join(hook_name);
        let state = state_of(&target, &want);

        if check_only {
            match state {
                State::Installed => {}
                State::Missing => problems.push(format!(
                    "{}: NOT INSTALLED (BUILT != WIRED). Run: cargo run -q -p cdcp_gate -- install-hooks",
                    target.display()
                )),
                State::Drifted => problems.push(format!(
                    "{}: differs from the committed {src_rel}. Run: cargo run -q -p cdcp_gate -- install-hooks --force",
                    target.display()
                )),
            }
            continue;
        }

        match state {
            State::Installed => {
                if !quiet {
                    println!("{NAME}: ok: {} already matches {src_rel}", target.display());
                }
            }
            State::Drifted if !force => problems.push(format!(
                "{} exists and differs from {src_rel}; refusing to overwrite another tool's hook. Inspect it, then re-run with --force",
                target.display()
            )),
            _ => {
                fs::create_dir_all(&hooks_dir)
                    .map_err(|e| GateError::error(format!("mkdir {}: {e}", hooks_dir.display())))?;
                fs::write(&target, &want)
                    .map_err(|e| GateError::error(format!("write {}: {e}", target.display())))?;
                make_executable(&target).map_err(GateError::error)?;
                installed += 1;
                if !quiet {
                    println!("{NAME}: installed {src_rel} -> {}", target.display());
                }
            }
        }
    }

    if !problems.is_empty() {
        return Err(GateError::Violation(problems));
    }
    if !quiet && !check_only {
        println!("{NAME}: ok: {} managed hook(s), {installed} written", MANAGED.len());
    }
    if !quiet && check_only {
        println!("{NAME}: ok: all {} managed hook(s) installed and current", MANAGED.len());
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(p: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perm = fs::metadata(p)
        .map_err(|e| format!("stat {}: {e}", p.display()))?
        .permissions();
    perm.set_mode(0o755);
    fs::set_permissions(p, perm).map_err(|e| format!("chmod {}: {e}", p.display()))
}

#[cfg(not(unix))]
fn make_executable(_p: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_detection() {
        let td = tempfile::tempdir().unwrap();
        let t = td.path().join("pre-commit");
        assert_eq!(state_of(&t, "body"), State::Missing);
        fs::write(&t, "body").unwrap();
        assert_eq!(state_of(&t, "body"), State::Installed);
        fs::write(&t, "other").unwrap();
        assert_eq!(state_of(&t, "body"), State::Drifted);
    }

    #[test]
    fn manages_at_least_one_hook() {
        assert!(!MANAGED.is_empty());
    }

    #[test]
    fn committed_shim_exists_and_is_thin() {
        // The shim must contain no decision logic: no conditionals, no loops, no
        // classification of files. Its whole body is one exec of the Rust binary.
        let root = crate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        for (src_rel, _) in MANAGED {
            let body = fs::read_to_string(root.join(src_rel)).unwrap();
            let code: Vec<&str> = body
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();
            for l in &code {
                for banned in ["&&", "||", "$(", "`", ";", "|"] {
                    assert!(
                        !l.contains(banned),
                        "{src_rel} line {l:?} contains {banned:?} — decision logic does not live in the shim"
                    );
                }
                for word in l.split_whitespace() {
                    for banned in [
                        "if", "then", "else", "elif", "fi", "case", "esac", "for", "while", "until",
                        "test", "[", "[[", "grep", "sed", "awk", "python", "python3",
                    ] {
                        assert!(
                            word != banned,
                            "{src_rel} line {l:?} uses {banned:?} — decision logic does not live in the shim"
                        );
                    }
                }
            }
            assert!(
                code.iter().any(|l| l.starts_with("exec ")),
                "{src_rel} must hand off with exec"
            );
            assert!(
                code.iter().any(|l| l.contains("cdcp_gate")),
                "{src_rel} must invoke the Rust gate binary"
            );
        }
    }
}
