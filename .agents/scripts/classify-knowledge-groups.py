#!/usr/bin/env python3
"""Assign knowledge_group + durable_class + disposition to every repo inventory row."""

from __future__ import annotations

import json
import re
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INVENTORY = ROOT / ".agents/memory/KNOWLEDGE_SOURCE_INVENTORY.json"
OUT_JSON = ROOT / ".agents/memory/KNOWLEDGE_GROUP_RECONCILIATION.json"
OUT_MD = ROOT / "docs/plans/active/agents-memory-framework/GROUP_RECONCILIATION.md"

LIFECYCLE_CORE = {"PLAN.md", "FINDINGS.md", "VERIFICATION.md", "AGENT_EVIDENCE.md"}


def classify_path(rel: str, zone: str, status: str) -> tuple[str, str, str]:
    """Return (knowledge_group, durable_class, disposition)."""
    p = Path(rel)
    name = p.name

    if status == "skipped_template":
        return "plans_template", "structural", "excluded_template"

    if zone == "repo_canonical":
        return "repo_canonical", "durable_high", "canonical_in_place"

    if zone == "repo_config":
        return "repo_config", "durable_high", "canonical_in_place"

    if zone == "agents_memory":
        if any(x in name for x in ("KNOWLEDGE", "MIGRATION", "MANIFEST", "RECONCILIATION")):
            return "agents_memory_meta", "structural", "inventory_artifact"
        if "/entries/" in rel and "/migrated/" not in rel and name.endswith(".md"):
            if name == "README.md":
                return "agents_memory_curated_readme", "durable_medium", "canonical_in_place"
            return "agents_memory_curated", "durable_high", "canonical_in_place"
        return "agents_memory_meta", "structural", "inventory_artifact"

    if zone == "skills_dot" or zone == "cursor":
        return zone, "durable_medium", status.replace("_", "-")

    if zone == "agents_other":
        if "/skills/" in rel:
            return "agents_skills_vendored", "durable_medium", status.replace("_", "-")
        if "/rules/" in rel or "/template/" in rel:
            return "agents_rules_template", "durable_high", status.replace("_", "-")
        return "agents_other", "durable_medium", status.replace("_", "-")

    if zone == "docs_plans":
        parts = p.parts
        if "_template" in parts:
            return "plans_template", "structural", "excluded_template"
        if "_audit-2026-10-13" in parts:
            return "plans_audit_slice", "durable_high", _disp(status)
        if len(parts) == 2:  # docs/plans/FOO.md
            return "plans_root_index", "durable_high", _disp(status)
        if "release-qualification-matrix/runs/" in rel:
            if name.endswith(".json"):
                return "plans_rqm_run_artifacts", "ephemeral", _disp(status)
            return "plans_rqm_run_docs", "durable_medium", _disp(status)
        if name in LIFECYCLE_CORE:
            return "plans_lifecycle_core", "durable_high", _dual(status)
        if name == "CHECKLIST.md":
            return "plans_checklist", "durable_medium", _disp(status)
        if name in {"requirements.md", "context.md", "tasks.md", "dod.md", "DESIGN.md", "API.md"}:
            return "plans_supplement", "durable_medium", _disp(status)
        if name.endswith(".json"):
            return "plans_json_sidecar", "ephemeral", _disp(status)
        if "evidence/" in rel or "screenshots/" in rel or "captures/" in rel:
            return "plans_evidence_path", "durable_low", _disp(status)
        return "plans_other_md", "durable_medium", _disp(status)

    if zone == "docs_other":
        if rel.startswith("docs/release/"):
            return "docs_release", "durable_high", _disp(status)
        if re.match(r"docs/\d+", rel):
            return "docs_architecture_numbered", "durable_high", _disp(status)
        return "docs_other", "durable_medium", _disp(status)

    return "unclassified", "unknown", status


def _disp(status: str) -> str:
    if status in ("mirrored_full_verified", "mirrored_full"):
        return "content_verified_full_mirror"
    if status in ("mirrored_truncated_verified", "mirror_truncated_not_allowed"):
        return "content_incomplete_truncated_mirror"
    if status in ("mirror_present_unverified", "mirrored_full_unverified", "mirrored_truncated_unverified"):
        return "mirror_present_unverified"
    if status == "mirrored_excerpt_in_plan_unit":
        return "migrated_excerpt_plan_unit"
    if status == "mirrored_excerpt_plan_unit_verified":
        return "content_verified_excerpt_plan_unit"
    if status == "pending":
        return "pending"
    if status in ("excluded_approved", "canonical_in_place", "inventory_artifact", "skipped_template"):
        return status.replace("_", "-")
    return status.replace("_", "-")


