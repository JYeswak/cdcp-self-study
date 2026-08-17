//! First-contact surfaces for `cdcp` (`bd-installability-sm4g.5`).
//!
//! Three product facts:
//!   1. `--version` / `-V` print the workspace package version, exit 0, stdout.
//!   2. Bare `cdcp` writes orientation to stdout and exits 0 (not stderr / 2).
//!   3. clap is built with `default-features = false` so colour cannot leak
//!      into a pipe. The planted known-bad is `CLICOLOR_FORCE=1`: that env
//!      used to force ANSI into piped output. Omitting it is a vacuous pass.

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve course-engine workspace root")
}

fn cdcp() -> Command {
    let mut cmd = Command::cargo_bin("cdcp").expect("cdcp binary");
    cmd.current_dir(workspace_root());
    cmd
}

/// `[workspace.package].version` from the engine Cargo.toml.
///
/// The binary must print this string, not a hard-coded crate constant that
/// could drift from the workspace pin.
fn workspace_package_version() -> String {
    let raw = fs::read_to_string(workspace_root().join("Cargo.toml"))
        .expect("read course-engine/Cargo.toml");
    let mut in_section = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_section = t == "[workspace.package]";
            continue;
        }
        if in_section {
            if let Some(rest) = t.strip_prefix("version") {
                let v = rest.trim().trim_start_matches('=').trim().trim_matches('"');
                assert!(
                    !v.is_empty(),
                    "workspace.package.version is empty — nothing to pin"
                );
                return v.to_string();
            }
        }
    }
    panic!("course-engine/Cargo.toml has no [workspace.package] version");
}

fn has_ansi(bytes: &[u8]) -> bool {
    bytes.contains(&0x1b)
}

/// One colour-environment the first-contact surfaces must survive.
///
/// `CLICOLOR_FORCE` is the planted known-bad: it is the case that failed
/// when clap's `color` feature was live. An empty list, or a list that
/// drops that row, is an ERROR — not a pass.
struct AnsiCase {
    name: &'static str,
    /// Extra env to apply after isolation (NO_COLOR / CLICOLOR_FORCE / TERM).
    env: &'static [(&'static str, &'static str)],
}

const ANSI_CASES: &[AnsiCase] = &[
    AnsiCase {
        name: "piped",
        env: &[],
    },
    AnsiCase {
        name: "NO_COLOR",
        env: &[("NO_COLOR", "1")],
    },
    AnsiCase {
        name: "TERM=dumb",
        env: &[("TERM", "dumb")],
    },
    AnsiCase {
        name: "CLICOLOR_FORCE",
        env: &[("CLICOLOR_FORCE", "1")],
    },
];

/// Invocations that exercise clap's renderer, not just our `println!`.
///
/// Bare + `--version` are our own writes (no ANSI regardless of clap).
/// `--help` and an unknown subcommand go through clap; those are the
/// surfaces that used to paint when `color` was on.
const ANSI_INVOCATIONS: &[&[&str]] = &[
    &[],
    &["--version"],
    &["--help"],
    &["definitely-not-a-cdcp-command"],
];

fn isolate_color_env(cmd: &mut Command, case: &AnsiCase) {
    // The parent process may carry colour env. Isolate so the case is
    // the only signal.
    cmd.env_remove("NO_COLOR");
    cmd.env_remove("CLICOLOR");
    cmd.env_remove("CLICOLOR_FORCE");
    cmd.env_remove("FORCE_COLOR");
    if case.name == "TERM=dumb" {
        cmd.env("TERM", "dumb");
    } else {
        cmd.env("TERM", "xterm-256color");
    }
    for (k, v) in case.env {
        cmd.env(*k, *v);
    }
}

#[test]
fn version_long_flag_prints_workspace_package_version() {
    let want = workspace_package_version();
    assert_eq!(
        want,
        env!("CARGO_PKG_VERSION"),
        "cdcp_cli version.workspace must equal [workspace.package].version"
    );
    let assert = cdcp().arg("--version").assert().success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        stdout,
        format!("{want}\n"),
        "--version must print exactly the workspace version on stdout"
    );
    assert!(
        stderr.is_empty(),
        "--version must not write stderr: {stderr:?}"
    );
}

#[test]
fn version_short_flag_prints_workspace_package_version() {
    let want = workspace_package_version();
    let assert = cdcp().arg("-V").assert().success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        stdout,
        format!("{want}\n"),
        "-V must print exactly the workspace version on stdout"
    );
    assert!(stderr.is_empty(), "-V must not write stderr: {stderr:?}");
}

