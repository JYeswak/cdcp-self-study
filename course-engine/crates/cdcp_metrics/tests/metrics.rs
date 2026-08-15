//! Computed PUE/WUE/CUE/ERE and rational comparison.

use cdcp_metrics::{
    Boundary, CarbonAccounting, EnergyWaterMix, Ewif, HydroReservoir, ItMeter, Metric, MetricKind,
    Ratio, ReusePolicy, ScopeItem, WaterScope, INCOMPARABLE,
};

fn pue_boundary(meter: ItMeter) -> Boundary {
    Boundary::pue(
        meter,
        [
            ScopeItem::ItEnergy,
            ScopeItem::Cooling,
            ScopeItem::Lighting,
            ScopeItem::UpsLosses,
        ],
        [ScopeItem::GeneratorTesting, ScopeItem::OfficeHvac],
    )
    .unwrap()
}

fn site_wue_boundary() -> Boundary {
    Boundary::wue(
        ItMeter::UpsOutput,
        WaterScope::Site,
        None,
        [
            ScopeItem::CoolingTowerEvaporation,
            ScopeItem::Blowdown,
            ScopeItem::Drift,
            ScopeItem::Humidification,
        ],
        [ScopeItem::FireWater, ScopeItem::EnergyWater],
    )
    .unwrap()
}

#[test]
fn pue_ratio_is_exact() {
    let m = Metric::pue(240, 200, pue_boundary(ItMeter::UpsOutput)).unwrap();
    assert_eq!(m.value(), Ratio::new(6, 5).unwrap());
    assert_eq!(m.kind(), MetricKind::Pue);
}

#[test]
fn wue_source_closed_form() {
    // Paper (LBNL-shaped numbers, exact rationals):
    //   EWIF = 87/20 L/kWh (4.35)
    //   PUE  = 6/5
    //   site = 9/25 L/kWh (0.36)
    //   source = (87/20)*(6/5) + 9/25
    //          = 522/100 + 9/25
    //          = 261/50 + 18/50
    //          = 279/50
    let ewif = Ewif::new(
        Ratio::new(87, 20).unwrap(),
        HydroReservoir::Included,
        EnergyWaterMix::NationalAverage,
    )
    .unwrap();
    let pue = Metric::pue(120, 100, pue_boundary(ItMeter::UpsOutput)).unwrap();
    let site = Metric::wue(36, 100, site_wue_boundary()).unwrap();
    let source = Metric::wue_source(&ewif, &pue, &site).unwrap();
    assert_eq!(source.value(), Ratio::new(279, 50).unwrap());
    assert_eq!(source.boundary().water_scope(), Some(WaterScope::Source));
    assert_eq!(source.boundary().hydro(), Some(HydroReservoir::Included));
    assert!(source
        .boundary()
        .includes()
        .contains(&ScopeItem::EnergyWater));
}

#[test]
fn cue_ere_closed_form() {
    let cue_b = Boundary::cue(
        ItMeter::PduOutput,
        CarbonAccounting::Scope1And2LocationBased,
        [ScopeItem::Scope1OnSite, ScopeItem::Scope2PurchasedEnergy],
        [ScopeItem::Scope3Upstream],
    )
    .unwrap();
    // 340 / 1000 = 17/50
    let c = Metric::cue(340, 1000, cue_b).unwrap();
    assert_eq!(c.value(), Ratio::new(17, 50).unwrap());

    let ere_b = Boundary::ere(
        ItMeter::UpsOutput,
        ReusePolicy::ActuallyConsumed,
        [
            ScopeItem::ItEnergy,
            ScopeItem::Cooling,
            ScopeItem::DistrictHeat,
        ],
        [ScopeItem::OfficeHvac],
    )
    .unwrap();
    // (200 - 50) / 100 = 3/2
    let e = Metric::ere(200, 50, 100, ere_b).unwrap();
    assert_eq!(e.value(), Ratio::new(3, 2).unwrap());
}

#[test]
fn comparison_is_rational_and_boundary_strict() {
    let a = Metric::pue(120, 100, pue_boundary(ItMeter::UpsOutput)).unwrap();
    let b = Metric::pue(150, 100, pue_boundary(ItMeter::UpsOutput)).unwrap();
    assert_eq!(a.compare(&b).unwrap(), std::cmp::Ordering::Less);

    let c = Metric::pue(120, 100, pue_boundary(ItMeter::ItInput)).unwrap();
    let err = a.compare(&c).expect_err("different IT meter");
    assert!(err.to_string().contains(INCOMPARABLE));
}

#[test]
fn site_18_and_source_18_cannot_compare() {
    // The same 9/5 figure is site water in LBNL 2016 and source water
    // in NREL 2003. Without the boundary they collapse. With it they
    // cannot be compared.
    let site = Metric::declared(
        MetricKind::Wue,
        Ratio::new(9, 5).unwrap(),
        site_wue_boundary(),
    )
    .unwrap();
    let source_b = Boundary::wue(
        ItMeter::UpsOutput,
        WaterScope::Source,
        Some(HydroReservoir::Excluded),
        [ScopeItem::CoolingTowerEvaporation, ScopeItem::EnergyWater],
        [ScopeItem::FireWater, ScopeItem::HydroReservoirEvaporation],
    )
    .unwrap();
    let source = Metric::declared(MetricKind::Wue, Ratio::new(9, 5).unwrap(), source_b).unwrap();
    assert_eq!(site.value(), source.value());
    let err = site.compare(&source).expect_err("site vs source");
    assert!(err.to_string().contains(INCOMPARABLE));
}

#[test]
fn display_refuses_to_print_a_bare_number() {
    let m = Metric::pue(120, 100, pue_boundary(ItMeter::UpsOutput)).unwrap();
    let text = m.to_string();
    assert!(text.starts_with("pue "), "{text}");
    assert!(text.contains("it_meter=ups-output"), "{text}");
    assert!(text.contains("includes={"), "{text}");
    assert!(text.contains("excludes={"), "{text}");
}
