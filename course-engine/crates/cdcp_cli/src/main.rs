//! cdcp CLI — grade / goldens / bank-hash / export-web / serve / doctor / health / repair / oracle-check / site / metrics / slo / snap-rewrite
#![forbid(unsafe_code)]

mod assemble;
mod metrics;
mod operator;
mod oracle;
mod site;
mod slo;
mod snap_rewrite;

use cdcp_bank::Bank;
use cdcp_core::{AnsweredItem, ChoiceLetter, ExamAttempt};
use cdcp_grade::{all_correct_attempt, all_wrong_attempt, grade, grade_digest};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "cdcp", about = "CDCP course engine CLI (GradeExact)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print bank_hash for bank/items
    BankHash {
        #[arg(long, default_value = "bank/items")]
        bank: PathBuf,
    },
    /// Licence three-field split: rights / redistribution / ai_ingestion
    CheckLicence {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Load compiled-in snapshots (licence + sha256). No network I/O.
    LoadSnapshots {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// OSHA facts: 1910.147 exclusion + 1910.333 isolation constraints.
    CheckOsha {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// content.lock [data] covers every snapshots.toml body + sidecar.
    VerifyDataLock {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Flip one vendored body byte in a TEMP tree and require RED.
        #[arg(long)]
        selftest: bool,
    },
    /// Write content.lock (bank + knowledge + modules + snapshot data).
    ContentLock {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Destination (default: <root>/content.lock).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Bank items directory (default: <root>/bank/items).
        #[arg(long)]
        bank: Option<PathBuf>,
    },
    /// Plan public/free/local corpus rows. Never opens a socket. access=paid is refused.
    FetchCorpus {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// sources.toml (default: <root>/knowledge/sources.toml)
        #[arg(long)]
        sources: Option<PathBuf>,
        /// Output directory for --write (default: <root>/knowledge/corpus/public)
        #[arg(long)]
        out: Option<PathBuf>,
        /// Plan only. Default when --write is absent. Never writes, never networks.
        #[arg(long)]
        dry_run: bool,
        /// Copy file:// sources only. access=paid is never written. HTTP is never fetched.
        #[arg(long)]
        write: bool,
        /// Snapshot header date YYYY-MM-DD (default: UTC today)
        #[arg(long)]
        fetched: Option<String>,
    },
    /// Computed site quantities vs published references. Vendored snapshots only; no network.
    #[command(visible_alias = "oracle")]
    OracleCheck {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Perturb one published ref and empty the ledger; both must RED.
        #[arg(long, alias = "self-test")]
        selftest: bool,
    },
    /// Climate / seismic / carbon / flood / power price from vendored snapshots. No network.
    Site {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Compiled location id (`ashburn`, …).
        #[arg(long)]
        location: Option<String>,
        /// WGS84 latitude (decimal degrees). Requires --lon.
        #[arg(long, allow_hyphen_values = true)]
        lat: Option<f64>,
        /// WGS84 longitude (decimal degrees). West is negative. Requires --lat.
        #[arg(long, allow_hyphen_values = true)]
        lon: Option<f64>,
    },
    /// PUE / WUE / CUE / ERE with an explicit Boundary. A bare number is RED.
    Metrics {
        /// TOML metric document path. Mutually exclusive with --doc.
        #[arg(long)]
        file: Option<PathBuf>,
        /// Inline TOML metric document. A bare number is a schema ERROR.
        #[arg(long, allow_hyphen_values = true)]
        doc: Option<String>,
    },
    /// Ledger tripwire for measured paraphrase pairs C3 cannot see
    VerifyParaphrasePairs {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Ledger path (default: <root>/registries/paraphrase_pairs.toml)
        #[arg(long)]
        ledger: Option<PathBuf>,
        /// Bank items directory (default: <root>/bank/items)
        #[arg(long)]
        bank: Option<PathBuf>,
        /// Run only the in-process known-bad plants
        #[arg(long)]
        selftest: bool,
    },
    /// Grade an attempt (all-correct / all-wrong / json answers)
    Grade {
        #[arg(long, default_value = "bank/items")]
        bank: PathBuf,
        #[arg(long)]
        fixture: PathBuf,
        #[arg(long, value_parser = ["all-correct", "all-wrong", "json"])]
        mode: String,
        /// When mode=json, path to answers file: [{item_id, chosen}, ...]
        #[arg(long)]
        answers: Option<PathBuf>,
    },
    /// Export approved bank items to Anki TSV/CSV/.apkg (learner product)
    ExportAnki {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// bank=all items; seed42=web bank_items export; keys=seed42 mock+keys
        #[arg(long, default_value = "bank", value_parser = ["bank", "seed42", "keys"])]
        source: String,
        /// Output directory (default: <root>/dist/anki)
        #[arg(long)]
        out: Option<PathBuf>,
        /// comma-separated: tsv,csv,apkg (default: tsv,apkg)
        #[arg(long, default_value = "tsv,apkg")]
        format: String,
        /// Filter by module number
        #[arg(long)]
        module: Option<i64>,
        /// Filter by tag or topic_id substring
        #[arg(long)]
        tag: Option<String>,
        /// Max cards
        #[arg(long)]
        limit: Option<i64>,
        /// Shuffle seed when --limit is set
        #[arg(long)]
        seed: Option<i64>,
        /// Anki deck name for .apkg
        #[arg(long, default_value = "CDCP Study")]
        deck_name: String,
        /// Export twice to temp and FAIL if .apkg bytes differ (does not write --out)
        #[arg(long)]
        check: bool,
    },
    /// Assemble a mock form. Non-letter assess kinds are refused (not flattened to A–D).
    Assemble {
        #[arg(long, default_value = "bank/items")]
        bank: PathBuf,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Typed assess items JSON: [{id, module, stem, item}, ...].
        /// A multi-select (or any non-letter kind) is a named refuse.
        #[arg(long)]
        assess: Option<PathBuf>,
        /// Write the assembled form JSON here. Default: stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Export browser exam packs (see web/data/README.md)
    ExportWeb {
        #[arg(long, default_value = "bank/items")]
        bank: PathBuf,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value = "web/data")]
        out: PathBuf,
        /// EXPLICIT fixture replay: skip the sampler and export exactly these item_ids.
        /// There is no implicit fixture at any seed (bd-golden-sampler-divergence-09q).
        #[arg(long)]
        fixture: Option<PathBuf>,
    },
    /// Serve web/ over HTTP for the local product (offline, local-only)
    Serve {
        #[arg(long, default_value = "web")]
        root: PathBuf,
        #[arg(long, default_value = "127.0.0.1:8766")]
        bind: String,
    },
    /// Compile the offline Learn surface (pages, hub, modules_index, topic_anchors)
    BuildLearn {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Compile the offline Reference surface (glossary + power cheatsheet)
    BuildReference {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Compile web/data/units_index.json (learner-visible Learn units)
    BuildUnits {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Compile web/data/glossary.json (learner-visible glossary)
    BuildGlossary {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Compile web/data/module_learn_slugs.js from knowledge/domains.toml
    BuildLearnSlugs {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Smoke the offline Learn surface (hub + module pages + honesty banner)
    SmokeLearn {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Static smoke: Learn UI chrome (TOC/math/continue/power embed). No browser.
    SmokeLearnChrome {
        /// Engine root (directory holding web/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Smoke item-review → Learn module/section links (L7-S2)
    SmokeFeedbackLinks {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Smoke present diagrams (honesty banner + data-diagram root)
    SmokeDiagrams {
        /// Engine root (directory holding docs/ + web/diagrams/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// L7-S5 a11y marker baseline over primary web/ page shells
    SmokeA11y {
        /// Engine root (directory holding web/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// L6-S3: every declared domain maps to a Learn page
    SmokeWeakLinks {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// L5 learner-pack shape: n_items==40, items len 40, no leaked `correct`
    CheckLearnerPack {
        /// Engine root (directory holding registries/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Pack JSON. Default: <root>/web/data/mock40_seed42.json
        #[arg(long)]
        pack: Option<PathBuf>,
    },
    /// M8-B/D: units index + unit shell + glossary + concept card assets
    SmokeLearnV2 {
        /// Engine root (directory holding web/). Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Preflight: bank loads, wasm present and fresh, goldens present, port bindable, python3 present
    Doctor {
        /// Engine root. Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Address `cdcp serve` would bind; doctor probes it and releases.
        #[arg(long, default_value = operator::DEFAULT_BIND)]
        bind: String,
    },
    /// Machine-readable tree status. `--robot` emits a versioned NDJSON envelope.
    Health {
        /// Engine root. Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Emit one versioned NDJSON object on stdout (schema_version required).
        #[arg(long)]
        robot: bool,
    },
    /// Idempotent rebuild of units, glossary, learn slugs, and export-web. Never re-freezes goldens.
    Repair {
        /// Engine root. Default: walk up from cwd.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Seed for export-web (default 42, the published form).
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    /// Parse slo.toml wall budgets / emit epoch-ms for smoke_slo.
    Slo {
        #[command(subcommand)]
        sub: SloCmd,
    },
    /// Rewrite a file for the check.sh snapshot CHARTER selftest. Not a gate.
    SnapRewrite {
        #[command(subcommand)]
        sub: SnapRewriteCmd,
    },
    /// Check or regenerate grade goldens
    Goldens {
        #[command(subcommand)]
        sub: GoldensCmd,
    },
}

#[derive(Subcommand)]
enum SloCmd {
    /// Print grade_ms, export_ms, bank_verify_ms (one integer per line).
    Budgets {
        /// slo.toml path (default: ./slo.toml).
        #[arg(long, default_value = "slo.toml")]
        file: PathBuf,
    },
    /// Print unix-epoch milliseconds.
    NowMs,
}

#[derive(Subcommand)]
enum SnapRewriteCmd {
    /// Require --from to occur once; write that occurrence as --to.
    ReplaceOnce {
        #[arg(long)]
        file: PathBuf,
        #[arg(long, allow_hyphen_values = true)]
        from: String,
        #[arg(long, allow_hyphen_values = true)]
        to: String,
    },
    /// CHARTER pair: skip-exec disables re-exec; delete-assert hollows the assert.
    Charter {
        #[arg(long)]
        file: PathBuf,
        /// `skip-exec` or `delete-assert`
        #[arg(long, value_parser = ["skip-exec", "delete-assert"])]
        kind: String,
    },
}

#[derive(Subcommand)]
enum GoldensCmd {
    Check {
        #[arg(long, default_value = "bank/items")]
        bank: PathBuf,
        #[arg(long, default_value = "goldens")]
        dir: PathBuf,
    },
    /// Generate goldens (requires UPDATE_GOLDENS=1)
    Generate {
        #[arg(long, default_value = "bank/items")]
        bank: PathBuf,
        #[arg(long, default_value = "goldens")]
        dir: PathBuf,
        #[arg(long, default_value = "goldens/fixtures/mock40_seed42.json")]
        fixture: PathBuf,
    },
    /// Regenerate the seed fixture FROM the Rust sampler (requires UPDATE_GOLDENS=1).
    ///
    /// This is the authoritative regeneration path for
    /// `goldens/fixtures/mock40_seed42.json` (bd-golden-sampler-divergence-09q).
    /// It replaces `python3 scripts/sample_mock.py`, whose MT19937 stream disagrees
    /// with `cdcp_assemble` and left the golden unreproducible.
    Fixture {
        #[arg(long, default_value = "bank/items")]
        bank: PathBuf,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value = "goldens/fixtures/mock40_seed42.json")]
        out: PathBuf,
    },
}

#[derive(Debug, Deserialize)]
struct SampleFixture {
    exam_id: String,
    seed: u64,
    item_ids: Vec<String>,
}

// ── goldens check: absent input must not read as success ────────────────────
// bd-goldens-check-is-file-hole-7v9p. The bank_hash pin used to sit behind
// `if bh_path.is_file()`. Deleting goldens/bank_hash.txt skipped the only
// comparison that reads it, and the step still printed ok and exited 0 — the
// sole difference was one fewer stdout line, which nothing diffs. Measured
// before the fix, against a temp copy of goldens/ with bank_hash.txt removed:
// exit 0, stdout "ok golden all-correct / ok golden all-wrong".
//
// THIS RAISES A FLOOR; IT DOES NOT PROVE THE GOLDENS ARE RIGHT. What is now
// enforced: every required golden is present, non-empty, and actually compared,
// and a scan that discovers nothing exits non-zero. What it CANNOT decide:
// whether a present, matching pin was frozen against a correct bank — a
// deliberate re-freeze of a wrong value still passes. That question belongs to
// the coupling ledger (cdcp_gate goldens-couplings) and human review of
// goldens/PROVENANCE.md.

/// Goldens `goldens check` REQUIRES under `--dir`. Absence of any of these is
/// an ERROR that names the file, never a skipped branch.
///
/// The list is COMPILED IN so emptying `goldens/` is not a way to pass. It is
/// deliberately the CLI's OWN floor and is not the same question as
/// `cdcp_gate::gates::goldens_couplings::REQUIRED_GOLDENS`, which decides
/// whether a golden is covered by a ledger row. Two gates, two questions;
/// neither may be the reason the other stays fail-open.
const REQUIRED_GOLDENS: &[&str] = &[
    "fixtures/mock40_seed42.json",
    "mock40_seed42_all_correct.sha256",
    "mock40_seed42_all_wrong.sha256",
    "bank_hash.txt",
];

/// Extensions discovery does not count as goldens. Prose next to the artifacts
/// (goldens/PROVENANCE.md) is documentation, not a pinned artifact. Anything
/// else — including an extensionless file — counts, which is the fail-closed
/// direction: an unrecognised file inflates the discovered set rather than
/// silently vanishing from it.
const DISCOVERY_SKIP_EXT: &[&str] = &["md"];

/// Comparisons `goldens check` must perform: the two grade digests plus the
/// bank_hash pin. Asserted at the end, so a future refactor that drops a leg
/// exits non-zero instead of printing one fewer line nobody reads.
const EXPECTED_COMPARISONS: usize = 3;

/// One row in a mode=json answers file: `[{ "item_id", "chosen" }, ...]`
#[derive(Debug, Deserialize)]
struct AnswerRow {
    item_id: String,
    chosen: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("cdcp: {e}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.cmd {
        Cmd::BankHash { bank } => {
            let b = Bank::load_dir(&bank).map_err(|e| e.to_string())?;
            println!("{}", b.bank_hash);
            Ok(())
        }
        Cmd::CheckLicence { root } => check_licence(root.as_deref()),
        Cmd::LoadSnapshots { root } => load_snapshots(root.as_deref()),
        Cmd::CheckOsha { root } => check_osha(root.as_deref()),
        Cmd::VerifyDataLock { root, selftest } => verify_data_lock(root.as_deref(), selftest),
        Cmd::ContentLock { root, out, bank } => {
            content_lock(root.as_deref(), out.as_deref(), bank.as_deref())
        }
        Cmd::FetchCorpus {
            root,
            sources,
            out,
            dry_run,
            write,
            fetched,
        } => fetch_corpus_cmd(
            root.as_deref(),
            sources.as_deref(),
            out.as_deref(),
            dry_run,
            write,
            fetched.as_deref(),
        ),
        Cmd::OracleCheck { root, selftest } => oracle::run(root.as_deref(), selftest),
        Cmd::Site {
            root,
            location,
            lat,
            lon,
        } => site::run(root.as_deref(), location.as_deref(), lat, lon),
        Cmd::Metrics { file, doc } => metrics::run(file.as_deref(), doc.as_deref()),
        Cmd::VerifyParaphrasePairs {
            root,
            ledger,
            bank,
            selftest,
        } => verify_paraphrase_pairs(
            root.as_deref(),
            ledger.as_deref(),
            bank.as_deref(),
            selftest,
        ),
        Cmd::Grade {
            bank,
            fixture,
            mode,
            answers,
        } => {
            let b = Bank::load_dir(&bank).map_err(|e| e.to_string())?;
            let fix: SampleFixture =
                serde_json::from_str(&fs::read_to_string(&fixture).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let attempt = match mode.as_str() {
                "all-correct" => all_correct_attempt(&b, &fix.exam_id, fix.seed, &fix.item_ids)
                    .map_err(|e| e.to_string())?,
                "all-wrong" => all_wrong_attempt(&b, &fix.exam_id, fix.seed, &fix.item_ids)
                    .map_err(|e| e.to_string())?,
                "json" => attempt_from_json_answers(&b, &fix, answers.as_deref())?,
                other => return Err(format!("unsupported mode: {other}")),
            };
            let report = grade(&b, &attempt).map_err(|e| e.to_string())?;
            let digest = grade_digest(&b, &attempt).map_err(|e| e.to_string())?;
            println!(
                "score={}/{} study_pass={} digest={}",
                report.score_correct, report.score_total, report.passed_study_signal, digest
            );
            Ok(())
        }
        Cmd::ExportAnki {
            root,
            source,
            out,
            format,
            module,
            tag,
            limit,
            seed,
            deck_name,
            check,
        } => export_anki(
            root.as_deref(),
            &source,
            out.as_deref(),
            &format,
            module,
            tag.as_deref(),
            limit,
            seed,
            &deck_name,
            check,
        ),
        Cmd::Assemble {
            bank,
            seed,
            assess,
            out,
        } => assemble::run(&bank, seed, assess.as_deref(), out.as_deref()),
        Cmd::ExportWeb {
            bank,
            seed,
            out,
            fixture,
        } => export_web(&bank, seed, &out, fixture),
        Cmd::Serve { root, bind } => serve(&root, &bind),
        Cmd::BuildLearn { root } => compile_learn(root.as_deref(), LearnKind::Learn),
        Cmd::BuildReference { root } => compile_learn(root.as_deref(), LearnKind::Reference),
        Cmd::BuildUnits { root } => compile_learn(root.as_deref(), LearnKind::Units),
        Cmd::BuildGlossary { root } => compile_learn(root.as_deref(), LearnKind::Glossary),
        Cmd::BuildLearnSlugs { root } => compile_learn(root.as_deref(), LearnKind::Slugs),
        Cmd::SmokeLearn { root } => compile_learn(root.as_deref(), LearnKind::Smoke),
        Cmd::SmokeLearnChrome { root } => smoke_learn_chrome(root.as_deref()),
        Cmd::SmokeFeedbackLinks { root } => smoke_feedback_links(root.as_deref()),
        Cmd::SmokeDiagrams { root } => smoke_diagrams(root.as_deref()),
        Cmd::SmokeA11y { root } => smoke_a11y(root.as_deref()),
        Cmd::SmokeWeakLinks { root } => smoke_weak_links(root.as_deref()),
        Cmd::CheckLearnerPack { root, pack } => {
            check_learner_pack(root.as_deref(), pack.as_deref())
        }
        Cmd::SmokeLearnV2 { root } => smoke_learn_v2(root.as_deref()),
        Cmd::Doctor { root, bind } => operator::doctor(root.as_deref(), &bind),
        Cmd::Health { root, robot } => operator::health(root.as_deref(), robot),
        Cmd::Repair { root, seed } => operator::repair(root.as_deref(), seed),
        Cmd::Slo { sub } => match sub {
            SloCmd::Budgets { file } => slo::emit_budgets(&file),
            SloCmd::NowMs => slo::emit_now_ms(),
        },
        Cmd::SnapRewrite { sub } => match sub {
            SnapRewriteCmd::ReplaceOnce { file, from, to } => {
                snap_rewrite::apply(&file, &snap_rewrite::Job::SwapOnce { from, to })
            }
            SnapRewriteCmd::Charter { file, kind } => {
                let leg = snap_rewrite::parse_leg(&kind)?;
                snap_rewrite::apply(&file, &snap_rewrite::Job::Charter(leg))
            }
        },
        Cmd::Goldens { sub } => match sub {
            GoldensCmd::Check { bank, dir } => goldens_check(&bank, &dir),
            // `.ok()` here is fail-CLOSED, the opposite of the goldens-check
            // hole: an unset, unreadable or non-UTF-8 UPDATE_GOLDENS collapses
            // to None and REFUSES. Absence blocks the write; only an explicit
            // "1" permits it.
            GoldensCmd::Generate { bank, dir, fixture } => {
                if std::env::var("UPDATE_GOLDENS").ok().as_deref() != Some("1") {
                    return Err(
                        "refusing to generate goldens without UPDATE_GOLDENS=1 (human review)"
                            .into(),
                    );
                }
                goldens_generate(&bank, &dir, &fixture)
            }
            GoldensCmd::Fixture { bank, seed, out } => {
                if std::env::var("UPDATE_GOLDENS").ok().as_deref() != Some("1") {
                    return Err(
                        "refusing to regenerate the fixture without UPDATE_GOLDENS=1 (human review)"
                            .into(),
                    );
                }
                goldens_fixture(&bank, seed, &out)
            }
        },
    }
}

/// Licence three-field split. An explicit `--root` is the tree under test
/// (fixture plants included). Walking up would find the live engine.
fn check_licence(root: Option<&Path>) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_evidence::resolve_engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let report = cdcp_evidence::scan_engine(&resolved);
    print!("{report}");
    if !report.is_clean() {
        return Err(format!(
            "licence policy RED (scanned={} faults={})",
            report.scanned,
            report.faults.len()
        ));
    }
    Ok(())
}

/// E1: compiled-in snapshot pins. An explicit `--root` is the tree under
/// test (fixture plants included). Walking up would find the live engine.
fn load_snapshots(root: Option<&Path>) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_data::engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let report = cdcp_data::load_compiled(&resolved).map_err(|e| e.to_string())?;
    print!("{report}");
    Ok(())
}

/// E3: OSHA / eCFR facts against the vendored snapshots and the live
/// curriculum. An explicit `--root` is the tree under test.
fn check_osha(root: Option<&Path>) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_data::engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let report = cdcp_data::check_osha(&resolved).map_err(|e| e.to_string())?;
    print!("{report}");
    if !report.is_clean() {
        return Err(format!(
            "OSHA facts RED (scanned={} faults={})",
            report.scanned,
            report.faults.len()
        ));
    }
    Ok(())
}

/// E: independent lock pin for every file `snapshots.toml` names.
fn verify_data_lock(root: Option<&Path>, selftest: bool) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_data::engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    if selftest {
        let msg = cdcp_data::selftest_flip_one_byte(&resolved).map_err(|e| e.to_string())?;
        print!("{msg}");
        return Ok(());
    }
    match cdcp_data::verify_data_lock(&resolved) {
        Ok(report) => {
            print!("{report}");
            Ok(())
        }
        Err(cdcp_data::DataError::DataLockFailed { faults }) => {
            eprintln!("verify_data_lock: FAIL");
            for f in &faults {
                eprintln!("  - {f}");
            }
            Err(format!(
                "content.lock [data] RED ({} fault(s))",
                faults.len()
            ))
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Author `content.lock`. Bank hash is computed in-process; goldens/bank_hash.txt
/// is the fallback the retired Python used when a live hash could not be obtained.
fn content_lock(
    root: Option<&Path>,
    out: Option<&Path>,
    bank: Option<&Path>,
) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_data::engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let bank_hash = resolve_bank_hash(&resolved, bank)?;
    let dest = match out {
        Some(p) => p.to_path_buf(),
        None => resolved.join(cdcp_data::LOCK_REL),
    };
    let report =
        cdcp_data::write_content_lock(&resolved, &bank_hash, &dest).map_err(|e| e.to_string())?;
    println!("{report}");
    Ok(())
}

/// Plan / copy public corpus rows. Default is dry-run (no write, no socket).
/// `--write` copies `file://` only. `access=paid` cannot reach the writer.
fn fetch_corpus_cmd(
    root: Option<&Path>,
    sources: Option<&Path>,
    out: Option<&Path>,
    dry_run_flag: bool,
    write: bool,
    fetched: Option<&str>,
) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_data::engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let dry_run = !write || dry_run_flag;
    let req = cdcp_data::FetchRequest {
        sources: sources
            .map(Path::to_path_buf)
            .unwrap_or_else(|| resolved.join(cdcp_data::SOURCES_REL)),
        out_dir: out
            .map(Path::to_path_buf)
            .unwrap_or_else(|| resolved.join(cdcp_data::OUT_DIR_REL)),
        dry_run,
        fetched: fetched
            .map(str::to_string)
            .unwrap_or_else(cdcp_data::today_utc_ymd),
        root: resolved,
    };
    let report = cdcp_data::fetch_corpus(&req).map_err(|e| e.to_string())?;
    print!("{report}");
    Ok(())
}

fn resolve_bank_hash(root: &Path, bank: Option<&Path>) -> Result<String, String> {
    let bank_dir = bank
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join("bank/items"));
    match Bank::load_dir(&bank_dir) {
        Ok(b) => {
            if b.bank_hash.len() == 64 && b.bank_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Ok(b.bank_hash);
            }
            Err(format!(
                "live bank-hash is not 64 hex chars (got {:?})",
                &b.bank_hash.chars().take(16).collect::<String>()
            ))
        }
        Err(e) => {
            let golden = root.join("goldens/bank_hash.txt");
            if golden.is_file() {
                let hx = fs::read_to_string(&golden)
                    .map_err(|ge| format!("read {}: {ge}", golden.display()))?
                    .trim()
                    .to_string();
                if hx.len() == 64 && hx.chars().all(|c| c.is_ascii_hexdigit()) {
                    eprintln!(
                        "content-lock: WARN: live bank-hash failed ({e}); \
                         using goldens/bank_hash.txt"
                    );
                    return Ok(hx);
                }
            }
            Err(format!("cannot obtain bank_hash: {e}"))
        }
    }
}

/// Build an `ExamAttempt` from a JSON answers file for mode=json.
///
/// Validates each letter (A–D) and that every `item_id` exists in the bank.
fn verify_paraphrase_pairs(
    root: Option<&Path>,
    ledger: Option<&Path>,
    bank: Option<&Path>,
    selftest: bool,
) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_learn::resolve_engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let req = cdcp_bank::paraphrase::Request {
        ledger: ledger
            .map(Path::to_path_buf)
            .unwrap_or_else(|| resolved.join(cdcp_bank::paraphrase::LEDGER_REL)),
        bank: bank
            .map(Path::to_path_buf)
            .unwrap_or_else(|| resolved.join(cdcp_bank::paraphrase::BANK_REL)),
        selftest_only: selftest,
    };
    let outcome = cdcp_bank::paraphrase::run(&req);
    print!("{}", outcome.stdout);
    eprint!("{}", outcome.stderr);
    if outcome.code != 0 {
        std::process::exit(outcome.code);
    }
    Ok(())
}

fn attempt_from_json_answers(
    bank: &Bank,
    fix: &SampleFixture,
    answers_path: Option<&Path>,
) -> Result<ExamAttempt, String> {
    let path = answers_path.ok_or_else(|| {
        "mode=json requires --answers <path> with [{item_id, chosen}, ...]".to_string()
    })?;
    let raw =
        fs::read_to_string(path).map_err(|e| format!("read answers {}: {e}", path.display()))?;
    let rows: Vec<AnswerRow> =
        serde_json::from_str(&raw).map_err(|e| format!("parse answers {}: {e}", path.display()))?;
    if rows.is_empty() {
        return Err("answers file is empty".into());
    }

    let mut answered = Vec::with_capacity(rows.len());
    for (i, row) in rows.into_iter().enumerate() {
        if bank.get(&row.item_id).is_none() {
            return Err(format!("unknown item_id at answers[{i}]: {}", row.item_id));
        }
        let chosen = ChoiceLetter::parse(&row.chosen).map_err(|e| {
            format!(
                "invalid chosen at answers[{i}] item_id={}: {e}",
                row.item_id
            )
        })?;
        answered.push(AnsweredItem {
            item_id: row.item_id,
            chosen,
        });
    }

    Ok(ExamAttempt {
        exam_id: fix.exam_id.clone(),
        seed: fix.seed,
        bank_hash: bank.bank_hash.clone(),
        answers: answered,
    })
}

/// Every non-prose file under `dir`, recursively. Used only to answer "did this
/// scan look at anything at all"; an empty result is an ERROR at the call site.
fn discover_goldens(dir: &Path, found: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read dir {}: {e}", dir.display()))?;
        let p = entry.path();
        if p.is_dir() {
            discover_goldens(&p, found)?;
        } else if p.is_file() {
            // `unwrap_or("")` is fail-CLOSED here: no extension means no skip.
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !DISCOVERY_SKIP_EXT.contains(&ext) {
                found.push(p);
            }
        }
    }
    Ok(())
}

/// Read a one-line pin file. An empty or whitespace-only pin is an ERROR: a
/// 0-byte file satisfies `is_file()` and pins nothing, so "the file is there"
/// is not the same claim as "a value was compared".
fn read_pin(path: &Path) -> Result<String, String> {
    let pin = fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?
        .trim()
        .to_string();
    if pin.is_empty() {
        return Err(format!(
            "{}: empty pin file — a 0-byte golden satisfies is_file() and pins nothing",
            path.display()
        ));
    }
    Ok(pin)
}

fn goldens_check(bank: &Path, dir: &Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!("goldens dir not found: {}", dir.display()));
    }
    // A checker that requires nothing certifies nothing. This cannot fire while
    // the const above is non-empty; it exists so emptying the list is a RED run
    // rather than a silently vacuous one.
    if REQUIRED_GOLDENS.is_empty() {
        return Err(
            "REQUIRED_GOLDENS is empty — a check that requires no goldens certifies nothing".into(),
        );
    }
    // Anti-vacuous discovery runs BEFORE the required-file check, deliberately.
    // Ordered the other way it is unreachable — zero discovered implies every
    // required file is missing — and an unreachable detector cannot be proven
    // to trip. This order also gives a directory holding only PROVENANCE.md its
    // own message: the scan looked and found nothing.
    let mut discovered = Vec::new();
    discover_goldens(dir, &mut discovered)?;
    if discovered.is_empty() {
        return Err(format!(
            "discovered 0 golden files under {} — an empty input set is a FAILURE, not a pass",
            dir.display()
        ));
    }
    let missing: Vec<String> = REQUIRED_GOLDENS
        .iter()
        .map(|rel| dir.join(rel))
        .filter(|p| !p.is_file())
        .map(|p| p.display().to_string())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "missing required golden(s): {} — an absent golden is an ERROR here, not a skipped comparison",
            missing.join(", ")
        ));
    }

    let b = Bank::load_dir(bank).map_err(|e| e.to_string())?;
    let fixture_path = dir.join("fixtures/mock40_seed42.json");
    let fix: SampleFixture = serde_json::from_str(
        &fs::read_to_string(&fixture_path)
            .map_err(|e| format!("read {}: {e}", fixture_path.display()))?,
    )
    .map_err(|e| format!("parse {}: {e}", fixture_path.display()))?;
    // A fixture pinning zero ids grades nothing; both attempts below would
    // digest the empty case, and `goldens generate` would freeze that as a
    // passing golden. (cdcp_grade rejects an empty attempt downstream — this
    // names the file instead of surfacing as an anonymous "empty attempt".)
    if fix.item_ids.is_empty() {
        return Err(format!(
            "{}: item_ids is empty — a fixture that pins no items grades nothing",
            fixture_path.display()
        ));
    }

    let mut compared = 0usize;
    let cases = [
        (
            "all-correct",
            all_correct_attempt(&b, &fix.exam_id, fix.seed, &fix.item_ids)
                .map_err(|e| e.to_string())?,
            dir.join("mock40_seed42_all_correct.sha256"),
        ),
        (
            "all-wrong",
            all_wrong_attempt(&b, &fix.exam_id, fix.seed, &fix.item_ids)
                .map_err(|e| e.to_string())?,
            dir.join("mock40_seed42_all_wrong.sha256"),
        ),
    ];

    for (name, attempt, path) in cases {
        let digest = grade_digest(&b, &attempt).map_err(|e| e.to_string())?;
        let expected = read_pin(&path)?;
        if digest != expected {
            return Err(format!(
                "GOLDEN MISMATCH {name}: got {digest} expected {expected} ({})",
                path.display()
            ));
        }
        compared += 1;
        println!("ok golden {name}");
    }

    // bank_hash pin — UNCONDITIONAL. Presence is enforced by REQUIRED_GOLDENS
    // above; the old `if bh_path.is_file()` guard here is the defect this
    // function was rewritten to remove (bd-goldens-check-is-file-hole-7v9p).
    let bh_path = dir.join("bank_hash.txt");
    let exp = read_pin(&bh_path)?;
    if exp != b.bank_hash {
        return Err(format!(
            "bank_hash drift: bank={} golden_pin={} ({})",
            b.bank_hash,
            exp,
            bh_path.display()
        ));
    }
    compared += 1;
    println!("ok bank_hash pin");

    if compared != EXPECTED_COMPARISONS {
        return Err(format!(
            "goldens check performed {compared} comparison(s), expected {EXPECTED_COMPARISONS} — a leg was dropped"
        ));
    }

    // Discovered files nobody compared. NOT fatal here: whether a newly added
    // golden is covered is the coupling ledger's question
    // (cdcp_gate goldens-couplings errors on a discovered file with no row),
    // and duplicating that rule against a hard-coded list would make every new
    // golden a two-place edit. Printed so it is visible in this command's own
    // output rather than only in another gate's.
    let unchecked: Vec<String> = discovered
        .iter()
        .filter(|p| {
            !REQUIRED_GOLDENS
                .iter()
                .any(|rel| p.as_path() == dir.join(rel).as_path())
        })
        .map(|p| p.display().to_string())
        .collect();
    println!(
        "goldens check: {compared} comparison(s) over {} discovered golden file(s) under {}; unchecked-here: {}",
        discovered.len(),
        dir.display(),
        if unchecked.is_empty() {
            "none".to_string()
        } else {
            unchecked.join(", ")
        }
    );
    Ok(())
}

fn goldens_generate(bank: &Path, dir: &Path, fixture: &Path) -> Result<(), String> {
    let b = Bank::load_dir(bank).map_err(|e| e.to_string())?;
    let fix: SampleFixture =
        serde_json::from_str(&fs::read_to_string(fixture).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let ac = all_correct_attempt(&b, &fix.exam_id, fix.seed, &fix.item_ids)
        .map_err(|e| e.to_string())?;
    let aw =
        all_wrong_attempt(&b, &fix.exam_id, fix.seed, &fix.item_ids).map_err(|e| e.to_string())?;
    let d1 = grade_digest(&b, &ac).map_err(|e| e.to_string())?;
    let d2 = grade_digest(&b, &aw).map_err(|e| e.to_string())?;
    fs::write(
        dir.join("mock40_seed42_all_correct.sha256"),
        format!("{d1}\n"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        dir.join("mock40_seed42_all_wrong.sha256"),
        format!("{d2}\n"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(dir.join("bank_hash.txt"), format!("{}\n", b.bank_hash))
        .map_err(|e| e.to_string())?;
    println!("wrote goldens under {}", dir.display());
    Ok(())
}

/// Regenerate the seed fixture from the **Rust** sampler (bd-golden-sampler-divergence-09q).
///
/// The fixture used to be produced by `scripts/sample_mock.py` (CPython MT19937). That stream
/// disagrees with `cdcp_assemble` (StdRng/ChaCha12) — measured at seed 42: 37 of 40 ids differed,
/// and the committed fixture no longer reproduced under *either* implementation because the bank
/// had drifted underneath it. `cdcp_assemble` is the authoritative sampler: it is the shipped
/// path, it enforces the C1 approved-only pool, and it is the only one that survives the Python
/// substrate migration.
///
/// `shuffle_choices` is false so `items[].correct` stays the raw bank letter — the fixture is an
/// assembly record, not a presented form. (Choice shuffling uses an independent rng, so
/// `item_ids` are identical either way.)
fn goldens_fixture(bank_dir: &Path, seed: u64, out: &Path) -> Result<(), String> {
    let b = Bank::load_dir(bank_dir).map_err(|e| e.to_string())?;
    let cfg = cdcp_assemble::AssembleConfig {
        shuffle_choices: false,
        ..Default::default()
    };
    let exam = cdcp_assemble::assemble(&b, seed, cfg).map_err(|e| e.to_string())?;

    let items: Vec<serde_json::Value> = exam
        .items
        .iter()
        .map(|it| {
            let bank_item = b.get(&it.id).expect("assembled id is in bank");
            serde_json::json!({
                "id": it.id,
                "module": it.module,
                "stem": it.stem,
                "choices": it.choices,
                "correct": it.correct,
                "topic_ids": bank_item.topic_ids,
            })
        })
        .collect();

    let payload = serde_json::json!({
        "exam_id": exam.exam_id,
        "seed": exam.seed,
        "n_items": exam.n_items,
        "bank_hash": exam.bank_hash,
        "item_ids": exam.item_ids,
        "modules": exam.modules,
        "items": items,
        "provenance": "cdcp goldens fixture (cdcp_assemble::assemble) — NOT scripts/sample_mock.py",
    });

    // Legitimately optional: `out` with no parent is a bare filename in the cwd,
    // which needs no mkdir. This guards a filesystem preparation step, not a
    // comparison — the write below still errors if the path is unusable.
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    write_json(out, &payload)?;
    println!(
        "goldens fixture: seed={seed} n_items={} bank_hash={} -> {}",
        exam.n_items,
        exam.bank_hash,
        out.display()
    );
    Ok(())
}

// ── export-web (L6-S5) ──────────────────────────────────────────────────────
// Contract: web/data/README.md.
//   EVERY seed → cdcp_assemble stratified sample, shuffle_choices = false.
//   --fixture <path> → EXPLICIT replay of a recorded item_ids list (opt-in only).
// Emits mock40_seed{N}.json (no correct letters), keys_seed{N}.json, bank_items_seed{N}.json.
//
// bd-golden-sampler-divergence-09q: seed 42 formerly PREFERRED
// goldens/fixtures/mock40_seed42.json whenever that path existed, so the sampler was never
// exercised at the one seed every gate pins. That implicit preference is deleted. The golden
// fixture is now regenerated FROM this sampler (`cdcp goldens fixture`), and
// crates/cdcp_cli/tests/cli.rs::golden_fixture_is_the_rust_sampler_output asserts the two agree.

#[derive(Debug, Deserialize)]
struct GoldenFixture {
    exam_id: String,
    seed: u64,
    item_ids: Vec<String>,
}

pub(crate) fn export_web(
    bank_dir: &Path,
    seed: u64,
    out: &Path,
    fixture: Option<PathBuf>,
) -> Result<(), String> {
    let b = Bank::load_dir(bank_dir).map_err(|e| e.to_string())?;

    // No implicit fixture at any seed. `--fixture` is an explicit, caller-chosen replay;
    // absent it, the sampler runs — including at seed 42.
    let (exam_id, item_ids, golden_pinned) = match fixture {
        Some(fp) => {
            let g: GoldenFixture =
                serde_json::from_str(&fs::read_to_string(&fp).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if g.seed != seed {
                return Err(format!("fixture seed {} != requested seed {seed}", g.seed));
            }
            (g.exam_id, g.item_ids, true)
        }
        None => {
            let cfg = cdcp_assemble::AssembleConfig {
                shuffle_choices: false,
                ..Default::default()
            };
            let exam = cdcp_assemble::assemble(&b, seed, cfg).map_err(|e| e.to_string())?;
            (exam.exam_id, exam.item_ids, false)
        }
    };

    fs::create_dir_all(out).map_err(|e| e.to_string())?;

    let mut learner = Vec::with_capacity(item_ids.len());
    let mut keys = Vec::with_capacity(item_ids.len());
    for id in &item_ids {
        let it = b
            .get(id)
            .ok_or_else(|| format!("item {id} missing from bank"))?;
        learner.push(serde_json::json!({
            "id": it.id, "stem": it.stem, "choices": it.choices
        }));
        keys.push(serde_json::json!({
            "item_id": it.id, "correct": it.correct, "explanation": it.explanation
        }));
    }

    let n = learner.len();
    let mock = serde_json::json!({
        "schema_version": 1, "exam_id": exam_id, "seed": seed,
        "bank_hash": b.bank_hash, "n_items": n, "items": learner,
        "answer_key_policy": format!("Learner pack omits correct letters. Keys live in keys_seed{seed}.json (e2e/harness + post-grade explanations only). WASM grade uses full BankItem array in bank_items_seed{seed}.json (includes correct; required for client-side GradeExact)."),
    });
    let keyfile = serde_json::json!({
        "schema_version": 1, "exam_id": exam_id, "seed": seed,
        "bank_hash": b.bank_hash, "n_items": n, "keys": keys,
        "policy": "e2e/harness and post-grade explanations only; not the learner exam payload",
    });
    // Full bank (identical across seeds) — WASM grade_digest_json needs `correct`.
    // to_value first: serde_json's Map is a BTreeMap, so keys land sorted —
    // matching the committed packs (struct order would not).
    let bank_items =
        serde_json::to_value(b.items.values().collect::<Vec<_>>()).map_err(|e| e.to_string())?;

    write_json(&out.join(format!("mock40_seed{seed}.json")), &mock)?;
    write_json(&out.join(format!("keys_seed{seed}.json")), &keyfile)?;
    write_json(
        &out.join(format!("bank_items_seed{seed}.json")),
        &bank_items,
    )?;

    println!(
        "export-web: seed={seed} n_items={n} bank_hash={} golden_pinned={golden_pinned}",
        b.bank_hash
    );
    println!("export-web: wrote 3 packs under {}", out.display());
    Ok(())
}

enum LearnKind {
    Learn,
    Reference,
    Units,
    Glossary,
    Slugs,
    Smoke,
}

fn smoke_learn_v2(root: Option<&Path>) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_learn::resolve_engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let outcome = cdcp_learn::learn_v2::run(&resolved);
    print!("{}", outcome.stdout);
    if outcome.code != 0 {
        std::process::exit(outcome.code);
    }
    Ok(())
}

fn check_learner_pack(root: Option<&Path>, pack: Option<&Path>) -> Result<(), String> {
    let outcome = match pack {
        Some(p) => cdcp_learn::learner_pack::check_path(p),
        None => {
            let resolved = match root {
                Some(p) => p.to_path_buf(),
                None => {
                    let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
                    cdcp_learn::resolve_engine_root(&start).map_err(|e| e.to_string())?
                }
            };
            cdcp_learn::learner_pack::run(&resolved)
        }
    };
    print!("{}", outcome.stdout);
    if outcome.code != 0 {
        return Err("L5 learner pack shape RED".into());
    }
    Ok(())
}

fn smoke_weak_links(root: Option<&Path>) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_learn::resolve_engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let outcome = cdcp_learn::weak_links::run(&resolved);
    print!("{}", outcome.stdout);
    if outcome.code != 0 {
        std::process::exit(outcome.code);
    }
    Ok(())
}

fn smoke_a11y(root: Option<&Path>) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_learn::resolve_engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let outcome = cdcp_learn::a11y::run(&resolved);
    print!("{}", outcome.stdout);
    if outcome.code != 0 {
        std::process::exit(outcome.code);
    }
    Ok(())
}

fn smoke_diagrams(root: Option<&Path>) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_learn::resolve_engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let outcome = cdcp_learn::diagrams::run(&resolved);
    print!("{}", outcome.stdout);
    if outcome.code != 0 {
        std::process::exit(outcome.code);
    }
    Ok(())
}

fn smoke_feedback_links(root: Option<&Path>) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_learn::resolve_engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let outcome = cdcp_learn::feedback::run(&resolved);
    print!("{}", outcome.stdout);
    if outcome.code != 0 {
        std::process::exit(outcome.code);
    }
    Ok(())
}

fn smoke_learn_chrome(root: Option<&Path>) -> Result<(), String> {
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_learn::resolve_engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let report = cdcp_learn::chrome::smoke(&resolved);
    print!("{}", report.stdout);
    if report.code != 0 {
        std::process::exit(report.code);
    }
    Ok(())
}

fn export_anki(
    root: Option<&Path>,
    source: &str,
    out: Option<&Path>,
    format: &str,
    module: Option<i64>,
    tag: Option<&str>,
    limit: Option<i64>,
    seed: Option<i64>,
    deck_name: &str,
    check: bool,
) -> Result<(), String> {
    // An explicit --root is the tree under test (including the all-retired
    // plant). Walking up would find the live engine and export the live bank.
    let resolved = match root {
        Some(p) => p.to_path_buf(),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            cdcp_anki::resolve_engine_root(&start).map_err(|e| e.to_string())?
        }
    };
    let mut req = cdcp_anki::Request::default_for(&resolved);
    req.source = match source {
        "bank" => cdcp_anki::Source::Bank,
        "seed42" => cdcp_anki::Source::Seed42,
        "keys" => cdcp_anki::Source::Keys,
        other => return Err(format!("unsupported --source {other}")),
    };
    if let Some(p) = out {
        req.out = p.to_path_buf();
    }
    req.format = format.to_string();
    req.module = module;
    req.tag = tag.map(|s| s.to_string());
    req.limit = limit;
    req.seed = seed;
    req.deck_name = deck_name.to_string();
    req.check = check;
    let outcome = cdcp_anki::run(&req);
    print!("{}", outcome.stdout);
    eprint!("{}", outcome.stderr);
    if outcome.code != 0 {
        std::process::exit(outcome.code);
    }
    Ok(())
}

