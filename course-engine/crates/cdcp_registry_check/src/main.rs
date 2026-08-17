//! Binary entrypoint for L1 claims constitution gate.
#![forbid(unsafe_code)]

use cdcp_registry_check::{resolve_repo_root, run};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let root = if let Some(a) = args.next() {
        PathBuf::from(a)
    } else {
        match resolve_repo_root(&env::current_dir().unwrap_or_else(|_| PathBuf::from("."))) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("cdcp_registry_check: {e}");
                return ExitCode::from(2);
            }
        }
    };

    match run(&root) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(2),
    }
}
