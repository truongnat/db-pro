# DB Pro — Phase H Goal: Advanced AI (Typed Assistants)

- Doc ID: `GOAL-PHASE-H`
- Phase: H — Advanced AI
- Release targets: v0.4 (H01), Later (H02–H03)
- Priority: P2 (H01), P3 (H02–H03)
- Authority: `docs/goals/goal-full-product.md` (master goal)
- Depends on: `D01–D04` (monitoring backend) and `F01–F03` (compare backend) for H02/H03;
  `A01` for any generated-DDL path; existing agent infrastructure for all milestones.
- Status: PLANNING. **The new agent tools described here are documentation only for this phase
  definition. No tool may be implemented until its backend (Phase D / Phase F) and its
  confirmation contract are in place.**
- Companions: `docs/goals/goal-phase-b-routines.md` (routine tools), `docs/goals/goal-phase-d-monitoring.md`,
  `docs/goals/goal-phase-f-compare-migration.md`, `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §7.

---

## 1. Problem

The agent layer is architecturally sound and functionally narrow.

What exists and works (source-verified):

- **A typed tool registry.** Nine tools are defined as JSON-schema function definitions in
  `crates/runtime/src/agent.rs:565-634`: `inspect_schema`, `inspect_table`, `inspect_columns`,
  `inspect_foreign_keys`, `get_current_query`, `patch_query`, `run_query`,
  `inspect_query_result`, `explain_query` (names at `:589-601`, parameter schemas at `:617-634`).
- **Every tool reuses canonical services.** `AgentToolExecutor` (`crates/runtime/src/agent_executor.rs:52-251`)
  routes to `SchemaApi`/`QueryApi` — `inspect_schema` → `schema_api().introspect`
  (`:81-86`), `run_query` → `query_api().execute_multi` (`:204-209`), `explain_query` →
  `query_api().explain` (`:223-228`). There is **no parallel database implementation** and no
  tool that opens its own connection. This invariant is the reason this phase can be built at
  all.
- **A real confirmation pipeline with anti-TOCTOU protection.** `ensure_execution_confirmation`
  (`agent_executor.rs:368-390`) requires `AgentMode::Agent` plus either an allowed decision or an
  explicit confirmation; `AgentWorkflow::require_confirmation` stores a
  `PendingAgentConfirmation` with an input fingerprint (`domain/agent_workflow.rs:37-49, 466-480`);
  `AgentRunOrchestrator::resume_confirmation` re-classifies SQL at approval time and escalates
  the required confirmation if the safety class worsened (`agent_orchestrator.rs:171-193`); the
  patch path verifies the document version before applying (`:196-241`).
- **Mode enforcement.** `AgentMode { Ask, Edit, Agent }` (`domain/agent.rs:57-61`) with an
  explicit tool allowlist per mode (`domain/agent_workflow.rs:519-542`), checked twice (workflow
  and executor).
- **Bounded outputs.** `MAX_AGENT_TABLES`, `MAX_AGENT_COLUMNS_PER_TABLE`, `MAX_AGENT_RELATIONS`,
  `MAX_AGENT_SAMPLE_ROWS`, `MAX_AGENT_EXPLAIN_CHARS`, `MAX_AGENT_TOOL_STEPS` bound every tool's
  input and output.

What is missing:

1. **No monitoring/AI administration tools.** There is no tool for sessions, active queries,
   locks, or plan-with-stats, because there is no monitoring backend (Phase D). Every entry in
   the capability matrix §7 for "Sessions/locks inspection", "Analyze Slow Query", "Index
   suggestion" is `MISSING`, explicitly blocked on Phase D.
2. **No compare/migration tools.** `compare_schema` and `generate_migration` are `MISSING` and
   blocked on Phase F's diff depth.
3. **No sampled-data analysis tool.** `inspect_query_result` summarizes an in-memory result; it
   cannot inspect a table's data without running a query the user wrote.
4. **The plan is text.** `explain_query` truncates at a character budget and the UI renders plain
   monospace text; the structured `ExplainPlanTree`/`PlanNode` widget exists but is used only in
   the component gallery (`crates/ui/src/component_gallery_view.rs:2217-2224`).
5. **Optimize is a prompt, not a tool.** "Optimize" exists as a prompt-level action with no
   dedicated tool, no plan input, and no statistics context (matrix §7: `PARTIAL`, target H).
6. **No routine-aware tools.** The agent cannot read a routine's source, parameters, or
   dependencies (Phase B adds the backend).

Consequence: DB Pro's agent can write and fix SQL in the editor, but it cannot help with the
operational work — the part of database work where an assistant with real context is most
valuable, and where the cost of a wrong action is highest.

## 2. Scope

### 2.1 In scope (by milestone)

| Milestone | Content |
|---|---|
| H01 | Tool-surface v2: implement read-only tools over the Phase D and Phase F backends (`list_sessions`, `list_active_queries`, `inspect_locks`, `explain_query_plan` with statistics context, `compare_schemas`), plus **documented contracts** for the mutating tools (`generate_migration`, `suggest_index`) including risk class, confirmation requirement, and the exact backend call each must use. Wire the real plan tree (`ExplainPlanTree`) into the query workspace. |
| H02 | Schema/Migration Assistant: given a diff, propose a migration plan, explain each operation, flag risks, and hand the reviewed plan to the F04 apply flow. **Never applies.** |
| H03 | Administration Assistant: slow-query analysis (plan + stats + lock context), index suggestion with preview + explicit apply confirmation, session/lock state summaries. |

### 2.2 Tool catalogue (target)

Read-only tier (auto-run only under the existing explicit setting; always safe to fall back to
confirmation):

| Tool | Parameters | Backend | Output bound |
|---|---|---|---|
| `list_sessions` | optional `filter` (user/database/state) | `MonitoringService::sessions` | ≤ 100 rows, query text truncated |
| `list_active_queries` | optional `min_duration_ms` | `MonitoringService::active_queries` | ≤ 50 rows |
| `inspect_locks` | optional `relation`, `blocked_only` | `MonitoringService::locks` | ≤ 100 rows + graph summary |
| `explain_query_plan` | `sql`, optional `with_stats` | `QueryService::explain` (+ Phase D stats for `with_stats`) | truncated plan + node summary |
| `inspect_sampled_data` | `table`, optional `limit` (≤ `MAX_AGENT_SAMPLE_ROWS`), optional `columns` | `TableDataService::fetch_rows` | ≤ sample cap, no writes |
| `inspect_routines` | optional `schema` | `RoutineService::list` | ≤ 50 routines with signatures |
| `inspect_routine_source` | `signature` | `RoutineService::source` | truncated source |
| `compare_schemas` | `source`, `target` (connection/snapshot refs) | `CompareService::compare` | summary + diff handle (not the full tree) |
| `analyze_query_performance` | `sql` or `pid` | `QueryService::explain` + `MonitoringService` | plan summary + stats + lock context |
| `suggest_indexes` | `sql` or `table` | plan + stats | candidate list with rationale — **no DDL execution** |

Mutating-shaped tier (never auto-run; preview + explicit confirmation; execution-time
re-classification):

| Tool | Produces | Execution path |
|---|---|---|
| `generate_migration` | a `SchemaMigrationPlan` (text + operation list) | Handed to the F03/F04 UI; the tool **never** applies |
| `fix_routine` / `generate_routine` | a `RoutineDefinition` patch preview | Handed to the A01 preview/apply flow |
| `apply_index` (H03 only, if approved) | an index DDL statement | Must go through A01 preview → confirmation → apply; requires its own safety review before implementation |

Explicitly absent by design: any tool that executes `CALL`, `DROP`, `ALTER`, `TRUNCATE`, a
migration apply, a session terminate, or a maintenance command. Those remain user actions in
their own UI surfaces (Phases A/D/F), which already have confirmation flows.

### 2.3 Out of scope

- Autonomous multi-step database changes (an agent loop that plans and applies several mutations).
- Self-scheduled or background agent runs.
- Model training/fine-tuning, embeddings, vector search over data.
- MCP integration (`LIM-008`).
- Agent access to credentials, keyring content, or connection strings.
- Agent-driven data modification of any kind outside the existing `patch_query` (editor) and
  `run_query` (confirmed execution) tools.

## 3. Non-goals

- **No autonomous destruction.** Stated in the master goal as binding (§1.4 rule 4) and repeated
  here because this phase is where the temptation is greatest: no tool may drop, truncate, alter,
  terminate, or vacuum. A user must click the confirmation in the owning surface.
- **No new database access path.** Every tool calls the canonical APIs. A tool that opens its own
  connection is a review-blocking defect (and would bypass policy, capability gating, and audit).
- **No unbounded context.** Every tool declares input and output budgets; a tool that can return
  a whole schema, a whole result set, or a whole plan without truncation is rejected.
- **No "silent" read-only auto-run for expensive reads.** A read-only tool that is expensive
  (a table scan for sampling, a full plan with `ANALYZE`) requires confirmation regardless of the
  read-only auto-run setting.
- **No AI-authored DDL applied directly.** Generated DDL always lands in a preview a human
  reviews (A01 or F03/F04).
- **No agent action that is invisible in the audit trail.** Every tool call is recorded with its
  inputs digest, risk class, and outcome.

## 4. Current implementation (code-verified baseline)

| Element | Reality | Evidence |
|---|---|---|
| Tool registry | 9 tools, JSON-schema parameters | `runtime/src/agent.rs:565-634` |
| Tool dispatch | Every tool → canonical `SchemaApi`/`QueryApi` | `runtime/src/agent_executor.rs:52-251` |
| No parallel DB access | Verified: no connector/pool use in the agent modules | `agent_executor.rs`, `agent_orchestrator.rs` |
| Modes | Ask / Edit / Agent with per-mode tool allowlists | `domain/agent.rs:57-61`; `domain/agent_workflow.rs:519-542` |
| Confirmation | Pending confirmation with input fingerprint; approval-time SQL re-classification and escalation | `agent_workflow.rs:466-480`; `agent_orchestrator.rs:120-194` |
| Anti-TOCTOU (patch) | Document version verified before applying | `agent_orchestrator.rs:196-241`; UI recheck `ui/src/agent_state.rs:302-326` |
| Idempotency/replay | Completed-call cache; duplicate `call_id` collision detection; step budget | `agent_orchestrator.rs:338-547` |
| Cancellation | Agent run cancellation aborts the task and cancels the in-flight DB query | `worker.rs:1037-1091` |
| Bounded outputs | `MAX_AGENT_*` caps | `agent_executor.rs` |
| Read-only auto-run | `execution_decision(mode, safety, allow_read_only_auto_run)` | `domain/agent.rs:383-397` |
| Plan rendering | `explain_query` truncates to `MAX_AGENT_EXPLAIN_CHARS`; UI shows plain monospace | `agent_executor.rs:213-239`; `ui/src/query_view.rs:463-481` |
| Structured plan widget | `ExplainPlanTree`/`PlanNode` exist, gallery-only | `ui/src/components/explain.rs`; gallery usage `component_gallery_view.rs:2217-2224` |
| Monitoring tools | **MISSING** (no backend) | matrix §7 rows `MISSING`, blocked on Phase D |
| Compare/migration tools | **MISSING** (diff too narrow) | `domain/schema_diff.rs` coverage; matrix §7 |
| Routine tools | **MISSING** (no parameter metadata) | Phase B §4 |
| Optimize | prompt-level action only | matrix §7 (`PARTIAL`, target H) |
| Sessions/locks context | **MISSING** | matrix §7 |

## 5. UX

### 5.1 Principles

1. **The agent proposes; the user disposes.** Anything that changes state produces a preview
   that flows into an existing confirmation surface. The agent panel never has its own
   "apply" button that bypasses the target surface's confirmation.
2. **Risk is visible before invocation.** Tool cards show a risk badge (`ReadOnly`,
   `Mutation`, `Destructive`) and, for read-only tools, whether they will auto-run under the
   current setting.
3. **Evidence is shown, not summarized away.** Plan nodes, lock graphs, and diff summaries are
   rendered with the same components the rest of the app uses, so the user can inspect what the
   model was given.
4. **Refusals are explicit.** When a capability is unavailable (SQLite sessions, monitoring on a
   read-only connection, a diff kind not compared), the tool returns an explicit unavailable
   result with the reason — never an empty success.
5. **Nothing happens in the background.** No scheduled agent runs, no retries without user
   action, no silent tool loops beyond the existing step budget.

### 5.2 Agent panel changes (H01)

```text
AGENT                                    [ Ask | Edit | Agent ]   [settings]

