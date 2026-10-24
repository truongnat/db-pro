#!/usr/bin/env python3
"""Migrate durable plan evidence from docs/plans into .agents/memory/entries/migrated/from-plans/."""

from __future__ import annotations

import hashlib
import json
import re
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PLANS = ROOT / "docs" / "plans"
OUT = ROOT / ".agents" / "memory" / "entries" / "migrated" / "from-plans"
MANIFEST = ROOT / ".agents" / "memory" / "MIGRATION_MANIFEST.json"
TODAY = date.today().isoformat()

SCAN_ROOTS = [
    PLANS / "active",
    PLANS / "completed",
    PLANS / "_audit-2026-10-13",
]

EVIDENCE_FILES = ("PLAN.md", "FINDINGS.md", "VERIFICATION.md", "AGENT_EVIDENCE.md")


def slugify(name: str) -> str:
    s = re.sub(r"[^a-zA-Z0-9]+", "-", name.lower()).strip("-")
    return s[:80] or "plan"


def classify(slug: str, rel_parent: str) -> str:
    s = slug.lower()
    if "_audit" in rel_parent or rel_parent.startswith("_audit"):
        return "pattern"
    if any(x in s for x in ("release", "qualification", "workflow", "agent-replay", "dispatch")):
        return "workflow"
    if any(x in s for x in ("fix", "hardening", "safety", "bug", "issue-")):
        return "bugfix"
    if any(
        x in s
        for x in (
            "theme",
            "component",
            "ui",
            "layout",
            "gallery",
            "visual",
            "token",
            "design",
            "focus",
            "disclosure",
        )
    ):
        return "pattern"
    return "decision"


def _digest_file(path: Path) -> tuple[str, int, str, bool]:
    text = path.read_text(encoding="utf-8", errors="replace")
    full_hash = hashlib.sha256(text.encode("utf-8")).hexdigest()
    nbytes = len(text.encode("utf-8"))
    if len(text) > 12000:
        excerpt = text[:12000] + "\n\n… [truncated for memory entry; see source file]\n"
        return excerpt, nbytes, full_hash, True
    return text, nbytes, full_hash, False


def pick_body(plan_dir: Path) -> tuple[str, list[str], list[dict]]:
    sources: list[str] = []
    parts: list[str] = []
    meta: list[dict] = []
    if plan_dir.is_file() and plan_dir.suffix == ".md":
        rel = plan_dir.relative_to(ROOT).as_posix()
        sources.append(rel)
        text, nbytes, full_hash, trunc = _digest_file(plan_dir)
        meta.append(
            {
                "path": rel,
                "bytes": nbytes,
                "sha256_full": full_hash,
                "truncated": trunc,
            }
        )
        parts.append(text)
        return "\n\n".join(parts), sources, meta
    for name in EVIDENCE_FILES:
        p = plan_dir / name
        if p.is_file():
            rel = p.relative_to(ROOT).as_posix()
            sources.append(rel)
            text, nbytes, full_hash, trunc = _digest_file(p)
            meta.append(
                {
                    "path": rel,
                    "bytes": nbytes,
                    "sha256_full": full_hash,
                    "truncated": trunc,
                }
            )
            parts.append(f"## Source: `{name}`\n\n{text}")
    return "\n\n".join(parts), sources, meta


def title_from_slug(slug: str) -> str:
    return slug.replace("-", " ").strip().title()[:120]


def write_entry(
    entry_id: str,
    slug: str,
    mtype: str,
    sources: list[str],
    body: str,
    note: str,
    excerpt_sha256: str,
) -> Path:
    OUT.mkdir(parents=True, exist_ok=True)
    fname = f"{TODAY}-plan-{slug}.md"
    path = OUT / fname
    title = f"Plan archive: {title_from_slug(slug)}"
    tags = ["migrated", "docs-plans", mtype, slug[:40]]
    fm = f"""---
schema_version: 1
id: "{entry_id}"
title: "{title}"
type: {mtype}
scope: project
status: needs-review
created: "{TODAY}"
updated: "{TODAY}"
verified_on: "unknown"
confidence: medium
tags: [{", ".join(tags)}]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "{excerpt_sha256}"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
{chr(10).join(f'  - "{s}"' for s in sources)}
related: []
supersedes: null
---

# Tóm tắt

{note}

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

{body}
"""
    path.write_text(fm, encoding="utf-8")
    return path


