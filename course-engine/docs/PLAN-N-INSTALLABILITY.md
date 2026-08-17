# PLAN — EPIC N: `cdcp` is installable in one command

Status: REVISED after adversarial review + independent falsification · 2026-08-17
Companion: EPIC M publishes the *corpus and graph*; this epic publishes the *tool*.

> **Revision note.** Draft 1 was reviewed hostilely and falsified independently. It
> contained a contradiction that would have shipped a brick (§W1), advertised a command
> that does not exist (`cdcp study`), and claimed a pipe-safety property the binary does
> not have. Those are corrected below. Two defects found during review outrank
> everything draft 1 contained: the mock exam is broken for 3 of 4 seeds it offers
> (§W3), and the wasm equivalence oracle tests a different binary than the one that
> ships (§W4).

---

## 1. Plain-language summary

Today, using `cdcp` requires cloning a repo and building a Rust workspace. After this
epic a person runs one command and is studying inside a minute. **The command that does
this does not exist yet and is the epic's core deliverable** (§W2) — today the nearest
thing is `cdcp serve`, which prints a URL and blocks.

**Why anyone would care.** The CDCP study material is the asset. It is currently
reachable only by people who can build Rust from source. The engine works; distribution
is the missing half.

---

## 2. The product boundary — measured, not assumed

| Layer | Size | Installed? |
|---|---|---|
| **Learner product** — `cdcp` binary + `web/` | **76 files, 2.52 MiB** | **YES** |
| Authoring / gate machinery — `registries/`, `knowledge/`, `goldens/`, `bank/` | 7.5 MB | no |
| Source-only — `crates/cdcp_data/snapshots.toml` (read live by `verify-data-lock`) | — | no |

**Falsified properly.** `web/` copied alone into a temp dir with no `registries/claims.toml`
at or above it, served via `cdcp serve --root <abs>`: all 80 files 200, an exhaustive
crawl of 443 `src`/`href` refs returned **0 bad**, zero external hosts, traversal
attempts 404. Grading works from the isolated copy — a headless replication of
`results.js` graded 40 pack items against 826 bank rows with 0 missing keys, and two
planted known-bads behaved (all-correct ≠ all-wrong digest; a zeroed `bank_hash` was
*rejected*, not graded). `serve` correctly honours `--root` — it is **not** one of the
six commands broken by §W9.

So the mechanism holds. "Complete" does not, for two reasons now tracked as §W3 and §W4.

**Non-goals.** Corpus redistribution (EPIC M, licence-gated). Homebrew tap, Docker,
crates.io, Windows, auto-update.

---

## 3. Workstreams, in dependency order

Critical path: **W1 → W2 → W7 → W8**. W3, W4, W5 are independent and should run in
parallel. W9, W10, W11 are off the installability path entirely.

### W1 — the release binary resolves its installed bundle (P0, the keystone)

**This is one indivisible bead. Splitting it ships a brick.**

Draft 1 said "delete the `CARGO_MANIFEST_DIR` fallback." That fallback is currently the
*only* mechanism by which the binary resolves anything outside a repo, and the anchor it
seeks (`registries/claims.toml`) is in §2's not-installed row. Deleting it without a
replacement resolver leaves a binary that can find neither `registries/` nor `web/`
(`serve --root` defaults to the bare relative string `"web"`, `main.rs:236`) and has no
env var to be told where either lives. Delete and replace are the same commit.

**Scope.**
- An anchor for an *installed* tree, which cannot be `registries/claims.toml`.
- Resolution order: `--root` flag > `CDCP_HOME` > `$XDG_DATA_HOME/cdcp` >
  `~/.local/share/cdcp` > cwd walk.
- Source-checkout vs installed precedence, with **the chosen root printed**. Silent
  precedence is the next fooled certificate.
- Remove the compile-time fallback from **5 definitions** — `cdcp_gate/src/root.rs:29`,
  `cdcp_learn/src/lib.rs:92`, `cdcp_evidence/src/licence.rs:623`,
  `cdcp_anki/src/lib.rs:1706`, `cdcp_registry_check/src/lib.rs:169` — **plus 1 delegate**,
  `cdcp_data::engine_root` (`lib.rs:460`), which inherits without copying.
