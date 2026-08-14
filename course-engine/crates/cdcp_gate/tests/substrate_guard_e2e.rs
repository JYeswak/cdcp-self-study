//! End-to-end: the guard binary against real git repos.
//!
//! Both legs are mandatory. The KNOWN-BAD leg establishes that the gate trips.
//! The KNOWN-GOOD leg establishes that it does not trip on the ordinary day — an
//! over-strict gate gets routed around, which is a slower death than no gate.
//!
//! Two families of known-bad live here, both confirmed by injection 2026-08-14
//! before the fix:
//!
//! * **bd-how** — the guard read candidate paths from the INDEX and the allowlist
//!   from the WORKING TREE, so a staged `.py` whose `[[allow]]` row was left
//!   unstaged passed at exit 0. The commit then carried the file and not the row.
//! * **bd-bo6i** — "wired" was a substring test, so `: "cargo run …"`,
//!   `true # cargo run …` and `cargo run … || true` all certified `wired=yes`.
//!   The text tests here only ever SUBTRACT; the assertion that check.sh actually
//!   stops is `--prove-wired`, which runs check.sh against a planted known-bad.

mod support;
use support::{good_row, header, Fixture, BIN};

const OK: i32 = 0;
const VIOLATION: i32 = 2;
const ERROR: i32 = 4;

// ─────────────────────────── KNOWN-GOOD ───────────────────────────────────

#[test]
fn good_baseline_passes() {
    let f = Fixture::new();
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(code, OK, "{out}");
    assert!(out.contains("ok:"), "{out}");
}

#[test]
fn good_allowlisted_file_with_a_valid_reason_passes() {
    let f = Fixture::new();
    f.write("scripts/verify_coverage.py", "print('coverage')\n");
    f.set_allowlist(&(f.read_allowlist() + &good_row("scripts/verify_coverage.py")));
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, OK, "{out}");
}

