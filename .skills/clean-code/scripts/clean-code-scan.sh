#!/usr/bin/env bash
# clean-code-scan.sh — Quét code smell phổ biến cho DB Pro (frontend TS/React + Rust)
# Usage: bash .skills/clean-code/scripts/clean-code-scan.sh [frontend|rust|all] [--with-linters] [--ci] [--diff]
#   frontend | rust | all   : phạm vi (mặc định all)
#   --with-linters          : chạy thêm eslint / prettier / cargo fmt / clippy
#   --ci                    : exit 1 nếu có ✗
#   --diff                  : chỉ quét file thay đổi so với origin/main (hoặc main)
#
# Script chỉ dùng grep/awk — không cần cài thêm gì. Đây là bộ lọc thô, không thay thế review.

set -u

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'
BOLD='\033[1m'

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"
cd "$PROJECT_ROOT" || exit 1

SCOPE="all"
WITH_LINTERS=0
CI_MODE=0
DIFF_MODE=0
for arg in "$@"; do
  case "$arg" in
    frontend|rust|all) SCOPE="$arg" ;;
    --with-linters) WITH_LINTERS=1 ;;
    --ci) CI_MODE=1 ;;
    --diff) DIFF_MODE=1 ;;
    -h|--help) sed -n '2,10p' "$0"; exit 0 ;;
  esac
done

# ─── Ngưỡng (đồng bộ với references/formatting.md) ─────────────────
FN_WARN_LINES=50
FN_FAIL_LINES=100
TS_FILE_WARN=400
TS_FILE_FAIL=600
RS_FILE_WARN=800
RS_FILE_FAIL=1200
MAX_PARAMS=3
MAX_SHOW=15   # số dòng ví dụ tối đa in ra mỗi mục

PASS=0
WARN=0
FAIL=0

section() { echo ""; echo -e "${BLUE}${BOLD}═══ $1 ═══${NC}"; echo ""; }
check() {
  local label="$1" status="$2" detail="$3"
  case "$status" in
    pass) echo -e "  ${GREEN}✓${NC} ${label}: ${detail}"; PASS=$((PASS + 1)) ;;
    warn) echo -e "  ${YELLOW}⚠${NC} ${label}: ${detail}"; WARN=$((WARN + 1)) ;;
    fail) echo -e "  ${RED}✗${NC} ${label}: ${detail}"; FAIL=$((FAIL + 1)) ;;
  esac
}
show() { # in tối đa MAX_SHOW dòng, thụt lề
  local shown=0
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    echo "      $line"
    shown=$((shown + 1))
    [ "$shown" -ge "$MAX_SHOW" ] && { echo "      …"; break; }
  done
}
count_lines() { local n; n="$(grep -c . 2>/dev/null)"; echo "${n:-0}"; }
strip_comments() { grep -vE '^[^:]+:[0-9]+:\s*(//|/\*|\*|///|//!)' ; } # bỏ dòng chỉ có comment khỏi kết quả grep

# ─── Danh sách file ─────────────────────────────────────────────────
list_files() { # $1 = ts|rs
  local ext="$1"
  if [ "$DIFF_MODE" -eq 1 ]; then
    local base
    base="$(git merge-base HEAD origin/main 2>/dev/null || git merge-base HEAD main 2>/dev/null || echo HEAD~1)"
    { git diff --name-only --diff-filter=ACMR "$base" --; git ls-files --others --exclude-standard; } | sort -u | while read -r f; do [ -f "$f" ] && echo "$f"; done
  else
    git ls-files
  fi | case "$ext" in
    ts) grep -E '^frontend/src/.*\.(ts|tsx)$' ;;
    rs) grep -E '^crates/.*\.rs$' ;;
  esac
}
prod_only() { # loại file test
  grep -vE '(__tests__/|\.test\.tsx?$|\.spec\.tsx?$|/tests?/|/benches/|/fixtures/|routeTree\.gen\.ts$|^frontend/src/dev/)'
}

