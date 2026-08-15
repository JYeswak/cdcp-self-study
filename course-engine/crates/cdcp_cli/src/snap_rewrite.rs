//! Snapshot / restore_safe CHARTER rewriter.
//!
//! Not a gate. `scripts/check.sh` L4 isolation used to spawn `python3 -c`
//! to (a) swap exactly one sentinel and (b) rewrite the two CHARTER legs —
//! skip the snapshot `exec`, then hollow out the isolation assert.
//! `scripts/restore_safe.inc.sh` used the same interpreter to weaken the
//! prove-rebuild mtime check (CHARTER-NEEDLE + -CHECK, next `if [ `).
//! Those jobs live here so the shell bodies have no live python3.
//! A rewrite that hits 0 or N≠1 targets is RED: silent no-op is how a
//! CHARTER pair lies.
//!
//! EXTRACT-THEN-DELETE (`bd-extract-checksh-charter-python-w8a8`,
//! `bd-extract-restore-safe-python-iiv8`).

use std::fs;
use std::path::Path;

/// Marker the production script tags on the snapshot `exec` line.
const EXEC_TAIL: &str = "CHARTER-NEEDLE-EXEC";
/// Token that distinguishes the real exec from a comment that only names it.
const EXEC_INVOKE: &str = "exec sh";
/// No-op the skip-exec leg writes in place of the exec line.
const EXEC_DISABLED: &str = ":   # CHARTER-NEEDLE-EXEC";
/// Left-trimmed opening of the live isolation assert (grep body).
const ASSERT_LIVE: &str = "snapshot_probe_assert() { grep";
/// Hollow assert the delete-assert leg writes; isolation then "passes".
const ASSERT_HOLLOW: &str = "snapshot_probe_assert() { return 0; }";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Job {
    /// Require `from` to occur once; write the first occurrence as `to`.
    SwapOnce {
        from: String,
        to: String,
    },
    Charter(CharterLeg),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CharterLeg {
    /// Disable the snapshot re-exec so a sheared source is what runs.
    SkipExec,
    /// Delete the isolation assert so a sheared source can still go GREEN.
    DeleteAssert,
    /// restore_safe prove-rebuild CHARTER: weaken the marked `if [ ` test.
    WeakenIf,
}

pub(crate) fn parse_leg(raw: &str) -> Result<CharterLeg, String> {
    match raw {
        "skip-exec" => Ok(CharterLeg::SkipExec),
        "delete-assert" => Ok(CharterLeg::DeleteAssert),
        "weaken-if" => Ok(CharterLeg::WeakenIf),
        other => Err(format!(
            "snap-rewrite: unknown charter kind {other:?} (want skip-exec, delete-assert, or weaken-if)"
        )),
    }
}

pub(crate) fn apply(path: &Path, job: &Job) -> Result<(), String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("snap-rewrite: cannot read {}: {e}", path.display()))?;
    let next = match job {
        Job::SwapOnce { from, to } => swap_once(&raw, from, to, path)?,
        Job::Charter(leg) => rewrite_charter(&raw, *leg, path)?,
    };
    fs::write(path, next)
        .map_err(|e| format!("snap-rewrite: cannot write {}: {e}", path.display()))?;
    Ok(())
}

fn swap_once(src: &str, from: &str, to: &str, path: &Path) -> Result<String, String> {
    if from.is_empty() {
        return Err("snap-rewrite: empty --from (a blank needle matches everything)".into());
    }
    let hits = src.matches(from).count();
    if hits != 1 {
        return Err(format!(
            "snap-rewrite: --from occurs {hits} time(s) in {} (want 1)",
            path.display()
        ));
    }
    // `matches` already proved there is exactly one non-overlapping hit.
    Ok(src.replacen(from, to, 1))
}

fn rewrite_charter(src: &str, leg: CharterLeg, path: &Path) -> Result<String, String> {
    match leg {
        CharterLeg::WeakenIf => weaken_if(src, path),
        CharterLeg::SkipExec | CharterLeg::DeleteAssert => rewrite_line_charter(src, leg, path),
    }
}

fn rewrite_line_charter(src: &str, leg: CharterLeg, path: &Path) -> Result<String, String> {
    let had_trailing_nl = src.ends_with('\n');
    let mut out = String::with_capacity(src.len() + 8);
    let mut hits = 0usize;
    for (i, line) in src.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if !row_matches(line, leg) {
            out.push_str(line);
            continue;
        }
        hits += 1;
        match leg {
            CharterLeg::SkipExec => out.push_str(EXEC_DISABLED),
            CharterLeg::DeleteAssert => {
                let pad = line.len() - line.trim_start().len();
                out.push_str(&line[..pad]);
                out.push_str(ASSERT_HOLLOW);
            }
            CharterLeg::WeakenIf => unreachable!("weaken-if uses weaken_if, not the line loop"),
        }
    }
    if hits != 1 {
        return Err(format!(
            "snap-rewrite: charter {leg:?} matched {hits} line(s) in {} (want 1)",
            path.display()
        ));
    }
    if had_trailing_nl {
        out.push('\n');
    }
    Ok(out)
}