/// The ordinary workflow bd-how must not make harder: the file and its row go in
/// together, so BOTH snapshots agree, so the gate stays quiet.
#[test]
fn good_staging_the_file_and_its_row_together_passes() {
    let f = Fixture::new();
    f.write("scripts/payload.py", "print('payload')\n");
    f.set_allowlist(&(f.read_allowlist() + &good_row("scripts/payload.py")));
    f.git(&[
        "add",
        "scripts/payload.py",
        "registries/substrate_allowlist.toml",
    ]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(
        code, OK,
        "the normal way to add a reasoned script must stay green: {out}"
    );
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(code, OK, "{out}");
}

#[test]
fn good_editing_an_existing_allowlisted_file_does_not_trip_it() {
    let f = Fixture::new();
    f.write("scripts/verify_bank.py", "print('bank v2 — edited')\n");
    f.git(&["add", "scripts/verify_bank.py"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(
        code, OK,
        "editing an allowlisted script must stay green: {out}"
    );
}

#[test]
fn good_adding_a_rust_file_anywhere_passes() {
    let f = Fixture::new();
    f.write(
        "crates/cdcp_gate/src/gates/verify_orphans.rs",
        "pub fn run() {}\n",
    );
    f.write("scripts/helper.rs", "fn main() {}\n");
    f.write("build.rs", "fn main() {}\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(
        code, OK,
        "the migration's own output must never be blocked: {out}"
    );
}

#[test]
fn good_non_scanned_surfaces_pass() {
    // bd-xmn5 SPLIT THIS TEST. It used to assert that docs/notes.py and
    // tests/voice-slop.sh PASS, which was true and was the bug: both were
    // measured at exit 0 while `scripts/check.sh` was invoking two real shell
    // gates from tests/. What survives here is the half that is still the rule —
    // the floor is the shell/python family, not a dragnet over every file.
    let f = Fixture::new();
    f.write("scripts/smoke.mjs", "console.log(1)\n");
    f.write("docs/notes.md", "# notes\n");
    f.write("web/data/x.json", "{}\n");
    f.write("web/assets/js/app.js", "console.log(1)\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, OK, "the gate is a floor, not a dragnet: {out}");
}

/// The other half, inverted. MEASURED against the built binary in a clone before
/// the fix: `docs/payload.py` staged -> exit 0, `tests/payload.sh` staged ->
/// exit 0.
#[test]
fn bad_a_script_outside_scripts_and_crates_is_red_now() {
    for p in [
        "docs/payload.py",
        "tests/payload.sh",
        ".flywheel/payload.sh",
        "web/payload.py",
    ] {
        let f = Fixture::new();
        f.write(p, "x = 1\n");
        f.git(&["add", "-A"]);
        let (code, out) = f.gate(&["substrate-guard", "--staged"]);
        assert_eq!(code, VIOLATION, "{p}: {out}");
        assert!(out.contains(p), "must name it: {out}");
    }
}

/// MEASURED before the fix, staged in a clone: `scripts/payload.PY` -> exit 0,
/// `scripts/payload.Py` -> exit 0, `scripts/payload.bash` -> exit 0,
/// `scripts/payload.zsh` -> exit 0. A rule a rename defeats is not a rule.
#[test]
fn bad_an_upper_case_or_other_family_extension_is_red_now() {
    for p in [
        "scripts/payload.PY",
        "scripts/payload.Py",
        "stray.SH",
        "scripts/payload.bash",
        "scripts/payload.zsh",
        "scripts/payload.ksh",
        "scripts/payload.pyw",
    ] {
        let f = Fixture::new();
        f.write(p, "x = 1\n");
        f.git(&["add", "-A"]);
        let (code, out) = f.gate(&["substrate-guard", "--staged"]);
        assert_eq!(code, VIOLATION, "{p}: {out}");
        assert!(out.contains(p), "must name it: {out}");
    }
}

/// MEASURED before the fix: `scripts/payload`, no extension, first line
/// `#!/usr/bin/env python3`, staged -> exit 0.
#[test]
fn bad_an_extensionless_shebang_script_is_red_now() {
    let f = Fixture::new();
    f.write("scripts/payload", "#!/usr/bin/env python3\nprint('pwn')\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("scripts/payload"), "{out}");
    assert!(out.contains("executable text script"), "{out}");
}

/// KNOWN-GOOD for the same leg: an extensionless DATA file is not a script, and
/// a gate that cannot tell them apart gets routed around.
#[test]
fn good_an_extensionless_data_file_is_not_a_script() {
    let f = Fixture::new();
    f.write("LICENSE", "All rights reserved.\n");
    f.write("scripts/FIXTURES", "one\ntwo\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, OK, "{out}");
}

/// The shebang leg reads BYTES. This repo tracks a mode-100755 `.wasm`; asking
/// "are the first two bytes `#!`" must not become an ERROR over a UTF-8 decode.
#[test]
fn good_a_tracked_executable_binary_is_not_an_error() {
    let f = Fixture::new();
    std::fs::write(
        f.path("scripts/blob.bin"),
        [0u8, 0x61, 0x73, 0x6d, 0xff, 0xfe],
    )
    .unwrap();
    f.git(&["add", "-A"]);
    f.git(&["update-index", "--chmod=+x", "scripts/blob.bin"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, OK, "a binary is not a shebang script: {out}");
}

/// MEASURED before the fix: a tracked directory symlink `scripts/linkdir` ->
/// an outside directory holding `hidden.py` was staged at exit 0, and
/// `hidden.py` was readable through it on disk. git reports mode 120000 and one
/// path; the tree beneath is not in this repository.
#[cfg(unix)]
#[test]
fn bad_a_tracked_symlink_is_red_because_the_gate_cannot_see_through_it() {
    let f = Fixture::new();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("hidden.py"), "print('hidden')\n").unwrap();
    std::os::unix::fs::symlink(outside.path(), f.path("scripts/linkdir")).unwrap();
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("scripts/linkdir"), "{out}");
    assert!(out.contains("SYMLINK"), "{out}");
}

/// ...and the bargain holds: a row clears it, because a row is a human saying
/// they looked at what the gate cannot.
#[cfg(unix)]
#[test]
fn good_a_tracked_symlink_with_a_row_passes() {
    let f = Fixture::new();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("hidden.py"), "print('hidden')\n").unwrap();
    std::os::unix::fs::symlink(outside.path(), f.path("scripts/linkdir")).unwrap();
    f.set_allowlist(&(f.read_allowlist() + &good_row("scripts/linkdir")));
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, OK, "{out}");
}

/// A row that exempts nothing is litter that reads like tracked debt.
#[test]
fn bad_a_row_for_a_file_the_gate_never_demands_is_dead_weight() {
    let f = Fixture::new();
    f.write("docs/notes.md", "# notes\n");
    f.set_allowlist(&(f.read_allowlist() + &good_row("docs/notes.md")));
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("dead weight"), "{out}");
}

/// ANTI-VACUOUS, and the leg the bead says `cargo build` does not have: the
/// presence scan must run against THIS repository under `cargo test`, not only
/// under `scripts/check.sh`. Measured 2026-08-14: neither
/// `cargo build --workspace` nor `cargo test --workspace` scanned the tree, so
/// an unlisted `.py` could be committed and every Rust test still passed.
#[test]
fn the_live_repo_tree_has_no_unlisted_non_rust_file() {
    use cdcp_gate::gates::substrate_guard as sg;
    let root = cdcp_gate::root::resolve(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("engine root");
    let text = std::fs::read_to_string(root.join(sg::REGISTRY_PATH)).expect("registry");
    let al = sg::parse_allowlist(&text).expect("parses");
    let entries = cdcp_gate::vcs::tracked_entries(&root).expect("git ls-files -s");
    assert!(
        entries.len() > 100,
        "scanned {} tracked files — a vacuous scan is an ERROR, not a pass",
        entries.len()
    );
    let entries: Vec<sg::Entry> = entries
        .into_iter()
        .map(|e| {
            let shebang = if sg::needs_content_probe(&e.path, &e.mode, &al.scan) {
                let bytes = std::fs::read(root.join(&e.path)).unwrap_or_default();
                sg::shebang_line(&bytes[..bytes.len().min(256)])
            } else {
                None
            };
            sg::Entry {
                path: e.path,
                mode: e.mode,
                shebang,
            }
        })
        .collect();
    let identified = entries
        .iter()
        .filter(|e| sg::scan_reason(e, &al.scan).is_some())
        .count();
    assert!(
        identified >= 40,
        "only {identified} entries identified as non-Rust — the scan found nothing to judge"
    );
    let v = sg::unlisted_entries(&entries, &al.allow, &al.scan);
    assert!(
        v.is_empty(),
        "{} unlisted non-Rust file(s) in this repo:\n  {}",
        v.len(),
        v.join("\n  ")
    );
}

#[test]
fn good_deleting_a_script_and_its_row_together_passes() {
    // The shrink-to-zero path: port to Rust, delete the script, delete the row.
    let f = Fixture::new();
    f.remove("scripts/verify_bank.py");
    f.set_allowlist(&(header("wired") + &good_row("scripts/check.sh")));
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, OK, "{out}");
}

/// Deleting a script and its row on the DESK, with nothing staged yet, is a
/// coherent working tree and a coherent index. Neither snapshot is a hybrid, so
/// neither goes RED. (Judging worktree candidates against the index allowlist —
/// bd-how's bug in mirror image — would fail this.)
#[test]
fn good_unstaged_deletion_of_a_script_and_its_row_passes() {
    let f = Fixture::new();
    f.remove("scripts/verify_bank.py");
    f.set_allowlist(&(header("wired") + &good_row("scripts/check.sh")));
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(code, OK, "{out}");
}

// ─────────────────────────── KNOWN-BAD ────────────────────────────────────

#[test]
fn bad_unlisted_python_is_red_and_names_the_file() {
    let f = Fixture::new();
    f.write("scripts/foo.py", "print('sneaked in')\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("scripts/foo.py"), "must name the file: {out}");
    assert!(
        out.contains("substrate_allowlist.toml"),
        "must name the remedy: {out}"
    );
}

#[test]
fn bad_unlisted_shell_is_red_and_names_the_file() {
    let f = Fixture::new();
    f.write("scripts/foo.sh", "echo sneaked in\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("scripts/foo.sh"), "{out}");
}

#[test]
fn bad_unlisted_at_engine_root_and_under_crates_is_red() {
    for path in ["stray.sh", "crates/cdcp_core/gen.py"] {
        let f = Fixture::new();
        f.write(path, "echo x\n");
        f.git(&["add", "-A"]);
        let (code, out) = f.gate(&["substrate-guard", "--staged"]);
        assert_eq!(code, VIOLATION, "{path}: {out}");
        assert!(out.contains(path), "{out}");
    }
}

// ───────── bd-how: the subject and the policy must share a snapshot ────────

/// THE bd-how REPRODUCTION. Stage the `.py`; leave its authorising row on the
/// desk. Measured at exit 0 on both legs before the fix — the approved tree had
/// never existed and never would.
#[test]
fn bad_row_left_unstaged_does_not_authorise_a_staged_script() {
    let f = Fixture::new();
    f.write("scripts/payload.py", "print('payload')\n");
    f.git(&["add", "scripts/payload.py"]);
    // The row exists — but only where the commit will not carry it.
    f.set_allowlist(&(f.read_allowlist() + &good_row("scripts/payload.py")));

    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(
        code, VIOLATION,
        "the commit would hold the file and not the row: {out}"
    );
    assert!(out.contains("scripts/payload.py"), "must name it: {out}");
    assert!(
        out.contains("this commit creates"),
        "must say WHICH snapshot is dirty: {out}"
    );

    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("scripts/payload.py"), "{out}");
}

/// The same hybrid in the other direction: the desk carries a wired check.sh
/// while the commit carries a neutered one. The hook used to read the desk copy.
#[test]
fn bad_neutered_check_sh_in_the_index_is_red_even_when_the_desk_copy_is_wired() {
    let f = Fixture::new();
    let wired = std::fs::read_to_string(f.path("scripts/check.sh")).unwrap();
    f.write("scripts/check.sh", "#!/bin/sh\necho nothing to see here\n");
    f.git(&["add", "scripts/check.sh"]);
    f.write("scripts/check.sh", &wired); // the desk still looks fine

    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(
        code, VIOLATION,
        "the commit removes the step; the desk copy is not the commit: {out}"
    );
    assert!(out.contains("BUILT != WIRED"), "{out}");
    assert!(out.contains("this commit creates"), "{out}");
}

/// An unstaged edit to the allowlist must not silently WIDEN what the commit
/// permits, and it must not silently NARROW it either: both snapshots are judged.
#[test]
fn bad_row_deleted_only_on_the_desk_is_red_for_the_working_tree() {
    let f = Fixture::new();
    // verify_bank.py stays tracked and on disk; its row leaves the desk copy.
    f.set_allowlist(&(header("wired") + &good_row("scripts/check.sh")));
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("scripts/verify_bank.py"), "{out}");
    assert!(out.contains("working tree only"), "{out}");
}

#[test]
fn bad_row_with_the_reason_stripped_is_a_schema_error() {
    let f = Fixture::new();
    let stripped = f
        .read_allowlist()
        .lines()
        .filter(|l| !l.starts_with("reason ="))
        .collect::<Vec<_>>()
        .join("\n");
    f.set_allowlist(&stripped);
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(
        code, ERROR,
        "a blank reason is an ERROR, not permission: {out}"
    );
    assert!(out.contains("reason"), "{out}");
}

#[test]
fn bad_row_with_an_empty_reason_string_is_a_schema_error() {
    let f = Fixture::new();
    let blanked = f.read_allowlist().replace(
        "reason = \"Grandfathered load-bearing gate; port tracked by the migration epic\"",
        "reason = \"\"",
    );
    f.set_allowlist(&blanked);
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(
        out.contains("never permission") || out.contains("empty `reason`"),
        "{out}"
    );
}

#[test]
fn bad_backdated_expires_is_red() {
    let f = Fixture::new();
    let backdated = f.read_allowlist().replace("2099-12-31", "2001-01-01");
    f.set_allowlist(&backdated);
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("EXPIRED"), "{out}");
}

#[test]
fn bad_registry_that_narrows_the_scan_is_an_error() {
    // The one-word disable attempt: drop "py" from the scanned extensions.
    let f = Fixture::new();
    let narrowed = f.read_allowlist().replace("\"py\", ", "");
    assert!(
        !narrowed.contains("\"py\""),
        "the fixture must actually drop py, or it tests nothing"
    );
    f.set_allowlist(&narrowed);
    f.write("scripts/foo.py", "print('now invisible?')\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("never narrow it"), "{out}");
}

#[test]
fn bad_stale_row_for_a_deleted_file_is_an_error() {
    let f = Fixture::new();
    f.remove("scripts/verify_bank.py");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("no file at this path"), "{out}");
}

#[test]
fn bad_missing_check_sh_step_is_red_when_wiring_is_declared_wired() {
    let f = Fixture::new();
    f.write("scripts/check.sh", "#!/bin/sh\necho nothing to see here\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, VIOLATION, "deleting the step must be noticed: {out}");
    assert!(out.contains("BUILT != WIRED"), "{out}");
}

// ─── bd-n1aj: a path the gate demands a row for must be able to have one ───
//
// REPRODUCED 2026-08-14 in a throwaway repo, before the fix, with a well-formed
// row present and the file tracked:
//   substrate-guard: ERROR: 1 schema error(s) in registries/substrate_allowlist.toml:
//   [[allow]] scripts/payload..py: `path` must be a normalised engine-root-relative path
//   exit 4 on the presence leg, exit 4 on --staged.
// Without the row the same file was exit 2 and named. So the gate demanded a row
// and then rejected every row that could satisfy it: the file could not be made
// green by complying. `scripts/a\b.py` measured identically — the `contains('\\')`
// leg of the same line. Fail-closed both ways, so the harm was never exposure; it
// was that the only way out was to route around the gate.

/// KNOWN-GOOD leg. Both files are ordinary sources in a mandatory root.
#[test]
fn good_a_filename_with_two_dots_or_a_backslash_can_be_allowlisted() {
    for path in ["scripts/payload..py", "scripts/a\\b.py", "weird..name.sh"] {
        let f = Fixture::new();
        f.write(path, "print('ordinary file, unusual name')\n");
        f.set_allowlist(&(f.read_allowlist() + &good_row(path)));
        f.git(&["add", "-A"]);

        let (code, out) = f.gate(&["substrate-guard", "--staged"]);
        assert_eq!(
            code, OK,
            "{path}: the gate demands a row for this file; the row must be accepted: {out}"
        );
        let (code, out) = f.gate(&["substrate-guard"]);
        assert_eq!(code, OK, "{path}: {out}");
    }
}

/// KNOWN-BAD leg, and the half that must NOT move: dropping the row is still RED,
/// by name. A widening that also stopped the file being caught would have traded
/// a trap for a hole.
#[test]
fn bad_a_filename_with_two_dots_and_no_row_is_still_red_and_named() {
    for path in ["scripts/payload..py", "scripts/a\\b.py", "weird..name.sh"] {
        let f = Fixture::new();
        f.write(path, "print('no row for me')\n");
        f.git(&["add", "-A"]);

        let (code, out) = f.gate(&["substrate-guard", "--staged"]);
        assert_eq!(code, VIOLATION, "{path}: {out}");
        assert!(out.contains(path), "{path} must be named: {out}");
        let (code, out) = f.gate(&["substrate-guard"]);
        assert_eq!(code, VIOLATION, "{path}: {out}");
        assert!(out.contains(path), "{out}");
    }
}

/// The widening is bounded: a `.`/`..` COMPONENT, and an absolute path, are still
/// malformed in a row. That is traversal, and a filename is not a traversal.
#[test]
fn bad_a_traversal_or_absolute_row_is_still_a_schema_error() {
    for path in [
        "../outside.py",
        "scripts/../../etc/passwd.sh",
        "scripts/./x.py",
        "/abs/path.py",
    ] {
        let f = Fixture::new();
        f.set_allowlist(&(f.read_allowlist() + &good_row(path)));
        f.git(&["add", "-A"]);
        let (code, out) = f.gate(&["substrate-guard", "--staged"]);
        assert_eq!(code, ERROR, "{path} must stay rejected: {out}");
        assert!(
            out.contains("normalised engine-root-relative path"),
            "{path}: rejected for the traversal reason, not incidentally: {out}"
        );
    }
}

// ───────── bd-bo6i: inert shell is not wiring ──────────────────────────────

/// All three of these were measured at `wired=yes`, exit 0, on 2026-08-14.
/// They are disqualified from the TEXT leg; the behavioural leg below is what
/// actually settles the question.
#[test]
fn bad_inert_invocations_in_check_sh_are_red() {
    for form in [
        ": \"cargo run -q -p cdcp_gate -- substrate-guard\"",
        "true # cargo run -q -p cdcp_gate -- substrate-guard",
        "cargo run -q -p cdcp_gate -- substrate-guard || true",
    ] {
        let f = Fixture::new();
        f.write("scripts/check.sh", &format!("#!/bin/sh\nset -eu\n{form}\n"));
        f.git(&["add", "-A"]);
        let (code, out) = f.gate(&["substrate-guard", "--staged"]);
        assert_eq!(code, VIOLATION, "{form}: {out}");
        assert!(out.contains("BUILT != WIRED"), "{form}: {out}");
        assert!(out.contains("inert"), "{form}: must say why: {out}");
    }
}

/// `[wiring].status` is a ratchet. Once a commit has said "wired", a later commit
/// may not quietly say "pending" — that word is the difference between RED and a
/// stderr murmur.
#[test]
fn bad_wiring_status_walked_back_to_pending_is_red() {
    let f = Fixture::new(); // HEAD carries status = "wired"
    f.write("scripts/check.sh", "#!/bin/sh\necho nothing to see here\n");
    f.set_allowlist(
        &f.read_allowlist()
            .replace("status = \"wired\"", "status = \"pending\""),
    );
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, VIOLATION, "{out}");
    assert!(out.contains("ratchet"), "{out}");
}

/// Repointing `check_sh` at any other file satisfied the wiring leg from a file
/// nothing runs. Measured exit 0 on 2026-08-14 with the real step deleted.
#[test]
fn bad_check_sh_repointed_at_another_file_is_an_error() {
    let f = Fixture::new();
    f.write(
        "docs/decoy_wiring.txt",
        "notes: cargo run -q -p cdcp_gate -- substrate-guard\n",
    );
    f.write("scripts/check.sh", "#!/bin/sh\necho nothing to see here\n");
    f.set_allowlist(&f.read_allowlist().replace(
        "check_sh = \"scripts/check.sh\"",
        "check_sh = \"docs/decoy_wiring.txt\"",
    ));
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("pinned"), "{out}");
}

