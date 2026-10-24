#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Cách dùng:
  search-memory.sh [--archive] <từ khóa tìm kiếm>

Tìm memory của project trước, sau đó đến global. Thao tác chỉ đọc.
Đặt PROJECT_ROOT hoặc AGENTS_HOME để ghi đè đường dẫn được phát hiện.
USAGE
}

include_archive=false
if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi
if [[ "${1:-}" == "--archive" ]]; then
  include_archive=true
  shift
fi
if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi
query="$*"

if [[ -n "${PROJECT_ROOT:-}" ]]; then
  project_root="$PROJECT_ROOT"
else
  project_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
fi
agents_home="${AGENTS_HOME:-$HOME/.agents}"

project_memory="$project_root/.agents/memory"
global_memory="$agents_home/memory"
paths=()
[[ -d "$project_memory" ]] && paths+=("$project_memory")
[[ -d "$global_memory" ]] && paths+=("$global_memory")

if [[ ${#paths[@]} -eq 0 ]]; then
  printf 'Không tìm thấy thư mục memory. Đã kiểm tra:\n  %s\n  %s\n' "$project_memory" "$global_memory" >&2
  exit 1
fi

printf 'Truy vấn: %s\n' "$query"
printf 'Thứ tự phạm vi: project → global%s\n\n' "$([[ "$include_archive" == true ]] && printf ' (bao gồm archive)' || printf ' (chỉ corpus hiện hành)')"

if command -v rg >/dev/null 2>&1; then
  args=(--line-number --heading --color never --smart-case --glob '*.md')
  if [[ "$include_archive" != true ]]; then
    args+=(--glob '!**/archive/**')
  fi
  # Chạy riêng từng đường dẫn để giữ thứ tự project trước global trong output.
  found=false
  for path in "${paths[@]}"; do
    if rg "${args[@]}" -- "$query" "$path"; then
      found=true
      printf '\n'
    fi
  done
  if [[ "$found" != true ]]; then
    printf 'Không có kết quả. Thử tìm bằng triệu chứng, lỗi chính xác, subsystem/file hoặc từ đồng nghĩa mô tả nguyên nhân.\n'
    exit 0
  fi
else
  printf 'Khuyến nghị cài ripgrep (rg) để hỗ trợ smart case và lọc archive. Đang dùng grep thay thế.\n' >&2
  grep_args=(-RniE --include='*.md')
  for path in "${paths[@]}"; do
    if [[ "$include_archive" != true ]]; then
      # Loại thư mục archive khỏi lượt tìm mặc định đầu tiên.
      find "$path" -type d -name archive -prune -o -type f -name '*.md' -print0 |
        xargs -0 -r grep -niE -- "$query" || true
    else
      grep "${grep_args[@]}" -- "$query" "$path" || true
    fi
  done
fi