Context: local-pg · public.orders · Query.sql · 1 result set · monitoring on

▸ Tool activity
   ✓ inspect_schema (public)                     read-only   41 ms
   ✓ explain_query_plan (…customer_id…)          read-only   88 ms   [view plan]
   ⋯ list_active_queries                         read-only   running

▸ Pending confirmation
   ⚠ run_query  "UPDATE orders SET status = 'x' WHERE …"           Destructive
     Target: staging-pg (Staging) · 1 statement · rollback: transactional
     [ Show SQL ]  [ Reject ]  [ Approve ]
```

- Each tool call renders a card with name, risk, duration, and a result affordance
  (`[view plan]`, `[view diff]`, `[view sessions]`) that opens the corresponding workspace
  surface rather than dumping raw text into the chat.
- The context line shows exactly what the agent can see, including whether monitoring/diff tools
  are available for the active connection.

### 5.3 Plan rendering (H01)

The query workspace's Explain output gains a tree view using the existing `ExplainPlanTree`
component:

- Node rows: operation, relation, cost, rows estimate, actual rows/time when available.
- Highlighting for the most expensive node by a documented rule (highest actual time, else
  highest cost) and for scans over large relations.
- Raw plan text remains available in a secondary tab (never removed — the tree is an addition).
- The agent's `explain_query_plan` result links to this view.

### 5.4 Migration assistant (H02)

```text
AGENT — Migration plan for staging-pg                                   [read-only]