// ───────── the handoff state: pending is loud, never silent ────────────────

#[test]
fn pending_wiring_reports_but_does_not_block() {
    // The handoff state: the gate exists, the check.sh step has not landed yet.
    // It must not block the very commit that wires it. HEAD must already say
    // "pending" here, or the ratchet — correctly — treats this as a walk-back.
    let f = Fixture::new();
    f.set_allowlist(
        &f.read_allowlist()
            .replace("status = \"wired\"", "status = \"pending\""),
    );
    f.git(&["add", "-A"]);
    f.commit("declare wiring pending");

    f.write("scripts/check.sh", "#!/bin/sh\necho nothing to see here\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, OK, "{out}");
    assert!(
        out.contains("PENDING WIRING"),
        "must still say so, loudly: {out}"
    );
}

#[test]
fn verify_wired_forces_the_assertion_even_while_pending() {
    let f = Fixture::new();
    f.set_allowlist(
        &f.read_allowlist()
            .replace("status = \"wired\"", "status = \"pending\""),
    );
    f.git(&["add", "-A"]);
    f.commit("declare wiring pending");

    f.write("scripts/check.sh", "#!/bin/sh\necho nothing to see here\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--verify-wired"]);
    assert_eq!(code, VIOLATION, "{out}");
}

#[test]
fn bad_blank_wiring_status_is_a_schema_error() {
    let f = Fixture::new();
    f.set_allowlist(
        &f.read_allowlist()
            .replace("status = \"wired\"", "status = \"\""),
    );
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("never permissive"), "{out}");
}

