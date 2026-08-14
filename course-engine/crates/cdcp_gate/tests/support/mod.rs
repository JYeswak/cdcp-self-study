//! Fixture builder: a throwaway git repo shaped like the engine, so the gate is
//! exercised against real `git ls-files` / `git diff --cached` output rather than
//! a mock of them.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub root: PathBuf,
}

pub fn good_row(path: &str) -> String {
    format!(
        "\n[[allow]]\npath = {path:?}\nreason = \"Grandfathered load-bearing gate; port tracked by the migration epic\"\nmigration_bead = \"bd-substrate-rust-migration-jhd.7\"\nexpires = \"2099-12-31\"\n"
    )
}

pub fn header(wiring_status: &str) -> String {
    format!(
        "schema_version = 1\n\n\
         [scan]\n\
         roots = [\"scripts\", \"crates\"]\n\
         extensions = [\"py\", \"pyw\", \"sh\", \"bash\", \"zsh\", \"ksh\"]\n\
         include_engine_root_files = true\n\n\
         [wiring]\n\
         status = {wiring_status:?}\n\
         check_sh = \"scripts/check.sh\"\n\
         invocation = \"cargo run -q -p cdcp_gate -- substrate-guard\"\n\
         bead = \"bd-substrate-rust-migration-jhd.1\"\n"
    )
}

impl Fixture {
    /// A repo with one allowlisted script, one Rust file, and a check.sh that
    /// already wires the gate.
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonicalize");
        let f = Fixture { dir, root };
        f.write("registries/claims.toml", "schema_version = 1\n");
        f.write(
            "scripts/check.sh",
            "#!/bin/sh\ncargo run -q -p cdcp_gate -- substrate-guard || exit 2\n",
        );
        f.write("scripts/verify_bank.py", "print('bank')\n");
        f.write("crates/cdcp_core/src/lib.rs", "pub fn f() {}\n");
        f.write("README.md", "# fixture\n");
        f.set_allowlist(
            &(header("wired")
                + &good_row("scripts/verify_bank.py")
                + &good_row("scripts/check.sh")),
        );
        f.git(&["init", "-q"]);
        f.git(&["add", "-A"]);
        f.commit("base");
        f
    }

    /// A repo git has been initialised in but which holds no files at all.
    pub fn empty() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonicalize");
        let f = Fixture { dir, root };
        f.write("registries/claims.toml", "schema_version = 1\n");
        f.set_allowlist(&header("pending"));
        f.git(&["init", "-q"]);
        f
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    pub fn write(&self, rel: &str, body: &str) {
        let p = self.path(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
    }

    pub fn remove(&self, rel: &str) {
        std::fs::remove_file(self.path(rel)).unwrap();
    }

    pub fn set_allowlist(&self, body: &str) {
        self.write("registries/substrate_allowlist.toml", body);
    }

    pub fn read_allowlist(&self) -> String {
        std::fs::read_to_string(self.path("registries/substrate_allowlist.toml")).unwrap()
    }

    pub fn git(&self, args: &[&str]) -> Output {
        let out = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .expect("git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    pub fn commit(&self, msg: &str) {
        self.git(&[
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "user.name=fixture",
            "commit",
            "-q",
            "--no-verify",
            "-m",
            msg,
        ]);
    }

    /// Run the gate binary against this fixture; returns (code, stdout+stderr).
    pub fn gate(&self, args: &[&str]) -> (i32, String) {
        run_gate(&self.root, args)
    }
}

pub fn run_gate(root: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(BIN)
        .current_dir(root)
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .expect("run cdcp_gate");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), s)
}
