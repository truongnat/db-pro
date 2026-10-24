---
schema_version: 1
id: "MEM-SRC-14e83fe3430fae78"
title: "Source mirror: docs/plans/MEMORY_MIGRATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/MEMORY_MIGRATION.md"
source_sha256: "14e83fe3430fae7802a00bb946dbcd6d9b0ab8c275ca00c276c61288864f1b7a"
mirror_body_sha256: "14e83fe3430fae7802a00bb946dbcd6d9b0ab8c275ca00c276c61288864f1b7a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan evidence → memory (pointer)

**Hoàn tất chỉ cho wave `v1-plan-evidence-excerpt`** — không phải toàn repo. Số liệu: [`SCOPE_AND_COMPLETION.md`](active/agents-memory-framework/SCOPE_AND_COMPLETION.md).

**Canonical owner:** [`docs/plans/active/agents-memory-framework/`](active/agents-memory-framework/PLAN.md)

Use that directory for PLAN, CHECKLIST, FINDINGS, VERIFICATION, and AGENT_EVIDENCE.

Quick commands:

```bash
.agents/scripts/search-memory.sh "<query>"
python3 .agents/scripts/migrate-plans-to-memory.py
```

Manifest: [`.agents/memory/MIGRATION_MANIFEST.json`](../../.agents/memory/MIGRATION_MANIFEST.json)
