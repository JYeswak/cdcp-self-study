//! cdcp CLI — grade / goldens / bank-hash
#![forbid(unsafe_code)]

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
    /// Check or regenerate grade goldens
    Goldens {
        #[command(subcommand)]
        sub: GoldensCmd,
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
}

#[derive(Debug, Deserialize)]
struct SampleFixture {
    exam_id: String,
    seed: u64,
    item_ids: Vec<String>,
}

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
        Cmd::Goldens { sub } => match sub {
            GoldensCmd::Check { bank, dir } => goldens_check(&bank, &dir),
            GoldensCmd::Generate { bank, dir, fixture } => {
                if std::env::var("UPDATE_GOLDENS").ok().as_deref() != Some("1") {
                    return Err(
                        "refusing to generate goldens without UPDATE_GOLDENS=1 (human review)"
                            .into(),
                    );
                }
                goldens_generate(&bank, &dir, &fixture)
            }
        },
    }
}

/// Build an `ExamAttempt` from a JSON answers file for mode=json.
///
/// Validates each letter (A–D) and that every `item_id` exists in the bank.
fn attempt_from_json_answers(
    bank: &Bank,
    fix: &SampleFixture,
    answers_path: Option<&Path>,
) -> Result<ExamAttempt, String> {
    let path = answers_path.ok_or_else(|| {
        "mode=json requires --answers <path> with [{item_id, chosen}, ...]".to_string()
    })?;
    let raw = fs::read_to_string(path).map_err(|e| format!("read answers {}: {e}", path.display()))?;
    let rows: Vec<AnswerRow> =
        serde_json::from_str(&raw).map_err(|e| format!("parse answers {}: {e}", path.display()))?;
    if rows.is_empty() {
        return Err("answers file is empty".into());
    }

    let mut answered = Vec::with_capacity(rows.len());
    for (i, row) in rows.into_iter().enumerate() {
        if bank.get(&row.item_id).is_none() {
            return Err(format!(
                "unknown item_id at answers[{i}]: {}",
                row.item_id
            ));
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

fn goldens_check(bank: &Path, dir: &Path) -> Result<(), String> {
    let b = Bank::load_dir(bank).map_err(|e| e.to_string())?;
    let fixture_path = dir.join("fixtures/mock40_seed42.json");
    let fix: SampleFixture =
        serde_json::from_str(&fs::read_to_string(&fixture_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

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
        let expected = fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?
            .trim()
            .to_string();
        if digest != expected {
            return Err(format!(
                "GOLDEN MISMATCH {name}: got {digest} expected {expected} ({})",
                path.display()
            ));
        }
        println!("ok golden {name}");
    }
    // bank_hash pin
    let bh_path = dir.join("bank_hash.txt");
    if bh_path.is_file() {
        let exp = fs::read_to_string(&bh_path)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string();
        if exp != b.bank_hash {
            return Err(format!(
                "bank_hash drift: bank={} golden_pin={}",
                b.bank_hash, exp
            ));
        }
        println!("ok bank_hash pin");
    }
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
