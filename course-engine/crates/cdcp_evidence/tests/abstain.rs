//! Query-time abstention: out of domain is a refusal, not a caveat.
//!
//! The load-bearing token is [`DOMAIN_CHECK`]. Deleting the bound check
//! (or the token the query body interpolates) makes
//! `selftest_delete_domain_check_is_nonzero` fail.

use cdcp_evidence::{
    CardError, Correlation, Evidence, ModelCard, ModelEvidence, ValidityDomain, ViolationKind,
    DOMAIN_CHECK,
};
use std::collections::BTreeMap;

/// Dittus–Boelter-shaped test card. Assumptions are the idealizations;
/// the validity box is the refuse boundary.
fn dittus_boelter() -> Correlation {
    let card = ModelCard::try_new(
        "dittus-boelter-heating",
        vec![
            "fully developed turbulent pipe flow".into(),
            "smooth circular duct".into(),
            "constant properties at bulk temperature".into(),
            "heating (n = 0.4)".into(),
        ],
        ValidityDomain::unconstrained()
            .with("Re", 1e4, 1e6)
            .with("Pr", 0.6, 160.0),
        0.25,
    )
    .expect("fixture card");
    Correlation::new(card)
}

fn in_domain_point() -> BTreeMap<String, f64> {
    let mut p = BTreeMap::new();
    p.insert("Re".into(), 2e4);
    p.insert("Pr".into(), 0.7);
    p
}

fn nu(point: &BTreeMap<String, f64>) -> f64 {
    let re = point["Re"];
    let pr = point["Pr"];
    0.023 * re.powf(0.8) * pr.powf(0.4)
}

#[test]
fn model_card_names_idealizing_assumptions() {
    let card = dittus_boelter().card().clone();
    assert!(!card.assumptions().is_empty());
    assert!(card
        .assumptions()
        .iter()
        .any(|a| a.contains("fully developed")));
    let ev = card.query(&in_domain_point()).expect("in domain");
    assert_eq!(ev.assumptions, card.assumptions());
    assert_eq!(ev.cards, vec![card.name().to_string()]);
}

#[test]
fn model_card_rejects_empty_or_blank_assumptions() {
    let d = ValidityDomain::unconstrained().with("x", 0.0, 1.0);
    assert!(matches!(
        ModelCard::try_new("c", vec![], d.clone(), 0.0),
        Err(CardError::EmptyAssumptions)
    ));
    assert!(matches!(
        ModelCard::try_new("c", vec!["   ".into()], d.clone(), 0.0),
        Err(CardError::BlankAssumption)
    ));
    assert!(matches!(
        ModelCard::try_new("  ", vec!["steady".into()], d, 0.0),
        Err(CardError::EmptyName)
    ));
}

#[test]
fn in_domain_query_returns_a_value() {
    let c = dittus_boelter();
    let ev = c
        .query(&in_domain_point(), nu)
        .expect("in-domain query must produce a value");
    assert!(ev.model.in_domain);
    assert!(ev.value > 0.0);
    assert!(
        ev.model
            .assumptions
            .iter()
            .any(|a| a.contains("fully developed")),
        "assumptions must travel with the value"
    );
}

#[test]
fn query_outside_bound_returns_abstention_not_a_value() {
    let c = dittus_boelter();
    let mut point = in_domain_point();
    point.insert("Re".into(), 2e6);
    match c.query(&point, nu) {
        Ok(ev) => panic!("out-of-domain query must not return a value: {ev:?}"),
        Err(abs) => {
            assert!(abs.is_domain_refusal(), "{abs}");
            assert!(
                abs.violations
                    .iter()
                    .any(|v| v.param == "Re" && v.kind == ViolationKind::OutOfRange),
                "{abs:?}"
            );
        }
    }
}

#[test]
fn missing_parameter_abstains() {
    let c = dittus_boelter();
    let mut point = BTreeMap::new();
    point.insert("Re".into(), 2e4);
    let err = c.query(&point, nu).expect_err("Pr missing");
    assert!(
        err.violations
            .iter()
            .any(|v| v.param == "Pr" && v.kind == ViolationKind::Missing),
        "{err:?}"
    );
}

/// Known-bad: query a correlation 1 ulp outside its bound → abstain.
/// The formula must not run; a value-with-caveat is the failure mode.
#[test]
fn known_bad_one_ulp_outside_bound_abstains() {
    let c = dittus_boelter();
    let hi = 1e6_f64;
    let mut on_bound = in_domain_point();
    on_bound.insert("Re".into(), hi);
    assert!(
        c.query(&on_bound, nu).is_ok(),
        "the inclusive endpoint is in-domain"
    );

    let mut one_ulp = on_bound;
    one_ulp.insert("Re".into(), hi.next_up());
    let result = c.query(&one_ulp, |_| {
        panic!("formula must not run 1 ulp outside the bound")
    });
    match result {
        Ok(ev) => panic!(
            "1 ulp outside must abstain, not return a value (in_domain={})",
            ev.model.in_domain
        ),
        Err(abs) => {
            assert!(abs.to_string().contains(DOMAIN_CHECK), "{abs}");
            assert!(
                abs.violations.iter().any(|v| {
                    v.param == "Re"
                        && v.kind == ViolationKind::OutOfRange
                        && v.value == Some(hi.next_up())
                }),
                "{abs:?}"
            );
        }
    }
}

