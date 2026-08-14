//! The dispatcher contract every future gate inherits.

mod support;
use std::process::Command;
use support::BIN;

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN).args(args).output().expect("run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn list_names_every_registered_gate() {
    let (code, stdout, _) = run(&["list"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("substrate-guard"), "{stdout}");
    assert!(stdout.contains("install-hooks"), "{stdout}");
}

#[test]
fn help_exits_zero_and_documents_the_exit_codes() {
    let (code, stdout, _) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("2 gate violation"), "{stdout}");
    assert!(stdout.contains("4 error"), "{stdout}");
}

#[test]
fn unknown_subcommand_is_usage_never_silent_success() {
    let (code, _, stderr) = run(&["verify-nothing"]);
    assert_eq!(code, cdcp_gate::exit::USAGE as i32, "{stderr}");
    assert!(stderr.contains("unknown subcommand"), "{stderr}");
}

#[test]
fn no_subcommand_is_usage() {
    let (code, _, _) = run(&[]);
    assert_eq!(code, cdcp_gate::exit::USAGE as i32);
}

#[test]
fn unknown_flag_on_a_gate_is_usage_not_a_silent_pass() {
    let (code, _, stderr) = run(&["substrate-guard", "--stagd"]);
    assert_eq!(code, cdcp_gate::exit::USAGE as i32, "{stderr}");
}

#[test]
fn gate_names_are_unique_and_kebab_case() {
    let mut seen = std::collections::BTreeSet::new();
    for g in cdcp_gate::registry::all() {
        assert!(seen.insert(g.name), "duplicate gate name {}", g.name);
        assert!(
            g.name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "gate name {:?} must be kebab-case",
            g.name
        );
        assert!(!g.summary.trim().is_empty(), "{} has no summary", g.name);
    }
    assert!(!seen.is_empty(), "an empty gate registry is an ERROR");
}
