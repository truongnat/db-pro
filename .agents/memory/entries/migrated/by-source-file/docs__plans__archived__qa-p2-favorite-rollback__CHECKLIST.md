---
schema_version: 1
id: "MEM-SRC-513574ef055710dd"
title: "Source mirror: docs/plans/archived/qa-p2-favorite-rollback/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/qa-p2-favorite-rollback/CHECKLIST.md"
source_sha256: "513574ef055710dd3a3ab707825680ad57f58fa786f866c72d13dd88e819ddb2"
mirror_body_sha256: "513574ef055710dd3a3ab707825680ad57f58fa786f866c72d13dd88e819ddb2"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist: Favorite Toggle Optimistic Update Rollback (QA-P2-20)

- [ ] Implement `onError` callback in `useToggleFavorite`
- [ ] Add unit test in `connection-queries.test.tsx` for error rollback
- [ ] Pass frontend quality gates (`typecheck`, `lint`, `format:check`, `check:tokens`, `test`, `build`)
- [ ] Pass Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`)
