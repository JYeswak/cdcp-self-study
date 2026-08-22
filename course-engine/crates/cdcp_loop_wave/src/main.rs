//! Wave entry point. Same binary for wave 1 and wave 2 (`--prior`).

use cdcp_loop_wave::{assemble_wave, render_stdout, write_report};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!(
            "loop-wave --root <engine> --harvest <franken-harvest.md> --out <dir> [--prior <wave-n.json>]"
        );
        return ExitCode::from(3);
    }
    let root = PathBuf::from(arg_value(&args, "--root").unwrap_or_else(|| ".".into()));
    let harvest = match arg_value(&args, "--harvest") {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("loop-wave: --harvest is required (fail closed)");
            return ExitCode::from(4);
        }
    };
    let out = PathBuf::from(arg_value(&args, "--out").unwrap_or_else(|| "docs/loop-waves".into()));
    let prior = arg_value(&args, "--prior").map(PathBuf::from);

    match assemble_wave(&root, &harvest, prior.as_deref()) {
        Ok(report) => match write_report(&out, &report) {
            Ok(json_path) => {
                let text = render_stdout(&report);
                print!("{text}");
                println!("wrote {}", json_path.display());
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("loop-wave: {err}");
                ExitCode::from(4)
            }
        },
        Err(err) => {
            eprintln!("loop-wave: {err}");
            ExitCode::from(4)
        }
    }
}
