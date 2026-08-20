#!/usr/bin/env bash
# cdcp install.sh — one command, receipt-driven, fails closed. [bd-installability-sm4g.7]
# --verify proves the installed prefix (inode/path + root), not an occupier on 8766. [.18]
#   curl -fsSL https://raw.githubusercontent.com/JYeswak/cdcp-self-study/main/course-engine/install.sh | bash
# Trust anchors are compiled in (D6). Env cannot retarget them.
set -euo pipefail
umask 022

PRODUCT_VERSION=0.1.0
GITHUB_OWNER=JYeswak
GITHUB_REPO=cdcp-self-study
CLONE_URL=https://github.com/JYeswak/cdcp-self-study.git
API_URL=https://api.github.com/repos/JYeswak/cdcp-self-study
ENGINE_SUBDIR=course-engine
VERIFY_SPEC=$'doctor\ntest\ndemo --no-open'

PREFIX=${HOME}/.local
PINNED= FROM_SOURCE=0 NO_MODIFY_PATH=0 DRY=0 UNINSTALL=0 VERIFY_ONLY=0
TARBALL= EXPECT_SHA= RELEASE_JSON=
STAGED= WORK= LOCK= FILELIST= CONFIG_TOUCHED=
ARTIFACT_URL= ARTIFACT_SHA= SOURCE_BUILD=false
VERIFY_SERVE_PID= VERIFY_SERVE_URL=

die() { echo "cdcp-install: ERROR: $*" >&2; exit 1; }
log() { echo "cdcp-install: $*"; }
have() { type -P "$1" >/dev/null 2>&1; }
need() { have "$1" || die "missing $1 on PATH"; }
lc() { printf '%s' "$1" | tr 'A-F' 'a-f'; }
json_esc() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

# D1: probe /dev/tty, never stdin; pair with read -t.
ask_tty() {
  [ -r /dev/tty ] && [ -w /dev/tty ] || \
    die "need confirmation but /dev/tty is unavailable (refusing stdin — that is curl|bash)"
  printf '%s' "$1" >/dev/tty
  local ans=
  read -t 30 -r ans </dev/tty || die "timed out waiting for confirmation on /dev/tty"
  case $ans in y|Y|yes|YES) return 0 ;; *) die "declined" ;; esac
}

# D4: awk NR==1 first field only. Neither tool = ERROR (never skip).
sha256_file() {
  if have sha256sum; then sha256sum "$1" | awk 'NR==1{print $1; exit}'
  elif have shasum; then shasum -a 256 "$1" | awk 'NR==1{print $1; exit}'
  else die "neither sha256sum nor shasum -a 256 is on PATH — refusing to skip verification"
  fi
}
require_checksum_tool() {
  have sha256sum || have shasum || \
    die "neither sha256sum nor shasum -a 256 is on PATH — refusing to skip verification"
}

cleanup() {
  # Only our verify-time prefix listener. Never an occupier we did not start
  # (a source-checkout `cdcp serve` on 8766 is not tool-broken).
  if [ -n "${VERIFY_SERVE_PID:-}" ]; then
    kill "$VERIFY_SERVE_PID" 2>/dev/null || true
    wait "$VERIFY_SERVE_PID" 2>/dev/null || true
    VERIFY_SERVE_PID=
  fi
  [ -n "$STAGED" ] && rm -f "$STAGED"
  [ -n "$WORK" ] && rm -rf "$WORK"
  if [ -n "$LOCK" ] && [ -d "$LOCK" ]; then
    [ "$(cat "$LOCK/pid" 2>/dev/null || true)" = "$$" ] && rm -rf "$LOCK"
  fi
  return 0
}
trap cleanup EXIT INT TERM HUP

usage() { cat <<'EOF'
Usage: install.sh [--prefix DIR] [--version [VER]] [--from-source] [--no-modify-path]
                  [--dry-run] [--uninstall] [--verify] [--tarball FILE] [--sha256 HEX]
                  [--release-json FILE] [--help]
One-liner:
  curl -fsSL https://raw.githubusercontent.com/JYeswak/cdcp-self-study/main/course-engine/install.sh | bash

Source fallback prerequisite: Cargo and a usable Rust toolchain. This installer
does not install Rust. If Cargo has no configured default, it will try the
installed stable toolchain (`cargo +stable`); otherwise configure one first,
for example with `rustup default stable`.
EOF
}

detect_triple() {
  case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) echo aarch64-apple-darwin ;;
    Darwin-x86_64) echo x86_64-apple-darwin ;;
    Linux-x86_64) echo x86_64-unknown-linux-gnu ;;
    Linux-aarch64|Linux-arm64) echo aarch64-unknown-linux-gnu ;;
    *) die "unsupported platform $(uname -s)-$(uname -m) (ship .tar.gz, never .zip)" ;;
  esac
}