I compared local-pg/public with staging-pg/public. 37 differences.

Risk summary
  • 5 tables added, 2 removed, 3 changed
  • 6 operations cannot be rolled back (listed below)
  • The dropped column public.orders.legacy_code holds data (≈1.2M rows)

Proposed order
  1. CREATE TYPE order_status …            (new type used by 2 columns)
  2. CREATE TABLE public.invoices …
  …
  37. CREATE OR REPLACE VIEW public.v_orders …

[ Open in Migration plan ]        ← hands the plan to the Phase F UI for review + apply
```

The assistant's output is a **plan document**, not an action. "Open in Migration plan" performs
the hand-off; the apply confirmation lives in Phase F's surface with its typed-name requirement.

### 5.5 Administration assistant (H03)

- **Slow query analysis**: takes a pid or SQL text, gathers the plan, the relation statistics,
  and (if present) lock context, then explains the bottleneck in terms of the evidence shown.
  The response links to the monitoring rows it used.
- **Index suggestion**: proposes candidate indexes with the reasoning (predicate/join columns,
  scan counts, selectivity estimates) and produces a **DDL preview** that the user can send to
  Phase A's index flow. The agent never creates an index.
- **Session/lock summary**: summarizes the current state ("3 sessions blocked by pid 4211 on
  public.orders for 4m12s") with a link to the Monitoring locks view; terminate remains a
  monitoring action.

### 5.6 Empty, error, unavailable states

- **Unavailable tool**: shown disabled with the reason ("Monitoring is not available on SQLite
  connections", "Requires the compare backend from Phase F").
- **Provider gap**: a tool whose result is partial (for example session visibility limited by
  role privileges) states the limitation in its output, matching Phase D's honesty rule.
- **Model/provider failure**: the existing provider error path is reused; no partial tool
  results are presented as complete.

## 6. Architecture

### 6.1 Invariants (carried forward, not relaxed)

```text
Agent tool call
  → AgentToolExecutor (mode + allowlist check)
  → canonical service API (SchemaApi | QueryApi | MonitoringApi | CompareApi | RoutineApi)
  → policy + capability gating inside the service
  → database
