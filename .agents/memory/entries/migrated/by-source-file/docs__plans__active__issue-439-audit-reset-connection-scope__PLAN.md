---
schema_version: 1
id: "MEM-SRC-5fb3fa84f676a406"
title: "Source mirror: docs/plans/active/issue-439-audit-reset-connection-scope/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-439-audit-reset-connection-scope/PLAN.md"
source_sha256: "5fb3fa84f676a4066955c596805f4fdcc3ad7fd18d10a977913203d9821eecc9"
mirror_body_sha256: "5fb3fa84f676a4066955c596805f4fdcc3ad7fd18d10a977913203d9821eecc9"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #439 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/439
- Fix: AuditState::reset_connection_scope clears audit_page, filters, selections, and bookmarks upon disconnect and connect, preventing audit events from leaking across connections.
