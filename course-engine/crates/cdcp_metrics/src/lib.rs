//! PUE / WUE / CUE / ERE with explicit [`Boundary`] declarations.
//!
//! The boundary is where the dishonesty lives. A PUE without an IT
//! meter point is Unrecognized by its own authoring body. A WUE of
//! 1.8 L/kWh without site-vs-source and a hydro-reservoir declaration
//! is the Green Grid default that excludes hydro twice (NREL/TP-550-33905
//! thermoelectric-only figure taken as a national average, hydro EWIF
//! set to 0). This crate makes that class of lie inexpressible:
//!
//! - every [`Metric`] carries a [`Boundary`] (what's in, what's out)
//! - a bare number is a schema ERROR
//! - comparison is integer/rational and refuses unequal boundaries
//! - the TGG "unknown" EWIF of 9/5 L/kWh with hydro excluded is RED
//!
//! Free-cooling hours are **consumed** from [`cdcp_data::free_cooling_hours`]
//! (the same count `cdcp_site::Climate` stores). This crate does not
//! re-derive the economizer threshold.
#![forbid(unsafe_code)]

mod boundary;
mod error;
mod free_cooling;
mod kind;
mod metric;
mod parse;
mod ratio;

pub use boundary::{
    Boundary, BoundarySpec, CarbonAccounting, EnergyWaterMix, HydroReservoir, ItMeter, ReusePolicy,
    ScopeItem, WaterScope,
};
pub use error::{
    MetricsError, BARE_NUMBER, EMPTY_BOUNDARY, EWIF_EXCLUDES_HYDRO_TWICE, INCOMPARABLE,
    MISSING_BOUNDARY,
};
pub use free_cooling::{free_cooling_hours, take_free_cooling_hours};
pub use kind::{MetricKind, KINDS};
pub use metric::{Ewif, Metric};
pub use parse::{parse_metric, require_boundary};
pub use ratio::Ratio;

#[cfg(test)]
mod unit {
    use super::*;

    fn production_src() -> &'static str {
        include_str!("lib.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests")
    }

    #[test]
    fn crate_forbids_unsafe() {
        let src = include_str!("lib.rs");
        assert!(src.contains("#![forbid(unsafe_code)]"));
        assert!(!production_src().contains("unsafe "));
        for extra in [
            include_str!("boundary.rs"),
            include_str!("error.rs"),
            include_str!("free_cooling.rs"),
            include_str!("kind.rs"),
            include_str!("metric.rs"),
            include_str!("parse.rs"),
            include_str!("ratio.rs"),
        ] {
            let prod = extra.split("#[cfg(test)]").next().unwrap_or(extra);
            assert!(!prod.contains("unsafe "), "unsafe token in module");
        }
    }

    #[test]
    fn kinds_are_the_four_named_in_the_bead() {
        assert_eq!(KINDS, ["pue", "wue", "cue", "ere"]);
        assert_eq!(KINDS.len(), 4);
    }

    #[test]
    fn production_has_no_socket_or_client() {
        let src = production_src();
        for needle in [
            "TcpStream",
            "UdpSocket",
            "TcpListener",
            "std::net",
            "::net::",
            "ToSocketAddrs",
            "reqwest",
            "ureq",
        ] {
            assert!(!src.contains(needle), "production mentions {needle}");
        }
    }

    #[test]
    fn display_always_prints_the_boundary() {
        let b = Boundary::pue(
            ItMeter::UpsOutput,
            [ScopeItem::ItEnergy, ScopeItem::Cooling, ScopeItem::Lighting],
            [ScopeItem::OfficeHvac],
        )
        .unwrap();
        let m = Metric::pue(120, 100, b).unwrap();
        let text = m.to_string();
        assert!(text.contains("pue"), "{text}");
        assert!(text.contains("it_meter="), "{text}");
        assert!(text.contains("includes="), "{text}");
        assert!(text.contains("excludes="), "{text}");
        assert!(!text.eq("6/5") && !text.eq("1.2"), "{text}");
    }
}