def _dual(status: str) -> str:
    if status == "mirrored_excerpt_in_plan_unit":
        return "dual_coverage_excerpt_and_full_mirror"
    return _disp(status)


def filesystem_gaps(inv_paths: set[str]) -> list[str]:
    missing = []
    for p in (ROOT / "docs").rglob("*"):
        if not p.is_file() or p.suffix.lower() not in {".md", ".json", ".txt", ".csv"}:
            continue
        if "/_archive/" in p.as_posix():
            continue
        rel = p.relative_to(ROOT).as_posix()
        if rel not in inv_paths:
            missing.append(rel)
    return missing


def main() -> None:
    inv = json.loads(INVENTORY.read_text(encoding="utf-8"))
    repo = inv["repo_sources"]
    inv_paths = {r["path"] for r in repo}
    gaps = filesystem_gaps(inv_paths)

    groups: dict[str, list] = defaultdict(list)
    for row in repo:
        g, durable, disp = classify_path(row["path"], row["zone"], row["migration_status"])
        row["knowledge_group"] = g
        row["durable_class"] = durable
        row["disposition"] = disp
        groups[g].append(row)

    group_summary = []
    for g, rows in sorted(groups.items()):
        st = Counter(r["disposition"] for r in rows)
        dur = Counter(r["durable_class"] for r in rows)
        group_summary.append(
            {
                "knowledge_group": g,
                "count": len(rows),
                "durable_class": dict(dur),
                "disposition": dict(st),
                "sample_paths": [r["path"] for r in rows[:3]],
            }
        )

    pending = sum(1 for r in repo if r.get("disposition") == "pending")
    unclassified = sum(1 for r in repo if r.get("knowledge_group") == "unclassified")
    v2 = inv.get("summary", {}).get("reconciliation_v2") or {}
    scope_ok = v2.get("scope_complete", False)
    docs_plans = sum(1 for r in repo if r["zone"] == "docs_plans")
    docs_other = sum(1 for r in repo if r["zone"] == "docs_other")
    docs_tree = len(
        [
            p
            for p in (ROOT / "docs").rglob("*")
            if p.is_file()
            and p.suffix.lower() in {".md", ".json", ".txt", ".csv"}
            and "/_archive/" not in p.as_posix()
        ]
    )

    v2_counts = v2.get("by_status") or {}
    truncated_paths = [
        r["path"]
        for r in repo
        if r.get("migration_status")
        in ("mirrored_truncated_verified", "mirror_truncated_not_allowed", "mirror_hash_mismatch")
        and r.get("truncated")
    ]

    recon = {
        "schema_version": 2,
        "classified_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "content_verification": {
            "artifact": ".agents/memory/KNOWLEDGE_RECONCILIATION_V2.json",
            "algorithm": (
                "Per source: sha256(raw bytes) == frontmatter source_sha256; "
                "sha256(mirror body after ---) == encode_mirror_body(source); "
                "full-byte mirror (no cap); truncated legacy mirrors fail scope."
            ),
            "by_status": v2_counts,
            "unresolved_count": v2.get("unresolved_count"),
            "truncated_verified_paths": truncated_paths,
            "scope_complete": scope_ok,
        },
        "filesystem": {
            "docs_tree_text_files": docs_tree,
            "inventory_docs_plans_rows": docs_plans,
            "inventory_docs_other_rows": docs_other,
            "missing_from_inventory": gaps,
        },
        "legacy_figures_resolved": {
            "note": "327 plans .md 'out of v1' → covered by v2 by-source-file + group plans_checklist/supplement/other",
            "v1_four_file_only": 605,
            "plans_zone_total": docs_plans,
        },
        "groups": group_summary,
        "repo_pending_disposition": pending,
        "unclassified_count": unclassified,
        "filesystem_scope_complete": len(gaps) == 0 and unclassified == 0,
        "content_scope_complete": scope_ok,
        "scope_complete": scope_ok and len(gaps) == 0 and unclassified == 0,
    }

    inv["repo_sources"] = repo
    inv["group_reconciliation"] = recon
    inv["summary"]["group_scope_complete"] = recon["scope_complete"]
    inv["summary"]["filesystem_gaps"] = len(gaps)
    INVENTORY.write_text(json.dumps(inv, indent=2) + "\n", encoding="utf-8")
    OUT_JSON.write_text(json.dumps(recon, indent=2) + "\n", encoding="utf-8")

    lines = [
        "# Đối soát nhóm nguồn kiến thức (repo)",
        "",
        f"**Cập nhật:** {recon['classified_at_utc'][:10]}",
        f"**`filesystem_scope_complete`:** `{recon['filesystem_scope_complete']}`",
        f"**`content_scope_complete` (v2 hash):** `{recon['content_scope_complete']}`",
        f"**`scope_complete` (cả hai):** `{recon['scope_complete']}`",
        "",
        "## Đối soát nội dung mirror (v2 — bắt buộc cho bảo toàn byte)",
        "",
        "Không dùng `mirror_present` / `migrated_full_mirror` (v1) làm bằng chứng.",
        "",
        f"| Chỉ số | Giá trị |",
        f"|---|---:|",
        f"| `mirrored_full_verified` | {v2_counts.get('mirrored_full_verified', 0)} |",
        f"| `mirrored_truncated_verified` | {v2_counts.get('mirrored_truncated_verified', 0)} |",
        f"| `excluded_approved` + artifact | {v2_counts.get('excluded_approved', 0) + v2_counts.get('inventory_artifact', 0)} |",
        f"| `unresolved_count` | {v2.get('unresolved_count', '—')} |",
        "",
        f"Artifact: `.agents/memory/KNOWLEDGE_RECONCILIATION_V2.json` — lệnh: `python3 .agents/scripts/reconcile-knowledge-sources-v2.py`.",
        "",
    ]
    if truncated_paths:
        lines.append("**Chưa mirror đủ byte (cần re-migrate hoặc exclusion):**")
        for tp in truncated_paths:
            lines.append(f"- `{tp}`")
        lines.append("")

    lines += [
        "## Filesystem vs inventory",
        "",
        f"| Chỉ số | Giá trị |",
        f"|---|---:|",
        f"| File text trong `docs/` (không `_archive`) | {docs_tree} |",
        f"| Dòng inventory `docs_plans` | {docs_plans} |",
        f"| Dòng inventory `docs_other` | {docs_other} |",
        f"| Thiếu trong inventory | {len(gaps)} |",
        "",
        "**Giải thích số cũ ~327 / ~1141:** v1 chỉ mirror 4 tên/plan → phần còn lại **không** ‘out of scope’ mà đã vào **v2 `by-source-file`** (`content_verified_full_mirror` sau `reconcile-knowledge-sources-v2.py`). Toàn `docs/` ngoài `_archive` được quét vào inventory (plans + docs_other + …).",
        "",
    ]
    if gaps:
        lines.append("### File thiếu (phải bổ sung inventory + migrate)\n")
        for g in gaps:
            lines.append(f"- `{g}`")
        lines.append("")

    lines += ["## Bảng nhóm", "", "| Nhóm | Số | durable (chính) | disposition (chính) |", "|---|---:|---|---|"]
    for gs in group_summary:
        d0 = max(gs["durable_class"], key=gs["durable_class"].get)
        s0 = max(gs["disposition"], key=gs["disposition"].get)
        lines.append(
            f"| `{gs['knowledge_group']}` | {gs['count']} | {d0} | {s0} |"
        )
    lines += [
        "",
        "Chi tiết JSON: `.agents/memory/KNOWLEDGE_GROUP_RECONCILIATION.json`",
        "",
        "## Durable ưu tiên (không mirror mù)",
        "",
        "- `durable_high`: lifecycle core, plans root, architecture numbered, release docs, rules — đọc trực tiếp hoặc dual excerpt+full.",
        "- `durable_medium`: CHECKLIST, supplement, vendored skills.",
        "- `ephemeral`: RQM `runs/**` JSON — mirror để search; không promote `active` hàng loạt.",
        "- `excluded_template`: `_template` only (5 files).",
        "",
    ]
    OUT_MD.write_text("\n".join(lines), encoding="utf-8")
    print(json.dumps({"scope_complete": recon["scope_complete"], "gaps": len(gaps), "groups": len(group_summary)}, indent=2))


if __name__ == "__main__":
    main()
