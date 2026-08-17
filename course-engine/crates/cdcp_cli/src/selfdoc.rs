//! Installed self-doc: `--info`, `quickstart`, `help <topic>`, `completion`.
//!
//! These are product surfaces (bd-installability-sm4g.15). They are not gates.
//! `--info` is a top-level flag (like `--version`), not a sixth mystery verb.
//! `help <topic>` is topic-help — there is no clap subcommand named `install`.
#![forbid(unsafe_code)]

use cdcp_root::{resolve_from_env, Via};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Product verbs a learner sees in `cdcp --help`. Authoring stays reachable
/// (`cdcp build-learn` still runs); it is hidden unless `CDCP_DEV=1`.
/// clap's auto `help` is kept. Empty is compile-fail — a hide that hides
/// everything is a brick.
///
/// W12–W16 grow this set on purpose after `.6` hide. `quickstart` and
/// `completion` are learner-visible. `--info` is a flag, not a verb.
pub(crate) const LEARNER_VISIBLE: &[&str] = &[
    "study",
    "doctor",
    "demo",
    "test",
    "repair",
    "quickstart",
    "completion",
    "help",
];

const _: () = assert!(
    LEARNER_VISIBLE.len() >= 8,
    "LEARNER_VISIBLE shrank below the operator verbs + clap help"
);

/// Version of `cdcp --info --json`. Bump when the envelope shape changes.
pub(crate) const INFO_SCHEMA_VERSION: u64 = 1;

/// Top-level keys of the info envelope. A test pins these names.
pub(crate) const INFO_JSON_FIELDS: &[&str] =
    &["schema_version", "version", "root", "kind", "via", "web"];

const _: () = assert!(
    INFO_SCHEMA_VERSION > 0,
    "INFO_SCHEMA_VERSION 0 is unversioned"
);
const _: () = assert!(
    !INFO_JSON_FIELDS.is_empty(),
    "empty INFO_JSON_FIELDS is an unversioned envelope"
);

/// Shells `cdcp completion` emits. Empty is compile-fail.
pub(crate) const COMPLETION_SHELLS: &[&str] = &["bash", "zsh", "fish"];

const _: () = assert!(
    !COMPLETION_SHELLS.is_empty(),
    "empty COMPLETION_SHELLS certifies nothing"
);

/// Topic-help names. `install` is deliberately not a clap subcommand.
/// Empty is compile-fail. The three required topics must stay present.
pub(crate) const HELP_TOPICS: &[&str] = &[
    "install",
    "doctor",
    "study",
    "demo",
    "test",
    "repair",
    "quickstart",
];

const _: () = assert!(
    HELP_TOPICS.len() >= 3,
    "help topics shrank below install/doctor/study"
);

/// First five minutes. Word count is a runtime gate (≥200). Empty is
/// compile-fail. The named verbs are the product path, not authoring.
pub(crate) const QUICKSTART: &str = "\
cdcp is a local-first CDCP study tool. It is not EPI certification software \
and it never phones home. After install.sh puts the binary on your PATH and \
the learner bundle under CDCP_HOME, or under $XDG_DATA_HOME/cdcp, or under \
~/.local/share/cdcp, these are the first five minutes.

Start by asking the binary where it is. cdcp --info prints the workspace \
version, the resolved root, and which precedence step chose that root: an \
explicit --root flag, the CDCP_HOME environment variable, the XDG data home \
(or ~/.local/share/cdcp), or a walk upward from the current directory. Add \
--json when a script needs a versioned envelope with schema_version.

Then prove the installed tree is whole. cdcp doctor probes the installed \
layer only: the web/ bundle, the shipped wasm magic, the install receipt \
when one exists, and a bindable local port. It does not require bank/, \
goldens/, content.lock, or python3. cdcp doctor --json emits a versioned \
envelope naming each probe.

cdcp test smokes that same installed tree. It checks the learner-pack \
shape, the wasm magic bytes, and the seed-42 assets that demo and study \
will serve. An empty test suite is a failure, not a pass. A missing wasm \
file is named by absolute path.