find_engine() {
  local src dir
  src=${BASH_SOURCE[0]:-$0}
  [ -f "$src" ] || return 1
  dir=$(CDPATH= cd -- "$(dirname "$src")" && pwd)
  if [ -f "$dir/Cargo.toml" ] && [ -d "$dir/crates/cdcp_cli" ]; then echo "$dir"; return 0; fi
  if [ -f "$dir/$ENGINE_SUBDIR/Cargo.toml" ] && [ -d "$dir/$ENGINE_SUBDIR/crates/cdcp_cli" ]; then
    echo "$dir/$ENGINE_SUBDIR"; return 0
  fi
  return 1
}

lock_acquire() {
  LOCK=${TMPDIR:-/tmp}/cdcp-install.lock
  if mkdir "$LOCK" 2>/dev/null; then printf '%s\n' "$$" >"$LOCK/pid"; return 0; fi
  local pid; pid=$(cat "$LOCK/pid" 2>/dev/null || true)
  if [ -n "$pid" ] && ! kill -0 "$pid" 2>/dev/null; then
    rm -rf "$LOCK"; mkdir "$LOCK" || die "cannot reclaim $LOCK"
    printf '%s\n' "$$" >"$LOCK/pid"; return 0
  fi
  die "install lock $LOCK is held (pid ${pid:-unknown})"
}

# Single-member streaming extract (zip-slip structurally impossible). tar.gz only.
extract_cdcp() {
  local tb=$1 dest=$2 list n=0 extra=0 member tv
  case $tb in *.zip) die "zip is refused (unzip propagates quarantine); ship .tar.gz" ;; esac
  [ -f "$tb" ] || die "tarball missing: $tb"
  list=$(tar -tzf "$tb") || die "not a readable .tar.gz: $tb"
  while IFS= read -r m; do
    [ -n "$m" ] || continue
    case $m in cdcp|./cdcp) n=$((n+1)); member=$m ;; *) extra=$((extra+1)) ;; esac
  done <<EOF
$list
EOF
  [ "$n" -eq 1 ] && [ "$extra" -eq 0 ] || \
    die "tarball must contain exactly one root-level regular member named cdcp (n=$n extra=$extra)"
  tv=$(tar -tvzf "$tb" | awk 'NR==1{print; exit}')
  case $tv in -*) ;; *) die "tarball member $member is not a regular file: $tv" ;; esac
  mkdir -p "$(dirname "$dest")"
  STAGED=${dest}.new.$$
  tar -xOf "$tb" "$member" >"$STAGED"
  chmod 0755 "$STAGED"
  mv -f "$STAGED" "$dest"   # D2: stage then atomic mv — not install -m 0755
  STAGED=
}

atomic_bin() {
  local src=$1 dest=$2
  [ -f "$src" ] || die "binary missing: $src"
  mkdir -p "$(dirname "$dest")"
  STAGED=${dest}.new.$$
  cp "$src" "$STAGED"; chmod 0755 "$STAGED"; mv -f "$STAGED" "$dest"; STAGED=
}

copy_web() {
  local src=$1 dst=$2 f rel
  [ -f "$src/index.html" ] || die "web/ is not a serveable bundle (missing $src/index.html)"
  mkdir -p "$dst"
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    rel=${f#"$src"/}
    mkdir -p "$(dirname "$dst/$rel")"
    cp "$f" "$dst/$rel"
    printf '%s\n' "$dst/$rel" >>"$FILELIST"
  done <<EOF
$(find "$src" -type f ! -name '.DS_Store' -print | sort)
EOF
  [ -s "$FILELIST" ] || die "copied 0 web/ files — empty bundle is an ERROR"
}

# D3: one-byte range request; never download-the-world --max-time 5.
preflight_url() {
  local code
  need curl
  code=$(curl -sS -L -o /dev/null -w '%{http_code}' -r 0-0 --max-time 30 "$1" || true)
  [ "$code" = 206 ] || [ "$code" = 200 ] || die "preflight $1 HTTP ${code:-000}"
}

select_asset() {
  awk -v t="$2" '
    /"name":/ { name=$0; sub(/.*"name":[[:space:]]*"/,"",name); sub(/".*/,"",name) }
    /"browser_download_url":/ {
      url=$0; sub(/.*"browser_download_url":[[:space:]]*"/,"",url); sub(/".*/,"",url)
      if (name ~ t && name ~ /\.tar\.gz$/ && name !~ /\.sha256/ && name !~ /\.zip/) print name "\t" url
    }
  ' "$1"
}

receipt_path() { echo "$SHARE/install-receipt.json"; }
kept_progress() { echo "$SHARE/var/attempts"; }

# Portable path/inode. pwd -P so /var and /private/var compare equal.
abspath() {
  local t=$1 d f
  if [ -d "$t" ]; then (CDPATH= cd -- "$t" && pwd -P)
  else
    d=$(dirname -- "$t"); f=$(basename -- "$t")
    printf '%s/%s\n' "$(CDPATH= cd -- "$d" && pwd -P)" "$f"
  fi
}
file_inode() {
  case "$(uname -s)" in
    Darwin) stat -f %i "$1" ;;
    *) stat -c %i "$1" ;;
  esac
}

