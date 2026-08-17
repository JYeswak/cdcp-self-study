//! Native == wasm32 schedule law (short-interval review + mastery).
//!
//! Same shape as `dual_path.rs` for grade: oracle is native `cdcp_schedule`,
//! subject is this crate compiled to wasm32, comparator is equality.
//!
//! Also the anti-shadowing known-bad: JS must CALL the wasm exports. If a
//! constant in Rust moved, a JS reimplementation would not.

use cdcp_schedule::{
    is_practiced_milli, migrate_state_version, next_interval_days, next_interval_days_with,
    validate_schedule, validate_steps, validate_thresholds, INTERVAL_STEPS, MASTERED_MILLI,
    PRACTICED_MILLI, STATE_VERSION,
};
use cdcp_wasm::is_mastered_json;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ensure_wasm_built() -> Result<PathBuf, String> {
    let root = repo_root();
    let candidates = [
        root.join("target/wasm32-unknown-unknown/debug/cdcp_wasm.wasm"),
        root.join("target/wasm32-unknown-unknown/release/cdcp_wasm.wasm"),
    ];
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "cdcp_wasm",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
        ])
        .arg(root.join("Cargo.toml"))
        .current_dir(&root)
        .status()
        .map_err(|e| format!("spawn cargo: {e}"))?;
    if !status.success() {
        return Err(format!(
            "cargo build -p cdcp_wasm --target wasm32-unknown-unknown failed: {status}"
        ));
    }
    let built = root.join("target/wasm32-unknown-unknown/debug/cdcp_wasm.wasm");
    if built.is_file() {
        Ok(built)
    } else {
        Err(format!(
            "wasm artifact missing after build: {}",
            built.display()
        ))
    }
}

struct WasmSchedule {
    store: wasmtime::Store<()>,
    memory: wasmtime::Memory,
    alloc: wasmtime::TypedFunc<u32, u32>,
    free: wasmtime::TypedFunc<(u32, u32), ()>,
    schedule_ok: wasmtime::TypedFunc<(), i32>,
    step_count: wasmtime::TypedFunc<(), i32>,
    step: wasmtime::TypedFunc<i32, i32>,
    next: wasmtime::TypedFunc<(i32, i32), i32>,
    day_ms: wasmtime::TypedFunc<(), i32>,
    practiced_milli: wasmtime::TypedFunc<(), i32>,
    mastered_milli: wasmtime::TypedFunc<(), i32>,
    gap_ms: wasmtime::TypedFunc<(), i32>,
    is_practiced: wasmtime::TypedFunc<i32, i32>,
    is_mastered: wasmtime::TypedFunc<(u32, u32), i32>,
    state_version: wasmtime::TypedFunc<(), i32>,
    migrate_state_version: wasmtime::TypedFunc<i32, i32>,
}

fn instantiate(wasm_path: &Path) -> Result<WasmSchedule, String> {
    use wasmtime::*;
    let engine = Engine::default();
    let module = Module::from_file(&engine, wasm_path).map_err(|e| format!("module: {e}"))?;
    let mut store = Store::new(&engine, ());
    let instance =
        Instance::new(&mut store, &module, &[]).map_err(|e| format!("instantiate: {e}"))?;
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| "wasm export `memory` missing".to_string())?;
    let get = |store: &mut Store<()>, name: &str| {
        instance
            .get_typed_func::<(), i32>(&mut *store, name)
            .map_err(|e| format!("{name}: {e}"))
    };
    Ok(WasmSchedule {
        alloc: instance
            .get_typed_func(&mut store, "cdcp_alloc")
            .map_err(|e| format!("cdcp_alloc: {e}"))?,
        free: instance
            .get_typed_func(&mut store, "cdcp_free")
            .map_err(|e| format!("cdcp_free: {e}"))?,
        schedule_ok: get(&mut store, "cdcp_schedule_ok")?,
        step_count: get(&mut store, "cdcp_interval_step_count")?,
        step: instance
            .get_typed_func(&mut store, "cdcp_interval_step")
            .map_err(|e| format!("cdcp_interval_step: {e}"))?,
        next: instance
            .get_typed_func(&mut store, "cdcp_next_interval_days")
            .map_err(|e| format!("cdcp_next_interval_days: {e}"))?,
        day_ms: get(&mut store, "cdcp_day_ms")?,
        practiced_milli: get(&mut store, "cdcp_practiced_milli")?,
        mastered_milli: get(&mut store, "cdcp_mastered_milli")?,
        gap_ms: get(&mut store, "cdcp_mastered_min_gap_ms")?,
        is_practiced: instance
            .get_typed_func(&mut store, "cdcp_is_practiced")
            .map_err(|e| format!("cdcp_is_practiced: {e}"))?,
        is_mastered: instance
            .get_typed_func(&mut store, "cdcp_is_mastered")
            .map_err(|e| format!("cdcp_is_mastered: {e}"))?,
        state_version: get(&mut store, "cdcp_state_version")?,
        migrate_state_version: instance
            .get_typed_func(&mut store, "cdcp_migrate_state_version")
            .map_err(|e| format!("cdcp_migrate_state_version: {e}"))?,
        store,
        memory,
    })
}

