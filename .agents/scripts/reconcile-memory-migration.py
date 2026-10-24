#!/usr/bin/env python3
"""Inventory in-scope sources, reconcile hashes/truncation vs mirror entries."""

from __future__ import annotations

import hashlib
import json
import re
from datetime import date, datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCOPE_PATH = ROOT / ".agents/memory/SOURCE_SCOPE_MANIFEST.json"
MANIFEST_PATH = ROOT / ".agents/memory/MIGRATION_MANIFEST.json"
RECON_PATH = ROOT / ".agents/memory/MIGRATION_RECONCILIATION.json"
MIRROR_DIR = ROOT / ".agents/memory/entries/migrated/from-plans"
PLANS = ROOT / "docs/plans"

EVIDENCE = ("PLAN.md", "FINDINGS.md", "VERIFICATION.md", "AGENT_EVIDENCE.md")
AUDIT_EXCLUDE = {"INDEX.md", "CAPTURE-EVIDENCE.md"}
MAX_EXCERPT = 12000


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> tuple[str, int]:
    data = path.read_bytes()
    return sha256_bytes(data), len(data)


def excerpt(text: str) -> tuple[str, bool]:
    if len(text) <= MAX_EXCERPT:
        return text, False
    marker = "\n\n… [truncated for memory entry; see source file]\n"
    return text[:MAX_EXCERPT] + marker, True


def load_scope() -> dict:
    return json.loads(SCOPE_PATH.read_text(encoding="utf-8"))


def inventory_plan_roots() -> list[Path]:
    roots = []
    for rel in ("active", "completed"):
        r = PLANS / rel
        if r.is_dir():
            roots.extend(sorted(p for p in r.iterdir() if p.is_dir()))
    return roots


def inventory_audit_files() -> list[Path]:
    ad = PLANS / "_audit-2026-10-13"
    if not ad.is_dir():
        return []
    return sorted(
        p for p in ad.glob("*.md") if p.name not in AUDIT_EXCLUDE
    )


def scan_plan_dir(plan_dir: Path) -> dict:
    all_md = {f.name: f for f in plan_dir.glob("*.md")}
    included = []
    excluded = []
    for name in EVIDENCE:
        f = all_md.get(name)
        if not f:
            continue
        if f.name in EVIDENCE:
            digest, nbytes = sha256_file(f)
            text = f.read_text(encoding="utf-8", errors="replace")
            ex, trunc = excerpt(text)
            included.append(
                {
                    "path": f.relative_to(ROOT).as_posix(),
                    "bytes": nbytes,
                    "sha256_full": digest,
                    "sha256_excerpt": sha256_bytes(ex.encode("utf-8")),
                    "truncated": trunc,
                    "included_in_mirror": True,
                }
            )
    for name, f in sorted(all_md.items()):
        if name in EVIDENCE:
            continue
        excluded.append(
            {
                "path": f.relative_to(ROOT).as_posix(),
                "reason": "out_of_scope_v1_filename",
            }
        )
    subdirs = [d.name for d in plan_dir.iterdir() if d.is_dir()]
    return {
        "plan_dir": plan_dir.relative_to(ROOT).as_posix(),
        "slug": plan_dir.name,
        "included_sources": included,
        "excluded_md_at_root": excluded,
        "subdirs_present": subdirs,
        "migratable": len(included) > 0,
    }


def scan_audit_file(md: Path) -> dict:
    digest, nbytes = sha256_file(md)
    text = md.read_text(encoding="utf-8", errors="replace")
    ex, trunc = excerpt(text)
    return {
        "plan_dir": md.parent.relative_to(ROOT).as_posix(),
        "slug": f"audit-{md.stem}",
        "included_sources": [
            {
                "path": md.relative_to(ROOT).as_posix(),
                "bytes": nbytes,
                "sha256_full": digest,
                "sha256_excerpt": sha256_bytes(ex.encode("utf-8")),
                "truncated": trunc,
                "included_in_mirror": True,
            }
        ],
        "excluded_md_at_root": [],
        "subdirs_present": [],
        "migratable": True,
    }


def parse_mirror_sources(entry_path: Path) -> list[str]:
    text = entry_path.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"(?ms)^sources:\n((?:  - \"[^\"]+\"\n)+)", text)
    if not m:
        return []
    return re.findall(r'  - "([^"]+)"', m.group(1))


def excerpt_body_for_unit(included: list[dict], is_audit_file: bool) -> str:
    by_path = {s["path"]: s for s in included}
    ordered = [by_path[p.relative_to(ROOT).as_posix()] for p in _ordered_evidence_paths(included)]
    parts = []
    for s in ordered:
        p = ROOT / s["path"]
        text = p.read_text(encoding="utf-8", errors="replace")
        ex, _ = excerpt(text)
        if is_audit_file and len(ordered) == 1:
            parts.append(ex)
        else:
            parts.append(f"## Source: `{p.name}`\n\n{ex}")
    return "\n\n".join(parts)


def _ordered_evidence_paths(included: list[dict]) -> list[Path]:
    paths = [ROOT / s["path"] for s in included]
    if len(paths) == 1:
        return paths
    order = {name: i for i, name in enumerate(EVIDENCE)}
    return sorted(paths, key=lambda p: order.get(p.name, 99))


def parse_mirror_excerpt_sha256(entry_path: Path) -> str | None:
    text = entry_path.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"^mirror_excerpt_sha256:\s*\"([a-f0-9]{64})\"", text, re.M)
    return m.group(1) if m else None