#[test]
fn bare_invocation_writes_orientation_to_stdout_and_exits_0() {
    let assert = cdcp().assert().success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stdout.trim().is_empty(),
        "bare cdcp must write orientation to stdout"
    );
    assert!(
        stdout.contains("--help"),
        "orientation must point at --help: {stdout}"
    );
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "orientation must name the workspace version: {stdout}"
    );
    assert!(
        stderr.is_empty(),
        "bare cdcp must not write stderr (that is the diagnostics-shaped help): {stderr:?}"
    );
}

/// Command names clap prints under `Commands:`. Empty is ERROR.
fn listed_commands(help: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_cmds = false;
    for line in help.lines() {
        if line.starts_with("Commands:") {
            in_cmds = true;
            continue;
        }
        if in_cmds && (line.starts_with("Options:") || line.starts_with("Arguments:")) {
            break;
        }
        if in_cmds {
            let t = line.trim_start();
            if let Some(name) = t.split_whitespace().next() {
                if name.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
                    out.push(name.to_string());
                }
            }
        }
    }
    out
}

#[test]
fn learner_help_is_five_product_verbs() {
    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .arg("--help")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let cmds = listed_commands(&stdout);
    assert!(
        !cmds.is_empty(),
        "learner --help listed zero commands — hide-everything is a brick:\n{stdout}"
    );
    // W12–W16 grow the learner set on purpose after .6 hide.
    for verb in [
        "study",
        "doctor",
        "demo",
        "test",
        "repair",
        "quickstart",
        "completion",
    ] {
        assert!(
            cmds.iter().any(|c| c == verb),
            "learner --help missing {verb}: {cmds:?}\n{stdout}"
        );
    }
    for authoring in [
        "bank-hash",
        "build-learn",
        "goldens",
        "export-web",
        "serve",
        "docs",
    ] {
        assert!(
            !cmds.iter().any(|c| c == authoring),
            "learner --help still lists authoring verb {authoring}: {cmds:?}"
        );
    }
    assert!(
        stdout.contains("--info"),
        "learner --help must list --info as a top-level flag: {stdout}"
    );
    assert!(
        !cmds.iter().any(|c| c == "info"),
        "--info leaked into Commands: (it is a flag, not a verb): {cmds:?}"
    );
}

#[test]
fn cdcp_dev_unhides_authoring() {
    let assert = cdcp().env("CDCP_DEV", "1").arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let cmds = listed_commands(&stdout);
    assert!(
        cmds.iter().any(|c| c == "bank-hash"),
        "CDCP_DEV=1 --help must list bank-hash: {cmds:?}\n{stdout}"
    );
    assert!(
        cmds.iter().any(|c| c == "build-learn"),
        "CDCP_DEV=1 --help must list build-learn: {cmds:?}"
    );
    assert!(
        cmds.len() > 10,
        "CDCP_DEV=1 --help should unhide authoring (got {}): {cmds:?}",
        cmds.len()
    );
}

#[test]
fn clap_disables_default_features_in_workspace_manifest() {
    let raw = fs::read_to_string(workspace_root().join("Cargo.toml"))
        .expect("read course-engine/Cargo.toml");
    let clap_line = raw
        .lines()
        .find(|l| l.contains("clap =") && l.contains("version"))
        .expect("workspace clap pin is a single line");
    assert!(
        clap_line.contains("default-features = false")
            || clap_line.contains("default-features=false"),
        "workspace clap must set default-features = false (color off by construction): {clap_line}"
    );
    assert!(
        !clap_line.contains("\"color\""),
        "workspace clap must not re-enable the color feature: {clap_line}"
    );
}

#[test]
fn first_contact_surfaces_emit_zero_ansi_under_controlled_environments() {
    // Anti-vacuous: an empty case list, or a list that drops CLICOLOR_FORCE,
    // is the exact hole this bead plants. Empty known-bad set is ERROR.
    assert!(
        !ANSI_CASES.is_empty(),
        "empty known-bad set is ERROR — colour cases must be named"
    );
    assert!(
        ANSI_CASES.iter().any(|c| c.name == "CLICOLOR_FORCE"),
        "CLICOLOR_FORCE is the planted known-bad; omitting it is a vacuous pass"
    );
    assert!(
        ANSI_INVOCATIONS.len() >= 4,
        "must exercise clap's renderer (--help / unknown command), not only our println!"
    );
    assert!(
        ANSI_INVOCATIONS.iter().any(|args| args.contains(&"--help")),
        "--help is the clap-rendered surface that used to paint"
    );

    for case in ANSI_CASES {
        for args in ANSI_INVOCATIONS {
            let mut cmd = cdcp();
            isolate_color_env(&mut cmd, case);
            cmd.args(*args);
            let output = cmd
                .output()
                .unwrap_or_else(|e| panic!("spawn cdcp {:?} under {}: {e}", args, case.name));
            assert!(
                !has_ansi(&output.stdout) && !has_ansi(&output.stderr),
                "ANSI leaked under {} args={args:?}\nstdout={:?}\nstderr={:?}",
                case.name,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