cdcp demo --no-open is the two-minute proof. It grades a planted \
all-correct attempt and a planted all-wrong attempt against the shipped \
wasm, prints two distinct digests, and prints the study URL. It does not \
block waiting for a browser. Drop --no-open if you want the URL opened \
on this machine.

cdcp study is the product command. It resolves the learner bundle, binds \
a local port (retrying nearby ports when 8766 is already taken), prints \
the URL, and opens a browser. cdcp study --no-open prints the URL only. \
cdcp serve is the same local HTTP bind without opening a browser; learners \
usually want study. Both stay offline and bind localhost only.

If something is missing, cdcp repair --dry-run reads the install receipt \
and plans restores. It never re-freezes goldens/. --apply is refused when \
the receipt would have to invent bytes the installed tree does not have.

Shell tab-completion lives in the binary: cdcp completion bash, also zsh \
and fish. Topic help is cdcp help install, cdcp help doctor, and \
cdcp help study — those are topics, not extra clap verbs. Authoring \
commands stay hidden unless CDCP_DEV=1.
";

const _: () = assert!(!QUICKSTART.is_empty(), "empty QUICKSTART is a brick");
const _: () = assert!(
    QUICKSTART.len() >= 800,
    "QUICKSTART shrank below a 200-word floor (bytes proxy)"
);

/// Intercept `--info` (any position) and `help <topic>` before clap.
///
/// `--info --json` / `--info --root PATH` are not clap parent flags — clap
/// would reject them. `help install` is not a clap subcommand; falling
/// through prints "unrecognized subcommand".
pub(crate) fn intercept<I, S>(args: I) -> Option<ExitCode>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();

    if flag_present(&args, "--info") {
        return Some(run_info(
            flag_value(&args, "--root").as_deref(),
            flag_present(&args, "--json"),
        ));
    }

    let positionals: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| !a.starts_with('-'))
        .collect();
    if let ["help", topic] = positionals.as_slice() {
        if let Some(body) = topic_body(topic) {
            print!("{body}");
            return Some(ExitCode::SUCCESS);
        }
    }
    None
}

pub(crate) fn print_quickstart() -> Result<(), String> {
    let n = word_count(QUICKSTART);
    if n < 200 {
        return Err(format!(
            "quickstart is {n} words — the product floor is 200"
        ));
    }
    print!("{QUICKSTART}");
    if !QUICKSTART.ends_with('\n') {
        println!();
    }
    Ok(())
}

pub(crate) fn print_completion(shell: &str) -> Result<(), String> {
    let script = completion_script(shell)?;
    print!("{script}");
    if !script.ends_with('\n') {
        println!();
    }
    Ok(())
}

pub(crate) fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