# lsof is optional. Missing lsof + an unproven 8766 = foreign (fail closed).
have_lsof() { have lsof || [ -x /usr/sbin/lsof ] || [ -x /usr/bin/lsof ]; }
_lsof() {
  if have lsof; then lsof "$@"
  elif [ -x /usr/sbin/lsof ]; then /usr/sbin/lsof "$@"
  elif [ -x /usr/bin/lsof ]; then /usr/bin/lsof "$@"
  else return 1
  fi
}
listen_pids() {
  _lsof -nP -iTCP:"$1" -sTCP:LISTEN -t 2>/dev/null || true
}
proc_args() { ps -p "$1" -o args= 2>/dev/null || ps -p "$1" -o command= 2>/dev/null || true; }
proc_cwd() {
  if [ -L "/proc/$1/cwd" ]; then readlink "/proc/$1/cwd" 2>/dev/null || true; return 0; fi
  _lsof -a -p "$1" -d cwd -Fn 2>/dev/null | awk '/^n/{print substr($0,2); exit}'
}
proc_exe() {
  if [ -L "/proc/$1/exe" ]; then readlink "/proc/$1/exe" 2>/dev/null || true; return 0; fi
  _lsof -a -p "$1" -d txt -Fn 2>/dev/null | awk '/^n/{print substr($0,2); exit}'
}

# Listener belongs to THIS prefix only if exe is the installed inode/path
# AND (--root is the install root or cwd is the install root / its web/).
# Same binary serving a source checkout is foreign.
is_prefix_listener() {
  local pid=$1 bin=$2 inode=$3 root=$4
  local exe cwd args einode exe_ok=0
  [ -n "$pid" ] || return 1
  kill -0 "$pid" 2>/dev/null || return 1
  args=$(proc_args "$pid")
  exe=$(proc_exe "$pid")
  cwd=$(proc_cwd "$pid")
  if [ -n "$exe" ] && [ -e "$exe" ]; then
    einode=$(file_inode "$exe" 2>/dev/null || true)
    [ -n "$einode" ] && [ "$einode" = "$inode" ] && exe_ok=1
    [ "$(abspath "$exe" 2>/dev/null || true)" = "$bin" ] && exe_ok=1
  fi
  case $args in *"$bin"*) exe_ok=1 ;; esac
  [ "$exe_ok" = 1 ] || return 1
  case $args in *"--root $root"*|*"--root=$root"*) return 0 ;; esac
  case $cwd in "$root"|"$root/web") return 0 ;; esac
  return 1
}
prefix_owns_port() {
  local port=$1 bin=$2 inode=$3 root=$4 pid
  for pid in $(listen_pids "$port"); do
    is_prefix_listener "$pid" "$bin" "$inode" "$root" && return 0
  done
  return 1
}

# Concrete 127.0.0.1:PORT. Never return 8766 — caller uses this only when
# 8766 is not ours. python3 is optional; installed cdcp doctor --bind :0
# is the fallback (binds, prints the real port, releases).
ephemeral_bind() {
  local addr out
  if have python3; then
    addr=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print("%s:%s"%s.getsockname()[:2]);s.close()' 2>/dev/null || true)
    if [ -n "$addr" ] && [ "$addr" != "127.0.0.1:8766" ]; then echo "$addr"; return 0; fi
  fi
  out=$("$1" doctor --root "$2" --bind 127.0.0.1:0 2>&1) || true
  addr=$(printf '%s\n' "$out" | awk '/ok doctor port \(/ {
    s=$0; sub(/.*\(/,"",s); sub(/[ )].*/,"",s); print s; exit
  }')
  if [ -n "$addr" ] && [ "$addr" != "127.0.0.1:8766" ]; then echo "$addr"; return 0; fi
  die "cannot allocate ephemeral bind — refusing to treat 8766 as proof"
}

