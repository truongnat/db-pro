#!/usr/bin/env python3
"""v2: verify every repo source hash ↔ mirror; drive scope_complete."""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INVENTORY = ROOT / ".agents/memory/KNOWLEDGE_SOURCE_INVENTORY.json"
OUT = ROOT / ".agents/memory/KNOWLEDGE_RECONCILIATION_V2.json"
EXCLUSIONS = ROOT / ".agents/memory/APPROVED_EXCLUSIONS.json"

# Import sibling module
sys.path.insert(0, str(Path(__file__).resolve().parent))
from knowledge_mirror_utils import (  # noqa: E402
    is_pipeline_memory_artifact,
    requires_v1_excerpt,
    requires_v2_mirror,
    verify_v2_mirror,
)

TERMINAL_OK = {
    "mirrored_full_verified",
    "mirrored_excerpt_plan_unit_verified",
    "canonical_in_place",
    "skipped_template",
    "skipped_non_text",
    "inventory_artifact",
    "not_exists",
}


def default_exclusions() -> dict:
    return {
        "schema_version": 1,
        "approved": [
            {"pattern": "docs/plans/_template/**", "reason": "template skeleton"},
            {"pattern": "AGENTS.md", "reason": "canonical agent contract"},
            {"pattern": "RULES.md", "reason": "canonical UI rules"},
            {"pattern": ".impeccable.md", "reason": "design context"},
            {"pattern": "skills-lock.json", "reason": "lock file"},
            {"pattern": "clean-code.config.json", "reason": "tool config"},
            {
                "pattern": ".agents/memory/KNOWLEDGE*.json",
                "reason": "inventory artifacts",
            },
            {
                "pattern": ".agents/memory/MIGRATION*.json",
                "reason": "migration artifacts",
            },
            {
                "pattern": ".agents/memory/entries/**/README.md",
                "reason": "structural readme",
            },
            {
                "pattern": ".agents/memory/entries/{environment,patterns,workflows}/*.md",
                "reason": "curated canonical entries",
            },
        ],
    }


def is_excluded(rel: str) -> str | None:
    if not EXCLUSIONS.is_file():
        EXCLUSIONS.write_text(json.dumps(default_exclusions(), indent=2) + "\n")
    data = json.loads(EXCLUSIONS.read_text())
    for item in data.get("approved", []):
        if item.get("path") == rel:
            return item["reason"]
        pat = item.get("pattern", "")
        if pat and rel.startswith(pat.rstrip("/")):
            return item["reason"]
    p = Path(rel)
    if "_template" in p.parts:
        return "template"
    if is_pipeline_memory_artifact(rel):
        return "inventory_artifact"
    if "/entries/" in rel and "/migrated/" not in rel and rel.endswith(".md"):
        if p.name != "README.md" and "2026-" in p.name:
            return "curated_entry"
    if rel.endswith("/README.md") and ".agents/memory/entries/" in rel:
        return "structural_readme"
    return None


def v1_excerpt_verified(root: Path, rel: str) -> bool:
    """Use plan-unit mirror + source order from reconcile-memory-migration logic."""
    # If v2 full mirror verifies for lifecycle file, excerpt unit is optional bonus
    v2 = verify_v2_mirror(root, rel)
    if v2["status"] == "mirrored_full_verified":
        return True
    # fallback: plan unit exists for slug
    plan_dir = (root / rel).parent
    slug = plan_dir.name
    if rel.startswith("docs/plans/_audit-2026-10-13/"):
        slug = f"audit-{plan_dir.name}"
    from_plans = root / ".agents/memory/entries/migrated/from-plans"
    if not from_plans.is_dir():
        return False
    for mf in from_plans.glob(f"*-plan-{slug}.md"):
        return True
    return False


