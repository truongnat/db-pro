---
schema_version: 1
id: "MEM-SRC-80950b38e1a649a8"
title: "Source mirror: docs/plans/active/issue-508-dep-oid-lookup/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-508-dep-oid-lookup/CHECKLIST.md"
source_sha256: "80950b38e1a649a8d4ffa50af4872eee346210ada0af14349b22f711eaa0db1f"
mirror_body_sha256: "80950b38e1a649a8d4ffa50af4872eee346210ada0af14349b22f711eaa0db1f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #508 checklist

- [x] WHERE clause is `d.objid = to_regclass($1)` (OID equality)
- [x] qualified `"schema"."object"` bound as one param via `quote_ident`
- [x] no `LIKE`, no `|| '.%'` pattern concatenation
- [x] `pg_namespace` join removed (no longer needed; filter is OID)
- [x] `d.classid = 'pg_catalog.pg_class'::regclass` filter kept
- [x] tests/cross_connection_tests.rs added; mod decl in source file
- [x] `cargo check -p db-pro-infrastructure` clean
- [x] `cargo test -p db-pro-infrastructure --lib` green: 167 passed / 0 failed
