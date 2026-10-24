---
schema_version: 1
id: "MEM-SRC-58bd7be444034fe7"
title: "Source mirror: docs/plans/active/macos-app-menu-refresh/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/macos-app-menu-refresh/CHECKLIST.md"
source_sha256: "58bd7be444034fe77aee350569d2e89881fc4bb96b5047ed79bc6a07ca0d48f6"
mirror_body_sha256: "58bd7be444034fe77aee350569d2e89881fc4bb96b5047ed79bc6a07ca0d48f6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

- [x] Confirm existing toolbar/palette refresh and connection-scoped cache invalidation APIs.
- [x] Define menu contents and the active-connection behavior, including New Connection, New Query, and Help.
- [x] Add a typed cache-invalidation option to schema introspection.
- [x] Build and inspect the native menu implementation with standard and capture-feature configurations.
- [x] Add regression coverage for Refresh Cache dispatch and no-active-connection behavior.
- [x] Run the native command-translator regression test.
- [x] Run formatting, targeted UI tests, and `cargo build --release --locked -p db-pro-native`.
- [ ] Observe menu actions in the live macOS app and record evidence.
- [ ] Verify Refresh and Refresh Cache against live SQLite and PostgreSQL connections independently.
- [ ] Obtain independent review and record final review outcome.
