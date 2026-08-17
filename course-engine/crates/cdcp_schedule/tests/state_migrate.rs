//! Versioned schedule-state + the one migration (v0 unversioned → v1).
//!
//! Unknown version is ERROR. An empty known-bad set is itself an ERROR.

use cdcp_schedule::{
    migrate_card, migrate_state_version, ScheduleCard, ScheduleError, STATE_VERSION,
    STATE_VERSION_UNVERSIONED,
};

/// Future / garbage versions that MUST stay RED. Empty = vacuous pass.
const KNOWN_BAD_VERSIONS: &[u32] = &[2, 99];

#[test]
fn known_bad_set_is_non_empty() {
    assert!(
        !KNOWN_BAD_VERSIONS.is_empty(),
        "empty known_bad is an ERROR, not a pass"
    );
}

#[test]
fn unversioned_migrates_to_current_identity_on_fields() {
    assert_eq!(STATE_VERSION_UNVERSIONED, 0);
    assert_eq!(STATE_VERSION, 1);
    assert_eq!(
        migrate_state_version(STATE_VERSION_UNVERSIONED).unwrap(),
        STATE_VERSION
    );
    let card = ScheduleCard {
        version: STATE_VERSION_UNVERSIONED,
        interval_days: 3,
        due_at_ms: 42,
        reps: 1,
        lapses: 2,
        updated_at_ms: 41,
    };
    let got = migrate_card(card).unwrap();
    assert_eq!(got.version, STATE_VERSION);
    assert_eq!(got.interval_days, 3);
    assert_eq!(got.due_at_ms, 42);
    assert_eq!(got.reps, 1);
    assert_eq!(got.lapses, 2);
    assert_eq!(got.updated_at_ms, 41);
}

#[test]
fn current_version_is_identity() {
    let card = ScheduleCard::new(3, 42, 1, 0, 41);
    assert_eq!(migrate_card(card).unwrap(), card);
    assert_eq!(migrate_state_version(STATE_VERSION).unwrap(), STATE_VERSION);
}

#[test]
fn known_bad_unknown_version_is_error() {
    for &version in KNOWN_BAD_VERSIONS {
        assert_eq!(
            migrate_state_version(version),
            Err(ScheduleError::UnknownVersion(version)),
            "version {version} must be ERROR"
        );
        let card = ScheduleCard {
            version,
            interval_days: 1,
            due_at_ms: 0,
            reps: 0,
            lapses: 0,
            updated_at_ms: 0,
        };
        assert_eq!(
            migrate_card(card),
            Err(ScheduleError::UnknownVersion(version))
        );
    }
}