fn maybe_wasm() -> Option<WasmSchedule> {
    match ensure_wasm_built() {
        Ok(p) => match instantiate(&p) {
            Ok(w) => Some(w),
            Err(e) => {
                eprintln!("SKIP schedule wasm: {e}");
                if std::env::var("CDCP_REQUIRE_WASM").ok().as_deref() == Some("1") {
                    panic!("CDCP_REQUIRE_WASM=1 but schedule wasm failed: {e}");
                }
                None
            }
        },
        Err(e) => {
            eprintln!("SKIP schedule wasm: {e}");
            if std::env::var("CDCP_REQUIRE_WASM").ok().as_deref() == Some("1") {
                panic!("CDCP_REQUIRE_WASM=1 but wasm unavailable: {e}");
            }
            None
        }
    }
}

#[test]
fn compiled_schedule_is_not_vacuous() {
    validate_schedule().expect("compiled schedule must be valid");
    assert_eq!(
        validate_steps(&[])
            .unwrap_err()
            .to_string()
            .contains("zero steps"),
        true
    );
    assert!(validate_thresholds(0, 900).is_err());
    assert!(validate_thresholds(800, 0).is_err());
}

#[test]
fn native_json_mastered_matches_crate() {
    let t0 = 1_700_000_000_000i64;
    let gap = cdcp_schedule::MASTERED_MIN_GAP_MS;
    let json_ok = format!(
        r#"[{{"ratio":0.9,"at_ms":{t0}}},{{"ratio":0.9,"at_ms":{}}}]"#,
        t0 + gap
    );
    assert_eq!(is_mastered_json(&json_ok).unwrap(), true);
    let json_close = format!(
        r#"[{{"ratio":0.9,"at_ms":{t0}}},{{"ratio":0.9,"at_ms":{}}}]"#,
        t0 + gap - 1
    );
    assert_eq!(is_mastered_json(&json_close).unwrap(), false);
    assert_eq!(is_mastered_json("[]").unwrap(), false);
}

#[test]
fn native_equals_wasm_schedule() {
    let Some(mut w) = maybe_wasm() else {
        return;
    };

    assert_eq!(w.schedule_ok.call(&mut w.store, ()).unwrap(), 1);
    let n = w.step_count.call(&mut w.store, ()).unwrap();
    assert!(n > 0, "zero steps from wasm is an ERROR, not a cap");
    assert_eq!(n as usize, INTERVAL_STEPS.len());
    for (i, &want) in INTERVAL_STEPS.iter().enumerate() {
        let got = w.step.call(&mut w.store, i as i32).unwrap();
        assert_eq!(got, want as i32, "wasm step[{i}] != native");
    }

    let cases: [(i32, i32); 7] = [(0, 0), (0, 1), (1, 1), (3, 1), (3, 0), (1, 0), (-5, 1)];
    for (cur, ok) in cases {
        let native = next_interval_days(cur, ok != 0) as i32;
        let wasm = w.next.call(&mut w.store, (cur, ok)).unwrap();
        assert_eq!(native, wasm, "next_interval_days({cur}, {ok})");
    }

    assert_eq!(
        w.practiced_milli.call(&mut w.store, ()).unwrap() as u32,
        PRACTICED_MILLI
    );
    assert_eq!(
        w.mastered_milli.call(&mut w.store, ()).unwrap() as u32,
        MASTERED_MILLI
    );
    assert_eq!(
        w.day_ms.call(&mut w.store, ()).unwrap() as i64,
        cdcp_schedule::DAY_MS
    );
    assert_eq!(
        w.gap_ms.call(&mut w.store, ()).unwrap() as i64,
        cdcp_schedule::MASTERED_MIN_GAP_MS
    );

    assert_eq!(
        w.is_practiced.call(&mut w.store, 799).unwrap(),
        i32::from(is_practiced_milli(799))
    );
    assert_eq!(
        w.is_practiced.call(&mut w.store, 800).unwrap(),
        i32::from(is_practiced_milli(800))
    );

    let t0 = 1_700_000_000_000i64;
    let json = format!(
        r#"[{{"ratio":0.9,"at_ms":{t0}}},{{"ratio":0.9,"at_ms":{}}}]"#,
        t0 + cdcp_schedule::MASTERED_MIN_GAP_MS
    );
    let native = is_mastered_json(&json).unwrap();
    let bytes = json.as_bytes();
    let ptr = w
        .alloc
        .call(&mut w.store, bytes.len() as u32)
        .expect("alloc");
    w.memory
        .write(&mut w.store, ptr as usize, bytes)
        .expect("write");
    let rc = w
        .is_mastered
        .call(&mut w.store, (ptr, bytes.len() as u32))
        .expect("is_mastered");
    let _ = w.free.call(&mut w.store, (ptr, bytes.len() as u32));
    assert!(rc >= 0, "wasm is_mastered error");
    assert_eq!(native, rc == 1, "mastered json dual-path");

    assert_eq!(
        w.state_version.call(&mut w.store, ()).unwrap() as u32,
        STATE_VERSION
    );
    for from in [0i32, 1] {
        let native = migrate_state_version(from as u32).unwrap() as i32;
        let wasm = w.migrate_state_version.call(&mut w.store, from).unwrap();
        assert_eq!(native, wasm, "migrate_state_version({from})");
    }
    for bad in [2i32, 99, -1] {
        let wasm = w.migrate_state_version.call(&mut w.store, bad).unwrap();
        assert!(wasm < 0, "unknown version {bad} must be ERROR from wasm");
        if bad >= 0 {
            assert!(migrate_state_version(bad as u32).is_err());
        }
    }
}