- The five are **not** byte-identical, and the differences are load-bearing:
  `cdcp_registry_check` walks 8 levels (others 12), hard-errors when `canonicalize` fails
  (others silently ignore), and carries a **`CDCP_REPO_ROOT` env override**
  (`lib.rs:147`) the others lack — an env-overridable root, i.e. our own D6 defect
  already inside the binary. Unify deliberately; do not paper over.

**Acceptance (W1′, falsifiable).** In a container with no copy of this repo and no Rust
toolchain, given only the release binary and the bundle at `$XDG_DATA_HOME/cdcp/web`:
(a) `cdcp study --no-open` binds a port and `curl -fsS http://127.0.0.1:$PORT/` returns
200 with a body containing `<title>`; (b) `cdcp --version` prints exactly the workspace
version, exit 0; (c) with the bundle **removed**, the same command exits 4 and stderr
names the absolute path it looked for; (d) `strings $(command -v cdcp) | grep -c '/Users/'`
is **0**. Empty container image = ERROR. Any of (a)–(d) unmeasured = ERROR.

### W2 — `cdcp study` (P0 — the product) · depends W1

Resolve bundle → bind a port → print and open the URL. Today `serve` binds a fixed 8766,
and on conflict exits 1 with a raw OS error; `doctor` reports the *tool* broken when
8766 is busy (`operator.rs:383-397`), which is a wrong verdict, not a diagnosis. `study`
retries or uses `:0`, prints the URL, opens a browser unless `--no-open`, and makes
`Ctrl-C` discoverable.

### W3 — the mock exam works for every seed the UI offers (P0 — live product bug)

`mock.html:57-62` offers seeds 42 / 7 / 1 / 99. **Measured: 42 → 200/200/200; 7, 1 and
99 → 404/404/404** for pack, bank and keys. The failure path (`mock.js:101-106`) then
tells the learner to run `cdcp export-web --seed N` — a source-checkout command an
installed learner cannot run.

This is broken **today, in the repo**, independent of installation. Either export the
three missing seeds or stop offering them. **Acceptance:** a test enumerates the seeds
the UI *offers* and asserts every one resolves; offering a seed with no data is RED.
An empty seed list is an ERROR, not a pass.

### W4 — the wasm oracle must test the artifact that ships (P0)

The shipped grader is `web/assets/wasm/cdcp_wasm.wasm` — tracked in git, 530,385 B,
`1fe5bc26…`, a `--release` build. The L4 dual-path oracle
(`crates/cdcp_wasm/tests/dual_path.rs:62,88`) loads
`target/wasm32-unknown-unknown/**debug**/cdcp_wasm.wasm` — 24,414,860 B, `89de94c8…`.
**Different binaries.** The native==wasm equivalence proof has never covered the artifact
the browser loads.

Supporting gaps: `check.sh:1156` asserts only `[ -f ]`; `check.sh` **never invokes**
`scripts/build_web_wasm.sh`, the only thing that refreshes the blob; and
`check_wasm` (`operator.rs:256-275`) checks existence, non-emptiness and `\0asm` magic
while its error strings say **"not fresh"** — it cannot detect staleness and claims to.

**Measured today:** the committed blob *is* byte-identical to a fresh `--release
--locked` build. So this is a latent hole, not an active wrong-answer bug. Freshness is
currently a property of discipline, not of mechanism.

**Acceptance.** `check.sh` rebuilds `cdcp_wasm` for `wasm32-unknown-unknown --release
--locked` and asserts `sha256` equality with the committed blob; the dual-path oracle
points at the shipped artifact. Planted known-bad: flip one byte in the committed blob →
RED **naming the wasm**, not a downstream digest. Second plant: change a grade-affecting
constant and rebuild only native → RED. Emits `INJECTIONS=2 SUITE=wasm-freshness` [[fact:fact-injections-enforced=yes]]. If
the wasm32 target is absent the step is **SKIPPED and the run may not be advertised as
full-green** — a skip must not read like a pass (`check.sh:1130` currently lets the whole
leg skip).

### W5 — first-contact fixes (P0, cheap, outsized blast radius)

