---
schema_version: 1
id: "MEM-SRC-5598b83c4380aa67"
title: "Source mirror: docs/plans/active/query-editor-intelligence/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/query-editor-intelligence/PLAN.md"
source_sha256: "5598b83c4380aa6782c9e9da1d4dfde8af6ed62114a398f7e4d6382a05e69241"
mirror_body_sha256: "5598b83c4380aa6782c9e9da1d4dfde8af6ed62114a398f7e4d6382a05e69241"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Query Editor Intelligence Layer

## Goal

Make query execution, explain, schema completion, and AI SQL prediction deterministic per
`QueryDocument`, with stale-response protection and a real async runtime boundary for prediction.

## Scope

- Carry document id, document version, and cursor anchor through prediction requests/events.
- Route prediction through the native translator and async runtime worker.
- Cancel superseded prediction requests and debounce typing-triggered requests.
- Keep execution and explain results associated with the originating document.
- Keep Query Workspace output selection associated with the originating document.
- Complete the query lifecycle with snapshot-based dirty state, saved-query updates,
  local draft/history persistence, and ordered multi-result output.
- Bound and normalize prediction output before rendering or accepting it.
- Improve local schema completion for mutation targets, CTEs, and subquery aliases.
- Preserve the existing native Query Workspace UI and completion behavior while adding focused tests.

## Non-goals

- Agent chat UI or a Query Workspace redesign.
- Database schema changes.
- Replacing the existing schema introspection provider.

## Acceptance criteria

- A prediction request contains document context and cannot mutate a newer document version.
- Native runtime executes prediction asynchronously through the configured provider.
- Typing does not issue an AI request until the debounce window expires.
- Switching document connection/schema cancels and invalidates pending prediction state.
- Query and explain completion update only the originating document.
- Save updates an existing saved query by stable id, while Save As creates a new saved query.
- Undoing back to the saved snapshot clears the dirty state.
- Query history survives restart with a bounded local retention policy.
- Multi-statement results preserve statement order and remain isolated per query document.
- PostgreSQL and SQLite continue to use their existing execution capabilities.