def migrate_plan_dir(plan_dir: Path, rel_base: str) -> dict | None:
    if not plan_dir.is_dir():
        return None
    slug = plan_dir.name
    if slug in ("_template", "evidence", "screenshots", "captures"):
        return None
    body, sources, source_files = pick_body(plan_dir)
    if not body.strip():
        return None
    mtype = classify(slug, rel_base)
    excerpt_sha = hashlib.sha256(body.encode()).hexdigest()
    h = excerpt_sha[:12]
    entry_id = f"MEM-PLAN-{slug[:32]}-{h}"
    note = f"Migrate từ `{plan_dir.relative_to(ROOT).as_posix()}/`. Loại suy luận: `{mtype}`."
    path = write_entry(entry_id, slug, mtype, sources, body, note, excerpt_sha)
    return {
        "slug": slug,
        "type": mtype,
        "path": path.relative_to(ROOT).as_posix(),
        "sources": sources,
        "source_files": source_files,
        "mirror_excerpt_sha256": excerpt_sha,
        "truncated_any": any(f["truncated"] for f in source_files),
    }


def migrate_audit_file(md: Path) -> dict | None:
    if md.name in ("INDEX.md", "CAPTURE-EVIDENCE.md") or md.suffix != ".md":
        return None
    slug = md.stem
    body, sources, source_files = pick_body(md)
    mtype = "pattern" if slug[0].isdigit() or slug.startswith("fix-") else "decision"
    excerpt_sha = hashlib.sha256(body.encode()).hexdigest()
    h = excerpt_sha[:12]
    entry_id = f"MEM-AUDIT-{slug[:24]}-{h}"
    note = f"Migrate từ audit slice `{md.relative_to(ROOT).as_posix()}`."
    path = write_entry(entry_id, f"audit-{slug}", mtype, sources, body, note, excerpt_sha)
    return {
        "slug": f"audit-{slug}",
        "type": mtype,
        "path": path.relative_to(ROOT).as_posix(),
        "sources": sources,
        "source_files": source_files,
        "mirror_excerpt_sha256": excerpt_sha,
        "truncated_any": any(f["truncated"] for f in source_files),
    }


def main() -> None:
    # Idempotent: replace prior auto-mirror only (not curated entries).
    if OUT.is_dir():
        for old in OUT.glob("*.md"):
            old.unlink()
    entries: list[dict] = []
    seen_slugs: set[str] = set()

    audit_dir = PLANS / "_audit-2026-10-13"
    if audit_dir.is_dir():
        for md in sorted(audit_dir.glob("*.md")):
            row = migrate_audit_file(md)
            if row and row["slug"] not in seen_slugs:
                seen_slugs.add(row["slug"])
                entries.append(row)

    for root in SCAN_ROOTS:
        if not root.is_dir():
            continue
        rel_base = root.relative_to(PLANS).as_posix()
        if root == audit_dir:
            continue
        for plan_dir in sorted(root.iterdir()):
            if not plan_dir.is_dir():
                continue
            row = migrate_plan_dir(plan_dir, rel_base)
            if row:
                key = f"{rel_base}/{row['slug']}"
                if key in seen_slugs:
                    continue
                seen_slugs.add(key)
                row["plan_root"] = rel_base
                entries.append(row)

    scope_path = ROOT / ".agents/memory/SOURCE_SCOPE_MANIFEST.json"
    scope_ref = scope_path.relative_to(ROOT).as_posix() if scope_path.is_file() else None
    manifest = {
        "schema_version": 1,
        "wave_id": "v1-plan-evidence-excerpt",
        "scope_manifest": scope_ref,
        "mirror_semantics": "excerpt",
        "max_excerpt_bytes_per_source_file": 12000,
        "migrated_at": TODAY,
        "tool": ".agents/scripts/migrate-plans-to-memory.py",
        "reconciliation_tool": ".agents/scripts/reconcile-memory-migration.py",
        "entry_count": len(entries),
        "entries": entries,
        "note": "Excerpt mirror only — run reconcile-memory-migration.py before claiming wave pass.",
    }
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"written": len(entries), "manifest": MANIFEST.relative_to(ROOT).as_posix()}))


if __name__ == "__main__":
    main()
