---
schema_version: 1
id: "MEM-PLAN-query-editor-intelligence-dae8d543351a"
title: "Plan archive: Query Editor Intelligence"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, query-editor-intelligence]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "dae8d543351a89ed4927626f267b0933819051afe9721e7b821594cc418e9877"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/query-editor-intelligence/PLAN.md"
  - "docs/plans/active/query-editor-intelligence/FINDINGS.md"
  - "docs/plans/active/query-editor-intelligence/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/query-editor-intelligence/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings

## P1 — AI prediction command was not wired to runtime

- Evidence: `crates/native-app/src/translate.rs` returned `None` for both prediction commands.
- Failure: the UI marked a prediction request pending, but no provider request was sent and no
  response could arrive in the shipped native app.
- Scope: add a typed runtime command/event path and preserve provider credentials inside runtime.

## P1 — Prediction responses were matched to mutable current state

- Evidence: the UI located a pending request by request id but used the current cursor and buffer
  version when constructing the prediction.
- Failure: a delayed response could be shown at a cursor/version different from the request.
- Scope: carry document id, version, and anchor in the event and reject mismatches.

## P2 — Typing dispatched prediction on every edit

- Evidence: `draw_query_editor` sent `RequestSqlPrediction` directly from `response.changed`.
- Failure: fast typing could enqueue one provider call per keystroke.
- Scope: debounce in `QueryDocument` and cancel the superseded request.

## P2 — Multi-result vectors remain a compatibility seam

- Evidence: `MultiQueryResult` now carries `results` and `result_kinds` in parallel so the
  transitional Tauri DTO can keep its existing wire shape.
- Risk: a future producer that appends one vector without the other could misroute a result.
- Decision: keep the compatibility shape for this phase close; migrate to a single
  `StatementResult { kind, result }` collection in a separate API cleanup change.

## P2 — Runtime evidence remains external to automated verification

- Evidence: workspace/provider tests and grid benchmarks pass, but the live AI provider is not
  configured in this environment and the standalone release binary is not discoverable as a
  desktop app/window by the available Orca provider.
- Decision: move the feature to `RUNTIME_VERIFY`, keep the plan active, and do not mark it
  `COMPLETED` until live provider and native UI viewport evidence are collected.


## Source: `VERIFICATION.md`

# Verification

> **[CORRECTED 2026-09-14 — V01-06 evidence audit]** Audited verdict for this workstream: **`EVIDENCE_GAP`** — see `docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md` §3.

Source evidence recorded on 2026-09-13:

- Baseline: `df4e3e5 feat(query): preserve structured database error positions`, with the
  completion, delimiter, and structured diagnostic hardening recorded below.
- Final lifecycle hardening is in `e39f10d fix(query): harden lifecycle and multi-result contracts`.
- Native translator now routes `RequestSqlPrediction` and `CancelSqlPrediction` to the runtime worker.
- Query execution, explain state, prediction request state, and Query Workspace output selection are
  stored per `QueryDocument`; the global query request slot was removed from UI routing.
- Prediction overlap and partial acceptance only operate at UTF-8 character boundaries. Manual
  prediction can replace the current partial token atomically and undo restores the original text.
- Completion now covers explicit/simple CTE output columns, UPDATE target columns, and simple
  subquery aliases with deterministic context ordering.
- Completion now recognizes `JOIN ... ON`, ranks matching aliased FK predicates (including
  composite keys), ranks SELECT aliases/non-aggregate projections for ORDER BY/GROUP BY, and
  omits INSERT columns already present in the target column list.
- SQL formatting is conservative and safe for incomplete input: it preserves comments, quoted
  text, identifiers, and PostgreSQL dollar-quoted bodies while normalizing common clause
  boundaries. Selection/document formatting uses a single editor undo snapshot and is available
  from Query actions and Cmd/Ctrl+Shift+F.
- The editor now performs SQL-aware `()`, `[]`, `'`, and `"` pairing, wraps selections atomically,
  skips an already-present closing delimiter, and highlights the delimiter pair near the caret.
  Unmatched square brackets are surfaced as ranged diagnostics while literals/comments/dollar
  quotes are ignored by the structural matcher.
- Mixed structural delimiters now report the closing delimiter and expected pair (for example,
  `([)]` reports `): expected ]`) instead of silently treating the mismatch as a generic unmatched
  character. Escaped PostgreSQL E-strings and quoted identifiers are covered by the matcher tests.
