//! Known-bad: assemble must refuse non-letter assess kinds.
//!
//! Bead: `bd-hardening-g-assess-64t.2`.
//!
//! A multi-select or numeric-range item in the assemble input is RED
//! (`AssembleError::NotLetterMcq`), never four shuffled strings. Flattening
//! those kinds back to A–D would be a better answer-key generator inside the
//! same recognition interface.
//!
//! FLOOR-RAISE: this suite asserts assemble will not present a non-letter
//! kind as a letter form. It cannot decide that a letter-MCQ item is any
//! good, and it does not migrate the 804-item bank.

use cdcp_assemble::{
    admit_assemble_kind, assemble_input, AssembleConfig, AssembleError, AssembleInput,
    LETTER_ASSEMBLE_KINDS,
};
use cdcp_assess::{
    lift_letter_mcq, Item, Quantity, Ratio, SequenceCredit, SetCredit, Tolerance, ToleranceKind,
    KINDS,
};
use cdcp_bank::{BankItem, ItemStatus};

const SEED: u64 = 42;

fn cfg() -> AssembleConfig {
    AssembleConfig {
        n_items: 1,
        max_per_module: 8,
        min_modules: 1,
        shuffle_choices: true,
    }
}

fn letter_bank_item(id: &str) -> BankItem {
    BankItem {
        id: id.to_string(),
        module: 1,
        stem: format!("stem for {id}"),
        choices: vec![
            "alpha".into(),
            "bravo".into(),
            "charlie".into(),
            "delta".into(),
        ],
        correct: "B".into(),
        explanation: "because reasons here".into(),
        topic_ids: vec![format!("topic-{id}")],
        objective_ids: Vec::new(),
        citation_ids: Vec::new(),
        tags: Vec::new(),
        bloom: "understand".into(),
        source_class: "original".into(),
        quantity_evidence: "qualitative_only".into(),
        status: ItemStatus::Approved,
    }
}

fn plant_item(kind: &str) -> Item {
    match kind {
        "multi-select" => Item::multi_select(
            ["alpha", "bravo", "charlie", "delta"],
            ["alpha", "charlie"],
            SetCredit::AllOrNothing,
        )
        .expect("valid multi-select plant"),
        "ordering" => Item::ordering(["first", "second", "third"], SequenceCredit::AllOrNothing)
            .expect("valid ordering plant"),
        "numeric-range" => Item::numeric_range(
            Quantity::new(Ratio::from_int(72), "kW").expect("quantity"),
            Tolerance::new(ToleranceKind::Absolute, Ratio::from_int(1)).expect("tolerance"),
        )
        .expect("valid numeric-range plant"),
        "topology-selection" => Item::topology_selection(
            ["ups", "sts", "pdu", "ats"],
            ["ups", "pdu"],
            SetCredit::Jaccard,
        )
        .expect("valid topology plant"),
        "procedural-sequence" => Item::procedural_sequence(
            ["isolate", "lock", "tag", "try"],
            SequenceCredit::AllOrNothing,
        )
        .expect("valid procedure plant"),
        other => panic!("no plant constructor for {other}"),
    }
}

fn refused_kinds() -> Vec<&'static str> {
    KINDS
        .iter()
        .copied()
        .filter(|k| !LETTER_ASSEMBLE_KINDS.contains(k))
        .collect()
}

