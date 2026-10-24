---
schema_version: 1
id: "MEM-SRC-412d0327df69d549"
title: "Source mirror: docs/plans/active/issue-508-dep-oid-lookup/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-508-dep-oid-lookup/VERIFICATION.md"
source_sha256: "412d0327df69d549595a23c83452f1a0f0c3eed0f92a817ca89eb8cb0381c35e"
mirror_body_sha256: "412d0327df69d549595a23c83452f1a0f0c3eed0f92a817ca89eb8cb0381c35e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
