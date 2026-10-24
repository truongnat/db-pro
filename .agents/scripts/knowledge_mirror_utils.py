"""Shared source ↔ mirror encoding and verification."""

from __future__ import annotations

import hashlib
import re
from pathlib import Path

# Full-byte mirror: no arbitrary cap (owner requirement: migration hết).
TRUNCATE_MARKER = "\n\n… [truncated at migrate cap]\n"
V1_TRUNCATE_MARKER = "\n\n… [truncated for memory entry; see source file]\n"
V1_MAX_CHARS = 12000

CANONICAL_IN_PLACE = {
    "AGENTS.md",
    "RULES.md",
    ".impeccable.md",
    "skills-lock.json",
    "clean-code.config.json",
}

# Pipeline outputs under .agents/memory — never inventory, mirror, or count as sources.
PIPELINE_MEMORY_ARTIFACT_BASENAMES = frozenset(
    {
        "KNOWLEDGE_SOURCE_INVENTORY.json",
        "KNOWLEDGE_RECONCILIATION_V2.json",
        "KNOWLEDGE_GROUP_RECONCILIATION.json",
        "MIGRATION_MANIFEST.json",
        "MIGRATION_RECONCILIATION.json",
        "SOURCE_SCOPE_MANIFEST.json",
        "APPROVED_EXCLUSIONS.json",
    }
)


def is_pipeline_memory_artifact(rel: str) -> bool:
    """True for migration/inventory JSON the pipeline writes (self-referential)."""
    rel = rel.replace("\\", "/")
    name = Path(rel).name
    if name in PIPELINE_MEMORY_ARTIFACT_BASENAMES:
        return True
    if rel.startswith(".agents/memory/") and name.endswith(".json"):
        if name.startswith("KNOWLEDGE_") or name.startswith("MIGRATION_"):
            return True
    return False


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_text(text: str) -> str:
    return sha256_bytes(text.encode("utf-8"))


def mirror_filename(rel: str) -> str:
    safe = rel.replace("/", "__")
    return safe if safe.endswith(".md") else f"{safe}.md"


def encode_mirror_body(rel: str, raw: bytes) -> tuple[str, bool, str]:
    """Return (body_text, truncated, source_sha256)."""
    source_sha = sha256_bytes(raw)
    truncated = False
    text = raw.decode("utf-8", errors="replace")
    if rel.endswith(".json"):
        text = f"```json\n{text}\n```\n"
    return text, truncated, source_sha


def parse_mirror_file(path: Path) -> dict:
    text = path.read_text(encoding="utf-8", errors="replace")
    if not text.startswith("---\n"):
        return {"error": "no_frontmatter", "path": str(path)}
    parts = text.split("---\n", 2)
    if len(parts) < 3:
        return {"error": "bad_frontmatter", "path": str(path)}
    # Body is verbatim after the closing `---\n` (do not strip: source may start with newline).
    fm, body = parts[1], parts[2]
    meta = {}
    for line in fm.splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            meta[k.strip()] = v.strip().strip('"')
    return {
        "meta": meta,
        "body": body,
        "body_sha256": sha256_text(body),
        "path": str(path),
    }


def requires_v2_mirror(rel: str, zone: str) -> bool:
    name = Path(rel).name
    if name in CANONICAL_IN_PLACE or rel in CANONICAL_IN_PLACE:
        return False
    if zone in ("repo_canonical", "repo_config"):
        return False
    if "_template" in Path(rel).parts:
        return False
    if zone == "agents_memory" and "/entries/" in rel and "/migrated/" not in rel:
        if name.endswith(".md"):
            return False
    if zone == "agents_memory" and ("KNOWLEDGE" in name or "MIGRATION" in name):
        return False
    if is_pipeline_memory_artifact(rel):
        return False
    return True


def requires_v1_excerpt(rel: str, zone: str) -> bool:
    if zone != "docs_plans":
        return False
    if Path(rel).name in ("PLAN.md", "FINDINGS.md", "VERIFICATION.md", "AGENT_EVIDENCE.md"):
        return True
    if "_audit-2026-10-13" in rel and rel.endswith(".md"):
        if Path(rel).name not in ("INDEX.md", "CAPTURE-EVIDENCE.md"):
            return True
    return False


def verify_v2_mirror(root: Path, rel: str) -> dict:
    src = root / rel
    mirror = root / ".agents/memory/entries/migrated/by-source-file" / mirror_filename(rel)
    if not src.is_file():
        return {"status": "source_not_exists", "rel": rel}
    raw = src.read_bytes()
    expected_body, truncated, source_sha = encode_mirror_body(rel, raw)
    expected_body_sha = sha256_text(expected_body)
    if not mirror.is_file():
        return {
            "status": "mirror_missing",
            "rel": rel,
            "source_sha256": source_sha,
            "truncated": truncated,
        }
    parsed = parse_mirror_file(mirror)
    if "error" in parsed:
        return {"status": "mirror_invalid", "rel": rel, **parsed}
    meta = parsed["meta"]
    fm_source_sha = meta.get("source_sha256", "")
    fm_trunc = meta.get("truncated", "false").lower() == "true"
    fm_body_sha = meta.get("mirror_body_sha256", "")
    body_ok = parsed["body_sha256"] == expected_body_sha
    sha_ok = fm_source_sha == source_sha
    trunc_flag_ok = fm_trunc == truncated
    meta_body_ok = (not fm_body_sha) or fm_body_sha == expected_body_sha
    if body_ok and sha_ok and trunc_flag_ok and meta_body_ok:
        return {
            "status": "mirrored_truncated_verified" if truncated else "mirrored_full_verified",
            "rel": rel,
            "mirror_path": mirror.relative_to(root).as_posix(),
            "source_sha256": source_sha,
            "truncated": truncated,
        }
    return {
        "status": "mirror_hash_mismatch",
        "rel": rel,
        "mirror_path": mirror.relative_to(root).as_posix(),
        "source_sha256": source_sha,
        "expected_body_sha256": expected_body_sha,
        "actual_body_sha256": parsed["body_sha256"],
        "fm_source_sha256": fm_source_sha,
        "truncated_expected": truncated,
        "truncated_fm": fm_trunc,
    }
