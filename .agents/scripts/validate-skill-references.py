#!/usr/bin/env python3
"""Ensure SKILL.md references to .agents/template/* resolve to existing files."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SKILLS_DIR = ROOT / ".agents/skills"
REF_RE = re.compile(r"\.agents/template/[a-zA-Z0-9_./-]+")


def template_refs(text: str) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for m in REF_RE.finditer(text):
        ref = m.group(0).rstrip("`).,;")
        if ref not in seen:
            seen.add(ref)
            out.append(ref)
    return out


def main() -> int:
    if not SKILLS_DIR.is_dir():
        print(f"Lỗi: không tìm thấy {SKILLS_DIR}", file=sys.stderr)
        return 1
    errors: list[str] = []
    checked = 0
    refs_total = 0
    for skill in sorted(SKILLS_DIR.glob("*/SKILL.md")):
        checked += 1
        for ref in template_refs(skill.read_text(encoding="utf-8", errors="replace")):
            refs_total += 1
            if not (ROOT / ref).is_file():
                errors.append(f"{skill.relative_to(ROOT)} → thiếu file: {ref}")
    if errors:
        for e in errors:
            print(f"FAIL {e}", file=sys.stderr)
        print(f"Kiểm tra thất bại: {len(errors)} lỗi ({refs_total} tham chiếu, {checked} skill).", file=sys.stderr)
        return 1
    print(f"OK: {refs_total} tham chiếu template trong {checked} skill(s) — tất cả tồn tại.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
