//! Scoring laws for the implemented kinds. Integers and rationals only.
use cdcp_assess::{
    lift_letter_mcq, score, score_digest, Item, Quantity, Ratio, Response, SequenceCredit,
    SetCredit, Tolerance, ToleranceKind,
};

#[test]
fn single_select_semantic_ids_not_letters() {
    let item = Item::single_select(
        ["utility", "genset", "both- paralleled", "neither"],
        "genset",
    )
    .unwrap();
    assert_eq!(item.kind_name(), "single-select");
    let ok = score(&item, &Response::single_select("genset").unwrap()).unwrap();
    assert!(ok.is_full());
    assert_eq!((ok.earned(), ok.out_of()), (1, 1));
    let miss = score(&item, &Response::single_select("utility").unwrap()).unwrap();
    assert!(miss.is_zero());
    assert_eq!((miss.earned(), miss.out_of()), (0, 1));
}

#[test]
fn single_select_unknown_id_is_error_not_zero() {
    let item = Item::single_select(["utility", "genset"], "genset").unwrap();
    let err = score(&item, &Response::single_select("battery").unwrap()).unwrap_err();
    assert!(matches!(err, cdcp_assess::AssessError::UnknownId(_)));
}

#[test]
fn lift_letter_mcq_preserves_letter_equality_grade() {
    let item = lift_letter_mcq("C").unwrap();
    assert!(score(&item, &Response::single_select("C").unwrap())
        .unwrap()
        .is_full());
    assert!(score(&item, &Response::single_select("A").unwrap())
        .unwrap()
        .is_zero());
    // The lift is still a single-select, not a four-letter core type.
    assert_eq!(item.kind_name(), "single-select");
}

#[test]
fn numeric_range_absolute_tolerance_integer() {
    let item = Item::numeric_range(
        Quantity::new(Ratio::from_int(72), "kW").unwrap(),
        Tolerance::new(ToleranceKind::Absolute, Ratio::from_int(1)).unwrap(),
    )
    .unwrap();
    assert_eq!(item.kind_name(), "numeric-range");
    for n in [71, 72, 73] {
        let r = Response::numeric_range(Quantity::new(Ratio::from_int(n), "kW").unwrap()).unwrap();
        assert!(
            score(&item, &r).unwrap().is_full(),
            "{n} kW should be inside ±1 kW of 72"
        );
    }
    for n in [70, 74] {
        let r = Response::numeric_range(Quantity::new(Ratio::from_int(n), "kW").unwrap()).unwrap();
        assert!(
            score(&item, &r).unwrap().is_zero(),
            "{n} kW should be outside ±1 kW of 72"
        );
    }
}

#[test]
fn numeric_range_relative_tolerance_rational() {
    // 100 A ± 1/10 of expected → [90, 110]
    let item = Item::numeric_range(
        Quantity::new(Ratio::from_int(100), "A").unwrap(),
        Tolerance::new(ToleranceKind::Relative, Ratio::new(1, 10).unwrap()).unwrap(),
    )
    .unwrap();
    let inside = Response::numeric_range(Quantity::new(Ratio::from_int(90), "A").unwrap()).unwrap();
    let edge = Response::numeric_range(Quantity::new(Ratio::from_int(110), "A").unwrap()).unwrap();
    let outside =
        Response::numeric_range(Quantity::new(Ratio::from_int(89), "A").unwrap()).unwrap();
    assert!(score(&item, &inside).unwrap().is_full());
    assert!(score(&item, &edge).unwrap().is_full());
    assert!(score(&item, &outside).unwrap().is_zero());
}

#[test]
fn numeric_range_fractional_expected() {
    // 5/2 kW ± 1/10 kW absolute → [2.4, 2.6]
    let item = Item::numeric_range(
        Quantity::new(Ratio::new(5, 2).unwrap(), "kW").unwrap(),
        Tolerance::new(ToleranceKind::Absolute, Ratio::new(1, 10).unwrap()).unwrap(),
    )
    .unwrap();
    let hit =
        Response::numeric_range(Quantity::new(Ratio::new(12, 5).unwrap(), "kW").unwrap()).unwrap(); // 2.4
    let miss =
        Response::numeric_range(Quantity::new(Ratio::new(23, 10).unwrap(), "kW").unwrap()).unwrap(); // 2.3
    assert!(score(&item, &hit).unwrap().is_full());
    assert!(score(&item, &miss).unwrap().is_zero());
}

#[test]
fn numeric_range_unit_mismatch_is_error_not_zero() {
    let item = Item::numeric_range(
        Quantity::new(Ratio::from_int(72), "kW").unwrap(),
        Tolerance::new(ToleranceKind::Absolute, Ratio::from_int(1)).unwrap(),
    )
    .unwrap();
    let got = Response::numeric_range(Quantity::new(Ratio::from_int(72), "MW").unwrap()).unwrap();
    assert!(matches!(
        score(&item, &got),
        Err(cdcp_assess::AssessError::UnitMismatch { .. })
    ));
}