TS_FILES="$(list_files ts)"
TS_PROD="$(echo "$TS_FILES" | prod_only)"
RS_FILES="$(list_files rs)"
RS_PROD="$(echo "$RS_FILES" | prod_only)"

grep_in() { # $1 = danh sách file (newline), $2.. = grep args
  local files="$1"; shift
  [ -z "$files" ] && return 0
  echo "$files" | xargs -d '\n' grep -nH "$@" 2>/dev/null
}

# ─── Helpers phân tích ─────────────────────────────────────────────
# Đếm độ dài hàm thô bằng cân bằng ngoặc nhọn từ dòng khai báo hàm.
# Không phải parser thật — đủ để gắn cờ ứng viên cần nhìn kỹ.
long_functions() { # $1 = files, $2 = regex mở đầu hàm
  local files="$1" start_re="$2"
  [ -z "$files" ] && return 0
  echo "$files" | xargs -d '\n' awk -v start_re="$start_re" -v warn="$FN_WARN_LINES" -v fail="$FN_FAIL_LINES" '
    FNR == 1 { infn = 0; intest = 0 }
    /^[[:space:]]*#\[cfg\(test\)\]/ { intest = 1 }
    intest { next }
    {
      if (!infn && $0 ~ start_re && $0 ~ /\{[[:space:]]*$/) {
        infn = 1; depth = 0; startline = FNR; name = $0
        sub(/^[[:space:]]+/, "", name); if (length(name) > 70) name = substr(name, 1, 70) "…"
      }
      if (infn) {
        n = gsub(/\{/, "{"); m = gsub(/\}/, "}")
        depth += n - m
        if (depth <= 0) {
          len = FNR - startline + 1
          if (len > warn) printf "%s:%d  (%d dòng)%s  %s\n", FILENAME, startline, len, (len > fail ? "  [FAIL]" : ""), name
          infn = 0
        }
      }
    }'
}

file_sizes() { # $1 = files, $2 warn, $3 fail
  local files="$1" warn="$2" fail="$3"
  [ -z "$files" ] && return 0
  echo "$files" | xargs -d '\n' wc -l 2>/dev/null | grep -v ' total$' | awk -v warn="$warn" -v fail="$fail" '
    $1 > warn { printf "%s  (%d dòng)%s\n", $2, $1, ($1 > fail ? "  [FAIL]" : "") }' | sort -t'(' -k2 -rn
}

report_list() { # $1 label, $2 output, $3 severity-if-nonempty (warn|fail), $4 extra note
  local label="$1" out="$2" sev="$3" note="${4:-}"
  local n; n="$(echo "$out" | count_lines)"
  if [ "$n" -eq 0 ]; then
    check "$label" pass "không có"
  else
    check "$label" "$sev" "$n chỗ${note:+ — $note}"
    echo "$out" | show
  fi
}

report_size_list() { # phân biệt warn/fail theo tag [FAIL]; chỉ chặn (✗) khi --diff (gate code mới), toàn repo là nợ hiện có → ⚠
  local label="$1" out="$2"
  local n nf; n="$(echo "$out" | count_lines)"; nf="$(echo "$out" | grep -c '\[FAIL\]')"
  if [ "$n" -eq 0 ]; then
    check "$label" pass "trong ngưỡng"
  elif [ "$nf" -gt 0 ] && [ "$DIFF_MODE" -eq 1 ]; then
    check "$label" fail "$n vượt ngưỡng cảnh báo, $nf vượt ngưỡng chặn"
    echo "$out" | show
  elif [ "$nf" -gt 0 ]; then
    check "$label" warn "$n vượt ngưỡng cảnh báo, $nf vượt ngưỡng chặn (nợ hiện có — dùng --diff để gate PR)"
    echo "$out" | show
  else
    check "$label" warn "$n vượt ngưỡng cảnh báo"
    echo "$out" | show
  fi
}

# ─── FRONTEND ───────────────────────────────────────────────────────
scan_frontend() {
  section "Frontend — Code Health (TypeScript / React)"
  local nfiles; nfiles="$(echo "$TS_PROD" | count_lines)"
  echo "  Quét $nfiles file production (bỏ qua test/fixtures/src/dev)"
  echo ""

  # 9. Debug logs
  report_list "console.log/warn/debug/debugger" \
    "$(grep_in "$TS_PROD" -E '(^|[^.a-zA-Z_])console\.(log|warn|debug|info|table|trace)\(|^\s*debugger;?')" fail \
    "xoá hoặc thay bằng logger có level (code-health.md §2)"

  # 7. Nuốt lỗi — catch trống hoàn toàn = ✗; catch chỉ có comment lý do = ⚠ (nên log thêm)
  report_list "catch {} trống hoàn toàn" \
    "$(echo "$TS_PROD" | xargs -d '\n' grep -nHE 'catch\s*(\([^)]*\))?\s*\{\s*\}' 2>/dev/null; \
       echo "$TS_PROD" | xargs -d '\n' awk '
         /catch[[:space:]]*(\([^)]*\))?[[:space:]]*\{[[:space:]]*$/ { start = FNR; line = $0; getline nxt
           if (nxt ~ /^[[:space:]]*\}/) printf "%s:%d: %s\n", FILENAME, start, line }' 2>/dev/null)" fail \
    "error-handling.md §3 — P1"
  report_list "catch chỉ có comment (bỏ qua có chủ ý)" \
    "$(echo "$TS_PROD" | xargs -d '\n' awk '
         /catch[[:space:]]*(\([^)]*\))?[[:space:]]*\{[[:space:]]*$/ { start = FNR; line = $0; getline nxt
           if (nxt ~ /^[[:space:]]*\/\//) { getline nxt2; if (nxt2 ~ /^[[:space:]]*\}/) printf "%s:%d: %s\n", FILENAME, start, line } }' 2>/dev/null)" warn \
    "chấp nhận nếu lý do đúng; cân nhắc logger.debug (error-handling.md §3)"

  # 6. Lách type
  report_list "any (explicit)" \
    "$(grep_in "$TS_PROD" -E ':\s*any\b|<any>|as any\b|any\[\]' | strip_comments)" warn \
    "thay bằng unknown + narrowing / generic (objects-and-data.md §6)"
  report_list "as unknown as / @ts-ignore / @ts-expect-error không lý do" \
    "$(grep_in "$TS_PROD" -E 'as unknown as|@ts-ignore|@ts-expect-error\s*$')" warn
  report_list "non-null assertion (!.)" \
    "$(grep_in "$TS_PROD" -E '[a-zA-Z0-9_\)\]]!\.|[a-zA-Z0-9_\)\]]!\[|[a-zA-Z0-9_\)\]]!;' | grep -vE '!==|!=')" warn \
    "ưu tiên ?. / early return"

  # 9. Tắt lint
  report_list "eslint-disable không lý do" \
    "$(grep_in "$TS_PROD" -E 'eslint-disable' | grep -vE '\-\-\s*\S+')" warn \
    "thêm '-- lý do' sau rule hoặc sửa code"

  # 9. Test residue (quét cả test)
  report_list ".only / .skip trong test" \
    "$(grep_in "$TS_FILES" -E '\b(it|test|describe)\.(only|skip)\(')" fail

  # 4. Comment rác / TODO mồ côi
  report_list "TODO/FIXME/HACK/XXX không có (#issue|owner)" \
    "$(grep_in "$TS_PROD" -E '(//|/\*|\*)\s*(TODO|FIXME|HACK|XXX)\b' | grep -vE '(TODO|FIXME|HACK|XXX)\((#[0-9]+|[a-zA-Z0-9_-]+)\)')" warn \
    "comments.md §4"
  report_list "code bị comment-out (heuristic)" \
    "$(grep_in "$TS_PROD" -E '^\s*//\s*((const|let|var|import|export|return|await)\b.*[;{}]\s*$|(if|for|while)\s*\(.*\)\s*\{\s*$|[a-zA-Z_$][a-zA-Z0-9_$.]*\(.*\);\s*$|\}\s*\)?;?\s*$)')" warn \
    "xoá — Git đã nhớ (comments.md)"

  # 2. Magic numbers (heuristic, loại 0/1/-1/2, index, ms/px hiển nhiên)
  report_list "magic number ≥ 3 chữ số ngoài hằng số (heuristic)" \
    "$(grep_in "$TS_PROD" -E '[^a-zA-Z0-9_."'"'"'#-][0-9]{3,}\b' \
        | grep -vE '^\S+:\s*(export\s+)?const\s+[A-Z0-9_]+\s*=' \
        | strip_comments \
        | grep -vE '(z-\[|w-\[|h-\[|className|tailwind|#[0-9a-fA-F]{3,}|0x|px|rem|em\b|\btest\b|localhost|1000\b|1024\b|version|Date\.UTC|toFixed|fontWeight|zIndex|aria-|=\{[0-9]+\}|style|width|height|duration|opacity)' )" warn \
    "informational — đặt tên hằng số nếu có ý nghĩa nghiệp vụ (naming.md §3)"

  # 3. Tham số hàm — chỉ định nghĩa (function / arrow / method có type annotation), không bắt call-site
  report_list "hàm > $MAX_PARAMS tham số (heuristic một dòng)" \
    "$(grep_in "$TS_PROD" -E '(function\s*[a-zA-Z_$]*\s*(<[^>]*>)?\(([^(){}]*,){'"$MAX_PARAMS"',}[^(){}]*\)|\(([^(){}]*,){'"$MAX_PARAMS"',}[^(){}]*\)\s*(:\s*[^=]+)?\s*=>|^\s*(public|private|protected|static|async|readonly|\s)*[a-zA-Z_$][a-zA-Z0-9_$]*\s*(<[^>]*>)?\(([^(){}]*:[^(){}]*,){'"$MAX_PARAMS"',}[^(){}]*\))' \
        | strip_comments | grep -vE '^[^:]+:[0-9]+:\s*(import|export \{)' )" warn \
    "gom thành options object (functions.md §3)"

  # 3. Boolean flag param (heuristic)
  report_list "tham số boolean flag (heuristic)" \
    "$(grep_in "$TS_PROD" -E '\(([^()]*,\s*)?[a-zA-Z_$][a-zA-Z0-9_$]*\??:\s*boolean(\s*=\s*(true|false))?[,)]' | grep -vE '(^\S+:\s*(interface|type)\s|Props|props|\bon[A-Z]|\bset[A-Z][a-zA-Z]*\s*[:(=]|resolve|enabled\??:|\(value: boolean\)|\(open: boolean\)|\(v: boolean\))')" warn \
    "cân nhắc tách hàm / enum (functions.md §3)"

  # 3. Hàm dài
  report_size_list "hàm dài (> $FN_WARN_LINES dòng, heuristic)" \
    "$(long_functions "$TS_PROD" '^[[:space:]]*(export[[:space:]]+)?(default[[:space:]]+)?(async[[:space:]]+)?function[[:space:]]+[A-Za-z_$][A-Za-z0-9_$]*|^[[:space:]]*(export[[:space:]]+)?const[[:space:]]+[A-Za-z_$][A-Za-z0-9_$]*[[:space:]]*(:[^=]+)?=[[:space:]]*(async[[:space:]]+)?(\([^)]*\)|[A-Za-z_$][A-Za-z0-9_$]*)[[:space:]]*(:[^=]+)?=>')"

  # 5. File dài
  report_size_list "file dài (> $TS_FILE_WARN dòng)" "$(file_sizes "$TS_PROD" "$TS_FILE_WARN" "$TS_FILE_FAIL")"

  # 8. DIP — component gọi invoke trực tiếp
  report_list "invoke() Tauri trực tiếp trong components/routes" \
    "$(grep_in "$(echo "$TS_PROD" | grep -E '/(components|routes)/')" -E "invoke\s*(<[^>]*>)?\s*\(\s*[\"'\`]")" warn \
    "đi qua service/DI để test được (design-principles.md — DIP)"

  # 6. Component định nghĩa trong component (heuristic: function PascalCase thụt lề ≥ 2 trong .tsx)
  report_list "component định nghĩa bên trong component (heuristic)" \
    "$(grep_in "$(echo "$TS_PROD" | grep -E '\.tsx$')" -E '^\s{2,}(const|function)\s+[A-Z][A-Za-z0-9]*\s*(=\s*(\([^)]*\)|[a-z]+)\s*=>|\()' | grep -vE '(useMemo|useCallback|React\.memo|forwardRef|^\S+:\s+(export|import))')" warn \
    "đưa ra module-level (functions.md §9)"
}

# ─── RUST ───────────────────────────────────────────────────────────
scan_rust() {
  section "Rust — Code Health"
  local nfiles; nfiles="$(echo "$RS_PROD" | count_lines)"
  echo "  Quét $nfiles file production (bỏ qua tests/benches)"
  echo ""

  # Loại bỏ nội dung #[cfg(test)] khỏi kết quả unwrap bằng cách quét theo file & dòng trước mod tests
  rs_prod_nontest() { # in "file:line:text" cho các dòng trước `#[cfg(test)]` đầu tiên
    [ -z "$RS_PROD" ] && return 0
    echo "$RS_PROD" | xargs -d '\n' awk '
      FNR == 1 { intest = 0 }
      /^[[:space:]]*#\[cfg\(test\)\]/ { intest = 1 }
      !intest { printf "%s:%d:%s\n", FILENAME, FNR, $0 }'
  }
  local NONTEST; NONTEST="$(rs_prod_nontest)"

  # 9. Debug output
  report_list "println!/dbg!/eprintln! ngoài main.rs" \
    "$(echo "$NONTEST" | grep -E '\b(println|dbg|eprintln)!\s*\(' | grep -vE '^crates/[^/]+/src/main\.rs:')" fail \
    "dùng tracing (code-health.md §2)"

  # 6/7. unwrap / expect trên đường dữ liệu
  report_list "unwrap()/expect() ngoài test" \
    "$(echo "$NONTEST" | grep -E '\.(unwrap|expect)\(' | grep -vE '(unwrap_or|unwrap_or_else|unwrap_or_default|expect_err|unwrap_err)')" warn \
    "trong commands/ và adapter là P1 — dùng ? / let-else (error-handling.md)"

  # 7. Nuốt lỗi
  report_list "let _ = <fallible> không comment lý do" \
    "$(echo "$RS_PROD" | xargs -d '\n' awk '
      FNR == 1 { prev = "" }
      /^[[:space:]]*let _ = / && prev !~ /^[[:space:]]*\/\// { printf "%s:%d: %s\n", FILENAME, FNR, $0 }
      { prev = $0 }' 2>/dev/null)" warn \
    "thêm comment lý do + tracing::debug, hoặc xử lý lỗi (error-handling.md §3)"
  report_list ".ok(); bỏ qua Result" \
    "$(echo "$NONTEST" | grep -E '\.ok\(\);\s*$')" warn
  report_list "unwrap_or_default() trên Result (heuristic)" \
    "$(echo "$NONTEST" | grep -E ':[0-9]+:.*\.unwrap_or_default\(\)' | grep -E ':[0-9]+:.*((try_get|parse|from_str|read[a-z_]*|open|fetch[a-z_]*|execute[a-z_]*|query[a-z_]*|connect|introspect[a-z_]*|load[a-z_]*)\([^)]*\)\s*\.unwrap_or_default|\.await\s*\.unwrap_or_default)')" warn \
    "lỗi IO/DB thành giá trị rỗng là nuốt lỗi"

  # 9. Placeholder
  report_list "todo!()/unimplemented!()" \
    "$(echo "$NONTEST" | grep -E '\b(todo|unimplemented)!\s*\(')" fail \
    "LSP: capability-gate với lý do rõ (design-principles.md)"

  # 9. allow không lý do
  report_list "#[allow(...)] không có comment lý do ngay trên" \
    "$(echo "$RS_PROD" | xargs -d '\n' awk '
      FNR == 1 { prev = "" }
      /^[[:space:]]*#!?\[allow\(/ && prev !~ /^[[:space:]]*\/\// { printf "%s:%d: %s\n", FILENAME, FNR, $0 }
      { prev = $0 }' 2>/dev/null)" warn

  # 6. unsafe không SAFETY
  report_list "unsafe không có // SAFETY:" \
    "$(echo "$RS_PROD" | xargs -d '\n' awk '
      FNR == 1 { prev = "" }
      /\bunsafe[[:space:]]*\{/ && prev !~ /SAFETY:/ && $0 !~ /SAFETY:/ { printf "%s:%d: %s\n", FILENAME, FNR, $0 }
      { prev = $0 }' 2>/dev/null)" fail

  # 6. as cast số (heuristic)
  report_list "cast 'as' giữa kiểu số có thể mất dữ liệu" \
    "$(echo "$NONTEST" | grep -E '\bas\s+(u8|u16|u32|i8|i16|i32|usize|isize)\b' | grep -vE '(as usize\]|\bas usize\)|// ok:|// safe:)')" warn \
    "ưu tiên try_from (objects-and-data.md §6)"

  # 4. TODO mồ côi / code comment-out
  report_list "TODO/FIXME/HACK/XXX không có (#issue|owner)" \
    "$(grep_in "$RS_PROD" -E '(//|/\*|\*)\s*(TODO|FIXME|HACK|XXX)\b' | grep -vE '(TODO|FIXME|HACK|XXX)\((#[0-9]+|[a-zA-Z0-9_-]+)\)')" warn
  report_list "code bị comment-out (heuristic)" \
    "$(grep_in "$RS_PROD" -E '^\s*//\s*((let|use|pub|return)\b.*;\s*$|(fn|impl|if|for|match)\b.*\{\s*$|[a-z_][a-z0-9_:.]*\(.*\)[;,]?\s*$|\}\s*[;,]?\s*$)' | grep -vE '^[^:]+:[0-9]+:\s*(///|//!)' )" warn

  # 8. DIP — core không import driver
  local core_leak; core_leak="$(grep -rnE '(^\s*(pub\s+)?use\s+(sqlx|rusqlite|tokio_postgres)\b|\b(sqlx|rusqlite|tokio_postgres)::)' crates/core/src 2>/dev/null | strip_comments)"
  report_list "crates/core import driver DB (sqlx/rusqlite)" "$core_leak" fail "vi phạm ports/adapters (design-principles.md — DIP)"

  # 3. Tham số hàm — đếm ':' trong ngoặc tròn sau khi bỏ '::' (self không có ':')
  report_list "fn > $MAX_PARAMS tham số (không tính self, heuristic một dòng)" \
    "$(grep_in "$RS_PROD" -E '^\s*(pub(\([a-z]+\))?\s+)?(async\s+)?(unsafe\s+)?fn\s+[a-z_][a-z0-9_]*\s*(<[^>]*>)?\(.*\)' \
        | awk -v max="$MAX_PARAMS" '{
            sig = $0; sub(/^[^(]*\(/, "", sig); sub(/\)[^)]*$/, "", sig)
            gsub(/::/, "", sig); gsub(/<[^<>]*>/, "", sig)
            n = gsub(/:/, ":", sig)
            if (n > max) print $0 }')" warn \
    "gom thành struct options / builder (functions.md §3)"

  # 3. Hàm dài
  report_size_list "fn dài (> $FN_WARN_LINES dòng, heuristic)" \
    "$(long_functions "$RS_PROD" '^[[:space:]]*(pub(\([a-z]+\))?[[:space:]]+)?(async[[:space:]]+)?(unsafe[[:space:]]+)?fn[[:space:]]+[a-z_][a-z0-9_]*')"

  # 5. File dài
  report_size_list "file dài (> $RS_FILE_WARN dòng)" "$(file_sizes "$RS_PROD" "$RS_FILE_WARN" "$RS_FILE_FAIL")"

  # 3. clone() dày đặc (heuristic — file có > 15 clone)
  report_list "file có > 15 .clone() (heuristic)" \
    "$(grep_in "$RS_PROD" -c '\.clone()' | awk -F: '$2 > 15 { printf "%s  (%d clone)\n", $1, $2 }')" warn \
    "kiểm tra borrow / Arc / Cow (functions.md §10)"
}

# ─── LINTERS ────────────────────────────────────────────────────────
run_linters() {
  section "Linters / Formatters (từ AGENTS.md)"
  if [ "$SCOPE" != "rust" ] && [ -d frontend/node_modules ]; then
    (cd frontend && pnpm run -s lint >/tmp/cc-eslint.log 2>&1) && check "eslint" pass "sạch" || { check "eslint" fail "xem /tmp/cc-eslint.log"; tail -n 20 /tmp/cc-eslint.log | show; }
    (cd frontend && pnpm run -s format:check >/tmp/cc-prettier.log 2>&1) && check "prettier" pass "sạch" || { check "prettier" fail "chạy pnpm run format"; tail -n 10 /tmp/cc-prettier.log | show; }
    (cd frontend && pnpm run -s typecheck >/tmp/cc-tsc.log 2>&1) && check "tsc" pass "sạch" || { check "tsc" fail "xem /tmp/cc-tsc.log"; tail -n 20 /tmp/cc-tsc.log | show; }
  elif [ "$SCOPE" != "rust" ]; then
    check "frontend linters" warn "bỏ qua — frontend/node_modules chưa cài (pnpm install)"
  fi
  if [ "$SCOPE" != "frontend" ] && command -v cargo >/dev/null 2>&1; then
    cargo fmt --all -- --check >/tmp/cc-fmt.log 2>&1 && check "cargo fmt" pass "sạch" || { check "cargo fmt" fail "chạy cargo fmt --all"; tail -n 10 /tmp/cc-fmt.log | show; }
    cargo clippy --workspace --all-targets -q >/tmp/cc-clippy.log 2>&1 && check "cargo clippy" pass "sạch" || { check "cargo clippy" fail "xem /tmp/cc-clippy.log"; grep -E '^(warning|error)' /tmp/cc-clippy.log | sort | uniq -c | sort -rn | head -n 10 | show; }
  elif [ "$SCOPE" != "frontend" ]; then
    check "rust linters" warn "bỏ qua — cargo không có trong PATH"
  fi
}

# ─── MAIN ───────────────────────────────────────────────────────────
echo -e "${BOLD}Clean Code Scan — DB Pro${NC}"
echo "  root : $PROJECT_ROOT"
echo "  scope: $SCOPE$([ "$DIFF_MODE" -eq 1 ] && echo ' (chỉ file thay đổi so với main)')"
echo "  refs : .skills/clean-code/references/"

case "$SCOPE" in
  frontend) scan_frontend ;;
  rust) scan_rust ;;
  all) scan_frontend; scan_rust ;;
esac
[ "$WITH_LINTERS" -eq 1 ] && run_linters

section "Tổng kết"
echo -e "  ${GREEN}✓ pass: $PASS${NC}   ${YELLOW}⚠ warn: $WARN${NC}   ${RED}✗ fail: $FAIL${NC}"
echo ""
echo "  ✗ = sửa trước khi mở PR · ⚠ = giải thích trong PR nếu giữ nguyên"
echo "  Heuristic grep/awk có thể báo nhầm — dùng như bộ lọc, không phải phán quyết."
echo "  Checklist review thủ công: .skills/clean-code/references/review-checklist.md"
echo ""

if [ "$CI_MODE" -eq 1 ] && [ "$FAIL" -gt 0 ]; then exit 1; fi
exit 0
