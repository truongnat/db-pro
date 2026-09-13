# Agent Workflow

## Goal

Add a safe orchestration layer that connects an AI agent to a specific
`QueryDocument`, bounded database context, explicit SQL patches, and the
existing query safety/runtime paths.

## Initial scope

- Define provider-neutral domain contracts for sessions, actions, tool calls,
  bounded schema/result context, and document-versioned SQL patches.
- Reuse the existing core SQL safety classifier through an agent permission
  decision, with mutations always confirmation-gated.
- Make patch application deterministic and reject stale document versions.
- Keep the existing offline/remote Agent prototype and native UI working while
  the new contracts become the seam for the next runtime slice.

## Non-goals

- No Agent Chat redesign or new app shell.
- No autonomous destructive/database mutations.
- No full-schema or full-result upload to a provider.
- No RAG/vector store, plugin system, or schema visualization.
- No Query Editor feature growth except blocking fixes.

## Architecture

`db-pro-core::domain::agent` owns provider-neutral models and safety decisions.
Runtime adapters will translate these contracts to `RuntimeCommand`/events and
the native UI will only dispatch actions and render state. `QueryDocument` and
the existing `QueryService` remain the source of truth for text and execution.

## Acceptance criteria for the foundation slice

- A patch includes document id and expected version and cannot apply to a
  different document, version, or non-UTF-8 range.
- Agent execution classification distinguishes read-only, mutating,
  destructive, and unknown SQL.
- Agent mode permissions never auto-run mutations or destructive SQL.
- Context builder limits tables, columns, relations, samples, cell length, and
  total serialized context deterministically.
- Tests cover stale patches, Unicode ranges, permission gates, bounded context,
  and result-summary truncation.

## Provider matrix

The foundation is provider-neutral and has no database I/O. PostgreSQL and
SQLite execution remain delegated to the existing capability/safety layer; live
provider and native UI evidence are pending for the later runtime slice.