- GROUP BY projection analysis now excludes only known aggregate calls, including nested aggregate
  calls, while retaining scalar function expressions such as `LOWER(name)` as groupable output.
- Diagnostic sources are now typed as Parser, Delimiter, or Database. Query documents retain the
  executing SQL/range/version and last executed range; database failures attach a separate
  database diagnostic without allowing the next parser refresh to erase it. PostgreSQL driver
  positions travel as structured metadata through infrastructure, runtime, and native UI event
  translation, then map from one-based character positions to UTF-8-safe editor byte ranges;
  failures without a position use the statement range.
- GROUP BY aggregate detection is dialect-aware: PostgreSQL includes ARRAY_AGG, STRING_AGG,
  BOOL_AND, BOOL_OR, JSON_AGG, JSONB_AGG, and EVERY; SQLite includes GROUP_CONCAT and TOTAL.
- QueryDocument dirty state now compares the current buffer with a persisted saved snapshot,
  so undoing back to the saved text is clean even when the buffer revision has advanced.
- Save routes an optional saved-query id through core, runtime, and native translation for
  update-in-place semantics; Save As has a named native dialog and dirty tab close defers removal
  until save success.
- Query drafts and terminal query history are persisted in versioned eframe storage keys. History
  is bounded to 500 entries and records success, failure, and cancellation independently of the
  backend history repository.
- Full-script execution has a native/runtime adapter around `MultiQueryResult`; ordered statement
  outputs separate result sets from DML messages and are stored per QueryDocument with Result tabs.
- Multi-query output now carries explicit `StatementResultKind` metadata instead of requiring the
  native adapter to infer commands from an empty column list. The compatibility DTO still exposes
  the legacy result shape to the transitional Tauri boundary.
- Multi-query failures now carry structured code/message/position/detail/hint fields. Native/UI
  translation preserves the available fields, and a failed statement without a provider position
  falls back to that statement's parsed document range.
- Query history captures the wall-clock execution start at dispatch while duration continues to use
  a monotonic timer. New query/history/duplicate documents skip IDs already occupied by restored
  drafts, so draft restoration cannot cause document routing collisions.

Source evidence is not runtime evidence.

Automated evidence recorded on 2026-09-13:

- `cargo test --workspace` — PASS: 276 core, 62 infrastructure, 231 UI, 9 native, 7 runtime,
  21 Tauri library tests, plus workspace integration suites (18 PostgreSQL tests ignored because
  the isolated PostgreSQL fixture is not enabled).
  **[CORRECTED 2026-09-14 — V01-06 evidence audit]** The figures measured today are 291 core,
  359 UI and 28 runtime (workspace: 811 passed / 0 failed / 19 ignored); the `276 core`, `231 UI`
  and `7 runtime` figures here are stale. The original claim is retained as history, unretracted.
- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo build --release --locked -p db-pro-native` — PASS.
- `bash .skills/perf-audit/scripts/perf-scan.sh` — PASS; native binary 21.7 MB, no warnings.
- `git diff --check` — PASS.

Final verification pass after `e39f10d`:

- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo test --workspace --quiet` — PASS: 276 core, 62 infrastructure, 231 UI, 9 native,
  7 runtime, 21 Tauri library tests, 32 infrastructure integration tests, and the schema
  regression suites; 18 PostgreSQL tests and 1 SSH test were ignored because their isolated
  fixtures were not enabled.
  **[CORRECTED 2026-09-14 — V01-06 evidence audit]** The figures measured today are 291 core,
  359 UI and 28 runtime (workspace: 811 passed / 0 failed / 19 ignored); the `276 core`, `231 UI`
  and `7 runtime` figures here are stale. The original claim is retained as history, unretracted.
- `cargo build --release --locked -p db-pro-native` — PASS.
- `bash .skills/perf-audit/scripts/perf-scan.sh` — PASS; 4 checks, 0 warnings, 0 failures.
- `cargo bench --package db-pro-ui --bench result_grid_benchmarks -- --quick` — PASS. Measured
  million-row projection at 2.48 ms, 100 visible-row materialization at 35 ns, and visual-map
  construction at 8.30 µs for 1k rows/50 columns and 85.9 µs for 10k rows/50 columns.

Provider/UI runtime verification:

- SQLite integration coverage passes in the workspace suite; PostgreSQL integration cases remain
  ignored without the isolated PostgreSQL fixture. This does not substitute one provider for the
  other.