fn assert_not_letter(err: AssembleError, id: &str, kind: &str) {
    match err {
        AssembleError::NotLetterMcq {
            id: got_id,
            kind: got_kind,
        } => {
            assert_eq!(got_id, id, "named error must name the planted id");
            assert_eq!(got_kind, kind, "named error must name the planted kind");
            let msg = format!(
                "{}",
                AssembleError::NotLetterMcq {
                    id: got_id,
                    kind: got_kind,
                }
            );
            assert!(
                msg.contains("will not flatten") && msg.contains("A–D"),
                "named error must say it will not flatten to A–D, got {msg}"
            );
        }
        other => panic!("planted {kind} must be NotLetterMcq, got {other:?}"),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Known-bad plants (the flatten temptation: four option strings + shuffle)
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn planted_multi_select_is_refused_not_flattened() {
    let letter = letter_bank_item("ok-letter");
    let plant = plant_item("multi-select");
    let input = [
        AssembleInput::LetterMcq(&letter),
        AssembleInput::Assess {
            id: "plant-ms",
            module: 1,
            stem: "select all that apply",
            item: &plant,
        },
    ];
    let err = assemble_input(&input, SEED, cfg()).expect_err("multi-select must be RED");
    assert_not_letter(err, "plant-ms", "multi-select");
}

#[test]
fn planted_numeric_range_is_refused_not_flattened() {
    let letter = letter_bank_item("ok-letter");
    let plant = plant_item("numeric-range");
    let input = [
        AssembleInput::LetterMcq(&letter),
        AssembleInput::Assess {
            id: "plant-nr",
            module: 1,
            stem: "nameplate kW",
            item: &plant,
        },
    ];
    let err = assemble_input(&input, SEED, cfg()).expect_err("numeric-range must be RED");
    assert_not_letter(err, "plant-nr", "numeric-range");
}

#[test]
fn every_non_letter_kind_is_refused() {
    let kinds = refused_kinds();
    assert!(
        !kinds.is_empty(),
        "empty refuse set is an ERROR — a gate with nothing to refuse cannot trip"
    );
    for kind in kinds {
        let plant = plant_item(kind);
        let input = [AssembleInput::Assess {
            id: "plant",
            module: 1,
            stem: "planted",
            item: &plant,
        }];
        let err = assemble_input(&input, SEED, cfg())
            .expect_err(&format!("{kind} must be RED, not four shuffled strings"));
        assert_not_letter(err, "plant", kind);
    }
}

#[test]
fn every_assess_kind_is_either_admitted_or_planted_red() {
    for kind in KINDS {
        if LETTER_ASSEMBLE_KINDS.contains(kind) {
            admit_assemble_kind("ok", kind).unwrap_or_else(|e| {
                panic!("{kind} is an admitted assemble kind, got {e:?}");
            });
        } else {
            assert!(
                refused_kinds().contains(kind),
                "kind {kind} has no refuse plant — a kind without a red fixture cannot be trusted"
            );
        }
    }
}

#[test]
fn unknown_kind_string_is_refused() {
    let err = admit_assemble_kind("x", "true-false").expect_err("unknown kind must fail closed");
    assert_not_letter(err, "x", "true-false");
}

#[test]
fn empty_assemble_input_is_an_error_not_an_empty_exam() {
    match assemble_input(&[], SEED, cfg()) {
        Err(AssembleError::EmptyInput) => {}
        other => panic!("empty input must be EmptyInput, got {other:?}"),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// GOOD controls (without these the plants could pass because nothing assembled)
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn letter_mcq_bank_item_is_admitted() {
    let item = letter_bank_item("ok-letter");
    let out = assemble_input(&[AssembleInput::LetterMcq(&item)], SEED, cfg())
        .expect("letter-mcq bank item must assemble");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "ok-letter");
    assert_eq!(out[0].choices.len(), 4);
    assert_eq!(out[0].original_correct, "B");
    assert!(
        matches!(out[0].correct.as_str(), "A" | "B" | "C" | "D"),
        "letter-mcq remapped correct must stay a letter, got {:?}",
        out[0].correct
    );
    assert_eq!(
        &out[0].choices[letter_pos(&out[0].correct)],
        "bravo",
        "shuffle must preserve answer text, not invent a flatten"
    );
}

#[test]
fn lift_letter_mcq_single_select_is_admitted() {
    let item = lift_letter_mcq("C").expect("lift");
    let out = assemble_input(
        &[AssembleInput::Assess {
            id: "lifted",
            module: 2,
            stem: "lifted letter",
            item: &item,
        }],
        SEED,
        AssembleConfig {
            shuffle_choices: false,
            ..cfg()
        },
    )
    .expect("lift_letter_mcq is the admitted single-select lift");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].choices, ["A", "B", "C", "D"]);
    assert_eq!(out[0].correct, "C");
    assert_eq!(out[0].original_correct, "C");
}

#[test]
fn semantic_single_select_is_admitted_and_not_rewritten_to_letters() {
    let item = Item::single_select(["utility", "genset", "both", "neither"], "genset")
        .expect("semantic single-select");
    let out = assemble_input(
        &[AssembleInput::Assess {
            id: "sem",
            module: 6,
            stem: "which source",
            item: &item,
        }],
        SEED,
        AssembleConfig {
            shuffle_choices: false,
            ..cfg()
        },
    )
    .expect("single-select is an admitted kind");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].choices, ["utility", "genset", "both", "neither"]);
    assert_eq!(out[0].correct, "genset");
    assert!(
        !matches!(out[0].correct.as_str(), "A" | "B" | "C" | "D"),
        "semantic single-select must not be flattened to a letter, got {:?}",
        out[0].correct
    );
}

fn letter_pos(letter: &str) -> usize {
    match letter {
        "A" => 0,
        "B" => 1,
        "C" => 2,
        "D" => 3,
        other => panic!("not a letter: {other}"),
    }
}
