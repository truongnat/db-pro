#!/usr/bin/env bash
set -euo pipefail

skills_dir="${1:-.agents/skills}"
if [[ ! -d "$skills_dir" ]]; then
  echo "Lỗi: không tìm thấy thư mục skills: $skills_dir" >&2
  exit 1
fi

errors=0
count=0
while IFS= read -r -d '' file; do
  [[ "$(basename "$file")" == "README.md" ]] && continue
  dir="$(basename "$(dirname "$file")")"
  count=$((count + 1))
  first_line="$(head -n 1 "$file")"
  if [[ "$first_line" != "---" ]]; then
    echo "FAIL $file: thiếu YAML frontmatter mở đầu bằng ---"
    errors=$((errors + 1)); continue
  fi
  name="$(awk 'NR>1 && $0=="---" {exit} NR>1 && /^name:/ {sub(/^name:[[:space:]]*/, ""); print; exit}' "$file")"
  description="$(awk 'NR>1 && $0=="---" {exit} NR>1 && /^description:/ {sub(/^description:[[:space:]]*/, ""); print; exit}' "$file")"
  if [[ -z "$name" || "$name" != "$dir" ]]; then
    echo "FAIL $file: name='$name' phải trùng tên thư mục '$dir'"
    errors=$((errors + 1))
  fi
  if [[ -z "$description" ]]; then
    echo "FAIL $file: thiếu description"
    errors=$((errors + 1))
  fi
  if [[ "$name" =~ [^a-z0-9-] || "$name" == -* || "$name" == *- || "$name" == *--* ]]; then
    echo "FAIL $file: name không đúng kebab-case: '$name'"
    errors=$((errors + 1))
  fi
done < <(find "$skills_dir" -mindepth 2 -maxdepth 2 -name SKILL.md -print0 | sort -z)

if (( count == 0 )); then
  echo "Lỗi: không tìm thấy SKILL.md nào trong $skills_dir" >&2
  exit 1
fi
if (( errors > 0 )); then
  echo "Kiểm tra thất bại: $errors lỗi trên $count skill(s)."
  exit 1
fi
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 "$script_dir/validate-skill-references.py"
echo "OK: đã kiểm tra $count skill(s), frontmatter cơ bản hợp lệ."