| Surface | Measured today | Required |
|---|---|---|
| `cdcp --version` / `-V` | `error: unexpected argument`, **exit 2**; zero `version` attrs workspace-wide | prints workspace version, exit 0 |
| `cdcp` bare | **stdout 0 bytes, stderr 3,944 bytes, exit 2** | oriented output on **stdout**, exit 0 |
| colour in pipes | `Cargo.toml:36` takes `clap` **without** `default-features = false`, so `anstream`/`anstyle` are live. Under a PTY bare `cdcp` emits **45 ANSI escapes**; `CLICOLOR_FORCE=1` forces colour into **piped** output | `default-features = false`, and the pinning test controls the environment |

*Draft 1 claimed the CLI was "pipe-safe by construction." That was wrong — pipe-safety
is clap's runtime TTY detection, which an env var overrides.*

### W6 — the installed binary is a five-command tool (P1) · depends W1

40 subcommands exist; genuinely learner-facing is **`serve`** — plus `doctor` and
`health` once they have a learner-shaped check list. `DOCTOR_CHECKS`
(`operator.rs:37-38`) is `bank, wasm, goldens, content.lock, port, python3`; four of six
read the not-installed layer, and `health_envelope` reads `bank/items`,
`bank/MANIFEST.toml` and `goldens/`. **So `health --robot` — the binary's only structured
output — is RED-by-construction on every installed machine.**

Hide the rest behind `CDCP_DEV=1` (`#[command(hide = true)]`): reachable for the source
workflow, invisible to a learner. Also drop `python3` from learner-facing checks —
measured, **no learner path needs it**; the 9 `.py` files are all differential oracles.

W12–W16 grow the learner set. After those land, `cdcp --help` lists the **operator
surface**, not a rigid five: `study` (or `serve` until W2), `doctor`, `demo`, `test`,
`repair`, plus read-only `health` / `quickstart`. Authoring stays behind `CDCP_DEV=1`.
W6 hide-commands waits on W12 so we do not hide the verbs we just added.

### W12–W16 — mirror br / ntm / franken operator surface (P1 · depends W1)

Joshua 2026-08-17: installability is not just curl|bash. Mirror the franken repos and
`br` / `bv` / `ntm`: **one-command install, then demo, doctor, repair, tests**.

Measured today (`cdcp` already has the *names*):

| Surface | Today | br / ntm | Required for installed learner |
|---|---|---|---|
| `doctor` | probes bank / goldens / content.lock / python3 — RED-by-construction on a bundle-only tree | `br doctor` (read-only default, `--repair`/`--dry-run`, `capabilities`, `robot-docs`, `undo`) · `ntm doctor --json` | probe **installed** layer only: `web/`, shipped wasm `\0asm`, receipt, bindable port. Missing wasm RED **names the path**. `--json`. Empty probe list is ERROR |
| `health` | `--robot` envelope reads bank + goldens — same RED-by-construction | `br doctor health` · `ntm health` / `--robot-health` | exit 0 on bundle-only; schema lists only installed facts |
| `repair` | rebuilds units/glossary/slugs/export-web; **no `--dry-run`** | `br doctor --repair --dry-run` through `mutate()`; refuses when irreversible | `--dry-run` mandatory; `--apply` mutates; receipt-driven bundle integrity; never re-freezes `goldens/` (already a law). Missing receipt is refuse, not guess (W7 D8) |
| `demo` | **does not exist** | franken showcase / `ntm --robot-docs=quickstart` | one command: bind (or print URL), run planted all-correct + all-wrong grade against the **shipped** wasm, print the 2-minute path. Empty planted set is ERROR |
| `test` / smoke | 10+ `smoke-*` authoring verbs | `br` tests live in CI; installers have `--verify` | one `cdcp test` on the **installed** tree: learner-pack shape + wasm magic + mock seed 42 assets 200. Empty suite is ERROR. Not `check.sh` |
| self-doc | clap `--help` / `--version` only (W5) | `br info`, `br robot-docs`, `ntm --robot-docs=quickstart`, `completions` | `--info` (version + resolved root + env), `quickstart`, `help <topic>`, `completion <shell>` |

**Still deferred (not these beads):** fleet robot mega-surface (`--robot-triage`, 70+ ntm robot verbs), `cdcp next` SRS, Homebrew/Docker. Those wait for a consumer. Doctor/repair/demo/test **have** a consumer: the stranger who just installed.

Anti-vacuity: a doctor that cannot construct a tree *without* bank/goldens is a vacuous pass and must FAIL. A demo that only prints "run serve" is not a demo.