# Sets VERIFY_SERVE_PID and VERIFY_SERVE_URL in the caller (must NOT run
# inside $() — a subshell would drop the pid and leave a leaked listener).
start_prefix_serve() {
  local bin=$1 root=$2 bind=$3 logf=$4
  local i=0 url=
  VERIFY_SERVE_URL=
  : >"$logf"
  "$bin" serve --root "$root" --bind "$bind" --no-open >"$logf" 2>&1 &
  VERIFY_SERVE_PID=$!
  while [ "$i" -lt 50 ]; do
    if ! kill -0 "$VERIFY_SERVE_PID" 2>/dev/null; then
      die "prefix serve exited early: $(cat "$logf" 2>/dev/null || true)"
    fi
    url=$(awk '/^cdcp serve: http/{print $3; exit}' "$logf")
    if [ -n "$url" ]; then
      VERIFY_SERVE_URL=$url
      return 0
    fi
    sleep 0.1
    i=$((i+1))
  done
  die "prefix serve did not print a URL: $(cat "$logf" 2>/dev/null || true)"
}

stop_prefix_serve() {
  [ -n "${VERIFY_SERVE_PID:-}" ] || return 0
  kill "$VERIFY_SERVE_PID" 2>/dev/null || true
  wait "$VERIFY_SERVE_PID" 2>/dev/null || true
  VERIFY_SERVE_PID=
}

url_hostport() {
  # http://127.0.0.1:1234/ → 127.0.0.1:1234
  local u=$1
  u=${u#http://}; u=${u#https://}; u=${u%%/*}
  printf '%s\n' "$u"
}

# D5: --verify always, before already-installed early exit. Empty list = ERROR.
# W7: prove the *installed* prefix. A foreign occupier on 8766 is not proof
# and is not killed. If demo prints a URL, that listener is ours or we bound
# ephemeral and printed that URL. Never curl 8766.
run_verify() {
  local bin=$BINDIR/cdcp n=0 line cmd work
  local installed_bin installed_inode install_root
  local demo_bind occupier_pids occupier_pid url hostport demo_out rc
  [ -n "$VERIFY_SPEC" ] || die "empty verify command list — test -x is a vacuous pass"
  [ -x "$bin" ] || die "installed binary not executable: $bin"
  [ -f "$SHARE/web/index.html" ] || die "installed web/ missing: $SHARE/web/index.html"

  installed_bin=$(abspath "$bin")
  installed_inode=$(file_inode "$installed_bin")
  install_root=$(abspath "$SHARE")
  log "verify: installed-bin=$installed_bin inode=$installed_inode"
  log "verify: install-root=$install_root"

  occupier_pids=$(listen_pids 8766)
  occupier_pid=$(printf '%s\n' "$occupier_pids" | awk 'NR==1{print; exit}')
  if prefix_owns_port 8766 "$installed_bin" "$installed_inode" "$install_root"; then
    demo_bind=127.0.0.1:8766
    log "verify: 8766 belongs to install prefix (pid=${occupier_pid:-unknown})"
  elif [ -n "$occupier_pid" ]; then
    log "verify: 8766 occupied by foreign pid=$occupier_pid — not our proof"
    log "verify: occupier args=$(proc_args "$occupier_pid")"
    start_prefix_serve "$installed_bin" "$install_root" "127.0.0.1:0" \
      "${WORK:-${TMPDIR:-/tmp}}/cdcp-verify-serve.log"
    demo_bind=$(url_hostport "$VERIFY_SERVE_URL")
    [ -n "$demo_bind" ] || die "prefix serve printed no bindable URL"
    is_prefix_listener "$VERIFY_SERVE_PID" "$installed_bin" "$installed_inode" "$install_root" || \
      die "prefix serve pid=$VERIFY_SERVE_PID is not the install prefix"
    log "verify: prefix-listener pid=$VERIFY_SERVE_PID bind=$demo_bind (ephemeral; occupier 8766 is not proof)"
  else
    # lsof listed nobody. No lsof at all → fail closed (do not claim 8766).
    if have_lsof; then
      demo_bind=127.0.0.1:8766
      log "verify: 8766 bindable (no occupier)"
    else
      demo_bind=$(ephemeral_bind "$installed_bin" "$install_root")
      log "verify: 8766 owner unlisted (no lsof) — not our proof"
      log "verify: demo-bind=$demo_bind (ephemeral; occupier 8766 is not proof)"
    fi
  fi

  work=$(mktemp -d "${TMPDIR:-/tmp}/cdcp-verify.XXXXXX")
  demo_out=$work/demo.out
  : >"$demo_out"
  (
    CDPATH= cd -- "$work" || exit 1
    unset CDCP_REPO_ROOT; export CDCP_HOME=$install_root PATH=$BINDIR:$PATH
    while IFS= read -r line; do
      [ -n "$line" ] || continue
      n=$((n+1))
      # shellcheck disable=SC2086
      set -- $line
      cmd=$1; shift
      if [ "$cmd" = demo ]; then
        echo "cdcp-install: verify: $installed_bin $cmd --root $install_root $* --bind $demo_bind"
        rc=0
        "$installed_bin" "$cmd" --root "$install_root" "$@" --bind "$demo_bind" >"$demo_out" 2>&1 || rc=$?
        cat "$demo_out"
        [ "$rc" -eq 0 ] || exit "$rc"
      else
        echo "cdcp-install: verify: $installed_bin $cmd --root $install_root $*"
        "$installed_bin" "$cmd" --root "$install_root" "$@"
      fi
    done <<EOF
$VERIFY_SPEC
EOF
    [ "$n" -gt 0 ] || exit 1
  ) || { stop_prefix_serve; rm -rf "$work"; die "verify failed (installed binary vs installed prefix)"; }

  url=$(awk '/^cdcp demo: http/{print $3; exit}' "$demo_out")
  if [ -n "$url" ]; then
    hostport=$(url_hostport "$url")
    case $hostport in
      127.0.0.1:8766|localhost:8766)
        if prefix_owns_port 8766 "$installed_bin" "$installed_inode" "$install_root"; then
          log "verify: demo-url=$url (listener belongs to install prefix)"
        elif [ -z "$(listen_pids 8766)" ]; then
          log "verify: demo-url=$url (8766 free — advertised default, not an occupier)"
        else
          stop_prefix_serve
          rm -rf "$work"
          die "demo printed $url but 8766 is a foreign occupier — not our proof"
        fi
        ;;
      *)
        [ "$hostport" = "$demo_bind" ] || {
          stop_prefix_serve
          rm -rf "$work"
          die "demo printed $url but verify bound $demo_bind"
        }
        if [ -n "${VERIFY_SERVE_PID:-}" ]; then
          is_prefix_listener "$VERIFY_SERVE_PID" "$installed_bin" "$installed_inode" "$install_root" || {
            stop_prefix_serve
            rm -rf "$work"
            die "demo-url=$url listener pid=$VERIFY_SERVE_PID is not the install prefix"
          }
        fi
        log "verify: demo-url=$url (ephemeral prefix listener, not occupier 8766)"
        ;;
    esac
  fi

  stop_prefix_serve
  rm -rf "$work"
  log "verify: measured green"
}