// ───────────────────── THE BEHAVIOURAL WIRING LEG ─────────────────────────
//
// No text test can establish that a shell line executes, so these run check.sh
// for real against a planted unlisted `.py` and ask whether check.sh ITSELF
// exits non-zero. The fixture check.sh invokes the gate BINARY directly rather
// than through `cargo run`: a nested cargo inside `cargo test` would contend for
// the target-directory lock, and the property under test is check.sh's control
// flow around the invocation, not cargo's.
//
// WHAT THIS ESTABLISHES: a RED verdict from this gate stops check.sh.
// WHAT IT DOES NOT: that any OTHER step in check.sh propagates its own failure,
// and that the working tree outside the index is clean.

fn probe_fixture(step: &str) -> Fixture {
    let f = Fixture::new();
    f.write(
        "scripts/check.sh",
        &format!(
            "#!/bin/sh\n\
             set -eu\n\
             cd \"$(dirname \"$0\")/..\"\n\
             echo \"==> cdcp_gate substrate-guard (S0 substrate floor)\"\n\
             {step}\n\
             echo \"check.sh: ok: S0 substrate floor\"\n"
        ),
    );
    f.git(&["add", "-A"]);
    f.commit("wire check.sh");
    f
}

fn live_step() -> String {
    format!(
        "\"{BIN}\" --root . substrate-guard --quiet || {{ echo \"check.sh: FAIL: substrate guard\" >&2; exit 2; }}"
    )
}