```

Rules:

1. No tool constructs SQL for execution except by delegating to a service that parameterizes and
   classifies it (`run_query` already does this via `execute_multi`).
2. No tool holds a `ConnectionHandle`, pool, or connector.
3. Every tool declares: name, description, JSON-schema parameters, risk class, output budget,
   and the canonical API it calls. This declaration lives beside the tool definition
   (`agent.rs:565-634`) so the registry stays the single source of truth.
4. Risk classes and mode allowlists are data, not scattered `if`s; a new tool that omits a risk
   class fails a test.
5. Tool results are bounded by the declared budget before they enter the model context; exceeding
   the budget truncates with an explicit marker (never a silent cut).

### 6.2 Confirmation pipeline extension

For tools whose *output* is a proposal rather than an action (`generate_migration`,
`suggest_indexes`), the pipeline changes shape:

```text
tool runs (read-only) → returns a proposal artifact (plan/diff/DDL preview)
   → the artifact is registered with the workflow (id + fingerprint)
   → the user opens it in the owning surface
   → the owning surface's own confirmation executes it
```

This is deliberately **not** the `PendingAgentConfirmation` path: approval of a *proposal* is not
approval of an *execution*. Execution confirmations remain where they are (Phase A/F surfaces),
with the same fingerprint discipline. The agent's proposal artifact carries:
`{ artifact_id, kind, target_connection, fingerprint, created_at, tool_call_id }`.

If a future milestone adds an execution-capable tool (for example `apply_index`), it must use the
existing `PendingAgentConfirmation` + `execution_decision` + approval-time re-classification
path; that is the only acceptable mechanism, and it requires an explicit safety review recorded
in the milestone's `FINDINGS.md`.

### 6.3 Context assembly

- New tools require new context: monitoring rows, plan nodes, diff summaries. Context assembly
  stays in `AgentContextBuilder` (`core/src/domain/agent_context.rs`) and remains bounded
  (existing pattern: summarized counts plus samples).
- Context must be **redacted**: no credentials, no bound values, no result rows beyond the
  existing sample caps. A test asserts that assembling context from a monitoring result set
  cannot include a session's `client_addr` in cleartext unless the tool explicitly declares it
  needs it (and then it is included deliberately, not by accident).
- Query text from monitoring is included only for queries relevant to the user's question
  (the tool passes filters); the agent must not receive the full session list by default.

### 6.4 Provider gating

Every new tool declares a capability requirement:

| Tool | Requires |
|---|---|
| `list_sessions`, `list_active_queries`, `inspect_locks` | `session_monitoring` / `lock_monitoring` (PostgreSQL, Phase D) |
| `explain_query_plan` with stats | `relation_size_metrics` + `explain` (both providers for explain; stats PostgreSQL-only) |
| `inspect_sampled_data` | `TableDataService` availability (both providers; no-PK tables remain readable) |
| `compare_schemas` | `schema_diff` at Phase F depth |
| `inspect_routines`, `fix_routine`, `generate_routine` | `functions` (PostgreSQL) |
| `suggest_indexes` / `apply_index` | Phase D stats for PostgreSQL; SQLite gets plan-only reasoning |

A tool invoked against a connection lacking its capability returns an explicit
`ToolUnavailableReason` — the executor must not attempt the call and fail deeper.

## 7. Domain model

Extends the existing agent domain (`domain/agent.rs`, `domain/agent_workflow.rs`):

```rust
pub enum AgentToolRisk { ReadOnly, ReadOnlyExpensive, Mutation, Destructive }