pub(crate) fn run_info(explicit: Option<&Path>, json: bool) -> ExitCode {
    let version = env!("CARGO_PKG_VERSION");
    match resolve_from_env(explicit) {
        Ok(root) => {
            let via = via_label(root.via);
            if json {
                emit_info_json(InfoEnvelope {
                    schema_version: INFO_SCHEMA_VERSION,
                    version,
                    root: Some(root.path.display().to_string()),
                    kind: Some(root.kind.as_str()),
                    via: Some(via),
                    web: Some(root.web.display().to_string()),
                    error: None,
                });
            } else {
                println!("cdcp {version}");
                println!("root: {}", root.path.display());
                println!("kind: {}", root.kind.as_str());
                println!("via: {via}");
                println!("web: {}", root.web.display());
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            let msg = err.to_string();
            if json {
                emit_info_json(InfoEnvelope {
                    schema_version: INFO_SCHEMA_VERSION,
                    version,
                    root: None,
                    kind: None,
                    via: None,
                    web: None,
                    error: Some(msg),
                });
            } else {
                println!("cdcp {version}");
                println!("root: (unresolved)");
                println!("via: (none)");
                println!("error: {msg}");
            }
            ExitCode::from(err.exit_code())
        }
    }
}

fn emit_info_json(env: InfoEnvelope) {
    // A broken envelope is a product bug: fail closed on stdout, not a
    // half-written object a consumer could mistake for v1.
    match serde_json::to_string(&env) {
        Ok(line) => println!("{line}"),
        Err(e) => {
            eprintln!("cdcp: --info --json envelope unparseable: {e}");
        }
    }
}

#[derive(Serialize)]
struct InfoEnvelope {
    schema_version: u64,
    version: &'static str,
    root: Option<String>,
    kind: Option<&'static str>,
    via: Option<&'static str>,
    web: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Precedence labels the acceptance names. XDG covers both `$XDG_DATA_HOME/cdcp`
/// and the default `~/.local/share/cdcp` slot.
fn via_label(via: Via) -> &'static str {
    match via {
        Via::Explicit => "--root",
        Via::CdcpHome => "CDCP_HOME",
        Via::XdgDataHome | Via::LocalShare => "XDG",
        Via::CwdWalk => "cwd-walk",
    }
}

fn flag_present(args: &[String], name: &str) -> bool {
    args.iter()
        .take_while(|a| a.as_str() != "--")
        .any(|a| a == name)
}

fn flag_value(args: &[String], name: &str) -> Option<PathBuf> {
    let mut iter = args.iter().map(String::as_str).take_while(|a| *a != "--");
    while let Some(a) = iter.next() {
        if a == name {
            return iter
                .next()
                .filter(|v| !v.starts_with('-'))
                .map(PathBuf::from);
        }
        if let Some(rest) = a.strip_prefix(name) {
            if let Some(v) = rest.strip_prefix('=') {
                if !v.is_empty() {
                    return Some(PathBuf::from(v));
                }
            }
        }
    }
    None
}

fn topic_body(topic: &str) -> Option<&'static str> {
    match topic {
        "install" => Some(TOPIC_INSTALL),
        "doctor" => Some(TOPIC_DOCTOR),
        "study" => Some(TOPIC_STUDY),
        "demo" => Some(TOPIC_DEMO),
        "test" => Some(TOPIC_TEST),
        "repair" => Some(TOPIC_REPAIR),
        "quickstart" => Some(TOPIC_QUICKSTART),
        _ => None,
    }
}

const TOPIC_INSTALL: &str = "\
cdcp help install — topic (not a clap subcommand)

There is no `cdcp install` verb. Installation is install.sh:

  curl -fsSL https://raw.githubusercontent.com/JYeswak/cdcp-self-study/main/course-engine/install.sh | bash

The script is receipt-driven and fails closed. It installs the binary
(typically ~/.local/bin/cdcp) and the learner bundle under CDCP_HOME,
or $XDG_DATA_HOME/cdcp, or ~/.local/share/cdcp. Env cannot retarget
the trust anchors.

After install:

  cdcp --info          version + resolved root + which step chose it
  cdcp doctor          preflight the installed layer
  cdcp test            smoke the installed tree
  cdcp demo --no-open  planted grade + study URL
  cdcp study           bind localhost and open the offline site

`cdcp --info` names the precedence step: --root, CDCP_HOME, XDG, or
cwd-walk. `cdcp help doctor` and `cdcp help study` are further topics.
";

const TOPIC_DOCTOR: &str = "\
cdcp help doctor — topic (not clap flag-list help)

`cdcp doctor` preflights the INSTALLED layer only:

  web/                 the learner bundle (index.html)
  wasm                 shipped web/assets/wasm/cdcp_wasm.wasm (\\0asm)
  receipt              install-receipt.json, when present
  port                 a bindable local address

It does not require bank/, goldens/, content.lock, or python3. Those
authoring probes stay behind CDCP_DEV=1. A missing wasm is RED and
names the absolute path. An empty probe list is a failure, not a pass.

  cdcp doctor              human report, exit 0 when every probe passes
  cdcp doctor --json       versioned envelope (schema_version + probes)
  cdcp doctor --root DIR   use DIR instead of CDCP_HOME / XDG / cwd-walk
  cdcp doctor --bind ADDR  probe ADDR (occupied default is not a fail)