#[test]
fn probe_certifies_a_check_sh_whose_failure_propagates() {
    let f = probe_fixture(&live_step());
    let (code, out) = f.gate(&["substrate-guard", "--prove-wired"]);
    assert_eq!(
        code, OK,
        "a planted unlisted .py must stop a correctly wired check.sh: {out}"
    );
    assert!(out.contains("PROVEN"), "{out}");
    assert!(
        out.contains("__cdcp_probe_unlisted__.py"),
        "must name the plant: {out}"
    );
}

#[test]
fn probe_rejects_the_three_inert_forms() {
    for step in [
        format!(": \"{BIN} --root . substrate-guard\""),
        format!("true # \"{BIN}\" --root . substrate-guard"),
        format!("\"{BIN}\" --root . substrate-guard --quiet || true"),
    ] {
        let f = probe_fixture(&step);
        let (code, out) = f.gate(&["substrate-guard", "--prove-wired"]);
        assert_eq!(
            code, VIOLATION,
            "an inert step must not certify wiring: {step}\n{out}"
        );
        assert!(
            out.contains("never invoked") || out.contains("discards its verdict"),
            "{step}: must say which failure it was: {out}"
        );
    }
}

#[test]
fn probe_refuses_to_nest_rather_than_recursing() {
    let f = probe_fixture(&live_step());
    // Simulating the child's environment: a probe inside a probe must stop.
    let out = std::process::Command::new(BIN)
        .current_dir(&f.root)
        .arg("--root")
        .arg(&f.root)
        .args(["substrate-guard", "--prove-wired"])
        .env("CDCP_SUBSTRATE_PROBE", "1")
        .output()
        .expect("run cdcp_gate");
    let text = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(out.status.code(), Some(ERROR), "{text}");
    assert!(
        text.contains("Refusing") || text.contains("not terminate"),
        "{text}"
    );
}