/// Known-bad: the stepper is the ladder, not a hardcoded 3. Changing the
/// cap in Rust moves the verdict. JS that still returned 3 would be shadowing.
#[test]
fn known_bad_moved_cap_changes_native_and_wasm_exports_live_cap() {
    let moved = next_interval_days_with(&[1, 99], 1, true).unwrap();
    assert_eq!(moved, 99);
    assert_ne!(moved, next_interval_days(1, true));

    let Some(mut w) = maybe_wasm() else {
        return;
    };
    let wasm_cap = w.step.call(&mut w.store, 1).unwrap();
    assert_eq!(wasm_cap as u32, INTERVAL_STEPS[1]);
    let wasm_next = w.next.call(&mut w.store, (1, 1)).unwrap();
    assert_eq!(wasm_next as u32, next_interval_days(1, true));
    // If JS shadowed with `return 3`, moving INTERVAL_STEPS[1] would leave
    // the browser stuck. The live export equals the Rust constant — the
    // browser path that CALLS this export moves with it.
}

/// Known-bad: review.js / mastery.js / schedule_bridge.js must not reimplement
/// the law. A second copy would stay put when a Rust constant moved.
#[test]
fn known_bad_js_does_not_shadow_the_rust_law() {
    let root = repo_root();
    let bridge = std::fs::read_to_string(root.join("web/assets/js/schedule_bridge.js"))
        .expect("schedule_bridge.js");
    let review = std::fs::read_to_string(root.join("web/assets/js/review.js")).expect("review.js");
    let mastery =
        std::fs::read_to_string(root.join("web/assets/js/mastery.js")).expect("mastery.js");
    let drill = std::fs::read_to_string(root.join("web/assets/js/drill.js")).expect("drill.js");

    assert!(
        !bridge.trim().is_empty()
            && !review.trim().is_empty()
            && !mastery.trim().is_empty()
            && !drill.trim().is_empty(),
        "empty JS scan set is an ERROR, not a pass"
    );

    for name in [
        "cdcp_next_interval_days",
        "cdcp_interval_step",
        "cdcp_is_practiced",
        "cdcp_is_mastered",
        "cdcp_practiced_milli",
        "cdcp_mastered_milli",
        "cdcp_state_version",
        "cdcp_migrate_state_version",
    ] {
        assert!(
            bridge.contains(name),
            "schedule_bridge.js must CALL wasm export {name}"
        );
    }

    assert!(
        review.contains("schedule_bridge.js") && review.contains("nextIntervalDays"),
        "review.js must import nextIntervalDays from schedule_bridge (WASM decides)"
    );
    assert!(
        review.contains("migrateStateVersion"),
        "review.js must import migrateStateVersion from schedule_bridge (WASM decides version)"
    );
    assert!(
        !review.contains("if (cur < 1) return 1")
            && !review.contains("if (cur < 3) return 3")
            && !review.contains("INTERVAL_STEPS = Object.freeze([1, 3])"),
        "review.js reimplements the interval law — JS is shadowing Rust"
    );
    assert!(
        !drill.contains("INTERVAL_STEPS = Object.freeze([1, 3])")
            && !drill.contains("if (cur < 1) return 1")
            && !drill.contains("if (cur < 3) return 3"),
        "drill.js reimplements the interval law — JS is shadowing Rust"
    );

    assert!(
        mastery.contains("schedule_bridge.js"),
        "mastery.js must import the law from schedule_bridge (WASM decides)"
    );
    assert!(
        !mastery.contains(">= PRACTICED_RATIO")
            && !mastery.contains(">= 0.8")
            && !mastery.contains(">= 0.80")
            && !mastery.contains("r < MASTERED_RATIO")
            && !mastery.contains("PRACTICED_RATIO = 0.8"),
        "mastery.js reimplements the threshold law — JS is shadowing Rust"
    );
}