awk_paths() {
  awk -v key="$2" '
    $0 ~ "\"" key "\"" { inf=1; next }
    inf && /"path"/ { line=$0; sub(/.*"path"[[:space:]]*:[[:space:]]*"/,"",line); sub(/".*/,"",line); print line }
    inf && /^[[:space:]]*],?[[:space:]]*$/ { inf=0 }
  ' "$1"
}

write_receipt() {
  local dest ts f hash first=1
  dest=$(receipt_path); ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  mkdir -p "$SHARE"
  {
    printf '{\n  "version": "%s",\n  "installed_at": "%s",\n  "triple": "%s",\n  "source_build": %s,\n' \
      "$(json_esc "$VERSION")" "$(json_esc "$ts")" "$(json_esc "$TRIPLE")" "$SOURCE_BUILD"
    printf '  "artifact": {"url": "%s", "sha256": "%s", "triple": "%s"},\n  "files": [\n' \
      "$(json_esc "$1")" "$(json_esc "$2")" "$(json_esc "$TRIPLE")"
    while IFS= read -r f; do
      [ -n "$f" ] && [ -f "$f" ] || continue
      hash=$(sha256_file "$f")
      [ "$first" -eq 1 ] && first=0 || printf ',\n'
      printf '    {"path": "%s", "sha256": "%s"}' "$(json_esc "$f")" "$(json_esc "$hash")"
    done <"$FILELIST"
    printf '\n  ],\n  "config_touched": ['
    [ -z "$CONFIG_TOUCHED" ] || \
      printf '\n    {"path": "%s", "kind": "path_line", "marker": "cdcp PATH"}\n  ' "$(json_esc "$CONFIG_TOUCHED")"
    printf '],\n  "learner_progress_kept": true,\n  "learner_progress_paths": ["%s"]\n}\n' \
      "$(json_esc "$(kept_progress)")"
  } >"$dest.new"
  mv -f "$dest.new" "$dest"
  log "receipt: $dest"
}

strip_path_block() {
  local rc=$1 tmp
  [ -f "$rc" ] || return 0
  tmp=$rc.cdcp-uninst.$$
  awk 'BEGIN{s=0} /# cdcp PATH \(managed by install.sh\)/{s=1;next} /# end cdcp PATH/{s=0;next} s==0{print}' \
    "$rc" >"$tmp"
  mv -f "$tmp" "$rc"
}

