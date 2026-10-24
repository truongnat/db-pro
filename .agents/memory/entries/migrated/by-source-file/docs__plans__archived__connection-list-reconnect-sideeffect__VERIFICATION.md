---
schema_version: 1
id: "MEM-SRC-a572ae4a62ae5fce"
title: "Source mirror: docs/plans/archived/connection-list-reconnect-sideeffect/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/connection-list-reconnect-sideeffect/VERIFICATION.md"
source_sha256: "a572ae4a62ae5fce11b1cb0031d3f2f2aeb40684868256eaeb3a230df539d440"
mirror_body_sha256: "a572ae4a62ae5fce11b1cb0031d3f2f2aeb40684868256eaeb3a230df539d440"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — QA-P1-09 Connection-list query reconnect side-effect

## Verification Plan

### Automated Unit Tests
- `frontend/src/modules/connection/__tests__/session-restoration.test.tsx`: Verify `restoreSession` behavior and one-shot execution.
- `frontend/src/modules/connection/__tests__/connection-queries.test.tsx`: Verify refetching `useConnectionList` does not trigger `service.connect()`.

### Quality Gates
- `cd frontend && npm run test`
- `cargo check -p db-pro-core -p db-pro-infrastructure`
