#!/usr/bin/env python3
"""Mirror pending repo knowledge files (full text) per KNOWLEDGE_SOURCE_INVENTORY."""

from __future__ import annotations

import json
import sys
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INVENTORY = ROOT / ".agents/memory/KNOWLEDGE_SOURCE_INVENTORY.json"
OUT = ROOT / ".agents/memory/entries/migrated/by-source-file"

sys.path.insert(0, str(Path(__file__).resolve().parent))
from knowledge_mirror_utils import (  # noqa: E402
    CANONICAL_IN_PLACE,
    encode_mirror_body,
    is_pipeline_memory_artifact,
    mirror_filename,
    sha256_text,
    verify_v2_mirror,
)


def is_excluded_path(rel: str) -> bool:
    if rel in CANONICAL_IN_PLACE:
        return True
    if is_pipeline_memory_artifact(rel):
        return True
    if "_template" in Path(rel).parts:
        return True
    if rel.startswith(".agents/memory/") and "/migrated/" in rel:
        return True
    return False


def write_mirror(rel: str, sha: str, content: str, truncated: bool, body_sha: str) -> Path:
    OUT.mkdir(parents=True, exist_ok=True)
    fname = mirror_filename(rel)
    path = OUT / fname
    today = date.today().isoformat()
    fm = f"""---
schema_version: 1
id: "MEM-SRC-{sha[:16]}"
title: "Source mirror: {rel}"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "{rel}"
source_sha256: "{sha}"
mirror_body_sha256: "{body_sha}"
truncated: {str(truncated).lower()}
created: "{today}"
updated: "{today}"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---

"""
    path.write_text(fm.rstrip() + "\n" + content, encoding="utf-8")
    return path


def main() -> None:
    inv = json.loads(INVENTORY.read_text(encoding="utf-8"))
    migrated = 0
    skipped = 0
    for row in inv.get("repo_sources", []):
        if row["migration_status"] not in (
            "pending",
            "pending_artifact_json",
            "mirror_present_unverified",
            "mirrored_full",
            "mirrored_full_unverified",
            "mirrored_truncated_unverified",
            "mirrored_truncated_verified",
            "mirrored_full_verified",
            "mirror_hash_mismatch",
            "mirror_truncated_not_allowed",
            "mirror_missing",
            "mirrored_excerpt_in_plan_unit",
            "mirrored_excerpt_in_plan_unit_verified",
            "excerpt_plan_unit_missing",
        ):
            continue
        if is_excluded_path(row["path"]):
            continue
        rel = row["path"]
        if row.get("migration_status") == "mirrored_full_verified":
            v2 = verify_v2_mirror(ROOT, rel)
            if v2.get("status") == "mirrored_full_verified":
                skipped += 1
                continue
        p = ROOT / rel
        if not p.is_file():
            row["migration_status"] = "not_exists"
            skipped += 1
            continue
        raw = p.read_bytes()
        text, truncated, sha = encode_mirror_body(rel, raw)
        body_sha = sha256_text(text)
        row["sha256_full"] = sha
        mp = write_mirror(rel, sha, text, truncated, body_sha)
        row["migration_status"] = (
            "mirrored_truncated_unverified" if truncated else "mirrored_full_unverified"
        )
        row["mirror_path"] = mp.relative_to(ROOT).as_posix()
        row["truncated"] = truncated
        row["verified"] = False
        migrated += 1
    rows = inv["repo_sources"] + inv.get("user_home_sources", [])
    by_status: dict[str, int] = {}
    for r in rows:
        by_status[r["migration_status"]] = by_status.get(r["migration_status"], 0) + 1
    inv["summary"] = {
        "total_rows": len(rows),
        "by_status": by_status,
        "repo_pending_count": None,
        "scope_complete": False,
        "note": "Run reconcile-knowledge-sources-v2.py for scope_complete",
        "last_migrate_v2": {"migrated": migrated, "skipped": skipped},
    }
    INVENTORY.write_text(json.dumps(inv, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(inv["summary"], indent=2))


if __name__ == "__main__":
    main()