do_uninstall() {
  local rec f leftover=0 dest
  rec=$(receipt_path); dest=$(kept_progress)
  # D8: no receipt → refuse. "complete" is a measurement. var/attempts is KEPT.
  [ -f "$rec" ] || die "no receipt at $rec — refusing to guess (will not delete var/attempts)"
  if [ "$DRY" = 1 ]; then
    log "dry-run uninstall (keep $dest)"
    awk_paths "$rec" files | while IFS= read -r f; do [ -n "$f" ] && log "dry-run: would remove $f"; done
    awk_paths "$rec" config_touched | while IFS= read -r f; do [ -n "$f" ] && log "dry-run: would strip $f"; done
    return 0
  fi
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    case $f in "$dest"|"$dest"/*) log "keeping learner progress $f"; continue ;; esac
    rm -f "$f" || die "failed to remove $f"
    if [ -e "$f" ] || [ -L "$f" ]; then echo "still present: $f" >&2; leftover=$((leftover+1))
    else log "removed $f"; fi
  done <<EOF
$(awk_paths "$rec" files)
EOF
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    strip_path_block "$f" || die "failed to strip $f"
    if grep -q 'cdcp PATH (managed by install.sh)' "$f" 2>/dev/null; then
      echo "PATH block still in $f" >&2; leftover=$((leftover+1))
    else log "stripped PATH block from $f"; fi
  done <<EOF
$(awk_paths "$rec" config_touched)
EOF
  rm -f "$rec" || die "failed to remove receipt"
  [ -e "$rec" ] && leftover=$((leftover+1))
  if [ -d "$SHARE/web" ]; then
    find "$SHARE/web" -depth -type d -empty -delete 2>/dev/null || true
  fi
  rmdir "$BINDIR" 2>/dev/null || true
  [ "$leftover" -eq 0 ] && [ ! -e "$BINDIR/cdcp" ] && [ ! -e "$SHARE/web" ] || \
    die "uninstall incomplete: leftover=$leftover (not asserting complete)"
  log "uninstall: measured complete (learner progress kept at $dest)"
}

maybe_path() {
  [ "$NO_MODIFY_PATH" = 1 ] && return 0
  case :$PATH: in *":$BINDIR:"*) return 0 ;; esac
  [ "$DRY" = 1 ] && { log "dry-run: would append $BINDIR to PATH"; return 0; }
  local rc
  case ${SHELL:-} in */zsh) rc=$HOME/.zshrc ;; */bash) rc=$HOME/.bashrc ;; *) rc=$HOME/.profile ;; esac
  if [ -f "$rc" ] && grep -q 'cdcp PATH (managed by install.sh)' "$rc"; then CONFIG_TOUCHED=$rc; return 0; fi
  printf '\n# cdcp PATH (managed by install.sh)\nexport PATH="%s:$PATH"\n# end cdcp PATH\n' "$BINDIR" >>"$rc"
  CONFIG_TOUCHED=$rc
  log "PATH: appended $BINDIR in $rc"
}

need_cargo() {
  have cargo || die "cargo not on PATH — source fallback requires Cargo and a Rust toolchain; refusing to install a toolchain (this is not rustup-init). Install Rust, then re-run --from-source."
}

maybe_copy_web_from_checkout() {
  local e
  e=$(find_engine || true)
  [ -n "$e" ] && [ -f "$e/web/index.html" ] && copy_web "$e/web" "$SHARE/web"
}

source_build() {
  local engine built target url sha tv got
  if engine=$(find_engine); then log "source-build from checkout $engine"
  else
    need git; need_cargo
    [ "$DRY" = 1 ] && { log "dry-run: would clone $CLONE_URL and cargo --release --locked"; return 0; }
    mkdir -p "$WORK/src"
    git clone --depth 1 "$CLONE_URL" "$WORK/src/repo" || die "git clone failed"
    if [ -n "$PINNED" ]; then
      git -C "$WORK/src/repo" fetch --tags --depth 1 origin "v$PINNED" 2>/dev/null || true
      git -C "$WORK/src/repo" checkout "v$PINNED" 2>/dev/null || \
        git -C "$WORK/src/repo" checkout "$PINNED" 2>/dev/null || \
        die "clone has no tag $PINNED — refusing to retarget a pinned artifact"
      got=$(git -C "$WORK/src/repo" describe --tags --exact-match 2>/dev/null || true)
      [ "$got" = "v$PINNED" ] || [ "$got" = "$PINNED" ] || \
        die "clone HEAD is '${got:-untagged}', not pinned $PINNED — refusing to retarget"
    fi
    if [ -f "$WORK/src/repo/$ENGINE_SUBDIR/Cargo.toml" ]; then engine=$WORK/src/repo/$ENGINE_SUBDIR
    elif [ -f "$WORK/src/repo/Cargo.toml" ]; then engine=$WORK/src/repo
    else die "clone has no course-engine Cargo.toml"; fi
  fi
  [ -f "$engine/Cargo.lock" ] || die "Cargo.lock missing — --locked requires it"
  if [ -n "$PINNED" ]; then
    tv=$(awk '/^version = "/ {gsub(/"/,"",$3); print $3; exit}' "$engine/Cargo.toml")
    [ -z "$tv" ] || [ "$tv" = "$PINNED" ] || die "checkout version $tv != pinned $PINNED — refusing to retarget"
  fi
  if [ "$DRY" = 1 ]; then log "dry-run: would cargo --release --locked -p cdcp_cli in $engine"; return 0; fi
  need_cargo
  log "cargo build --release --locked -p cdcp_cli (D7)"
  # W1 (d): stable has no trim-paths. Remap PWD + HOME so the binary
  # embeds no /Users/ or /home/runner paths.
  (CDPATH= cd -- "$engine" && \
    RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }--remap-path-prefix=${PWD}= --remap-path-prefix=${HOME}=" \
    cargo build --release --locked -p cdcp_cli) || \
    die "cargo --release --locked -p cdcp_cli failed"
  target=${CARGO_TARGET_DIR:-$engine/target}
  built=$target/release/cdcp
  atomic_bin "$built" "$BINDIR/cdcp"
  printf '%s\n' "$BINDIR/cdcp" >>"$FILELIST"
  copy_web "$engine/web" "$SHARE/web"
  sha=$(sha256_file "$BINDIR/cdcp")
  url=source://$engine
  have git && git -C "$engine" rev-parse HEAD >/dev/null 2>&1 && \
    url=source://$(git -C "$engine" rev-parse HEAD)
  ARTIFACT_URL=$url ARTIFACT_SHA=$sha SOURCE_BUILD=true
}