See also: cdcp test, cdcp demo --no-open, cdcp --info, cdcp help install.
";

const TOPIC_STUDY: &str = "\
cdcp help study — topic (not clap flag-list help)

`cdcp study` is the product command. It resolves the learner bundle,
binds a local port, prints the URL, and opens a browser. Occupied
8766 retries nearby ports, then an ephemeral port. The listener is
localhost only. Nothing is uploaded.

  cdcp study                 bind, print URL, open a browser
  cdcp study --no-open       print the URL only (scripts / CI)
  cdcp study --root DIR      bundle, engine root, or CDCP home
  cdcp study --bind ADDR     preferred address (fallback still applies)

`cdcp serve` is the same local HTTP bind without opening a browser.
Learners usually want study. serve stays reachable; it is hidden from
`cdcp --help` unless CDCP_DEV=1.

A missing bundle exits 4 and names the absolute path that was looked
for. See cdcp --info for which precedence step chose the root, and
cdcp doctor / cdcp test before you study a fresh install.
";

const TOPIC_DEMO: &str = "\
cdcp help demo — topic

`cdcp demo` grades a planted all-correct attempt and a planted
all-wrong attempt against the shipped wasm, then prints the study URL.
It does not block. `cdcp demo --no-open` skips the browser. An empty
planted set is an error, not a pass.
";

const TOPIC_TEST: &str = "\
cdcp help test — topic

`cdcp test` smokes the installed tree: learner-pack shape, wasm magic,
and seed-42 assets. It is not check.sh. An empty suite is an error.
A missing wasm is named by absolute path.
";

const TOPIC_REPAIR: &str = "\
cdcp help repair — topic

`cdcp repair` is receipt-driven. Default is --dry-run (writes nothing).
--apply is idempotent and refuses to invent bytes. goldens/ is never
a repair target.
";

const TOPIC_QUICKSTART: &str = "\
cdcp help quickstart — topic

Run `cdcp quickstart` for the full first-five-minutes guide (doctor,
test, demo, study / serve). This topic is the pointer, not the guide.
";

fn completion_script(shell: &str) -> Result<String, String> {
    let cmds = LEARNER_VISIBLE.join(" ");
    let topics = HELP_TOPICS.join(" ");
    let shells = COMPLETION_SHELLS.join(" ");
    match shell {
        "bash" => Ok(format!(
            r#"# cdcp bash completion (learner verbs)
_cdcp() {{
  local cur
  COMPREPLY=()
  cur="${{COMP_WORDS[COMP_CWORD]}}"
  if [ "${{COMP_CWORD}}" -eq 1 ]; then
    COMPREPLY=( $(compgen -W "{cmds}" -- "${{cur}}") )
    return 0
  fi
  case "${{COMP_WORDS[1]}}" in
    completion)
      COMPREPLY=( $(compgen -W "{shells}" -- "${{cur}}") )
      ;;
    help)
      COMPREPLY=( $(compgen -W "{topics}" -- "${{cur}}") )
      ;;
  esac
}}
complete -F _cdcp cdcp
"#
        )),
        "zsh" => Ok(format!(
            r#"#compdef cdcp
_cdcp() {{
  local -a cmds topics shells
  cmds=({cmds})
  topics=({topics})
  shells=({shells})
  if (( CURRENT == 2 )); then
    _describe 'command' cmds
    return
  fi
  case ${{words[2]}} in
    completion) _describe 'shell' shells ;;
    help) _describe 'topic' topics ;;
  esac
}}
_cdcp
"#
        )),
        "fish" => Ok(format!(
            r#"# cdcp fish completion (learner verbs)
complete -c cdcp -f
complete -c cdcp -n "__fish_use_subcommand" -a "{cmds}"
complete -c cdcp -n "__fish_seen_subcommand_from completion" -a "{shells}"
complete -c cdcp -n "__fish_seen_subcommand_from help" -a "{topics}"
"#
        )),
        other => Err(format!(
            "unsupported shell: {other} (want {})",
            COMPLETION_SHELLS.join(", ")
        )),
    }
}
