---
schema_version: 1
id: "MEM-SRC-039d81f266168d2f"
title: "Source mirror: docs/plans/active/issue-508-dep-oid-lookup/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-508-dep-oid-lookup/FINDINGS.md"
source_sha256: "039d81f266168d2f775d84f590a3c587768ef73a2d0d2f67853f399f80d39d1e"
mirror_body_sha256: "039d81f266168d2f775d84f590a3c587768ef73a2d0d2f67853f399f80d39d1e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