### W7 — `install.sh` (P1, ~150 lines) · depends W1, W2, W12–W16

Derived from three reference installers (~7,190 lines). **Copy what they got right;
refuse eight measured defects.**

| # | Defect | Our rule |
|---|---|---|
| D1 | `[ -t 0 ]` is **false under `curl \| bash`** (stdin is the script), so a `(y/N)` prompt silently becomes "yes, always" and auto-installs a rustup nightly | probe `/dev/tty`, never stdin; pair with `read -t` |
| D2 | `install -m 0755` cannot replace a running binary on Linux (`ETXTBSY`) | stage in the destination dir, atomic `mv` |
| D3 | network preflight downloads the whole artifact and discards it, `--max-time 5` | one-byte range request |
| D4 | `awk '{print $1}'` concatenates fields on multi-line checksum files | `awk 'NR==1{print $1; exit}'` |
| D5 | `--verify` silently skipped when already installed | `--verify` always runs, before any early exit |
| D6 | trust anchors env-overridable, so a hostile environment retargets verification | anchors are the security boundary; **not** read from env |
| D7 | source fallback builds without `--locked` | `--release --locked` always |
| D8 | uninstaller discards every exit status and prints `Uninstall complete!` unconditionally | every status checked; "complete" is a **measurement**, never an assertion |

**Copy verbatim:** single-member streaming extract (makes zip-slip structurally
impossible); `type -P`, not `command -v` (an alias cannot fake verification); prove the
clone matches the tag in the source fallback; refuse to retarget a pinned artifact;
`--verify` as a planted-failure gate run under a sandboxed `HOME`/`XDG_CONFIG_HOME`.

**The receipt — the piece the whole family is missing.** No reference installer records
what it did, which is why their uninstaller is a hardcoded guess that misses the PATH
line it wrote and stops at the first binary it finds. Write
`~/.local/share/cdcp/install-receipt.json` (version, artifact url+sha256+triple, every
file with checksum, config touched, `source_build`), and drive `--uninstall`,
`--dry-run` and bundle-integrity `--repair` from it. Decide and record: what `--uninstall`
does with learner progress (`var/`, the attempts store), and what it does when **no
receipt exists** — refusing is defensible, guessing is D8.

**`--verify` must prove it is talking to its own server.** During review a stale `cdcp
serve` from an earlier session held a port; `curl` returned 200 from the *real repo*
while the new bind failed `EADDRINUSE`. Absent an `lsof` check, that would have certified
an isolated bundle while reading the repo — a fooled certificate of the same family as
W1. `--verify` asserts the listener's PID and root are its own.

**Bundle/binary version match.** They ship as separate files, so partial install (binary
replaced, bundle write failed) is the *default* multi-file failure mode. The bundle
carries a version stamp; the binary refuses on mismatch rather than serving a stale app.

**Post-install operator bar (Joshua 2026-08-17 — franken / br / ntm).**
`install.sh --verify` (and a successful install's last step) runs the *installed*
binary against the *installed* prefix, never the source checkout:

```
cdcp doctor && cdcp test && cdcp demo --no-open
```

Empty command list is ERROR. A verify that only checks `test -x "$BIN"` is a vacuous
pass of the same family as W1. This is why W7 depends W12–W16, not only W1/W2.

Installer-workmanship that fits the ~150-line budget: `set -euo pipefail`, `umask 022`,
`trap cleanup EXIT`, SHA256 via `sha256sum` or `shasum -a 256` (neither present is
ERROR), mkdir-based lock, `--dry-run` / `--uninstall` / `--verify` / `--prefix`.
**Non-goals** (DCG-specific, no learner consumer): gum UI, AI-agent hook auto-config,
skill-tarball install.

**macOS Gatekeeper — resolved, no signing needed.** Apple DTS states plainly that
*"Unix-y networking tools, like `curl` and `scp`, don't quarantine the files they
download."* No quarantine attribute ⇒ Sequoia's tightening never engages. Corroboration:
Homebrew's own `rg` and `jq` are ad-hoc signed, not notarized. **One correction to
Apple's own docs, measured on macOS 26.2: `unzip` *does* propagate quarantine** (a
sentinel quarantine UUID survived into the extracted binary); `tar` does not. So **ship
`.tar.gz`, never `.zip`** — an evidence-backed rule, and neither Apple's docs nor the
usual secondary sources can be cited for the zip case.

