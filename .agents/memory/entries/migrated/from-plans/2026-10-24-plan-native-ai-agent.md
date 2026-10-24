---
schema_version: 1
id: "MEM-PLAN-native-ai-agent-4f55a94ced26"
title: "Plan archive: Native Ai Agent"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, native-ai-agent]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "4f55a94ced269c69858a576083eee6653b67ba8fd92e9f04b23950334c1eb0dd"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/native-ai-agent/PLAN.md"
  - "docs/plans/completed/native-ai-agent/FINDINGS.md"
  - "docs/plans/completed/native-ai-agent/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/native-ai-agent/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Native AI Agent

## Status

`COMPLETED`

## Goal

Turn the native Agent preview into a database-aware copilot for the Rust/egui
workspace. The first slice must work offline and produce inspectable SQL drafts
without executing mutations.

## Scope

- Add a pure Rust template responder with connection/schema context.
- Add native agent conversation state and a functional composer.
- Render assistant text, SQL drafts and write-risk warnings.
- Insert a proposed SQL draft into the active query document.
- Keep the agent visually inside the database workspace, not as a detached chat.

## Out of scope

- Network LLM calls or provider/API-key storage.
- Automatic query execution or mutation approval.
- MCP/tool execution and autonomous database changes.

## Architecture

`egui composer -> AgentMessage -> pure template responder -> AgentMessage`

The responder receives only non-secret context: driver, connection label,
table names and column names. SQL drafts are data for the query editor; they are
never executed by the Agent action itself. A later provider slice can replace
the responder behind the same message contract in the runtime layer.

## Acceptance criteria

- Empty state offers useful database-aware starter prompts.
- Cmd/Ctrl+Enter and Send produce a user message plus an assistant response.
- Responses can include SQL and Insert places it in the active query tab.
- Mutating drafts are visibly marked as requiring confirmation and never run.
- Pure responder tests cover empty schema, table selection, read-only SQL and mutation warning.


## Source: `FINDINGS.md`

# Findings

- The native panel was presentation-only: its input was disabled and it had no message state or command path.
- The existing frontend has template-agent behavior, but native schema DTOs currently expose only summary table and column names. The native first slice therefore generates safe, inspectable drafts from that summary and does not pretend to have foreign-key context it does not receive.
- Native architecture forbids putting credentials in UI state. Provider configuration and network calls remain a later runtime concern.


## Source: `VERIFICATION.md`

# Verification

## Automated

- `cargo fmt --all -- --check` passed.
- `cargo check -p db-pro-ui -p db-pro-native` passed.
- `cargo test -p db-pro-ui` passed: 10 tests.
- `cargo check --workspace` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace` passed with no failures; the PostgreSQL integration set remains ignored because it needs an external provider.

## Runtime

Native runtime review passed with the raw `cargo run -p db-pro-native` binary:

- light shell and Agent empty state rendered in `/tmp/db-pro-agent-v3-empty.png`;
- active SQLite context rendered as `Native Test · SQLite · 2 tables`;
- read-only SQL response rendered in `/tmp/db-pro-agent-sql.png`;
- mutation warning rendered in `/tmp/db-pro-agent-mutation.png`;
- Insert into Query activated the Query tab and populated the editor without executing the draft.

The provider/API-backed Codex integration is intentionally a later slice; this
plan only covers the offline native copilot contract and safe draft workflow.

