//! cdcp CLI — grade / goldens / bank-hash / export-web / serve
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
    /// Export browser exam packs (see web/data/README.md)
    ExportWeb {
        #[arg(long, default_value = "bank/items")]
        bank: PathBuf,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value = "web/data")]
        out: PathBuf,
        /// Override the golden fixture (seed 42 uses goldens/fixtures/mock40_seed42.json)
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
        Cmd::ExportWeb {
            bank,
            seed,
            out,
            fixture,
        } => export_web(&bank, seed, &out, fixture),
        Cmd::Serve { root, bind } => serve(&root, &bind),
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

// ── export-web (L6-S5) ──────────────────────────────────────────────────────
// Contract: web/data/README.md.
//   seed 42 → prefers goldens/fixtures/mock40_seed42.json when present (golden-pinned)
//   seed N  → cdcp_assemble stratified sample, shuffle_choices = false (practice only)
// Emits mock40_seed{N}.json (no correct letters), keys_seed{N}.json, bank_items_seed{N}.json.

#[derive(Debug, Deserialize)]
struct GoldenFixture {
    exam_id: String,
    seed: u64,
    item_ids: Vec<String>,
}

fn export_web(
    bank_dir: &Path,
    seed: u64,
    out: &Path,
    fixture: Option<PathBuf>,
) -> Result<(), String> {
    let b = Bank::load_dir(bank_dir).map_err(|e| e.to_string())?;

    // Seed 42 is golden-pinned: prefer the fixture so browser packs match GradeExact digests.
    let default_fixture = PathBuf::from("goldens/fixtures/mock40_seed42.json");
    let pinned = fixture
        .or_else(|| (seed == 42 && default_fixture.is_file()).then(|| default_fixture.clone()));

    let (exam_id, item_ids, golden_pinned) = match pinned {
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

fn write_json<T: serde::Serialize>(path: &Path, v: &T) -> Result<(), String> {
    let mut s = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    s.push('\n');
    fs::write(path, s).map_err(|e| format!("{}: {e}", path.display()))
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