download_extract() {
  local url=$1 sha=$2 tb got
  [ "$DRY" = 1 ] && { log "dry-run: would download $url → $BINDIR/cdcp"; return 0; }
  preflight_url "$url"
  tb=$WORK/cdcp.tar.gz
  need curl
  curl -sS -L --max-time 300 -o "$tb" "$url" || die "download failed: $url"
  got=$(sha256_file "$tb")
  [ "$(lc "$got")" = "$(lc "$sha")" ] || die "sha256 mismatch: got $got want $sha — installing nothing"
  extract_cdcp "$tb" "$BINDIR/cdcp"
  printf '%s\n' "$BINDIR/cdcp" >>"$FILELIST"
  maybe_copy_web_from_checkout
  ARTIFACT_URL=$url ARTIFACT_SHA=$sha SOURCE_BUILD=false
}

from_tarball() {
  local got
  case $TARBALL in *.zip) die "zip is refused; ship .tar.gz" ;; esac
  [ -f "$TARBALL" ] || die "tarball missing: $TARBALL"
  [ -n "$EXPECT_SHA" ] || die "--tarball requires --sha256 (refusing unverified artifact)"
  got=$(sha256_file "$TARBALL")
  [ "$(lc "$got")" = "$(lc "$EXPECT_SHA")" ] || \
    die "sha256 mismatch: got $got want $EXPECT_SHA — installing nothing"
  [ "$DRY" = 1 ] && { log "dry-run: would extract $TARBALL"; return 0; }
  extract_cdcp "$TARBALL" "$BINDIR/cdcp"
  printf '%s\n' "$BINDIR/cdcp" >>"$FILELIST"
  maybe_copy_web_from_checkout
  ARTIFACT_URL=file://$TARBALL ARTIFACT_SHA=$EXPECT_SHA SOURCE_BUILD=false
}

# A release that exists with zero matching assets is ERROR (not success, not fallback).
handle_release() {
  local hits hit name url sha shafile
  hits=$(select_asset "$1" "$TRIPLE" || true)
  [ -n "$hits" ] || die "no .tar.gz asset matching triple $TRIPLE — ERROR (not a fall-through to success)"
  hit=$(printf '%s\n' "$hits" | awk 'NR==1{print; exit}')
  name=$(printf '%s\n' "$hit" | awk -F'\t' 'NR==1{print $1; exit}')
  url=$(printf '%s\n' "$hit" | awk -F'\t' 'NR==1{print $2; exit}')
  [ -n "$url" ] || die "asset $name has no browser_download_url"
  if [ -n "$EXPECT_SHA" ]; then sha=$EXPECT_SHA
  else
    shafile=$WORK/asset.sha256; need curl
    curl -sS -L --max-time 60 -o "$shafile" "$url.sha256" || \
      die "missing checksum file $url.sha256 — refusing to skip verification"
    sha=$(awk 'NR==1{print $1; exit}' "$shafile")
  fi
  [ -n "$sha" ] || die "empty checksum for $name"
  download_extract "$url" "$sha"
}