// ───────── bd-ip10: the vacuity check reads ROWS, not bytes ────────────────
//
// `--prove-wired` refuses to run when the registry the snapshot carries already
// exempts the plant — otherwise the known-bad is not bad and the run certifies
// nothing. That precondition used to be `reg_text.contains(PROBE_PLANT)`, a raw
// byte scan. Measured 2026-08-14: it took scripts/check.sh RED with ZERO
// [[allow]] rows for the plant, over the registry's OWN COMMENT warning nobody
// to add one. It now reads parsed [[allow]] rows.
//
// The probe reads the INDEX, so every fixture here stages its registry.

const PLANT: &str = "scripts/__cdcp_probe_unlisted__.py";

/// THE bd-ip10 PROOF, end to end: the registry documents the rule in the very
/// words that used to trip it, and the probe runs anyway.
#[test]
fn probe_runs_when_a_comment_names_the_plant() {
    let f = probe_fixture(&live_step());
    f.set_allowlist(&format!(
        "# NEVER add a row for {PLANT} — that is the plant\n\
         # --prove-wired uses, and listing it makes the probe vacuous (the gate ERRORs).\n{}",
        f.read_allowlist()
    ));
    f.git(&["add", "-A"]);
    assert!(
        std::fs::read_to_string(f.path("registries/substrate_allowlist.toml"))
            .unwrap()
            .contains(PLANT),
        "the fixture must actually name the plant, or it tests nothing"
    );

    let (code, out) = f.gate(&["substrate-guard", "--prove-wired"]);
    assert_eq!(
        code, OK,
        "a comment is not an [[allow]] row — a gate that cannot be described in its own registry has documentation for a liability: {out}"
    );
    assert!(out.contains("PROVEN"), "{out}");
}

