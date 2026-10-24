---
schema_version: 1
id: "MEM-SRC-df540822a089a477"
title: "Source mirror: docs/plans/active/ui-query-workspace/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-query-workspace/CHECKLIST.md"
source_sha256: "df540822a089a477f2d3d6e83f8598f2d9f7edee3c5d3462bb853ddde5c0e966"
mirror_body_sha256: "df540822a089a477f2d3d6e83f8598f2d9f7edee3c5d3462bb853ddde5c0e966"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — UI05 Query Workspace

- [x] Audit and migrate Query header actions to canonical `Button`.
- [x] Audit and migrate Query find/search bar actions to canonical `Button`.
- [x] Audit and migrate SQL snippets inserters to canonical `Button`.
- [x] Extend non-regression test in `crates/ui/src/components/mod.rs` to include `query_view.rs`.
- [x] Run full workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release --locked -p db-pro-native`).
- [x] Document verification.
