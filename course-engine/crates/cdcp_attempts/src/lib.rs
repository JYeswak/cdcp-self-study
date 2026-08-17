//! Local-first attempt-event log. Capture only — no psychometrics.
//!
//! Records the fields EPIC L names. Export is explicit opt-in (default OFF).
//! An empty store is a schema ERROR on read/export, never a zero-N analysis.
//! This crate refuses to compute IRT, item difficulty, or discrimination.
//! A minimum-N warning is part of the public surface so later analysis
//! cannot ship without naming the floor.
#![forbid(unsafe_code)]

mod error;
mod event;
mod store;

pub use error::{AttemptError, EMPTY_STORE, EXPORT_NOT_OPTED_IN, PSYCHOMETRICS_REFUSED};
pub use event::{AttemptEvent, AttemptMode, SCHEMA_VERSION};
pub use store::{AttemptLog, JSONL_NAME, SQLITE_NAME};

/// Designed-in floor. Item-level statistics are sample-dependent and
/// unreliable below this N. This crate does not compute those quantities
/// at any N; the constant exists so a later analysis crate cannot ship
/// without naming the floor.
pub const MIN_N_FOR_ITEM_STATS: u64 = 200;

/// Warning text attached to an export receipt when `n < MIN_N_FOR_ITEM_STATS`.
pub const MIN_N_WARNING_MESSAGE: &str = "item-level statistics are sample-dependent and unreliable at small N; do not analyse until observed_n >= required_n";

/// Named warning. Presence means "do not analyse", not "analyse with caution".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimumNWarning {
    pub observed_n: u64,
    pub required_n: u64,
    pub message: &'static str,
}

/// `Some` when `observed_n` is below [`MIN_N_FOR_ITEM_STATS`].
pub fn minimum_n_warning(observed_n: u64) -> Option<MinimumNWarning> {
    if observed_n < MIN_N_FOR_ITEM_STATS {
        Some(MinimumNWarning {
            observed_n,
            required_n: MIN_N_FOR_ITEM_STATS,
            message: MIN_N_WARNING_MESSAGE,
        })
    } else {
        None
    }
}

/// Export is OFF unless the caller constructs [`ExportPolicy::opt_in`].
/// `bool` defaults to `false`, so [`Default`] is mechanically opt-in-OFF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExportPolicy {
    opt_in: bool,
}

impl ExportPolicy {
    pub fn off() -> Self {
        Self { opt_in: false }
    }

    pub fn opt_in() -> Self {
        Self { opt_in: true }
    }

    pub fn is_opted_in(self) -> bool {
        self.opt_in
    }
}

/// Result of an opted-in export. Always carries the minimum-N warning
/// when the exported N is below the designed-in floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportReceipt {
    pub event_count: u64,
    pub minimum_n: Option<MinimumNWarning>,
}

/// Always `Err`. This crate captures events; it does not estimate IRT.
pub fn estimate_item_response_model(
    _events: &[AttemptEvent],
) -> Result<std::convert::Infallible, AttemptError> {
    Err(AttemptError::PsychometricsRefused("IRT"))
}

/// Always `Err`. Difficulty is sample-dependent; this crate will not compute it.
pub fn compute_item_difficulty(
    _events: &[AttemptEvent],
) -> Result<std::convert::Infallible, AttemptError> {
    Err(AttemptError::PsychometricsRefused("item difficulty"))
}

/// Always `Err`. Discrimination is sample-dependent; this crate will not compute it.
pub fn compute_item_discrimination(
    _events: &[AttemptEvent],
) -> Result<std::convert::Infallible, AttemptError> {
    Err(AttemptError::PsychometricsRefused("item discrimination"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AttemptEvent {
        AttemptEvent::new(
            "item-v1",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "learner-aa11",
            AttemptMode::Quiz,
            1,
            "A",
            true,
            1500,
            1_724_000_000_000,
            0,
        )
        .unwrap()
    }

    #[test]
    fn export_policy_defaults_off() {
        assert!(!ExportPolicy::default().is_opted_in());
        assert!(!ExportPolicy::off().is_opted_in());
        assert!(ExportPolicy::opt_in().is_opted_in());
    }

    #[test]
    fn minimum_n_warning_trips_below_floor_and_clears_at_floor() {
        let w = minimum_n_warning(0).expect("n=0 must warn");
        assert_eq!(w.required_n, MIN_N_FOR_ITEM_STATS);
        assert_eq!(w.observed_n, 0);
        assert!(minimum_n_warning(MIN_N_FOR_ITEM_STATS - 1).is_some());
        assert!(minimum_n_warning(MIN_N_FOR_ITEM_STATS).is_none());
        assert!(minimum_n_warning(MIN_N_FOR_ITEM_STATS + 1).is_none());
    }

    #[test]
    fn record_roundtrips_sqlite_and_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let mut log = AttemptLog::open(dir.path()).unwrap();
        let event = sample();
        log.record(&event).unwrap();
        let loaded = log.events().unwrap();
        assert_eq!(loaded, vec![event.clone()]);

        let jsonl = std::fs::read_to_string(dir.path().join(JSONL_NAME)).unwrap();
        let parsed: AttemptEvent = serde_json::from_str(jsonl.trim()).unwrap();
        assert_eq!(parsed, event);
        assert_eq!(log.count().unwrap(), 1);
    }

    #[test]
    fn export_with_opt_in_writes_jsonl_and_carries_minimum_n_warning() {
        let dir = tempfile::tempdir().unwrap();
        let mut log = AttemptLog::open(dir.path()).unwrap();
        log.record(&sample()).unwrap();
        let mut buf = Vec::new();
        let receipt = log.export_jsonl(&ExportPolicy::opt_in(), &mut buf).unwrap();
        assert_eq!(receipt.event_count, 1);
        assert!(receipt.minimum_n.is_some());
        let line = std::str::from_utf8(&buf).unwrap();
        let parsed: AttemptEvent = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(parsed.item_version, "item-v1");
    }

    #[test]
    fn constructor_rejects_empty_fields_and_unset_clock() {
        assert!(matches!(
            AttemptEvent::new("", "h", "p", AttemptMode::Learn, 1, "A", false, 0, 1, 0),
            Err(AttemptError::EmptyField("item_version"))
        ));
        assert!(matches!(
            AttemptEvent::new("i", "h", "p", AttemptMode::Learn, 1, "A", false, 0, 0, 0),
            Err(AttemptError::TimestampUnset)
        ));
    }

    #[test]
    fn psychometrics_are_refused_even_with_events() {
        let events = [sample()];
        assert!(matches!(
            estimate_item_response_model(&events),
            Err(AttemptError::PsychometricsRefused("IRT"))
        ));
        assert!(matches!(
            compute_item_difficulty(&events),
            Err(AttemptError::PsychometricsRefused("item difficulty"))
        ));
        assert!(matches!(
            compute_item_discrimination(&events),
            Err(AttemptError::PsychometricsRefused("item discrimination"))
        ));
    }
}