fn row_matches(line: &str, leg: CharterLeg) -> bool {
    match leg {
        CharterLeg::SkipExec => {
            let tail = line.trim_end();
            tail.ends_with(EXEC_TAIL) && tail.contains(EXEC_INVOKE)
        }
        CharterLeg::DeleteAssert => line.trim_start().starts_with(ASSERT_LIVE),
        CharterLeg::WeakenIf => false,
    }
}

/// Marker is assembled so a comment that names the parts cannot match it.
/// Same construction the retired restore_safe python3 used.
fn weaken_mark() -> String {
    format!("{}{}", "CHARTER-NEEDLE", "-CHECK")
}

/// Find exactly one `CHARTER-NEEDLE-CHECK` line; rewrite the next `if [ `
/// test to `if false && [ `. Zero markers, two markers, or a marker whose
/// next line is not that test is RED — a no-op weaken is how the pair lies.
fn weaken_if(src: &str, path: &Path) -> Result<String, String> {
    let mark = weaken_mark();
    let had_trailing_nl = src.ends_with('\n');
    let rows: Vec<&str> = src.lines().collect();
    let mut out = String::with_capacity(src.len() + 16);
    let mut hits = 0usize;
    let mut mutated = false;
    let mut i = 0usize;
    while i < rows.len() {
        if i > 0 {
            out.push('\n');
        }
        let line = rows[i];
        if !line.contains(&mark) {
            out.push_str(line);
            i += 1;
            continue;
        }
        hits += 1;
        out.push_str(line);
        i += 1;
        if i < rows.len() {
            out.push('\n');
            let next = rows[i];
            if next.trim_start().starts_with("if [ ") {
                out.push_str(&next.replacen("if [ ", "if false && [ ", 1));
                mutated = true;
            } else {
                out.push_str(next);
            }
            i += 1;
        }
    }
    if hits != 1 {
        return Err(format!(
            "snap-rewrite: charter WeakenIf matched {hits} marker line(s) in {} (want 1)",
            path.display()
        ));
    }
    if !mutated {
        return Err(format!(
            "snap-rewrite: charter WeakenIf found the marker but the next line is not an `if [ ` test in {}",
            path.display()
        ));
    }
    if had_trailing_nl {
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "cdcp_snap_rewrite_{}_{}_{name}",
            std::process::id(),
            nanos
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("scratch dir");
        dir.join("script.sh")
    }

    fn wipe(path: &Path) {
        if let Some(dir) = path.parent() {
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn swap_once_rewrites_the_single_needle() {
        let path = scratch("swap_ok");
        fs::write(&path, "alpha INTACT omega\n").unwrap();
        apply(
            &path,
            &Job::SwapOnce {
                from: "INTACT".into(),
                to: "SHEARED".into(),
            },
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "alpha SHEARED omega\n");
        wipe(&path);
    }

    #[test]
    fn swap_once_zero_hits_is_red() {
        let path = scratch("swap_zero");
        fs::write(&path, "nothing to swap\n").unwrap();
        let err = apply(
            &path,
            &Job::SwapOnce {
                from: "INTACT".into(),
                to: "SHEARED".into(),
            },
        )
        .unwrap_err();
        assert!(err.contains("0 time"), "{err}");
        assert_eq!(fs::read_to_string(&path).unwrap(), "nothing to swap\n");
        wipe(&path);
    }

    #[test]
    fn swap_once_two_hits_is_red() {
        let path = scratch("swap_two");
        fs::write(&path, "INTACT and INTACT\n").unwrap();
        let err = apply(
            &path,
            &Job::SwapOnce {
                from: "INTACT".into(),
                to: "SHEARED".into(),
            },
        )
        .unwrap_err();
        assert!(err.contains("2 time"), "{err}");
        assert_eq!(fs::read_to_string(&path).unwrap(), "INTACT and INTACT\n");
        wipe(&path);
    }

    #[test]
    fn empty_from_is_red() {
        let path = scratch("empty_from");
        fs::write(&path, "x\n").unwrap();
        let err = apply(
            &path,
            &Job::SwapOnce {
                from: String::new(),
                to: "y".into(),
            },
        )
        .unwrap_err();
        assert!(err.contains("empty --from"), "{err}");
        wipe(&path);
    }

    #[test]
    fn skip_exec_disables_the_tagged_exec_line() {
        let path = scratch("skip_ok");
        fs::write(
            &path,
            "#!/bin/sh\n\
             exec sh \"$copy\" \"$@\" || die   # CHARTER-NEEDLE-EXEC\n\
             echo after\n",
        )
        .unwrap();
        apply(&path, &Job::Charter(CharterLeg::SkipExec)).unwrap();
        let got = fs::read_to_string(&path).unwrap();
        assert!(got.contains(EXEC_DISABLED), "{got}");
        assert!(!got.contains(EXEC_INVOKE), "{got}");
        assert!(got.contains("echo after"), "{got}");
        wipe(&path);
    }

    #[test]
    fn skip_exec_ignores_a_comment_that_only_names_the_marker() {
        let path = scratch("skip_comment");
        fs::write(
            &path,
            "# CHARTER-NEEDLE-EXEC\n\
             echo stay\n",
        )
        .unwrap();
        let err = apply(&path, &Job::Charter(CharterLeg::SkipExec)).unwrap_err();
        assert!(err.contains("matched 0"), "{err}");
        wipe(&path);
    }

    #[test]
    fn delete_assert_hollows_the_grep_body() {
        let path = scratch("del_ok");
        fs::write(
            &path,
            "  snapshot_probe_assert() { grep -q SHEARED \"$1\" && return 1; return 0; }\n\
             echo stay\n",
        )
        .unwrap();
        apply(&path, &Job::Charter(CharterLeg::DeleteAssert)).unwrap();
        let got = fs::read_to_string(&path).unwrap();
        assert!(
            got.contains("  snapshot_probe_assert() { return 0; }"),
            "{got}"
        );
        assert!(!got.contains("grep -q"), "{got}");
        wipe(&path);
    }

    #[test]
    fn delete_assert_two_hits_is_red() {
        let path = scratch("del_two");
        fs::write(
            &path,
            "snapshot_probe_assert() { grep a; }\n\
             snapshot_probe_assert() { grep b; }\n",
        )
        .unwrap();
        let err = apply(&path, &Job::Charter(CharterLeg::DeleteAssert)).unwrap_err();
        assert!(err.contains("matched 2"), "{err}");
        wipe(&path);
    }

    #[test]
    fn missing_file_is_red() {
        let path = scratch("missing");
        let _ = fs::remove_file(&path);
        let err = apply(
            &path,
            &Job::SwapOnce {
                from: "a".into(),
                to: "b".into(),
            },
        )
        .unwrap_err();
        assert!(err.contains("cannot read"), "{err}");
        wipe(&path);
    }

    #[test]
    fn parse_leg_rejects_unknown() {
        let err = parse_leg("hollow-assert").unwrap_err();
        assert!(err.contains("unknown charter kind"), "{err}");
        assert!(err.contains("weaken-if"), "{err}");
        assert_eq!(parse_leg("skip-exec").unwrap(), CharterLeg::SkipExec);
        assert_eq!(
            parse_leg("delete-assert").unwrap(),
            CharterLeg::DeleteAssert
        );
        assert_eq!(parse_leg("weaken-if").unwrap(), CharterLeg::WeakenIf);
    }

    #[test]
    fn weaken_if_rewrites_the_next_test() {
        let path = scratch("weaken_ok");
        // Assemble the marker the same way the retired python3 did so this
        // fixture comment cannot become a second hit.
        let mark = format!("{}{}", "CHARTER-NEEDLE", "-CHECK");
        fs::write(
            &path,
            format!(
                "  # {mark} — mutate the next test\n\
                 if [ \"$_pr_after\" -eq \"$_pr_before\" ]; then\n\
                 echo stay\n"
            ),
        )
        .unwrap();
        apply(&path, &Job::Charter(CharterLeg::WeakenIf)).unwrap();
        let got = fs::read_to_string(&path).unwrap();
        assert!(
            got.contains("if false && [ \"$_pr_after\" -eq \"$_pr_before\" ]; then"),
            "{got}"
        );
        assert!(got.contains("echo stay"), "{got}");
        assert!(
            !got.contains("\nif [ \"$_pr_after\""),
            "live if-test must be gone: {got}"
        );
        wipe(&path);
    }

    #[test]
    fn weaken_if_zero_markers_is_red() {
        let path = scratch("weaken_zero");
        fs::write(&path, "if [ 1 -eq 1 ]; then\necho stay\n").unwrap();
        let err = apply(&path, &Job::Charter(CharterLeg::WeakenIf)).unwrap_err();
        assert!(err.contains("matched 0"), "{err}");
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "if [ 1 -eq 1 ]; then\necho stay\n"
        );
        wipe(&path);
    }

    #[test]
    fn weaken_if_two_markers_is_red() {
        let path = scratch("weaken_two");
        let mark = format!("{}{}", "CHARTER-NEEDLE", "-CHECK");
        fs::write(
            &path,
            format!(
                "# {mark}\n\
                 if [ 1 -eq 1 ]; then\n\
                 # {mark}\n\
                 if [ 2 -eq 2 ]; then\n"
            ),
        )
        .unwrap();
        let err = apply(&path, &Job::Charter(CharterLeg::WeakenIf)).unwrap_err();
        assert!(err.contains("matched 2"), "{err}");
        wipe(&path);
    }

    #[test]
    fn weaken_if_marker_without_if_test_is_red() {
        let path = scratch("weaken_no_if");
        let mark = format!("{}{}", "CHARTER-NEEDLE", "-CHECK");
        let body = format!("# {mark}\necho not-an-if\n");
        fs::write(&path, &body).unwrap();
        let err = apply(&path, &Job::Charter(CharterLeg::WeakenIf)).unwrap_err();
        assert!(err.contains("not an `if [ ` test"), "{err}");
        assert_eq!(fs::read_to_string(&path).unwrap(), body);
        wipe(&path);
    }
}