do_install() {
  local code json
  VERSION=${PINNED:-$PRODUCT_VERSION}
  [ -n "$TARBALL" ] && { from_tarball; return; }
  [ "$FROM_SOURCE" = 1 ] && { source_build; return; }
  if [ -n "$RELEASE_JSON" ]; then
    [ -f "$RELEASE_JSON" ] || die "--release-json missing: $RELEASE_JSON"
    handle_release "$RELEASE_JSON"; return
  fi
  json=$WORK/release.json
  if [ -n "$PINNED" ]; then
    need curl
    code=$(curl -sS -L -o "$json" -w '%{http_code}' --max-time 30 "$API_URL/releases/tags/v$PINNED" || true)
    [ "$code" = 200 ] || die "pinned v$PINNED has no GitHub release (HTTP ${code:-000}) — refusing to retarget"
    handle_release "$json"; return
  fi
  if have curl; then
    code=$(curl -sS -L -o "$json" -w '%{http_code}' --max-time 30 "$API_URL/releases/latest" || true)
    if [ "$code" = 200 ]; then handle_release "$json"; return; fi
    [ "$code" = 404 ] && log "no GitHub release (0 tags); source-build" || \
      log "GitHub latest HTTP ${code:-000}; source-build"
  else log "curl not on PATH; source-build"; fi
  source_build
}

while [ $# -gt 0 ]; do
  case $1 in
    --help|-h) usage; exit 0 ;;
    --version)
      if [ -n "${2:-}" ] && [ "${2#-}" = "$2" ]; then PINNED=$2; shift 2
      else echo "cdcp-install $PRODUCT_VERSION"; exit 0; fi ;;
    --prefix) [ -n "${2:-}" ] || die "--prefix needs an argument"; PREFIX=$2; shift 2 ;;
    --from-source) FROM_SOURCE=1; shift ;;
    --no-modify-path) NO_MODIFY_PATH=1; shift ;;
    --dry-run) DRY=1; shift ;;
    --uninstall) UNINSTALL=1; shift ;;
    --verify) VERIFY_ONLY=1; shift ;;
    --tarball) [ -n "${2:-}" ] || die "--tarball needs an argument"; TARBALL=$2; shift 2 ;;
    --sha256) [ -n "${2:-}" ] || die "--sha256 needs an argument"; EXPECT_SHA=$2; shift 2 ;;
    --release-json) [ -n "${2:-}" ] || die "--release-json needs an argument"; RELEASE_JSON=$2; shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done

# D6: ignore env retargets (anchors are not env).
[ -z "${CDCP_INSTALL_URL:-}${GITHUB_REPOSITORY:-}${CDCP_REPO:-}" ] || \
  log "ignoring CDCP_INSTALL_URL / GITHUB_REPOSITORY / CDCP_REPO (trust anchors are not env)"

PREFIX=${PREFIX%/}
BINDIR=$PREFIX/bin
SHARE=$PREFIX/share/cdcp
TRIPLE=$(detect_triple)
VERSION=${PINNED:-$PRODUCT_VERSION}

[ "$UNINSTALL" = 1 ] && [ "$VERIFY_ONLY" = 1 ] && [ "$DRY" != 1 ] && \
  die "use one of --uninstall or --verify"

INSTALLING=1
if [ "$UNINSTALL" = 1 ]; then INSTALLING=0
elif [ "$VERIFY_ONLY" = 1 ] && [ -z "$TARBALL" ] && [ "$FROM_SOURCE" = 0 ] && [ -z "$RELEASE_JSON" ]; then
  INSTALLING=0
fi

require_checksum_tool
lock_acquire
WORK=$(mktemp -d "${TMPDIR:-/tmp}/cdcp-inst.XXXXXX")
FILELIST=$WORK/files.txt
: >"$FILELIST"

[ "$UNINSTALL" = 1 ] && { do_uninstall; exit 0; }

# D5: --verify before any already-installed early exit.
if [ "$VERIFY_ONLY" = 1 ] && [ "$INSTALLING" = 0 ]; then run_verify; exit 0; fi
if [ "$VERIFY_ONLY" = 1 ] && [ "$INSTALLING" = 1 ] && [ -x "$BINDIR/cdcp" ] && [ -f "$(receipt_path)" ]; then
  run_verify; log "already installed at $PREFIX; verify green (no reinstall)"; exit 0
fi

if [ "$DRY" = 1 ]; then
  log "dry-run install prefix=$PREFIX triple=$TRIPLE version=$VERSION from_source=$FROM_SOURCE"
  do_install; exit 0
fi

do_install
maybe_path
[ -f "$BINDIR/cdcp" ] || die "install wrote no binary"
write_receipt "${ARTIFACT_URL:-source}" "${ARTIFACT_SHA:-$(sha256_file "$BINDIR/cdcp")}" "$SOURCE_BUILD"
run_verify
log "install: measured complete (verify green) prefix=$PREFIX"
exit 0
