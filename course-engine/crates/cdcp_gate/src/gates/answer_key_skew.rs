use crate::registry::{GateCtx, GateError};
use cdcp_bank::Bank;
use toml::Value;
const POLICY: &str = "registries/answer_key_skew.toml";
const LETTERS: [&str; 4] = ["A", "B", "C", "D"];
pub const NAME: &str = "answer-key-skew";
pub const SUMMARY: &str = "approved answer-key distribution stays within registry band";
#[rustfmt::skip]
pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&[])?;
    let path = ctx.root.join(POLICY);
    let raw = std::fs::read_to_string(&path).map_err(|e| GateError::error(format!("read {}: {e}", path.display())))?;
    let table: Value = raw.parse().map_err(|e| GateError::error(format!("parse {}: {e}", path.display())))?;
    let row = table.get("tolerance").and_then(Value::as_array).and_then(|rows| rows.first()).and_then(Value::as_table).filter(|r| r.get("gate").and_then(Value::as_str) == Some(NAME)).ok_or_else(|| GateError::error(format!("{}: missing [[tolerance]] row", path.display())))?;
    let min = row.get("min_share").and_then(Value::as_float).ok_or_else(|| GateError::error("tolerance.min_share must be a number"))?;
    let max = row.get("max_share").and_then(Value::as_float).ok_or_else(|| GateError::error("tolerance.max_share must be a number"))?;
    if !(0.0..=1.0).contains(&min) || !(0.0..=1.0).contains(&max) || min > max { return Err(GateError::error("invalid answer-key tolerance band")); }
    let bank = Bank::load_dir(&ctx.root.join("bank/items")).map_err(|e| GateError::error(format!("load bank/items: {e}")))?;
    let mut counts = [0usize; 4];
    for item in bank.items.values().filter(|i| i.is_approved() && i.kind.is_letter_form()) { counts[LETTERS.iter().position(|l| *l == item.correct).ok_or_else(|| GateError::error(format!("{}: correct key is not A-D", item.id)))?] += 1; }
    let n = counts.iter().sum::<usize>();
    if n == 0 { return Err(GateError::error("zero approved single-select items (vacuous scan)")); }
    let summary = format!("approved single-select={n}; A={} ({:.1}%), B={} ({:.1}%), C={} ({:.1}%), D={} ({:.1}%)", counts[0], counts[0] as f64 * 100.0 / n as f64, counts[1], counts[1] as f64 * 100.0 / n as f64, counts[2], counts[2] as f64 * 100.0 / n as f64, counts[3], counts[3] as f64 * 100.0 / n as f64);
    let outside = LETTERS.iter().zip(counts).filter_map(|(l, c)| { let p = c as f64 / n as f64; (p < min || p > max).then_some(*l) }).collect::<Vec<_>>();
    if outside.is_empty() { println!("{NAME}: PASS: {summary}; band={:.1}%..{:.1}%", min * 100.0, max * 100.0); Ok(()) } else { Err(GateError::violation([format!("{summary}; band={:.1}%..{:.1}%; outside={}", min * 100.0, max * 100.0, outside.join(","))])) }
}
#[cfg(test)]
#[rustfmt::skip]
mod tests { use super::*; fn root(n: &str) -> GateCtx { GateCtx::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/answer_key_skew").join(n), vec![]) } #[test] fn known_bad_fixture_is_red() { assert_eq!(run(&root("bad")).unwrap_err().code(), crate::exit::VIOLATION); } #[test] fn uniform_fixture_is_green() { assert!(run(&root("uniform")).is_ok()); } }
