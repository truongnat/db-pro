---
schema_version: 1
id: "MEM-SRC-291847747263e0e9"
title: "Source mirror: docs/plans/active/issue-508-dep-oid-lookup/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-508-dep-oid-lookup/PLAN.md"
source_sha256: "291847747263e0e990de041d07d7601492c64e906d1a036783ef9585fc0e3f27"
mirror_body_sha256: "291847747263e0e990de041d07d7601492c64e906d1a036783ef9585fc0e3f27"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #508 — dependency lookup matches text, not OID

## Goal
`get_object_dependencies` must select dependencies by catalog OID, not by comparing
`d.objid::regclass::text` — whose rendering drops the schema under `search_path` — and
must not use `LIKE` wildcards on user-supplied names.

## Scope
- `crates/infrastructure/src/postgres/cross_connection.rs`: filter becomes
  `d.objid = to_regclass($1)` with one bound parameter holding the fully-qualified,
  per-part double-quoted `"schema"."object"` string built by a new `quote_ident` helper.
- `to_regclass` (not `$1::regclass`) so a missing/unparseable name yields NULL → empty
  result instead of a query error.
- New `tests/cross_connection_tests.rs` (`#[cfg(test)] #[path]` split per convention):
  `quote_ident` unit tests + a source-shape regression asserting OID equality and no LIKE.

## Context
Commit 55dc35a4 ("fix: commit temp", 2026-10-16) had already replaced the original
`source_name LIKE $1 || '.%'` with exact `nspname`/`relname` equality — which already
killed the wildcard bug but still compared names, not OIDs. This change completes the
fix per the issue's prescribed shape.

## Non-goals
- Same `pg_depend` row shape and ORDER BY preserved; `classid` filter on
  `pg_catalog.pg_class` kept.
- No changes to `list_partitions`/`list_tablespaces`/`rename_schema_object`.

## Acceptance
- OID equality filter; no LIKE; `%`/`_`/dots in names cannot escape into pattern syntax.
