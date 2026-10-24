---
schema_version: 1
id: "MEM-SRC-c0bc9c3b4c272775"
title: "Source mirror: docs/plans/active/monitoring-event-correlation/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/monitoring-event-correlation/CHECKLIST.md"
source_sha256: "c0bc9c3b4c272775146295105c473d7e4885a9aed9d4f94344b7a9dfa95e313c"
mirror_body_sha256: "c0bc9c3b4c272775146295105c473d7e4885a9aed9d4f94344b7a9dfa95e313c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

## Implementation
- [x] Idle repaint scheduled only for active, connected, auto-refresh Monitor.
- [x] Snapshot/workload request identity tracked and validated.
- [x] Users/privileges/memberships/RLS request identity tracked and validated.
- [x] Audit, pg_settings, FDW, replication, and event-trigger loaded responses validate request and active connection.
- [x] Data-diff responses validate request, source, target, and active source connection.
- [x] Mutation-triggered inventory refreshes replace pending request identity.
- [x] Audit state is cleared on disconnect and connection switch.
- [x] Loaded events preserve the originating connection ID across runtime/native protocol.

## Verification
- [x] Monitoring repaint delay regression test.
- [x] Request/connection matching regression test.
- [x] Monitoring and security focused tests pass.
- [x] App/runtime/native cross-crate check and formatting pass.
- [x] Diff/callsite audit complete.

## Delivery
- [ ] Rebase on latest `origin/main`.
- [ ] Commit, push, and draft PR linked with `Fixes #489` and `Fixes #473`.