pub struct AgentToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: serde_json::Value,      // JSON schema
    pub risk: AgentToolRisk,
    pub modes: &'static [AgentMode],        // allowlist
    pub required_capabilities: &'static [CapabilityKey],
    pub input_budget: ToolBudget,           // e.g. max rows/seconds the tool may request
    pub output_budget: ToolBudget,          // truncation limits before context insertion
}

pub enum AgentToolOutcome {
    Data { json: serde_json::Value, truncated: bool },
    Proposal { artifact: ProposalArtifact },
    Unavailable { reason: ToolUnavailableReason },
    ConfirmationRequired { kind: AgentConfirmationKind },
    Failed { error: String, retryable: bool },
}

pub struct ProposalArtifact {
    pub artifact_id: ProposalArtifactId,
    pub kind: ProposalKind,                 // MigrationPlan | RoutineDefinition | IndexDdl
    pub target_connection: ConnectionId,
    pub fingerprint: String,
    pub created_at: OffsetDateTime,
    pub tool_call_id: String,
}
```

`AgentToolRisk::ReadOnlyExpensive` is new and important: it distinguishes "safe and cheap"
(auto-run eligible) from "safe but expensive" (always confirm), which is the honest way to avoid
an agent triggering a table scan or a full plan with `ANALYZE` without the user asking.

## 8. APIs, services, ports

### 8.1 Registry changes

`tool_definitions()` (`runtime/src/agent.rs:565-634`) becomes a list of `AgentToolSpec` values.
The wire format (JSON schema sent to the provider) is generated from the spec, so the wire
contract, the risk class, the mode allowlist, and the capability requirement cannot drift apart.
Tests assert: every tool has a spec, the wire schema is derived from the spec, every spec names an
existing canonical API call, and no spec is missing a risk class.

### 8.2 Executor changes

`AgentToolExecutor::execute` (`agent_executor.rs:52-62`) keeps its match-based dispatch but
each arm:

1. checks `required_capabilities` against `DatabaseCapabilities` for the session's connection,
2. enforces the mode allowlist (existing),
3. enforces risk-appropriate confirmation (`ReadOnly` → existing auto-run setting;
   `ReadOnlyExpensive`/`Mutation` → confirmation; `Destructive` → confirmation),
4. calls the canonical API,
5. truncates the result to `output_budget` and records the truncation flag,
6. records the call in the audit/tool-activity trail.

### 8.3 Services consumed

| Tool | Service | Method |
|---|---|---|
| `list_sessions` | `MonitoringService` | `sessions` |
| `list_active_queries` | `MonitoringService` | `active_queries` |
| `inspect_locks` | `MonitoringService` | `locks` |
| `analyze_query_performance` | `QueryService` + `MonitoringService` | `explain`, `relation_stats`, `locks` |
| `suggest_indexes` | `QueryService` + `MonitoringService` | `explain`, `relation_stats` |
| `inspect_sampled_data` | `TableDataService` | `fetch_rows` (bounded, no PK requirement) |
| `compare_schemas` | `CompareService` | `compare` |
| `inspect_routines`, `inspect_routine_source` | `RoutineService` | `list`, `source` |
| `generate_migration` | `MigrationPlanner` | `plan` (pure) |
| `fix_routine`, `generate_routine` | `RoutineService` + A01 planner | `plan_definition` |

No new ports are introduced by Phase H itself; it consumes the ports created by Phases A–F. If a
tool needs data no service exposes, the correct response is to add it to the owning phase's
service, not to reach around the service layer.

### 8.4 Runtime/UI wiring

```text
UiCommand::AgentToolInventory    { request_id, connection_id }        // for the panel's availability display
UiEvent::AgentToolInventoryLoaded{ request_id, tools: Vec<ToolAvailability> }
```

Tool invocation itself continues through the existing agent run path
(`RunAgent`/`StartAgentWorkflow`/`ContinueAgentWorkflow`); proposal artifacts are surfaced as
`AgentWorkflowEvent` payloads and opened by the owning workspace surface:

```text
AgentWorkflowEvent::Proposal { artifact }     → UI stores it and renders an "Open in …" action
```

## 9. Provider behavior

| Tool | PostgreSQL | SQLite |
|---|---|---|
| `list_sessions`, `list_active_queries`, `inspect_locks` | Available (Phase D) with `permissions_limited` honesty when the role cannot see all sessions | Unavailable with reason |
| `analyze_query_performance` | Available; stats from `pg_stat_user_tables`, locks from `pg_locks` | Plan-only analysis; stats/locks explicitly unavailable (no fake numbers) |
| `suggest_indexes` | Plan + stats + scan counts | Plan-only; the suggestion states that statistics were unavailable |
| `explain_query_plan` | JSON plan (existing), plus stats when `with_stats` is confirmed | `EXPLAIN QUERY PLAN` (existing) — the tree is derived from the limited node set and the UI states the difference |
| `inspect_sampled_data` | Available | Available |
| `compare_schemas` | Phase F depth | Phase F depth (SQLite kinds) |
| `inspect_routines`, `fix_routine`, `generate_routine` | Available after Phase B | Unavailable with reason |
| `generate_migration` | Phase F planner | Phase F planner (with the rebuild caveats from Phase F §9.2) |

Cross-provider: `compare_schemas` with mixed providers returns the Phase F structural comparison,
and `generate_migration` returns the documented refusal. The agent must relay those refusals
verbatim in meaning, not rephrase them as suggestions.

## 10. UI structure

| Need | Component | Status |
|---|---|---|
| Tool cards with risk + result actions | `ToolCall`, `RiskLevel`, `StatusBadge` (`components/agent_primitives.rs`) | reuse/extend |
| Confirmation card | `ExecutionApproval` + `ConfirmationDialog` | reuse |
| Plan tree | `ExplainPlanTree`/`PlanNode` (`components/explain.rs`) | **wire** (currently gallery-only) |
| Diff rendering | `DiffViewer` (`components/diff.rs`) | **wire** (Phase F) |
| Session/lock results | Monitoring views (Phase D) via an "open in Monitoring" action | reuse |
| Tool availability list | `ObjectList`/`Badge` in the panel's settings/context area | new small composition |
| Proposal artifact link | `Button`/`Card` inside the tool card | new small composition |
| Activity/audit trail | `ActivityLog` + existing `AgentAuditEntry` | reuse |

Interaction rules:

- A tool result never renders raw JSON by default; each tool has a typed renderer, with a
  "show raw" escape hatch for debugging.
- "Open in …" actions navigate to the owning surface and preserve the artifact reference (so the
  user can return to the conversation and re-open it).
- Rejecting a confirmation returns control to the user without retrying the tool.
- The step budget and any truncation are visible in the run summary ("stopped after N steps",
  "result truncated at X").

## 11. Safety model

| Tool class | Auto-run | Confirmation | Execution |
|---|---|---|---|
| `ReadOnly` (cheap) | Only under the existing explicit `allow_read_only_auto_run` | Otherwise | Immediate read |
| `ReadOnlyExpensive` | Never | Required, with an estimated cost statement | Immediate read after approval |
| `Mutation` | Never | Required | Through the owning surface's preview/apply |
| `Destructive` | Never | Required + owning-surface strong confirmation | Through the owning surface |
| Proposal-producing (`generate_migration`, `suggest_indexes`) | Allowed as read-only computation | n/a for producing the artifact | The artifact's execution is confirmed in the owning surface |

Binding rules (from the master goal §10.6, restated for this phase):

1. No tool opens its own database connection.
2. No tool bypasses the application layer.
3. Any AI-generated mutation is previewed and explicitly confirmed by the user, with
   execution-time re-classification retained for execution-capable tools.
4. No autonomous index creation, ever.
5. No scheduled or background agent execution.
6. The agent never performs a maintenance or session-termination action; those are Phase D
   actions with their own confirmations.
7. Every tool call is audited (name, risk, inputs digest, capability checks, outcome,
   truncation).
8. Model-provided input is untrusted: tool parameters are validated against the declared schema
   and cross-checked against real state (for example, a `pid` must exist in the current session
   list before any action tool could use it in a future milestone).

## 12. Testing strategy

### 12.1 Unit

- Registry integrity: every tool has a spec; wire schema derived from the spec; risk class
  present; mode allowlist present; capability requirement present; canonical API call declared.
- Mode allowlists: Ask cannot call mutating tools; Edit can patch but not run; Agent can run with
  confirmation (existing tests extended to the new tools).
- Confirmation decisions: `ReadOnlyExpensive` always requires confirmation; read-only auto-run
  applies only to `ReadOnly`; risk escalation at approval time still works for execution-capable
  tools.
- Output budgets: every tool truncates at its declared bound and sets the truncation flag;
  a fixture that exceeds the bound produces the marker.
- Capability gating: each tool returns `Unavailable` with a reason on SQLite (or on a connection
  missing the capability) and **does not call** the backend (mock assertion).
- Context redaction: assembling context from monitoring/diff results cannot include credentials
  or bound values (fixture-based assertion).
- Proposal artifacts: ids and fingerprints are stable and unique; the same diff produces the same
  plan fingerprint (reusing Phase F's determinism test).

### 12.2 Workflow / integration

- Replay and idempotency: the existing `agent_orchestrator.rs` tests extend to the new tools
  (duplicate `call_id`, cached outcomes, step budget).
- Proposal hand-off: a `generate_migration` proposal opened in the Phase F UI applies with the
  Phase F fingerprint check; a stale proposal (source changed) is rejected with the drift message.
- Live PostgreSQL (ignored suite): each read-only tool against a fixture with a seeded slow query
  and a lock conflict; assert the tool output matches the Monitoring service output (the agent
  must not compute a different answer).
- SQLite: every monitoring-dependent tool returns `Unavailable` with the reason and issues no SQL.
- No-autonomy assertions: a code-level test (or review checklist with grep evidence) proves no new
  tool calls an execution path; the executor's dispatch table contains no
  `apply`/`execute_ddl`/`terminate` arms.

### 12.3 UI

- Tool cards: risk badge, availability state, truncation indicator, result renderers, "open in …"
  navigation.
- Plan tree rendering from a real plan (PostgreSQL) and from SQLite's limited plan; raw-text tab
  still available.
- Confirmation variants for `ReadOnlyExpensive` vs `Mutation` vs `Destructive`.
- Proposal artifact flow: open in the owning surface, apply, return to the conversation.
- Regression: existing agent tests (mode isolation, IME, multi-tab) keep passing.

## 13. Runtime verification

**H01**

1. Open the agent panel on a PostgreSQL connection; capture the tool inventory showing available
   and unavailable tools with reasons.
2. Ask for active queries; capture the tool card, the result, and the "open in Monitoring" action;
   verify the numbers match the Monitoring view.
3. Ask for a plan of a selected query; capture the plan tree, the expensive-node highlight, and the
   raw-text tab.
4. Ask for an `EXPLAIN ANALYZE`-backed plan; capture the cost statement and the confirmation before
   execution.
5. On SQLite, capture the monitoring tools shown as unavailable with the reason and the assertion
   that no query was issued.
6. Capture one rejected confirmation and confirm nothing executed.

**H02** (Later)

7. Compare two schemas via the agent; capture the summary and the risk list.
8. Open the proposed plan in the Migration plan surface; capture the review and the fingerprint
   hand-off.
9. Change the source schema, then attempt to apply the stale proposal; capture the drift refusal.

**H03** (Later)

10. Analyze a seeded slow query; capture the plan, the statistics used, and the explanation's link
    to the monitoring rows.
11. Request index suggestions; capture candidates with rationale and the DDL preview.
12. Send a candidate to Phase A's index flow; capture the normal preview/confirmation and confirm
    the agent itself did not execute anything.

## 14. Milestone order

| Order | ID | Title | Release | Pri | Prerequisites | Rationale |
|---:|---|---|---|---|---|---|
| 1 | H01 | AI Tool Surface v2 (read-only tools + tool contracts + plan rendering) | v0.4 | P2 | Phases B, D, F backends (tools are gated on them, so partial delivery is possible as backends land) | The registry/spec refactor, capability gating, budgets, and plan rendering are the foundation; shipping them before any proposal tool keeps the risk low |
| 2 | H02 | Schema / Migration Assistant | Later | P3 | H01, F04 | Needs a proven plan→review→apply path; the assistant must hand off, not execute |
| 3 | H03 | Administration Assistant (slow query, index suggestion, sessions) | Later | P3 | H01, D03, D04 | Needs monitoring depth and statistics to give non-speculative advice; index suggestion carries the highest "looks harmless but isn't" risk |

Ordering rules: H01 must land before any proposal-producing tool. H02 and H03 are independent of
each other. If Phase D or F slips, H01 ships only the tools whose backends exist and marks the rest
`Unavailable` — the registry must never advertise a tool it cannot serve.

## 15. Definition of done

1. **Plan folder** per milestone; `docs/plans/STATUS.md` matches.
2. **P0 = 0, P1 = 0.**
3. **No new database access path** exists: every tool calls a canonical service; review evidence
   shows no connector/pool usage in the agent modules.
4. **Every tool is a spec**: name, description, JSON-schema parameters, risk class, mode allowlist,
   capability requirements, input/output budgets, and the canonical API it calls. Tests enforce
   completeness and wire-schema derivation.
5. **Capability gating is honest**: unavailable tools are disabled with a reason and issue no
   backend call (mock-asserted); SQLite gets explicit unavailability rather than empty results.
6. **`ReadOnlyExpensive` is distinct** from `ReadOnly`: expensive reads always confirm, and the
   existing read-only auto-run setting cannot bypass that.
7. **Plan rendering is real**: `ExplainPlanTree` is wired into the query workspace and fed by real
   plans on both providers, with raw text still available.
8. **Proposal artifacts never execute**: `generate_migration`/`suggest_indexes` produce artifacts
   consumed by the owning surface's confirmation; a test proves no code path from a proposal to an
   execution.
9. **No autonomous destructive behavior** exists: no tool performs DDL, session termination,
   maintenance, or migration apply; grep-based review evidence is recorded in `VERIFICATION.md`.
10. **Audit trail**: every tool call is recorded with name, risk, inputs digest, capability checks,
    outcome, and truncation flag; visible in the activity surface.
11. **Context is redacted**: no credentials, connection strings, or bound values enter the model
    context (fixture-based test).
12. **Live PostgreSQL evidence** for each read-only tool, plus SQLite unavailability evidence, in
    each milestone's `VERIFICATION.md`; where a backend phase has not landed, the tool is recorded
    as `Unavailable` — not as "implemented".
13. **Quality gates executed and recorded** (fmt, clippy with stated tauri scope,
    `cargo test --workspace`, SQLite suite, PostgreSQL live suite).
14. **Docs updated**: capability matrix §7 rows move off `MISSING`/`PARTIAL` for shipped tools;
    master goal §9 and Phase H's tool catalogue reflect what exists versus what is documented;
    every documented-but-unimplemented tool is marked as such in the matrix.
15. **Non-goals respected**: no autonomous mutation, no scheduled runs, no unbounded context, no
    MCP, no embeddings, no direct execution from proposals.

Phase H is complete when the agent can inspect sessions, active queries, locks, plans, sampled
data, routine metadata, and schema differences on supported providers; render real plan trees in
the workspace; produce migration plans and index suggestions as reviewable artifacts that flow
into the existing confirmed apply paths; and demonstrably cannot execute a mutation, terminate a
session, or apply a migration on its own — with PostgreSQL evidence, SQLite unavailability
evidence, and a full audit trail.
