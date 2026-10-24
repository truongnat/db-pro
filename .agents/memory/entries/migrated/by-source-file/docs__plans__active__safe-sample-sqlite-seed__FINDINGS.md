---
schema_version: 1
id: "MEM-SRC-12a830b9ccf3d6a6"
title: "Source mirror: docs/plans/active/safe-sample-sqlite-seed/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/safe-sample-sqlite-seed/FINDINGS.md"
source_sha256: "12a830b9ccf3d6a6f969791be1486ef3baf807e107f197e4e313f60647161a26"
mirror_body_sha256: "12a830b9ccf3d6a6f969791be1486ef3baf807e107f197e4e313f60647161a26"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — Safe SQLite Sample Seeding

## Root Cause

Startup used a fixed `/tmp/db_pro_sample.db`, followed any existing path (including symlinks), and registered it as `Sample E-Commerce (SQLite)` with `readonly = false`. Fixture initialization errors were ignored.

## Fix Strategy

Use `runtime.data_dir()/sample-ecommerce.sqlite`. Atomically reserve a new file with `create_new`; initialize it with a DB Pro SQLite application id and marker. If the path already exists, reject symlinks and only reuse a regular SQLite file whose marker and application id match. Do not register the connection if validation fails. Mark it read-only and migrate a prior built-in sample entry away from the legacy `/tmp` path.
