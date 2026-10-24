---
schema_version: 1
id: "MEM-PLAN-issue-508-dep-oid-lookup-388a5a9bdd2f"
title: "Plan archive: Issue 508 Dep Oid Lookup"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-508-dep-oid-lookup]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "388a5a9bdd2f27c080fabbd04757424a23f892ed07c9e8dab25f4e19ca88ca28"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-508-dep-oid-lookup/PLAN.md"
  - "docs/plans/active/issue-508-dep-oid-lookup/FINDINGS.md"
  - "docs/plans/active/issue-508-dep-oid-lookup/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-508-dep-oid-lookup/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# #508 findings

- Original bug (pre-55dc35a4): subquery rendered `d.objid::regclass::text` then matched
  `source_name LIKE $1 || '.%'`. regclass text omits the schema when the object is in
  `search_path`, so qualified patterns missed rows; unescaped `%`/`_` in object names
  were interpreted as wildcards.
- 55dc35a4 had replaced this with `n.nspname = $1 AND c.relname = $2` — correct
  equality, no wildcards, but still name-based rather than OID-based.
- Fix per issue: `d.objid = to_regclass($1)`. `to_regclass` parses regclass input
  syntax, so each part is double-quoted via `quote_ident` — mixed-case names and names
  containing dots/quotes resolve to exactly that relation; never a search_path sibling.
- `to_regclass` chosen over `$1::regclass` cast: it returns NULL instead of raising for
  names that don't resolve, preserving "object has no known deps" → empty Vec.
- regclass text columns in SELECT remain display-only; that rendering ambiguity does
  not affect which rows match.


## Source: `VERIFICATION.md`

# #508 verification

- `quote_ident_wraps_and_doubles_inner_quotes` — plain, spaced, dotted, and quoted
  identifiers all render as single quoted parts.
- `qualified_name_binds_schema_and_object_verbatim` — `%`/`_` stay literal inside
  `"my%schema"."tab_le"`.
- `dependency_lookup_filters_by_oid_not_like_pattern` — source-shape regression:
  `get_object_dependencies` body contains `d.objid = to_regclass($1)`; contains no
  `LIKE` and no `$1 ||`; qualified param built via `quote_ident`.
- `cargo check -p db-pro-infrastructure` — clean.
- `cargo test -p db-pro-infrastructure --lib` — 167 passed / 0 failed, including the three
  tests above.

