//! `cdcp_gate` — the Rust home for this repo's gates.
//!
//! # Why this crate exists
//!
//! Measured 2026-08-14: the engine was ~70% Python and shell by volume, with 17
//! Python scripts wired into `scripts/check.sh` as load-bearing gates. Three were
//! added the day after the correct Rust pattern was written down. Prohibition in
//! doctrine did not stop it. This crate is the gate that does, plus the scaffold
//! the 17 ports (bd-substrate-rust-migration-jhd.2 .. .18) each extend by adding
//! exactly one file to `src/gates/`.
//!
//! # Shape
//!
//! - `exit`     — the exit-code conventions every gate shares.
//! - `registry` — the gate contract (`GateCtx`, `GateError`) and lookup.
//! - `gates/`   — one file per gate; `build.rs` registers them automatically.
//! - `root`     — engine-root resolution.
//! - `vcs`      — thin git plumbing, invoked by argv, never through a shell.
//! - `date`     — civil-date arithmetic for `expires`.

#![forbid(unsafe_code)]

pub mod date;
pub mod exit;
pub mod gates;
pub mod registry;
pub mod root;
pub mod vcs;