/// Known-bad, unchanged: a real row really would make the run vacuous.
#[test]
fn probe_errors_when_an_allow_row_lists_the_plant() {
    let f = probe_fixture(&live_step());
    f.set_allowlist(&(f.read_allowlist() + &good_row(PLANT)));
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--prove-wired"]);
    assert_eq!(code, ERROR, "an exempt known-bad certifies nothing: {out}");
    assert!(out.contains("vacuous"), "{out}");
    assert!(out.contains(PLANT), "must name it: {out}");
}

/// Known-bad, NEW. Bytes stay readable when rows do not: a parse without this
/// branch would let a malformed registry make the plant exempt in silence, which
/// is the one thing the byte scan could not do.
#[test]
fn probe_errors_when_the_staged_registry_does_not_parse() {
    let f = probe_fixture(&live_step());
    f.set_allowlist("schema_version = 1\n[scan\nroots = [\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--prove-wired"]);
    assert_eq!(
        code, ERROR,
        "a registry that will not parse must never clear the plant: {out}"
    );
    assert!(out.contains("does not parse"), "{out}");
    assert!(out.contains("ERROR, not a pass"), "{out}");
}

/// The other route to a meaningless plant: leave it unlisted but put it out of
/// scope. The gate's compiled-in floor already makes that RED elsewhere; here it
/// must stop the probe rather than let it certify against a plant nothing scans.
#[test]
fn probe_errors_when_the_staged_scan_excludes_the_plant() {
    let f = probe_fixture(&live_step());
    f.set_allowlist(&f.read_allowlist().replace("\"py\", ", ""));
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--prove-wired"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("outside the scanned surface"), "{out}");
}

