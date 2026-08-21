//! Fail-closed physical workspace identity preflight.

fn main() {
    let start = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("workspace-identity: cannot read pwd: {error}");
            std::process::exit(2);
        }
    };
    match cdcp_root::verify_workspace_identity(&start) {
        Ok(identity) => println!("{}", identity.receipt_line()),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
