//! `cdcp_gate <subcommand> [args]` — dispatcher.
//!
//! Holds no gate logic of its own: it resolves the engine root, looks the
//! subcommand up in the generated registry, and maps the outcome onto the shared
//! exit codes. An unknown subcommand is a USAGE error, never a silent success —
//! a typo must not read as "the gate passed".

#![forbid(unsafe_code)]

use cdcp_gate::registry::{self, GateCtx};
use cdcp_gate::{exit, root};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn usage(code: u8) -> ExitCode {
    let mut s = String::from(
        "cdcp_gate — Rust-native repo gates\n\n\
         usage: cdcp_gate [--root <dir>] <subcommand> [args]\n\
                cdcp_gate list\n\n\
         subcommands:\n",
    );
    for g in registry::all() {
        s.push_str(&format!("  {:<18} {}\n", g.name, g.summary));
    }
    s.push_str(
        "\nexit codes: 0 ok · 2 gate violation · 3 usage · 4 error (unreadable input / vacuous scan)\n",
    );
    if code == exit::OK {
        print!("{s}");
    } else {
        eprint!("{s}");
    }
    ExitCode::from(code)
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mut root_override: Option<PathBuf> = None;
    let mut rest: Vec<String> = Vec::new();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--root" => {
                let Some(v) = argv.get(i + 1) else {
                    eprintln!("cdcp_gate: --root needs a directory");
                    return ExitCode::from(exit::USAGE);
                };
                root_override = Some(PathBuf::from(v));
                i += 2;
            }
            _ => {
                rest.push(argv[i].clone());
                i += 1;
            }
        }
    }

    let Some(sub) = rest.first().cloned() else {
        return usage(exit::USAGE);
    };
    if sub == "--help" || sub == "-h" || sub == "help" {
        return usage(exit::OK);
    }
    if sub == "list" {
        for g in registry::all() {
            println!("{}\t{}", g.name, g.summary);
        }
        return ExitCode::SUCCESS;
    }

    let Some(gate) = registry::find(&sub) else {
        eprintln!("cdcp_gate: unknown subcommand {sub:?}");
        return usage(exit::USAGE);
    };

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let resolved = match root_override {
        Some(p) => match p.canonicalize() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("cdcp_gate: --root {}: {e}", p.display());
                return ExitCode::from(exit::ERROR);
            }
        },
        None => match root::resolve(&cwd) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("cdcp_gate: {e}");
                return ExitCode::from(exit::ERROR);
            }
        },
    };

    let ctx = GateCtx::new(resolved, rest[1..].to_vec());
    match (gate.run)(&ctx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            e.report(gate.name);
            ExitCode::from(e.code())
        }
    }
}