### W8 — release engineering (P1) · depends W5, W11

No `.github/` at all, **0 git tags**, 0 releases. Artifacts are `.tar.gz` (W7). Day-one
triples: `aarch64-apple-darwin` primary, `x86_64-unknown-linux-gnu` secondary.
**Blocked by W11** — a release workflow gated on `check.sh` cannot be green while
`check.sh` is red.

### W9 — `--root` is silently ignored by six commands (P1, parallel, off critical path)

`compile_learn` (`main.rs:1642-1648`) treats `--root` as a *starting point for an upward
walk*, then escapes to the compile-time path. Measured: `build-learn --root <emptydir>`
→ **exit 0**, nothing written to the empty dir, real repo mutated. All six
(`main.rs:716-721`) route through this one function.

Demoted from draft 1's blocker list: no installed learner can reach these commands, so
they fool no installer test. But it is a real P0 for the *authoring* workflow — and it
corrupts the very bundle being shipped, since `web/` is generated, not static.
**Acceptance:** each of the six, given `--root <emptydir>`, exits non-zero and writes
nothing; a sentinel file planted in the repo is untouched.

### W10 — generators are freshness-asserted in check.sh (P2, parallel)

`check.sh:1171` runs `build-learn` and asserts only that it *ran*. Stale artifacts stay
green. **Already caught one live bug** (2026-08-17): the committed Learn surface had
drifted from `content/modules/` — module 01 advertised `estimate_minutes: 24` for a
42-minute module (word_count 3,484 vs 6,149). Fixed and committed; the *guard* is not.
Applies to `build-learn`, `build-reference`, `build-units`, `build-glossary`,
`build-learn-slugs`, `export-web`.

### W11 — the `gate-shrink` ratchet is RED (P0 for W8 only)

Measured: `cdcp_gate` **50,852 > ceiling 49,422**. `check.sh` fails at the L1 constitution
step today. Raising the ceiling is weakening a gate and is escalation-only; the fix is
extraction or deletion.

*(Separately: `cdcp_registry_check` treats its first positional arg as the root, so
`--help` is read as a path and reports `registries/claims.toml missing` for a file that
plainly exists. Small, but it is an agent's first instinctive command.)*

---

## 4. Deliberately deferred

**Un-deferred 2026-08-17 (Joshua):** learner `doctor` / `repair --dry-run` / `demo` /
`test` / `--info`+`quickstart`+`completion`. Those now have a named consumer (the
installed stranger). See W12–W16.

**Still deferred:** ntm-scale robot mega-surface (`--robot-triage`, 70+ `--robot-*`
verbs), six-code exit-dictionary *drift guard*, `cdcp next` (SRS — EPIC I leftover
store, not install). The four-way README↔installer drift guard stays cut until W7
and W8 exist — guarding an empty set is a vacuous pass.

---

## 5. Rule Zero audit

PRODUCT: W1 (only paired with the resolver — the deletion alone is a regression), W2,
W3, W4, W5, W6, W7, W8, W12–W16 (doctor / repair / demo / test / self-doc). NOT-PRODUCT:
W10, W11, and the remaining §4 deferrals. W9 is product for the *authoring* workflow,
not the learner.

Draft 1 was roughly half gate-work. The tell was structural: it specified the installer
in exquisite detail while never making the payload able to run at the destination.

## 6. Rigor layers

L1 claims — §3 cites file:line and a measured result throughout. **Caveat: §W7's D1–D8
citations were fetched from remote repos and are not verifiable from this machine**; they
carry less weight than §3's and must not be advertised as equal.
L3 external oracle — *a machine that is not the build machine*: a container with no repo,
the only environment where W1 can be falsified.
L4 gates proven to trip — every gate here ships a planted known-bad.
L7 — `--locked` on every build path.

**Anti-vacuity for this epic:** an installer that finds no matching release asset must
ERROR. An installer that can compute no checksum (neither `shasum -a 256` nor
`sha256sum`) must ERROR. Both would otherwise report exactly like success.

## 7. Gated

Tagging and publishing to GitHub Releases is external and irreversible — escalation-only.
Everything in §3 is local and reversible. The first `git tag v0.1.0` is Josh's call.