def main() -> None:
    scope = load_scope()
    plan_scans = [scan_plan_dir(d) for d in inventory_plan_roots()]
    audit_scans = [scan_audit_file(p) for p in inventory_audit_files()]
    all_scans = plan_scans + audit_scans

    migratable = [s for s in all_scans if s["migratable"]]
    empty = [s for s in all_scans if not s["migratable"]]

    all_included_files: list[dict] = []
    for s in all_scans:
        all_included_files.extend(s["included_sources"])

    truncated = [f for f in all_included_files if f["truncated"]]
    total_source_bytes = sum(f["bytes"] for f in all_included_files)

    mirror_files = sorted(MIRROR_DIR.glob("*.md")) if MIRROR_DIR.is_dir() else []
    manifest_entries = []
    if MANIFEST_PATH.is_file():
        manifest_entries = json.loads(MANIFEST_PATH.read_text()).get("entries", [])

    mirror_by_slug = {}
    for mf in mirror_files:
        # filename: YYYY-MM-DD-plan-<slug>.md
        stem = mf.stem
        if "-plan-" in stem:
            slug = stem.split("-plan-", 1)[1]
            mirror_by_slug[slug] = mf

    slug_mismatches = []
    for s in migratable:
        slug = s["slug"]
        if slug not in mirror_by_slug:
            slug_mismatches.append({"slug": slug, "issue": "missing_mirror_file"})

    manifest_by_slug = {e.get("slug"): e for e in manifest_entries}
    hash_mismatches = []
    for s in migratable:
        slug = s["slug"]
        mf = mirror_by_slug.get(slug)
        if not mf:
            continue
        is_audit = slug.startswith("audit-")
        expected_excerpt = excerpt_body_for_unit(s["included_sources"], is_audit)
        expected_sha = sha256_bytes(expected_excerpt.encode("utf-8"))
        file_sha = parse_mirror_excerpt_sha256(mf)
        man = manifest_by_slug.get(slug) or {}
        man_sha = man.get("mirror_excerpt_sha256")
        if expected_sha != file_sha or (man_sha and man_sha != expected_sha):
            hash_mismatches.append(
                {
                    "slug": slug,
                    "mirror": mf.relative_to(ROOT).as_posix(),
                    "expected": expected_sha,
                    "file": file_sha,
                    "manifest": man_sha,
                }
            )

    # Repo-wide context (not migrated)
    plans_md_total = len(list(PLANS.rglob("*.md")))
    docs_md_total = len(list((ROOT / "docs").rglob("*.md"))) if (ROOT / "docs").is_dir() else 0

    recon = {
        "schema_version": 1,
        "wave_id": scope.get("wave_id"),
        "reconciled_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "scope_manifest": SCOPE_PATH.relative_to(ROOT).as_posix(),
        "summary": {
            "plan_dirs_scanned": len(plan_scans),
            "audit_files_scanned": len(audit_scans),
            "migratable_units": len(migratable),
            "empty_plan_dirs_no_v1_files": len(empty),
            "included_source_files": len(all_included_files),
            "truncated_source_files": len(truncated),
            "truncated_bytes_lost_estimate": sum(
                max(0, f["bytes"] - MAX_EXCERPT) for f in truncated
            ),
            "total_included_source_bytes": total_source_bytes,
            "mirror_files_on_disk": len(mirror_files),
            "manifest_entry_count": len(manifest_entries),
            "docs_plans_md_total": plans_md_total,
            "docs_md_total": docs_md_total,
            "plans_md_not_in_v1_scope_estimate": plans_md_total
            - len(all_included_files),
        },
        "pass_v1": {
            "counts_match": len(mirror_files) == len(migratable)
            and len(manifest_entries) == len(migratable),
            "no_missing_mirror": len(slug_mismatches) == 0,
            "no_body_drift": len(hash_mismatches) == 0,
            "overall": (
                len(mirror_files) == len(migratable)
                and len(manifest_entries) == len(migratable)
                and len(slug_mismatches) == 0
                and len(hash_mismatches) == 0
            ),
        },
        "truncated_files": [
            {"path": f["path"], "bytes": f["bytes"], "sha256_full": f["sha256_full"]}
            for f in truncated
        ],
        "empty_plan_dirs_sample": [e["plan_dir"] for e in empty[:30]],
        "empty_plan_dirs_count": len(empty),
        "slug_mismatches": slug_mismatches,
        "hash_mismatches": hash_mismatches,
        "units": [
            {
                "slug": s["slug"],
                "plan_dir": s["plan_dir"],
                "included": s["included_sources"],
                "excluded_root_md_count": len(s["excluded_md_at_root"]),
            }
            for s in all_scans
        ],
        "explicit_completion_statement": (
            "Wave v1-plan-evidence-excerpt: mirror "
            f"{len(all_included_files)} source files from "
            f"{len(migratable)} plan/audit units. "
            f"NOT migrated: ~{plans_md_total - len(all_included_files)} other docs/plans/*.md "
            f"and ~{docs_md_total - plans_md_total} docs outside docs/plans. "
            "Excerpts truncated at 12KB per file where noted."
        ),
    }

    RECON_PATH.write_text(json.dumps(recon, indent=2) + "\n", encoding="utf-8")

    man = json.loads(MANIFEST_PATH.read_text()) if MANIFEST_PATH.is_file() else {}
    man["last_reconciliation"] = RECON_PATH.relative_to(ROOT).as_posix()
    man["reconciliation_pass_v1"] = recon["pass_v1"]["overall"]
    MANIFEST_PATH.write_text(json.dumps(man, indent=2) + "\n", encoding="utf-8")

    print(json.dumps(recon["summary"], indent=2))
    print("pass_v1", recon["pass_v1"])


if __name__ == "__main__":
    main()