/// Meta-test: delete the domain check → this selftest is non-zero.
/// Keys on the query *body*, not a comment.
#[test]
fn selftest_delete_domain_check_is_nonzero() {
    let src = include_str!("../src/card.rs");
    let mut parts = src.split("fn query<");
    let _before = parts.next().expect("split");
    let body = parts.next().expect("Correlation::query is missing");
    // Cut at the next item so we do not match later tests / impls.
    let body = body.split("impl ").next().unwrap_or(body);
    assert!(
        body.contains("DOMAIN_CHECK"),
        "delete the domain check → selftest non-zero"
    );
    assert!(
        body.contains("query(") || body.contains("check("),
        "delete the domain check → selftest non-zero"
    );
}

#[test]
fn in_domain_false_cannot_be_reset_true_by_any_composition() {
    let out = Evidence::exact(1.0).with_model(ModelEvidence {
        in_domain: false,
        ..ModelEvidence::none()
    });
    let inn = Evidence::exact(2.0).with_model(ModelEvidence::none());

    let via_model = ModelEvidence::combine(&out.model, &inn.model);
    assert!(!via_model.in_domain, "ModelEvidence::combine must AND");
    let via_model_rev = ModelEvidence::combine(&inn.model, &out.model);
    assert!(!via_model_rev.in_domain);

    let via_ev = Evidence::combine(&out, &inn, 0.0);
    assert!(!via_ev.model.in_domain, "Evidence::combine must AND");
    let via_ev_rev = Evidence::combine(&inn, &out, 0.0);
    assert!(!via_ev_rev.model.in_domain);

    match Evidence::combine_queried(Ok(out.clone()), Ok(inn.clone()), 0.0) {
        Ok(ev) => panic!("combine_queried must not mint a value from in_domain=false: {ev:?}"),
        Err(abs) => assert!(
            abs.violations
                .iter()
                .any(|v| v.kind == ViolationKind::Propagated),
            "{abs:?}"
        ),
    }
    if let Ok(ev) = Evidence::combine_queried(Ok(inn), Ok(out), 0.0) {
        panic!("reverse combine_queried must also refuse: {ev:?}");
    }
}

#[test]
fn combine_queried_any_abstention_wins() {
    let ok = dittus_boelter()
        .query(&in_domain_point(), nu)
        .expect("in domain");
    let abs = {
        let mut point = in_domain_point();
        point.insert("Re".into(), 1e6_f64.next_up());
        dittus_boelter().query(&point, nu).expect_err("ood")
    };
    assert!(
        Evidence::<f64>::combine_queried::<f64, f64>(Err(abs.clone()), Ok(ok.clone()), 0.0)
            .is_err()
    );
    assert!(Evidence::<f64>::combine_queried::<f64, f64>(Ok(ok), Err(abs), 0.0).is_err());
}

#[test]
fn combine_queried_two_in_domain_values_compose() {
    let a = dittus_boelter()
        .query(&in_domain_point(), nu)
        .expect("in domain");
    let b = dittus_boelter()
        .query(&in_domain_point(), nu)
        .expect("in domain");
    let c = Evidence::combine_queried(Ok(a), Ok(b), 1.0).expect("both in domain");
    assert!(c.model.in_domain);
}

#[test]
fn reversed_and_nan_on_the_card_are_honest() {
    let reversed = ModelCard::try_new(
        "reversed",
        vec!["endpoints normalize".into()],
        ValidityDomain::unconstrained().with("x", 5.0, 1.0),
        0.0,
    )
    .unwrap();
    let (lo, hi) = reversed.validity().bound("x").unwrap();
    assert!(lo <= hi);
    let mut p = BTreeMap::new();
    p.insert("x".into(), 3.0);
    assert!(reversed.query(&p).is_ok());

    let nan = ModelCard::try_new(
        "nan-bound",
        vec!["NaN is unusable, not dropped".into()],
        ValidityDomain::unconstrained().with("x", f64::NAN, 1.0),
        0.0,
    )
    .unwrap();
    p.insert("x".into(), 0.5);
    let err = nan.query(&p).expect_err("unusable");
    assert!(err
        .violations
        .iter()
        .any(|v| v.kind == ViolationKind::Unusable));
}
