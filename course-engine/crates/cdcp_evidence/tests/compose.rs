//! Conservative composition: property test, known-bad inward plant, meta-test.
//!
//! The load-bearing assertion is [`CONSERVATIVENESS_ASSERTION`]. Deleting it
//! (or the `is_tighter_than` / `is_wider_than` checks that implement it)
//! makes `selftest_delete_conservativeness_assertion_is_nonzero` fail.

use cdcp_evidence::{
    Evidence, ModelEvidence, NumericalCertificate, NumericalKind, SensitivitySummary,
    StatisticalCertificate, ValidityDomain,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

/// Token the property test must interpolate. The selftest keys on this
/// identifier appearing *inside* `combine_never_produces_tighter_enclosure_or_wider_domain`.
const CONSERVATIVENESS_ASSERTION: &str =
    "combine() never produces a tighter enclosure or wider validity domain than either input";

fn arb_finite() -> impl Strategy<Value = f64> {
    any::<f64>().prop_filter("finite", |x| x.is_finite())
}

fn arb_ordered_pair() -> impl Strategy<Value = (f64, f64)> {
    (arb_finite(), arb_finite()).prop_map(|(a, b)| if a <= b { (a, b) } else { (b, a) })
}

fn arb_discrepancy() -> impl Strategy<Value = f64> {
    prop_oneof![Just(0.0), (0.0f64..8.0), Just(f64::INFINITY)]
}

fn arb_domain() -> impl Strategy<Value = ValidityDomain> {
    proptest::collection::btree_map("[a-z]{1,3}", arb_ordered_pair(), 0..3).prop_map(|m| {
        let mut d = ValidityDomain::unconstrained();
        for (k, (lo, hi)) in m {
            d = d.with(k, lo, hi);
        }
        d
    })
}

fn arb_model() -> impl Strategy<Value = ModelEvidence> {
    (
        proptest::collection::vec("[a-z]{1,6}", 0..3),
        proptest::collection::vec("[a-z]{1,8}", 0..3),
        arb_domain(),
        arb_discrepancy(),
        any::<bool>(),
    )
        .prop_map(
            |(cards, assumptions, validity, discrepancy_rel, in_domain)| {
                ModelEvidence::try_new(cards, assumptions, validity, discrepancy_rel, in_domain)
                    .expect("generator only emits valid discrepancy")
            },
        )
}

fn arb_numerical() -> impl Strategy<Value = NumericalCertificate> {
    prop_oneof![
        arb_finite().prop_map(NumericalCertificate::exact),
        arb_ordered_pair().prop_map(|(lo, hi)| NumericalCertificate::enclosure(lo, hi)),
        arb_ordered_pair().prop_map(|(lo, hi)| NumericalCertificate::estimate(lo, hi)),
    ]
}

fn arb_stat() -> impl Strategy<Value = StatisticalCertificate> {
    prop_oneof![
        Just(StatisticalCertificate::None),
        (0.0f64..10.0, 0.01f64..0.99)
            .prop_map(|(e, alpha)| StatisticalCertificate::EValue { e, alpha }),
        (0.0f64..10.0, 0.01f64..0.99).prop_map(|(half_width, confidence)| {
            StatisticalCertificate::HalfWidth {
                half_width,
                confidence,
            }
        }),
    ]
}

fn arb_sensitivity() -> impl Strategy<Value = SensitivitySummary> {
    proptest::collection::btree_map("[a-z]{1,4}", arb_finite(), 0..3)
        .prop_map(|d_qoi| SensitivitySummary { d_qoi })
}

fn arb_evidence() -> impl Strategy<Value = Evidence<f64>> {
    (
        arb_finite(),
        arb_numerical(),
        arb_stat(),
        arb_model(),
        arb_sensitivity(),
    )
        .prop_map(
            |(value, numerical, statistical, model, sensitivity)| Evidence {
                qoi: value,
                value,
                numerical,
                statistical,
                model,
                sensitivity,
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// For any two Evidence values, combine() never produces a tighter
    /// enclosure or a wider validity domain than either input.
    #[test]
    fn combine_never_produces_tighter_enclosure_or_wider_domain(
        a in arb_evidence(),
        b in arb_evidence(),
    ) {
        let c = Evidence::combine(&a, &b, a.value);
        assert!(
            !c.numerical.is_tighter_than(&a.numerical)
                && !c.numerical.is_tighter_than(&b.numerical)
                && !c.model.validity.is_wider_than(&a.model.validity)
                && !c.model.validity.is_wider_than(&b.model.validity),
            "{CONSERVATIVENESS_ASSERTION}; a={:?} b={:?} c.num={:?} c.val={:?}",
            a.numerical.kind,
            b.numerical.kind,
            (c.numerical.lo, c.numerical.hi),
            c.model.validity.bounds()
        );
    }
}

/// Known-bad: inward rounding of the hull. The same predicate the property
/// test uses MUST go red on this plant. If this is green, the property test
/// cannot detect a compose that rounds inward.
#[test]
fn known_bad_inward_rounding_trips_conservativeness() {
    let left = NumericalCertificate::exact(1.0);
    let right = NumericalCertificate::enclosure(0.0, 2.0);
    // Live compose is outward and must stay conservative.
    let live = NumericalCertificate::combine(&left, &right);
    assert!(
        !live.is_tighter_than(&left) && !live.is_tighter_than(&right),
        "live outward compose must not trip the predicate: {live:?}"
    );

    // Plant: hull of [1,1] ∪ [0,2] = [0,2], then INWARD one ulp.
    let inward = NumericalCertificate {
        kind: NumericalKind::Enclosure,
        lo: 0.0_f64.next_up(),
        hi: 2.0_f64.next_down(),
    };
    assert!(
        inward.is_tighter_than(&left) || inward.is_tighter_than(&right),
        "inward-rounding composition must be tighter than an input — \
         if this assertion is green the property test cannot go RED"
    );

    // Degenerate plant: exact ⊕ exact, inward inverts the point interval.
    let point = NumericalCertificate::exact(1.0);
    let inward_point = NumericalCertificate {
        kind: NumericalKind::Enclosure,
        lo: 1.0_f64.next_up(),
        hi: 1.0_f64.next_down(),
    };
    assert!(
        inward_point.is_tighter_than(&point),
        "inward rounding of an exact point must be tighter than the point"
    );
}

/// Meta-test: delete the conservativeness assertion → this selftest is
/// non-zero. Keys on the property-test *body*, not a comment.
#[test]
fn selftest_delete_conservativeness_assertion_is_nonzero() {
    let src = include_str!("compose.rs");
    let mut parts = src.split("fn combine_never_produces_tighter_enclosure_or_wider_domain");
    let _before = parts.next().expect("split");
    let body = parts.next().expect(
        "property test combine_never_produces_tighter_enclosure_or_wider_domain is missing",
    );
    // Cut at the next top-level item so we do not match later tests.
    let body = body
        .split("fn known_bad_inward_rounding")
        .next()
        .unwrap_or(body);
    assert!(
        body.contains("CONSERVATIVENESS_ASSERTION"),
        "delete the conservativeness assertion → selftest non-zero"
    );
    assert!(
        body.contains("is_tighter_than") && body.contains("is_wider_than"),
        "delete the conservativeness assertion → selftest non-zero"
    );
}

/// Evidence::combine is the same hull+intersect+add laws.
#[test]
fn evidence_combine_intersects_and_adds() {
    let a = Evidence::enclosed(1.0, 0.0, 2.0).with_model(
        ModelEvidence::try_new(
            vec!["card-a".into()],
            vec!["steady".into()],
            ValidityDomain::unconstrained().with("Re", 1e4, 1e5),
            0.10,
            true,
        )
        .unwrap(),
    );
    let b = Evidence::enclosed(1.5, 1.0, 3.0).with_model(
        ModelEvidence::try_new(
            vec!["card-b".into()],
            vec!["isothermal".into()],
            ValidityDomain::unconstrained().with("Re", 5e4, 2e5),
            0.05,
            false,
        )
        .unwrap(),
    );
    let c = Evidence::combine(&a, &b, 1.2);
    assert!(!c.numerical.is_tighter_than(&a.numerical));
    assert!(!c.numerical.is_tighter_than(&b.numerical));
    assert!(!c.model.validity.is_wider_than(&a.model.validity));
    assert!(!c.model.validity.is_wider_than(&b.model.validity));
    assert!(!c.model.in_domain, "false must propagate");
    let expected = 0.10 + 0.05;
    assert!(
        !(c.model.discrepancy_rel < expected),
        "discrepancy must not shrink below the sum"
    );
    let mut point_in = BTreeMap::new();
    point_in.insert("Re".into(), 7e4);
    let mut point_out = BTreeMap::new();
    point_out.insert("Re".into(), 2e4);
    assert!(c.model.validity.contains(&point_in));
    assert!(!c.model.validity.contains(&point_out));
}

#[test]
fn statistical_half_widths_add() {
    let a = StatisticalCertificate::HalfWidth {
        half_width: 0.2,
        confidence: 0.95,
    };
    let b = StatisticalCertificate::HalfWidth {
        half_width: 0.3,
        confidence: 0.90,
    };
    match StatisticalCertificate::combine(&a, &b) {
        StatisticalCertificate::HalfWidth {
            half_width,
            confidence,
        } => {
            assert!(!(half_width < 0.5), "half-widths add conservatively");
            assert!(confidence <= 0.90);
        }
        other => panic!("expected half-width, got {other:?}"),
    }
}

#[test]
fn sensitivity_keeps_larger_magnitude() {
    let mut a = SensitivitySummary::default();
    a.d_qoi.insert("k".into(), 1.0);
    let mut b = SensitivitySummary::default();
    b.d_qoi.insert("k".into(), -3.0);
    let c = SensitivitySummary::combine(&a, &b);
    let v = c.d_qoi.get("k").copied().unwrap();
    assert!(v < 0.0 && v.abs() >= 3.0);
}