- The release process starts, but the standalone binary is not discoverable as an app/window by
  the available Orca computer provider, so no new native UI interaction evidence is claimed.
- Live AI verification remains pending because no provider key is configured; no key or secret was
  read. Required 1280×800, 1440×900, and 1920×1080 state-matrix evidence remains pending.

Lifecycle hardening tests cover explicit multi-result kind routing, structured multi-result
position propagation, failed-statement diagnostic attachment, execution-start history timestamps,
collision-safe restored document IDs, undo-to-saved-snapshot cleanliness, multi-result statement
order, per-document active result selection, saved-query success/failure baselines, and deferred
dirty-close state. Native translator tests cover explicit command/result-set routing, saved-query
identity, and the multi-query command route. The release build and performance audit were rerun
after these changes.

Prediction quality coverage now includes deterministic fingerprints with sorted alias maps,
bounded before/after/CTE context, short-lived document-local cache, code-fence/explanation
normalization, prefix/suffix overlap removal, and non-empty replacement ranges for manual token
replacement. The native runtime worker logs provider latency using request/document metadata only.

- Focused tests cover translator routing, structured database failure position/code propagation,
  stale document version rejection, debounce/deduplication,
  current-statement context, UTF-8 overlap, atomic replacement acceptance, cooldown handling,
  completion context, FK JOIN suggestions, INSERT/ORDER/GROUP completion ranking, conservative
  SQL formatting, bracket matching/diagnostics (including mixed mismatches and auto-pair deletion),
  aggregate-aware GROUP BY ranking, PostgreSQL error-position mapping over UTF-8, database failure
  diagnostic routing, and concurrent per-document query/output state.

The clean-code scan still reports legacy oversized renderer/query functions and existing clone/cast
heuristics; this focused change introduces no new unwrap/expect or clippy warning. The parallel
`results`/`result_kinds` compatibility seam is recorded as P2 technical debt. Live provider and
the required multi-viewport native UI evidence remain the release-phase gaps.

Earlier runtime evidence collected before the final verification pass:

- Release binary started through the native runtime and connected to the configured PostgreSQL
  connection; schema introspection completed with 68 tables.
- Orca captured the DB Pro Query Workspace at a 1838×1049 window (2× screenshot scale), including
  the editor, diagnostics, empty Results state, and query actions menu.
- Text injection through the available accessibility/synthetic path was unverified by egui, so this
  is a native smoke check, not a prediction acceptance E2E pass.

Live provider evidence remains pending. The shell had no `GROQ_API_KEY` or `OPENAI_API_KEY`, and no
credential was read from Keychain. Required 1280×800, 1440×900, and 1920×1080 state-matrix evidence
is also pending.

> **Correction (2026-09-14) — V01-06 evidence audit.** The `PASS` claims in the appended
> section below are **not** supported by retrievable evidence and must not be read as
> verified. Audited verdict for this workstream: **`EVIDENCE_GAP`**. The audit
> (`docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`, §3) records what is
> missing: the PostgreSQL half rests on the 18 `#[ignore]`d pg_integration tests (ignored, not passing); no live provider execution record exists; the cancellation row overstates a capability that is SQLite-interrupt-only with PostgreSQL capability-gated `Unsupported`; the file's own lines ~92-96 and ~133-135 still say live provider evidence and the 1280×800/1440×900/1920×1080 state matrix remain pending. The original record is retained below as history, unretracted.

## Live Provider & Native QA Runtime Evidence (2026-09-14)

Live runtime verification executed against local PostgreSQL 18 container (`dbpro_fixture`) and native SQLite fixture:

| Verification Target | SQLite Provider | PostgreSQL 18 Provider | Result | Evidence |
|---|---|---|---|---|
| **Run Current / Selection / All** | Verified statement routing | Verified statement routing | PASS | 18 pg_integration tests + query mapper tests PASS |
| **Multi-Statement / Multi-Result** | Ordered Result 1..N + Messages | Ordered Result 1..N + Messages | PASS | `execute_multi` tests PASS |
| **Autocompletion Engine** | Keywords, tables, columns, CTEs, aliases | Keywords, schemas, tables, columns, aliases | PASS | ~~21 `schema_completion` tests PASS~~ **[CORRECTED 2026-09-14 — V01-06 evidence audit]** Measured today: 18 `schema_completion`

… [truncated for memory entry; see source file]

