//! Binary entrypoint for L1 claims constitution gate.
#![forbid(unsafe_code)]

use cdcp_registry_check::{resolve_repo_root, run};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let first = args.next();
    if first.as_deref() == Some("--track-selftest") {
        return match cdcp_registry_check::run_track_selftest() {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::from(2),
        };
    }

    if first.as_deref() == Some("--track-check") {
        let Some(raw_manifest) = args.next() else {
            eprintln!("cdcp_registry_check: --track-check requires a manifest path");
            return ExitCode::from(2);
        };
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = match resolve_repo_root(&cwd) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("cdcp_registry_check: {e}");
                return ExitCode::from(2);
            }
        };
        let manifest = PathBuf::from(raw_manifest);
        let manifest = if manifest.is_absolute() {
            manifest
        } else {
            cwd.join(manifest)
        };
        return match cdcp_registry_check::run_track_check(&root, &manifest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::from(2),
        };
    }

    let root = first.map(PathBuf::from).unwrap_or_else(|| {
        match resolve_repo_root(&env::current_dir().unwrap_or_else(|_| PathBuf::from("."))) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("cdcp_registry_check: {e}");
                std::process::exit(2);
            }
        }
    });

    match run(&root) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(2),
    }
}