fn compile_learn(root: Option<&Path>, kind: LearnKind) -> Result<(), String> {
    let start = match root {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().map_err(|e| format!("cwd: {e}"))?,
    };
    let resolved = cdcp_learn::resolve_engine_root(&start).map_err(|e| e.to_string())?;
    let outcome = match kind {
        LearnKind::Learn => cdcp_learn::build::write_learn(&resolved).map_err(|e| e.to_string())?,
        LearnKind::Reference => {
            cdcp_learn::reference::write_reference(&resolved).map_err(|e| e.to_string())?
        }
        LearnKind::Units => cdcp_learn::units::write_units(&resolved).map_err(|e| e.to_string())?,
        LearnKind::Glossary => {
            cdcp_learn::glossary::write_glossary(&resolved).map_err(|e| e.to_string())?
        }
        LearnKind::Slugs => cdcp_learn::slugs::write_slugs(&resolved).map_err(|e| e.to_string())?,
        LearnKind::Smoke => cdcp_learn::smoke::run(&resolved),
    };
    print!("{}", outcome.stdout);
    if outcome.code != 0 {
        std::process::exit(outcome.code);
    }
    Ok(())
}

fn write_json<T: serde::Serialize>(path: &Path, v: &T) -> Result<(), String> {
    let mut s = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    s.push('\n');
    // Byte-compare before write so `repair` (and a second export-web) leave
    // mtimes untouched when the packs already match. goldens_fixture also
    // goes through here; it is still gated by UPDATE_GOLDENS=1 at the call
    // site, and still never invoked from repair.
    let _ = operator::write_bytes_if_changed(path, s.as_bytes())?;
    Ok(())
}

