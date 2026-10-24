#!/usr/bin/env python3
"""Build master inventory of knowledge sources (repo + user agent home)."""

from __future__ import annotations

import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / ".agents/memory/KNOWLEDGE_SOURCE_INVENTORY.json"

sys.path.insert(0, str(Path(__file__).resolve().parent))
from knowledge_mirror_utils import (  # noqa: E402
    is_pipeline_memory_artifact,
    mirror_filename,
    verify_v2_mirror,
)
MIRROR_PLANS = ROOT / ".agents/memory/entries/migrated/from-plans"
MIRROR_BY_FILE = ROOT / ".agents/memory/entries/migrated/by-source-file"

TEXT_SUFFIXES = {".md", ".json", ".txt", ".csv"}
def sha256_file(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def should_skip(path: Path) -> bool:
    s = path.as_posix()
    for bad in ("/.git/", "/node_modules/", "/target/", "/target-build/", "/_archive/"):
        if bad in s:
            return True
    if "entries/migrated/from-plans" in s or "entries/migrated/by-source-file" in s:
        return True
    try:
        rel = path.relative_to(ROOT).as_posix()
    except ValueError:
        rel = path.as_posix()
    if is_pipeline_memory_artifact(rel):
        return True
    return False


def zone_for(rel: Path) -> str:
    s = rel.as_posix()
    if s.startswith("docs/plans/"):
        return "docs_plans"
    if s.startswith("docs/"):
        return "docs_other"
    if s.startswith(".agents/memory/"):
        return "agents_memory"
    if s.startswith(".agents/"):
        return "agents_other"
    if s.startswith(".skills/"):
        return "skills_dot"
    if s.startswith(".cursor/"):
        return "cursor"
    if s in {"AGENTS.md", "RULES.md", ".impeccable.md"}:
        return "repo_canonical"
    if s in {"skills-lock.json", "clean-code.config.json"}:
        return "repo_config"
    return "repo_other"


def migration_status(rel: Path, zone: str) -> tuple[str, str | None]:
    """Return (status, mirror_rel_or_none)."""
    p = ROOT / rel
    if not p.is_file():
        return "not_exists", None
    if p.suffix.lower() not in TEXT_SUFFIXES:
        return "skipped_non_text", None
    if "_template" in rel.parts and zone == "docs_plans":
        return "skipped_template", None
    mirror = MIRROR_BY_FILE / mirror_filename(rel.as_posix())
    if mirror.is_file():
        v2 = verify_v2_mirror(ROOT, rel.as_posix())
        if v2.get("status") == "mirrored_full_verified":
            return "mirrored_full_verified", v2.get("mirror_path")
        if v2.get("status") == "mirrored_truncated_verified":
            return "mirrored_truncated_verified", v2.get("mirror_path")
        if v2.get("status") == "mirror_hash_mismatch":
            return "mirror_hash_mismatch", v2.get("mirror_path")
        return "mirror_present_unverified", mirror.relative_to(ROOT).as_posix()
    # covered by v1 plan unit? (heuristic: only the 4 standard names)
    if zone == "docs_plans" and rel.name in (
        "PLAN.md",
        "FINDINGS.md",
        "VERIFICATION.md",
        "AGENT_EVIDENCE.md",
    ):
        return "mirrored_excerpt_in_plan_unit", None
    if zone == "docs_plans" and rel.parts[1:2] == ("_audit-2026-10-13",) and rel.suffix == ".md":
        if rel.name not in ("INDEX.md", "CAPTURE-EVIDENCE.md"):
            return "mirrored_excerpt_in_plan_unit", None
    if "runs/" in rel.as_posix() and "release-qualification-matrix" in rel.as_posix():
        if rel.suffix == ".json":
            return "pending_artifact_json", None
    return "pending", None


def scan_repo() -> list[dict]:
    rows = []
    for p in ROOT.rglob("*"):
        if not p.is_file() or should_skip(p):
            continue
        if p.suffix.lower() not in TEXT_SUFFIXES:
            continue
        try:
            rel = p.relative_to(ROOT)
        except ValueError:
            continue
        # limit to meaningful trees
        top = rel.parts[0] if rel.parts else ""
        if top not in {
            "docs",
            ".agents",
            ".skills",
            ".cursor",
        } and rel.as_posix() not in {
            "AGENTS.md",
            "RULES.md",
            ".impeccable.md",
            "skills-lock.json",
            "clean-code.config.json",
        }:
            continue
        zone = zone_for(rel)
        status, mirror = migration_status(rel, zone)
        rows.append(
            {
                "id": sha256_file(p)[:16],
                "path": rel.as_posix(),
                "zone": zone,
                "bytes": p.stat().st_size,
                "sha256_full": sha256_file(p),
                "migration_status": status,
                "mirror_path": mirror,
            }
        )
    return rows


def scan_user_home() -> list[dict]:
    home = Path.home()
    rows = []
    targets = [
        (home / ".agents/memory", "user_agents_memory"),
        (home / ".agents/skills", "user_agents_skills"),
        (home / ".agents/agents", "user_agents_agents"),
        (home / ".agents/scripts", "user_agents_scripts"),
        (home / ".cursor/skills", "user_cursor_skills"),
    ]
    for base, zone in targets:
        if not base.is_dir():
            rows.append(
                {
                    "id": f"missing-{zone}",
                    "path": str(base),
                    "zone": zone,
                    "bytes": 0,
                    "sha256_full": None,
                    "migration_status": "not_exists",
                    "mirror_path": None,
                }
            )
            continue
        for p in base.rglob("*"):
            if not p.is_file():
                continue
            if p.suffix.lower() not in TEXT_SUFFIXES and p.name != "SKILL.md":
                if p.suffix not in {".md", ".json", ".txt", ".sh"}:
                    continue
            status = "catalogued_user_home"
            if zone == "user_agents_memory" and p.read_text(encoding="utf-8", errors="replace").strip():
                body = p.read_text(encoding="utf-8", errors="replace")
                if "TODO" in body and len(body) < 2000:
                    status = "user_skeleton_only"
            rows.append(
                {
                    "id": sha256_file(p)[:16] if p.stat().st_size else f"empty-{p.name}",
                    "path": str(p),
                    "zone": zone,
                    "bytes": p.stat().st_size,
                    "sha256_full": sha256_file(p) if p.stat().st_size else None,
                    "migration_status": status,
                    "mirror_path": None,
                    "note": "User home — not copied into repo; inventory only unless db-pro-specific",
                }
            )
    return rows


def summarize(rows: list[dict]) -> dict:
    by_status = {}
    by_zone = {}
    for r in rows:
        by_status[r["migration_status"]] = by_status.get(r["migration_status"], 0) + 1
        by_zone[r["zone"]] = by_zone.get(r["zone"], 0) + 1
    repo = [r for r in rows if "user_home" not in r.get("zone", "") and not str(r.get("path", "")).startswith(str(Path.home()))]
    user = [r for r in rows if r not in repo]
    return {
        "total_rows": len(rows),
        "repo_source_count": len(repo),
        "user_home_catalog_count": len(user),
        "by_status": by_status,
        "by_zone": by_zone,
        "repo_pending_count": None,
        "scope_complete": False,
        "user_home_note": "User paths catalogued in inventory; content not copied into repo unless db-pro-specific entry added.",
        "note": "scope_complete only via reconcile-knowledge-sources-v2.py",
    }


def main() -> None:
    repo_rows = scan_repo()
    user_rows = scan_user_home()
    doc = {
        "schema_version": 1,
        "built_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "repo_root": str(ROOT),
        "user_home": str(Path.home()),
        "summary": summarize(repo_rows + user_rows),
        "repo_sources": repo_rows,
        "user_home_sources": user_rows,
        "completion_rule": (
            "scope_complete only when reconcile-knowledge-sources-v2.py reports true: "
            "mirrored_full_verified (full bytes), approved exclusions, or v1 excerpt verified. "
            "Truncated legacy mirrors fail scope."
        ),
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(doc["summary"], indent=2))


if __name__ == "__main__":
    main()
