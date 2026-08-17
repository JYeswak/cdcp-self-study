//! Known-bad injection per kind + meta-test that the set is non-empty.
//!
//! A planted schema failure that scores green is a broken gate. Each kind
//! below has at least one input that MUST return Err.

use cdcp_assess::{
    ashburn_tmy3_free_cooling_hours_int, Item, Quantity, Ratio, Response, SequenceCredit,
    SetCredit, Tolerance, ToleranceKind, KINDS,
};

struct Plant {
    kind: &'static str,
    why: &'static str,
    item_json: &'static str,
}

/// One planted failure per kind. An empty list is itself a test failure.
fn plants() -> Vec<Plant> {
    vec![
        Plant {
            kind: "single-select",
            why: "one option is vacuous",
            item_json: r#"{"kind":"single-select","options":["only"],"correct":"only"}"#,
        },
        Plant {
            kind: "single-select",
            why: "empty options",
            item_json: r#"{"kind":"single-select","options":[],"correct":"x"}"#,
        },
        Plant {
            kind: "multi-select",
            why: "empty correct set",
            item_json: r#"{"kind":"multi-select","options":["a","b"],"correct":[],"credit":"all-or-nothing"}"#,
        },
        Plant {
            kind: "ordering",
            why: "empty sequence",
            item_json: r#"{"kind":"ordering","elements":[],"credit":"all-or-nothing"}"#,
        },
        Plant {
            kind: "ordering",
            why: "adjacent-pairs on a 1-step key (out_of would be 0)",
            item_json: r#"{"kind":"ordering","elements":["only"],"credit":"adjacent-pairs"}"#,
        },
        Plant {
            kind: "numeric-range",
            why: "bare JSON number is not a quantity",
            item_json: r#"{"kind":"numeric-range","expected":72,"tolerance":{"kind":"absolute","magnitude":{"num":1,"den":1}}}"#,
        },
        Plant {
            kind: "numeric-range",
            why: "missing units (bare number)",
            item_json: r#"{"kind":"numeric-range","expected":{"value":{"num":72,"den":1}},"tolerance":{"kind":"absolute","magnitude":{"num":1,"den":1}}}"#,
        },
        Plant {
            kind: "numeric-range",
            why: "empty units string",
            item_json: r#"{"kind":"numeric-range","expected":{"value":{"num":72,"den":1},"units":""},"tolerance":{"kind":"absolute","magnitude":{"num":1,"den":1}}}"#,
        },
        Plant {
            kind: "numeric-range",
            why: "missing tolerance",
            item_json: r#"{"kind":"numeric-range","expected":{"value":{"num":72,"den":1},"units":"kW"}}"#,
        },
        Plant {
            kind: "numeric-range",
            why: "negative tolerance",
            item_json: r#"{"kind":"numeric-range","expected":{"value":{"num":72,"den":1},"units":"kW"},"tolerance":{"kind":"absolute","magnitude":{"num":-1,"den":1}}}"#,
        },
        Plant {
            kind: "topology-selection",
            why: "correct id not in the universe",
            item_json: r#"{"kind":"topology-selection","elements":["a","b"],"correct":["z"],"credit":"jaccard"}"#,
        },
        Plant {
            kind: "procedural-sequence",
            why: "duplicate step id",
            item_json: r#"{"kind":"procedural-sequence","steps":["a","a"],"credit":"all-or-nothing"}"#,
        },
        Plant {
            kind: "procedural-sequence",
            why: "credit policy omitted (no implicit partial)",
            item_json: r#"{"kind":"procedural-sequence","steps":["a","b"]}"#,
        },
    ]
}

#[test]
fn known_bad_set_is_non_empty() {
    let p = plants();
    assert!(!p.is_empty(), "empty known-bad set is an ERROR, not a pass");
}

#[test]
fn every_kind_has_at_least_one_known_bad() {
    let p = plants();
    for kind in KINDS {
        assert!(
            p.iter().any(|x| x.kind == *kind),
            "kind {kind} has no known-bad plant — a kind without a red fixture cannot be trusted"
        );
    }
}

#[test]
fn every_known_bad_is_red() {
    for plant in plants() {
        let result = Item::from_json(plant.item_json);
        assert!(
            result.is_err(),
            "planted {} ({}) scored green: {:?}",
            plant.kind,
            plant.why,
            result
        );
    }
}

#[test]
fn sequence_credit_has_no_serde_default() {
    // A missing `credit` must not become all-or-nothing or position-matches.
    let err = Item::from_json(r#"{"kind":"ordering","elements":["a","b","c"]}"#).unwrap_err();
    match err {
        cdcp_assess::AssessError::Json(_) => {}
        other => panic!("missing credit must be a schema/json error, got {other:?}"),
    }
}

#[test]
fn rust_constructors_reject_the_same_plants() {
    assert!(Item::single_select(["only"], "only").is_err());
    assert!(Item::multi_select(["a", "b"], Vec::<&str>::new(), SetCredit::AllOrNothing).is_err());
    assert!(Item::ordering(Vec::<&str>::new(), SequenceCredit::AllOrNothing).is_err());
    assert!(Item::ordering(["only"], SequenceCredit::AdjacentPairs).is_err());
    assert!(Quantity::new(Ratio::from_int(72), "").is_err());
    assert!(Tolerance::new(ToleranceKind::Absolute, Ratio::from_int(-1)).is_err());
    assert!(Item::topology_selection(["a", "b"], ["z"], SetCredit::Jaccard).is_err());
    assert!(Item::procedural_sequence(["a", "a"], SequenceCredit::AllOrNothing).is_err());
    assert!(ashburn_tmy3_free_cooling_hours_int(-1).is_err());
}

#[test]
fn scoring_sources_contain_no_floating_types() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = 0usize;
    let mut hits = Vec::new();
    for ent in std::fs::read_dir(&root).expect("src") {
        let ent = ent.unwrap();
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        files += 1;
        let text = std::fs::read_to_string(&path).unwrap();
        for (i, line) in text.lines().enumerate() {
            // Comments may mention the ban; only flag type tokens.
            let code = line.split("//").next().unwrap_or(line);
            if code.contains("f32") || code.contains("f64") {
                hits.push(format!("{}:{}:{code}", path.display(), i + 1));
            }
        }
    }
    assert!(files >= 4, "empty scan of src/ is an ERROR (found {files})");
    assert!(
        hits.is_empty(),
        "floating-point type in scoring crate:\n  {}",
        hits.join("\n  ")
    );
}

#[test]
fn response_bare_number_is_schema_error() {
    let err = Response::from_json(r#"{"kind":"numeric-range","submitted":72}"#).unwrap_err();
    match err {
        cdcp_assess::AssessError::Json(_) | cdcp_assess::AssessError::BareNumber => {}
        other => panic!("bare submitted number must be schema ERROR, got {other:?}"),
    }
    let err = Response::from_json(
        r#"{"kind":"numeric-range","submitted":{"value":{"num":72,"den":1},"units":""}}"#,
    )
    .unwrap_err();
    match err {
        cdcp_assess::AssessError::Json(_) | cdcp_assess::AssessError::BareNumber => {}
        other => panic!("empty units must be BareNumber, got {other:?}"),
    }
}