// ───────────────────────── ANTI-VACUOUS ───────────────────────────────────

#[test]
fn zero_files_scanned_is_an_error_not_a_pass() {
    let f = Fixture::empty();
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(
        code, ERROR,
        "a vacuous scan must never report like a clean one: {out}"
    );
    assert!(out.contains("scanned 0 files"), "{out}");
}

/// bd-xmn5 RETIRED the old shape of this test rather than rewriting it. It used
/// to stage `docs/only-this.md` into an otherwise empty repo and assert ERROR,
/// because `docs/` was outside the scan roots and so the whole tree resolved to
/// nothing in scope. The scan is the whole tree now, so that state is no longer
/// constructible — the branch survives in `run` as the floor for the case where
/// `WHOLE_TREE_SCOPE` is ever turned off, and the vacuity property that is still
/// constructible is asserted directly above and here.
///
/// The anti-vacuous claim that remains, and that this asserts: a tree with files
/// in it must be REPORTED as scanned, so a run that scanned nothing cannot read
/// like a run that found nothing wrong.
#[test]
fn the_receipt_states_how_much_was_actually_scanned() {
    let f = Fixture::new();
    f.write("docs/only-this.md", "# ordinary content\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, OK, "{out}");
    assert!(out.contains("scanned="), "{out}");
    assert!(
        out.contains("identified_non_rust="),
        "the receipt must say how many entries it judged, not just that it ran: {out}"
    );
    assert!(
        !out.contains("scanned=0") && !out.contains("identified_non_rust=0"),
        "a scan that judged nothing must not report like a clean one: {out}"
    );
}

#[test]
fn missing_registry_is_an_error_not_a_pass() {
    let f = Fixture::new();
    std::fs::remove_file(f.path("registries/substrate_allowlist.toml")).unwrap();
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(code, ERROR, "{out}");
}

/// A commit that deletes the policy is not a commit with nothing to check.
#[test]
fn registry_absent_from_the_index_is_an_error_not_a_pass() {
    let f = Fixture::new();
    f.git(&[
        "rm",
        "-q",
        "--cached",
        "registries/substrate_allowlist.toml",
    ]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("not in the index"), "{out}");
}

/// A check.sh that cannot be read means the wiring leg was NOT evaluated. An
/// unevaluated leg must never report like a passed one.
#[test]
fn unreadable_check_sh_is_an_error_not_a_pass() {
    let f = Fixture::new();
    // Keep the row honest — the row's own "file is gone" check is a different
    // assertion, and this test is about the wiring leg.
    f.set_allowlist(&(header("wired") + &good_row("scripts/verify_bank.py")));
    f.remove("scripts/check.sh");
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(
        out.contains("wiring leg cannot be evaluated"),
        "must say the leg did not run: {out}"
    );
}

#[test]
fn check_sh_absent_from_the_index_is_an_error_not_a_pass() {
    let f = Fixture::new();
    f.set_allowlist(&(header("wired") + &good_row("scripts/verify_bank.py")));
    f.git(&["add", "-A"]);
    f.git(&["rm", "-q", "--cached", "scripts/check.sh"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("not in the index"), "{out}");
}
