---
schema_version: 1
id: "MEM-PLAN-issue-365-auto-increment-type-d8b60576087b"
title: "Plan archive: Issue 365 Auto Increment Type"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-365-auto-increment-type]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d8b60576087bb3690f3d55afa606b4926d5c0c7c6a40664c243fa09925a6e3aa"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-365-auto-increment-type/PLAN.md"
  - "docs/plans/active/issue-365-auto-increment-type/FINDINGS.md"
  - "docs/plans/active/issue-365-auto-increment-type/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-365-auto-increment-type/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #365: auto_increment overrides type wrong — SQLite INTEGER PK / Postgres Alter SERIAL

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `build_mutation_request` rejects `auto_increment && !is_pk` on all drivers, rejects Postgres `Alter + auto_increment` (SERIAL is create-only), and only rewrites the type for the (now PK-only) create path.


## Source: `FINDINGS.md`

# Findings — #365

schema_workbench_mutation.rs:45-74. Validation covers MySQL AUTO_INCREMENT-needs-key implicitly via the PK requirement.


## Source: `VERIFICATION.md`

# Verification — #365

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).


