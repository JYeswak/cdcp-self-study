# cdcp_wasm

WASM dual-path grade surface (ORACLE-GAUNTLET L4 / L5 browser grade).

## API

- Rust: `grade_digest_json(bank_json, attempt_json) -> Result<String, String>`
- wasm32 C ABI: `cdcp_alloc` / `cdcp_free` / `cdcp_grade_digest` / `cdcp_last_ptr` / `cdcp_last_len`

Browser glue: [`../../web/assets/js/grade_bridge.js`](../../web/assets/js/grade_bridge.js).

## Build & install for the static web root

```bash
# from course-engine/ (preferred)
./scripts/build_web_wasm.sh          # release
./scripts/build_web_wasm.sh --debug  # faster

# Headless dual-path smoke vs goldens (Node 18+):
node scripts/smoke_results_wasm.mjs
```

Manual equivalent:

```bash
rustup target add wasm32-unknown-unknown
cargo build -p cdcp_wasm --target wasm32-unknown-unknown --release
mkdir -p web/assets/wasm
cp target/wasm32-unknown-unknown/release/cdcp_wasm.wasm web/assets/wasm/
```

If the `wasm32-unknown-unknown` target is missing, export-web + JS glue still ship;
browser grade is unavailable until the wasm artifact is present. Headless e2e can
use the native oracle:

```bash
cargo run -q -p cdcp_cli -- grade \
  --fixture goldens/fixtures/mock40_seed42.json --mode all-correct
```

## Bank JSON for grade

Use the pack from `cdcp export-web`:

- `web/data/bank_items_seed42.json` — full bank `BankItem` array (hash matches `cdcp bank-hash`)
- Attempt `bank_hash` must equal that bank’s hash (same as learner pack `bank_hash`)