// ── serve (V11) ─────────────────────────────────────────────────────────────
// Minimal local-only static server for web/. Pure std: no new dependencies,
// nothing listens beyond the bind address, no upload/exec surface.

fn serve(root: &Path, bind: &str) -> Result<(), String> {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    if !root.is_dir() {
        return Err(format!("web root not found: {}", root.display()));
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let listener = TcpListener::bind(bind).map_err(|e| format!("bind {bind}: {e}"))?;
    println!("cdcp serve: http://{bind}/  (root {})", root.display());
    println!("cdcp serve: Ctrl-C to stop");

    // The guards in this loop are per-CONNECTION liveness, not verdicts about an
    // artifact: a dropped socket or an unreadable request line must not take the
    // server down, and neither one grants access to anything. The access verdict
    // is the traversal guard below, which is fail-closed.
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut line = String::new();
        if BufReader::new(&stream).read_line(&mut line).is_err() {
            continue;
        }
        let mut parts = line.split_whitespace();
        // Fail-closed defaults: a request line with no verb yields "", which is
        // neither GET nor HEAD and is answered 405; a request line with no target
        // yields "/", which is served as index.html or 404s. Neither default can
        // widen what is reachable.
        let method = parts.next().unwrap_or("");
        let raw = parts.next().unwrap_or("/");
        if method != "GET" && method != "HEAD" {
            let _ =
                stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let path = raw.split('?').next().unwrap_or("/");
        let rel = if path == "/" {
            "index.html"
        } else {
            path.trim_start_matches('/')
        };

        // Path traversal guard: resolve, then require the result stay under root.
        // This IS a verdict, and it is fail-CLOSED in both legs: a canonicalize
        // failure becomes None via `.ok()` (404, never "assume it is fine"), and
        // the `starts_with` filter turns any escape into None as well. A file
        // that cannot be resolved is refused, not served.
        let candidate = root.join(rel);
        let resolved = candidate
            .canonicalize()
            .ok()
            .filter(|p| p.starts_with(&root));
        let (status, body, ctype) = match resolved {
            Some(p) if p.is_file() => match fs::read(&p) {
                Ok(bytes) => ("200 OK", bytes, content_type(&p)),
                Err(_) => (
                    "500 Internal Server Error",
                    b"read error".to_vec(),
                    "text/plain",
                ),
            },
            _ => ("404 Not Found", b"not found".to_vec(), "text/plain"),
        };
        let head = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\n\
             X-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(head.as_bytes());
        if method == "GET" {
            let _ = stream.write_all(&body);
        }
    }
    Ok(())
}

/// Content-Type for a served file. The `unwrap_or("")` fallback is a labelling
/// decision, not a verdict: an unknown or absent extension becomes
/// `application/octet-stream`, which (with the `nosniff` header above) is the
/// conservative answer. It cannot make an unreachable file reachable.
fn content_type(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => "application/octet-stream",
    }
}
