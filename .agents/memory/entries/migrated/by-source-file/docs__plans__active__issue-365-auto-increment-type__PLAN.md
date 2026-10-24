---
schema_version: 1
id: "MEM-SRC-24c7d6e35cf870e8"
title: "Source mirror: docs/plans/active/issue-365-auto-increment-type/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-365-auto-increment-type/PLAN.md"
source_sha256: "24c7d6e35cf870e826c0621f61d49e80bd201273928e3f5a99f1a009a74767c7"
mirror_body_sha256: "24c7d6e35cf870e826c0621f61d49e80bd201273928e3f5a99f1a009a74767c7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #365: auto_increment overrides type wrong — SQLite INTEGER PK / Postgres Alter SERIAL

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `build_mutation_request` rejects `auto_increment && !is_pk` on all drivers, rejects Postgres `Alter + auto_increment` (SERIAL is create-only), and only rewrites the type for the (now PK-only) create path.
