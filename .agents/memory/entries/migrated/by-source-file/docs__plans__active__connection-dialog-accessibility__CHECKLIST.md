---
schema_version: 1
id: "MEM-SRC-d26f8f60c55185fe"
title: "Source mirror: docs/plans/active/connection-dialog-accessibility/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/connection-dialog-accessibility/CHECKLIST.md"
source_sha256: "d26f8f60c55185fe291079822524e7a320a4af8a280a2f489a60d4ce0371e67f"
mirror_body_sha256: "d26f8f60c55185fe291079822524e7a320a4af8a280a2f489a60d4ce0371e67f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Native UI Regression Repairs — Checklist

## Planning
- [x] All four owner-reported regressions recorded with concrete source/runtime evidence
- [x] Scope, non-goals, and PostgreSQL/SQLite applicability explicit

## Implementation
- [x] Legacy sidebar rows use the DB Pro theme-aware Button component
- [x] Modal focus trap allows egui Tab/Shift+Tab traversal and reclaims focus outside the modal
- [x] Required markers and inline errors added for validated connection fields and Save Query As
- [x] Agent API-key field is visibly marked required; Save remains disabled while blank
- [x] Port zero rejected to match the existing 1–65535 error contract
- [x] Global noninteractive strokes, resting standard-input outlines, and dialog separators removed;
  intentional search/focus/error outlines retained
- [x] Connection modal Cancel/window close sends cancellation for the active test request; failed dispatch keeps the dialog open.
- [x] Pending connection tests show a separate Cancel Test button; the tested cancel path dispatches
  cancellation and keeps the modal open; modal Cancel remains separate.
- [x] Runtime worker drops the in-flight test future and removes cancellation state.
- [x] SQLite connection test uses a standalone actor rather than a registered connector actor.

## Tests and quality gates
- [x] Pinned Rust format, workspace check, Clippy, workspace tests (1,793 passed / 41 ignored), and native release build pass.
- [x] Clean-code diff scan reports 0 errors and 4 warning categories.
- [x] Runtime worker cancellation closes a stalled local PostgreSQL-protocol socket and emits no completion event.
- [x] SQLite uses an unregistered standalone actor; MySQL uses a local pool; SQL Server uses a local client instead of persistent connector registration.
- [ ] Live cancellation against PostgreSQL, SQLite, MySQL, and SQL Server is pending independently for each provider.

## Review and runtime
- [x] Self-review complete; P0 = 0, P1 = 0
- [ ] Independent review / owner keyboard interaction
- [x] Light/dark normal/error captures reviewed at 1280×800 and effective 1440×838 / 1920×838
- [ ] Exact 1440×900 and 1920×1080 viewports; current display caps available height at 838 logical px
- [x] VERIFICATION.md and AGENT_EVIDENCE.md contain exact commands, findings, and evidence
- [x] STATUS.md remains RUNTIME_VERIFY until open runtime/review evidence is collected
