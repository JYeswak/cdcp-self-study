//! End-to-end: the guard binary against real git repos.
//!
//! Both legs are mandatory. The KNOWN-BAD leg proves the gate trips. The
//! KNOWN-GOOD leg proves it does not trip on the ordinary day — an over-strict
//! gate gets routed around, which is a slower death than no gate at all.

mod support;
use support::{good_row, header, Fixture};

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
    let f = Fixture::new();
    f.write("docs/notes.py", "x=1\n");
    f.write("tests/voice-slop.sh", "echo hi\n");
    f.write("scripts/smoke.mjs", "console.log(1)\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard", "--staged"]);
    assert_eq!(code, OK, "the gate is a floor, not a dragnet: {out}");
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
    let narrowed = f
        .read_allowlist()
        .replace("extensions = [\"py\", \"sh\"]", "extensions = [\"sh\"]");
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

#[test]
fn pending_wiring_reports_but_does_not_block() {
    // The handoff state: the gate exists, the check.sh step has not landed yet.
    // It must not block the very commit that wires it.
    let f = Fixture::new();
    f.write("scripts/check.sh", "#!/bin/sh\necho nothing to see here\n");
    f.set_allowlist(
        &f.read_allowlist()
            .replace("status = \"wired\"", "status = \"pending\""),
    );
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
    f.write("scripts/check.sh", "#!/bin/sh\necho nothing to see here\n");
    f.set_allowlist(
        &f.read_allowlist()
            .replace("status = \"wired\"", "status = \"pending\""),
    );
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

#[test]
fn zero_in_scope_files_is_an_error_not_a_pass() {
    let f = Fixture::empty();
    f.write("docs/only-this.md", "# nothing in scope\n");
    f.git(&["add", "-A"]);
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(code, ERROR, "{out}");
    assert!(out.contains("in scope"), "{out}");
}

#[test]
fn missing_registry_is_an_error_not_a_pass() {
    let f = Fixture::new();
    std::fs::remove_file(f.path("registries/substrate_allowlist.toml")).unwrap();
    let (code, out) = f.gate(&["substrate-guard"]);
    assert_eq!(code, ERROR, "{out}");
}
