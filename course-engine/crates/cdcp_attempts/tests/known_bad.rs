//! Known-bad plants: empty store, export without opt-in, psychometric
//! fields in src. An empty plant list is itself a test failure.

use cdcp_attempts::{
    compute_item_difficulty, compute_item_discrimination, estimate_item_response_model,
    AttemptError, AttemptEvent, AttemptLog, AttemptMode, ExportPolicy, EMPTY_STORE,
    EXPORT_NOT_OPTED_IN, PSYCHOMETRICS_REFUSED,
};

struct Plant {
    why: &'static str,
}

/// One planted failure per required red path. An empty list is itself a
/// test failure (anti-vacuous).
fn plants() -> Vec<Plant> {
    vec![
        Plant {
            why: "empty store is an ERROR on read, never a zero-N analysis",
        },
        Plant {
            why: "export without opt-in is an ERROR even when events exist",
        },
        Plant {
            why: "IRT / difficulty / discrimination are refused at any N",
        },
    ]
}

fn sample() -> AttemptEvent {
    AttemptEvent::new(
        "item-v1",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "learner-aa11",
        AttemptMode::Mock,
        1,
        "B",
        false,
        800,
        1_724_000_000_000,
        0,
    )
    .unwrap()
}

#[test]
fn known_bad_set_is_non_empty() {
    let p = plants();
    assert!(!p.is_empty(), "empty known-bad set is an ERROR, not a pass");
    for plant in &p {
        assert!(
            !plant.why.is_empty(),
            "a plant with an empty why cannot be reviewed"
        );
    }
}

#[test]
fn empty_store_is_error_on_read_and_on_opted_in_export() {
    let dir = tempfile::tempdir().unwrap();
    let log = AttemptLog::open(dir.path()).unwrap();
    assert_eq!(log.count().unwrap(), 0);

    match log.events() {
        Err(AttemptError::EmptyStore) => {}
        other => panic!("empty store events() must be EmptyStore, got {other:?}"),
    }

    let mut buf = Vec::new();
    match log.export_jsonl(&ExportPolicy::opt_in(), &mut buf) {
        Err(AttemptError::EmptyStore) => {}
        other => panic!("empty store export must be EmptyStore, got {other:?}"),
    }
    assert!(
        EMPTY_STORE.contains("empty"),
        "EMPTY_STORE token must stay load-bearing"
    );
}

#[test]
fn export_without_opt_in_is_error() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = AttemptLog::open(dir.path()).unwrap();
    log.record(&sample()).unwrap();

    let mut buf = Vec::new();
    match log.export_jsonl(&ExportPolicy::default(), &mut buf) {
        Err(AttemptError::ExportNotOptedIn) => {}
        other => panic!("default policy export must be ExportNotOptedIn, got {other:?}"),
    }
    match log.export_jsonl(&ExportPolicy::off(), &mut buf) {
        Err(AttemptError::ExportNotOptedIn) => {}
        other => panic!("off policy export must be ExportNotOptedIn, got {other:?}"),
    }
    assert!(buf.is_empty(), "refused export must write no bytes");
    assert!(
        EXPORT_NOT_OPTED_IN.contains("opt-in"),
        "EXPORT_NOT_OPTED_IN token must stay load-bearing"
    );
}

#[test]
fn psychometrics_are_refused_on_empty_and_non_empty() {
    assert!(matches!(
        estimate_item_response_model(&[]),
        Err(AttemptError::PsychometricsRefused(_))
    ));
    let events = [sample()];
    assert!(matches!(
        compute_item_difficulty(&events),
        Err(AttemptError::PsychometricsRefused(_))
    ));
    assert!(matches!(
        compute_item_discrimination(&events),
        Err(AttemptError::PsychometricsRefused(_))
    ));
    assert!(
        PSYCHOMETRICS_REFUSED.contains("refuses"),
        "PSYCHOMETRICS_REFUSED token must stay load-bearing"
    );
}

/// No `f32`/`f64` and no `difficulty`/`discrimination` field in src.
/// Comments and refuse-function names may mention the banned words.
#[test]
fn src_has_no_float_types_and_no_psychometric_fields() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = 0usize;
    let mut float_hits = Vec::new();
    let mut field_hits = Vec::new();
    for ent in std::fs::read_dir(&root).expect("src") {
        let ent = ent.unwrap();
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        files += 1;
        let text = std::fs::read_to_string(&path).unwrap();
        for (i, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or(line);
            if code.contains("f32") || code.contains("f64") {
                float_hits.push(format!("{}:{}:{code}", path.display(), i + 1));
            }
            // Field binding: `difficulty:` / `discrimination:` — not a
            // function name like compute_item_difficulty(.
            if field_token(code, "difficulty") || field_token(code, "discrimination") {
                field_hits.push(format!("{}:{}:{code}", path.display(), i + 1));
            }
        }
    }
    assert!(files >= 3, "empty scan of src/ is an ERROR (found {files})");
    assert!(
        float_hits.is_empty(),
        "floating-point type in attempt crate:\n  {}",
        float_hits.join("\n  ")
    );
    assert!(
        field_hits.is_empty(),
        "psychometric field in attempt crate:\n  {}",
        field_hits.join("\n  ")
    );
}

fn field_token(code: &str, name: &str) -> bool {
    let mut rest = code;
    while let Some(at) = rest.find(name) {
        let after = &rest[at + name.len()..];
        let trimmed = after.trim_start();
        if trimmed.starts_with(':') && !trimmed.starts_with("::") {
            return true;
        }
        rest = &rest[at + name.len()..];
    }
    false
}