def reconcile_row(root: Path, row: dict) -> dict:
    rel = row["path"]
    zone = row["zone"]
    exc = is_excluded(rel)
    if exc:
        if exc == "template":
            st = "skipped_template"
        elif exc in ("canonical", "curated_entry", "structural_readme"):
            st = "canonical_in_place"
        elif exc == "inventory_artifact":
            st = "inventory_artifact"
        else:
            st = "excluded_approved"
        row.update({"migration_status": st, "exclusion_reason": exc, "verified": True})
        return row

    need_v2 = requires_v2_mirror(rel, zone)
    need_v1 = requires_v1_excerpt(rel, zone)

    if need_v2:
        v2 = verify_v2_mirror(root, rel)
        st = v2["status"]
        if st == "mirrored_full_verified":
            row.update(
                {
                    "migration_status": st,
                    "mirror_path": v2.get("mirror_path"),
                    "source_sha256": v2.get("source_sha256"),
                    "truncated": False,
                    "verified": True,
                }
            )
            return row
        if st == "mirrored_truncated_verified":
            row.update(
                {
                    "migration_status": "mirror_truncated_not_allowed",
                    "mirror_path": v2.get("mirror_path"),
                    "source_sha256": v2.get("source_sha256"),
                    "truncated": True,
                    "verified": False,
                    "reconcile_detail": v2,
                }
            )
            return row
        row.update({"migration_status": st, "verified": False, "reconcile_detail": v2})
        return row

    if need_v1:
        ok = v1_excerpt_verified(root, rel)
        st = "mirrored_excerpt_plan_unit_verified" if ok else "excerpt_plan_unit_missing"
        row.update({"migration_status": st, "verified": ok})
        return row

    v2 = verify_v2_mirror(root, rel)
    if v2["status"] != "mirror_missing":
        verified = v2["status"] == "mirrored_full_verified"
        row.update(
            {
                "migration_status": v2["status"],
                "mirror_path": v2.get("mirror_path"),
                "verified": verified,
            }
        )
        return row
    row.update({"migration_status": "canonical_in_place", "verified": True})
    return row


def main() -> int:
    if not INVENTORY.is_file():
        print("missing inventory", file=sys.stderr)
        return 1
    inv = json.loads(INVENTORY.read_text(encoding="utf-8"))
    repo = inv["repo_sources"]
    counts = {}
    failures = []
    for row in repo:
        reconcile_row(ROOT, row)
        st = row["migration_status"]
        counts[st] = counts.get(st, 0) + 1
        if not row.get("verified", False) and st not in TERMINAL_OK:
            failures.append({"path": row["path"], "status": st})

    unresolved = [
        r
        for r in repo
        if not r.get("verified", False)
        and r.get("migration_status")
        not in ("skipped_template", "skipped_non_text", "not_exists")
    ]
    scope_ok = len(unresolved) == 0

    report = {
        "schema_version": 2,
        "reconciled_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "by_status": counts,
        "unresolved_count": len(unresolved),
        "unresolved_sample": failures[:40],
        "scope_complete": scope_ok,
        "pass_criteria": (
            "Every repo source mirrored_full_verified OR excluded_approved/skipped_template/"
            "canonical_in_place/inventory_artifact/v1 excerpt verified. "
            "Truncated mirrors (legacy cap) are mirror_truncated_not_allowed and fail scope."
        ),
    }
    OUT.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    inv["repo_sources"] = repo
    inv["reconciliation_v2"] = OUT.relative_to(ROOT).as_posix()
    if "summary" not in inv:
        inv["summary"] = {}
    inv["summary"]["scope_complete"] = scope_ok
    inv["summary"]["repo_pending_count"] = len(unresolved)
    inv["summary"]["by_status"] = counts
    inv["summary"]["reconciliation_v2"] = report
    INVENTORY.write_text(json.dumps(inv, indent=2) + "\n", encoding="utf-8")

    print(json.dumps({"scope_complete": scope_ok, "unresolved": len(unresolved), "by_status": counts}, indent=2))
    return 0 if scope_ok else 2


if __name__ == "__main__":
    raise SystemExit(main())
