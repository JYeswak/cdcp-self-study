//! Shared exit-code conventions for every gate in this crate.
//!
//! Every ported gate (bd-substrate-rust-migration-jhd.2 .. .18) returns these and
//! only these. `scripts/check.sh` treats any non-zero as RED; the codes exist so a
//! caller can tell "the gate ran and the assertion failed" apart from "the gate
//! could not honestly run at all".

/// Gate ran; every assertion held.
pub const OK: u8 = 0;

/// Gate ran; at least one assertion FAILED. This is the ordinary RED.
pub const VIOLATION: u8 = 2;

/// The gate was invoked wrongly (unknown subcommand, bad flag). Not a verdict on
/// the tree.
pub const USAGE: u8 = 3;

/// The gate could not honestly evaluate: unreadable/ill-formed registry, git
/// unavailable, or a vacuous scan (zero files). Never confuse this with OK —
/// a deliverable that was never checked reports exactly like one that passed,
/// which is the whole reason this code is separate.
pub const ERROR: u8 = 4;