#[test]
fn ordering_all_or_nothing_refuses_partial() {
    let key = ["isolate", "verify", "ground", "work"];
    let item = Item::ordering(key, SequenceCredit::AllOrNothing).unwrap();
    assert_eq!(item.kind_name(), "ordering");
    let exact = Response::ordering(key).unwrap();
    assert!(score(&item, &exact).unwrap().is_full());
    // First two correct, last two swapped — partial is REFUSED.
    let swapped = Response::ordering(["isolate", "verify", "work", "ground"]).unwrap();
    let s = score(&item, &swapped).unwrap();
    assert!(s.is_zero(), "all-or-nothing must not award partial credit");
}

#[test]
fn ordering_position_matches_is_explicit_partial() {
    let item = Item::ordering(
        ["isolate", "verify", "ground", "work"],
        SequenceCredit::PositionMatches,
    )
    .unwrap();
    let swapped = Response::ordering(["isolate", "verify", "work", "ground"]).unwrap();
    let s = score(&item, &swapped).unwrap();
    assert_eq!((s.earned(), s.out_of()), (1, 2)); // 2/4 reduces
}

#[test]
fn ordering_adjacent_pairs_is_explicit_partial() {
    let item = Item::ordering(
        ["isolate", "verify", "ground", "work"],
        SequenceCredit::AdjacentPairs,
    )
    .unwrap();
    // (isolate,verify) present; (verify,ground) broken; (ground,work) broken.
    let got = Response::ordering(["isolate", "verify", "work", "ground"]).unwrap();
    let s = score(&item, &got).unwrap();
    assert_eq!((s.earned(), s.out_of()), (1, 3));
}

#[test]
fn procedural_sequence_all_or_nothing_and_prefix_partial() {
    let key = ["alarm", "investigate", "isolate", "restore"];
    let refuse = Item::procedural_sequence(key, SequenceCredit::AllOrNothing).unwrap();
    let partial = Item::procedural_sequence(key, SequenceCredit::PositionMatches).unwrap();
    assert_eq!(refuse.kind_name(), "procedural-sequence");
    let prefix =
        Response::procedural_sequence(["alarm", "investigate", "restore", "isolate"]).unwrap();
    assert!(score(&refuse, &prefix).unwrap().is_zero());
    let s = score(&partial, &prefix).unwrap();
    assert_eq!((s.earned(), s.out_of()), (1, 2)); // 2/4
}

#[test]
fn multi_select_all_or_nothing_and_jaccard() {
    let opts = ["A-side", "B-side", "tie", "spare"];
    let nothing = Item::multi_select(opts, ["A-side", "B-side"], SetCredit::AllOrNothing).unwrap();
    let jaccard = Item::multi_select(opts, ["A-side", "B-side"], SetCredit::Jaccard).unwrap();
    let subset = Response::multi_select(["A-side"]).unwrap();
    assert!(score(&nothing, &subset).unwrap().is_zero());
    // ∩=1 ∪=2 → 1/2
    let s = score(&jaccard, &subset).unwrap();
    assert_eq!((s.earned(), s.out_of()), (1, 2));
    let exact = Response::multi_select(["B-side", "A-side"]).unwrap();
    assert!(score(&nothing, &exact).unwrap().is_full());
    assert!(score(&jaccard, &exact).unwrap().is_full());
}

#[test]
fn topology_selection_jaccard() {
    let item = Item::topology_selection(
        ["utility", "ats", "ups", "pdu", "it"],
        ["utility", "ats", "ups", "pdu"],
        SetCredit::Jaccard,
    )
    .unwrap();
    assert_eq!(item.kind_name(), "topology-selection");
    let extra = Response::topology_selection(["utility", "ats", "ups", "pdu", "it"]).unwrap();
    // ∩=4 ∪=5 → 4/5
    let s = score(&item, &extra).unwrap();
    assert_eq!((s.earned(), s.out_of()), (4, 5));
}

#[test]
fn kind_mismatch_is_error() {
    let item = Item::single_select(["a", "b"], "a").unwrap();
    let resp = Response::multi_select(["a"]).unwrap();
    assert!(matches!(
        score(&item, &resp),
        Err(cdcp_assess::AssessError::KindMismatch { .. })
    ));
}

#[test]
fn identical_fixtures_byte_identical_digests() {
    let item = Item::numeric_range(
        Quantity::new(Ratio::new(5, 2).unwrap(), "kW").unwrap(),
        Tolerance::new(ToleranceKind::Absolute, Ratio::new(1, 10).unwrap()).unwrap(),
    )
    .unwrap();
    let resp =
        Response::numeric_range(Quantity::new(Ratio::new(12, 5).unwrap(), "kW").unwrap()).unwrap();
    let a = score_digest(&item, &resp).unwrap();
    let b = score_digest(&item, &resp).unwrap();
    assert_eq!(a, b);
    // Re-parse through JSON (the dual-path payload) and match.
    let item_json = serde_json::to_string(&item).unwrap();
    let resp_json = serde_json::to_string(&resp).unwrap();
    assert_eq!(
        a,
        cdcp_assess::score_digest_json(&item_json, &resp_json).unwrap()
    );
}
