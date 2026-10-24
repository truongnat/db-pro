---
schema_version: 1
id: "MEM-PLAN-safe-sample-sqlite-seed-7adedb53d926"
title: "Plan archive: Safe Sample Sqlite Seed"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, safe-sample-sqlite-seed]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "7adedb53d926fb724ca02b28cf76c866f7afe0623bf3a59495e0998b82e07653"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/safe-sample-sqlite-seed/PLAN.md"
  - "docs/plans/active/safe-sample-sqlite-seed/FINDINGS.md"
  - "docs/plans/active/safe-sample-sqlite-seed/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/safe-sample-sqlite-seed/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan — Safe SQLite Sample Seeding

## Objective

Fix issue #492 so startup never registers an arbitrary or symlinked `/tmp` file as the writable Sample E-Commerce database.

## Target Branch

`fix/safe-sample-sqlite-seed`

## Severity

P1 — startup could silently expose a user-owned SQLite database as a writable demo connection.

## Scope

- Store the fixture under the app's resolved per-user data directory.
- Create the fixture only at a no-overwrite path; validate app id and fixture marker before reuse.
- Reject symlinks/unrecognized existing files and keep the seeded connection read-only.
- Migrate the prior built-in `/tmp/db_pro_sample.db` connection entry to the verified per-user fixture.
- Add regression coverage for existing files, symlinks, idempotence, and read-only configuration.


## Source: `FINDINGS.md`

# Findings — Safe SQLite Sample Seeding

## Root Cause

Startup used a fixed `/tmp/db_pro_sample.db`, followed any existing path (including symlinks), and registered it as `Sample E-Commerce (SQLite)` with `readonly = false`. Fixture initialization errors were ignored.

## Fix Strategy

Use `runtime.data_dir()/sample-ecommerce.sqlite`. Atomically reserve a new file with `create_new`; initialize it with a DB Pro SQLite application id and marker. If the path already exists, reject symlinks and only reuse a regular SQLite file whose marker and application id match. Do not register the connection if validation fails. Mark it read-only and migrate a prior built-in sample entry away from the legacy `/tmp` path.


## Source: `VERIFICATION.md`

# Verification — Safe SQLite Sample Seeding

- `cargo test -p db-pro-native --bin db-pro-native sample_seed -- --nocapture`: 3 passed, 0 failed.
- `cargo check -p db-pro-native`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Focused regressions confirm unknown pre-existing bytes and a valid unmarked SQLite database are preserved, fixture reuse leaves its bytes unchanged, sample config is read-only, and symlinks are rejected without modifying their targets.
- Fixture creation opens its reserved file with `SQLITE_OPEN_NOFOLLOW` and without `SQLITE_OPEN_CREATE`, so SQLite refuses a symlink swapped into the reserved path.
- Legacy connection migration now additionally requires the SQLite driver.

