---
schema_version: 1
id: "MEM-SRC-26337eca470bc770"
title: "Source mirror: docs/plans/active/ui-query-workspace/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-query-workspace/FINDINGS.md"
source_sha256: "26337eca470bc7702eebc8abbbade6921b3fb2aee824b754789f7b86280355f2"
mirror_body_sha256: "26337eca470bc7702eebc8abbbade6921b3fb2aee824b754789f7b86280355f2"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — UI05 Query Workspace

## Audit Findings

1. `crates/ui/src/query_view.rs`:
   - Header `Run` and `Stop` used legacy `primary_button_with_icon` and `secondary_button_with_icon`.
   - `Builder` used legacy `secondary_button`.
   - `More actions` used `compact_icon_button`.
   - Editor find bar used `compact_icon_button` for previous/next match and close.
   - SQL snippets list used legacy `compact_button`.
