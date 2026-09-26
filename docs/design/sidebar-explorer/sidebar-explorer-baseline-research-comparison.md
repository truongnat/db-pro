# Sidebar Explorer — Baseline vs Feature Research Comparison

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · research/comparison lane |
| Issue(s) | n/a |
| Task state | Done |
| Research baseline SHA | `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b` — baseline and feature-research source snapshot |
| Current source SHA | `7bbfb08e289fa2b77867d40feb0d886dbc097d31` — used only for the documented filter-drift check |
| Branch / PR | `main` · no PR |
| Scope interpretation | Compare the source-observed Sidebar Explorer baseline with the capability-gap research and use the implementation roadmap to resolve priority or scope ambiguity. |
| Out of scope | Implementing Explorer changes; native UI traversal; PostgreSQL or SQLite runtime verification; validating every external product capability independently. |

## 2. Progress checkpoint

- Completed acceptance rows: [x] establish document roles, [x] compare feature areas, [x] identify overlap and gaps, [x] identify stale or ambiguous claims, [x] recommend implementation order.
- Remaining acceptance rows: none for this comparison.
- Findings / risks: no confirmed P0/P1/P2 correctness defect. The priorities below are product recommendations, not defect severities.
- Tests already run: none; this is a documentation/source comparison. No native UI or provider runtime gate was executed.
- Dependency / blocker changes: none.

## 3. Document roles

The two documents are complementary, not competing specifications:

```text
Baseline: source-observed behavior at a fixed SHA
    ↓
Feature Research: competitive gap analysis and candidate capabilities
    ↓
Implementation Roadmap: scope, dependency and priority resolution
```

### Baseline

`sidebar-explorer-baseline.md` records the Explorer implementation visible in source at the research baseline SHA. Its strongest evidence is implementation structure:

- the `egui` view → typed action → `DbProApp` reducer → state/runtime pipeline;
- action variants and their effects;
- workspace destinations;
- connection, transaction, staged-change and confirmation guards;
- table-row offscreen rendering and navigation-cache behavior.

It is a source inventory, not proof of native UI behavior or PostgreSQL/SQLite runtime support.

### Feature Research

`sidebar-explorer-feature-research.md` compares that baseline with documented DBeaver, DataGrip and pgAdmin capabilities. It defines eleven candidate gaps, F1–F11. These are proposals, not implementation commitments or runtime evidence.

### Roadmap

`sidebar-explorer-implementation-roadmap.md` is the tie-breaker when Research is broader than the current architecture or assigns a priority without sufficient dependency analysis. It explicitly preserves the working navigation/action pipeline and rejects a broad object-browser rewrite.

## 4. Feature comparison matrix

| Area | Baseline proves at `a21504e…` | Feature Research proposes | Comparison verdict |
|---|---|---|---|
| Connection lifecycle | Connect, disconnect, reconnect, refresh, lifecycle states, and guards for staged changes/open transactions. | Connection groups, favorites, environment markers, granular retry/cancel. | Preserve lifecycle and guards. Grouping is organization work, not a replacement lifecycle. |
| Tree hierarchy | Connection → database → schema → Tables/Views/Functions/Triggers. Functions are capability-gated. | User-defined connection folders, pinned objects and recent objects. | Extend the existing hierarchy only after persistence ownership is defined. |
| Object filtering | One `explorer_search`, normalized to lowercase; baseline describes table-focused matching. | Filter every supported object kind; type scope; contains/starts-with/exact/regex; owner/comment/system filters; counts, clear state and persistence. | Highest-value usability gap, but the baseline claim is now partially stale; see §6. |
| Object coverage | Tables, Views, Functions/Procedures and Triggers. Table detail includes columns, foreign keys and indexes. | PostgreSQL sequences, materialized views, types, extensions, FDW, policies and other kinds; SQLite-specific indexes, pragmas and virtual tables. | Candidate list only. Every kind requires provider introspection, capability metadata, rendering/actions and independent runtime evidence. |
| Object actions | Rich table actions; View supports modify/drop; Function lacks modify/drop; Trigger lacks query/modify; Create Table opens a query draft. | Typed Create/Modify/Drop workflows for each supported kind. | Audit action-to-runtime support before adding menu items. Generate SQL, plan mutation and execute mutation remain separate. |
| Metadata preview | Table metadata can expand in-tree; definitions and richer object views open another workspace. | Keyboard-accessible quick documentation for tables, views, routines and triggers. | Proposed P1. Start with metadata already available and label unavailable fields instead of inventing data. |
| Schema Compare | A separate Schema Compare activity exists; Explorer does not define source/target selection. | Select source/target from connection, schema or object context and preview a diff/script. | Valuable, but requires stable identity, explicit source/target context, invalid-pair handling and no automatic mutation. |
| ER Diagram | Opens from connection context. | Open with schema/table scope, show related tables, save/export layout. | Add selected scope only after the Diagram workspace accepts and labels that scope without stale cross-connection data. |
| Refresh/status | Connection/schema introspection with shared loading/error feedback. | Folder/object refresh, stale timestamps, node-level loading, cancel and retry. | Begin with truthful connection/schema scope. Do not display per-folder refresh or cancel controls until runtime supports them. |
| Dependencies/navigation | Foreign keys are displayed in table details. | Go to referenced object, incoming/outgoing dependencies, Find Usages and dependency graph. | FK navigation can precede a complete dependency graph. Never infer full dependency coverage from visible FK rows. |
| Keyboard access | Some shortcuts exist, including refresh and context actions; complete tree navigation is not established. | Arrow/Enter/Space/F2/Cmd-or-Ctrl+F/Escape and focus restoration. | Proposed P2, but new tree interactions should be designed with keyboard behavior from the start. |
| Persistence | Sidebar width and some persistent `egui` node IDs exist. | Persist expanded paths, filters, favorites, recent objects and compare selection. | Define whether state belongs to application, connection, database or schema before persistence is added. |
| Safety | Delete/drop confirmations and connection/transaction/staged-change guards exist. | Dependency warnings, migration preview and explicit apply confirmation. | Research must preserve baseline safety. A visible action must not imply unsupported SQL or bypass confirmation policy. |

## 5. Detailed findings

### 5.1 Areas where the documents agree

Both documents establish that Explorer already has a credible navigator foundation:

- connection catalog and lifecycle;
- database/schema/table navigation;
- table data, structure, DDL and SQL-generation entry points;
- columns, foreign keys and indexes;
- Views, Functions/Procedures and Triggers folders;
- integration with Query, Table, Schema Object, Schema Workbench, Diagram and Agent workspaces;
- mutation and connection safety guards.

The Research correctly treats those capabilities as the starting point rather than proposing a replacement tree.

### 5.2 Baseline-only implementation detail

The Baseline is stronger where implementation correctness depends on exact contracts:

- action enum boundaries and reducer ownership;
- which context-menu actions select a table first;
- exact workspace activated by each action;
- query templates generated for SELECT/INSERT/UPDATE/DELETE;
- the difference between a direct row click and a chevron click;
- connection-switch, transaction and delete-confirmation guards;
- offscreen table-row rendering and navigation-cache matching.

The Research does not supersede these contracts. An implementation based only on the Research could accidentally bypass a reducer, duplicate state ownership or weaken a guard.

### 5.3 Research-only capability gaps

The Research adds product-level needs not represented by the source inventory:

1. **F1 — deep object filtering:** explicit scope/mode, filter state, per-folder results and persistence.
2. **F2 — organization:** connection groups, favorites, pinned and recent objects.
3. **F3 — quick documentation:** inspect metadata without leaving Explorer.
4. **F4 — provider depth:** provider-specific object families.
5. **F5 — action consistency:** typed Create/Modify/Drop workflows.
6. **F6 — compare context:** choose source/target directly from Explorer.
7. **F7 — scoped diagrams:** schema/table/relationship scope.
8. **F8 — refresh trust:** granular loading/error/stale state.
9. **F9 — dependency navigation:** references, usages and dependency graph.
10. **F10 — keyboard navigation:** complete and discoverable tree controls.
11. **F11 — persisted context:** expansion, selection, filters and favorites.

### 5.4 Priority mismatch: connection organization

Feature Research labels connection folders/favorites as P1. The Roadmap moves this work behind filter, refresh/status, metadata preview, selected-context workflows and action safety, effectively treating it as P2 organization work.

The Roadmap order is safer. Groups and favorites improve organization, but they do not solve incorrect filter semantics, stale metadata ambiguity or unsupported action exposure.

### 5.5 Object coverage must remain capability-driven

The PostgreSQL and SQLite object lists in F4 are research candidates, not evidence that DB Pro introspects or operates on those kinds. A supported object family needs all four layers:

```text
provider introspection
→ explicit capability metadata
→ Explorer rendering and action contract
→ independent provider/runtime verification
```

A folder must not appear because another IDE exposes it. PostgreSQL-only folders must not appear on SQLite, and no provider may receive generated DDL for an unsupported operation.

### 5.6 Current-source drift: F1 is partially implemented

The Baseline and Research correctly describe the filter behavior at `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`. They are no longer a complete description of source at `7bbfb08e289fa2b77867d40feb0d886dbc097d31`.

At the current source SHA:

- `crates/ui/src/explorer_schema_objects_view.rs:43-77` normalizes one search query and passes it to Views, Functions and Triggers as well as Tables.
- `crates/ui/src/explorer_schema_object_folders_view.rs:30-153` filters View, Function and Trigger names with case-normalized `contains` matching, computes matched counts and shows a `No matching …` empty label.

Therefore F1 should now be classified **partially implemented**, not table-only.

Still missing from the complete F1 proposal:

- object-kind scope and visibility controls;
- starts-with, exact and regex modes;
- owner, comment and system/user-object fields;
- an explicit applied-filter indicator and clear/reset action;
- persistence ownership per connection/schema;
- a defined policy for total count versus matched count.

## 6. Decision and recommended order

### Decision

Use the Baseline as the implementation contract, the Research as a candidate capability catalogue, and the Roadmap as the priority/dependency authority. Do not implement the Research as a flat checklist.

### Recommended order

1. **Reconcile F1 documentation with current source**, then complete filter state, count semantics and clear/reset behavior.
2. **Make refresh status truthful** at connection/schema scope before exposing finer-grained controls.
3. **Add quick metadata preview** using data already supplied by introspection.
4. **Audit provider/action capability and safety** before adding Create/Modify/Drop affordances.
5. **Define stable source/target context**, then add Explorer-to-Schema-Compare entry points.
6. **Add schema/table Diagram scope** only after the Diagram workspace accepts explicit scope and freshness identity.
7. **Expand provider object families incrementally**, one introspected/capability-gated kind at a time.
8. **Add folders, favorites, complete keyboard behavior and persistence** after state ownership is explicit.

## 7. Research / audit handoff

- **Source date:** 2026-09-26.
- **Source references:**
  - `docs/design/sidebar-explorer/sidebar-explorer-baseline.md`
  - `docs/design/sidebar-explorer/sidebar-explorer-feature-research.md`
  - `docs/design/sidebar-explorer/sidebar-explorer-implementation-roadmap.md`
  - `crates/ui/src/explorer_schema_objects_view.rs:43-77` at `7bbfb08e289fa2b77867d40feb0d886dbc097d31`
  - `crates/ui/src/explorer_schema_object_folders_view.rs:30-153` at `7bbfb08e289fa2b77867d40feb0d886dbc097d31`
- **Factual findings:** Baseline and Research share source baseline `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`; Research declares itself non-runtime evidence; current source applies the shared search query to Tables, Views, Functions and Triggers.
- **Inference:** The recommended ordering is a product/architecture decision derived from dependencies and safety constraints, not a measured runtime result.
- **Decision / recommendation:** Preserve the existing action/reducer pipeline; reconcile F1 first; deliver reliability and context before breadth.
- **Unresolved questions:** filter persistence ownership; exact filter modes; provider/object support matrix; compare selection lifetime; Diagram scope contract; metadata fields allowed in quick preview.
- **Downstream tasks activated:** none.

## 8. Review outcome

n/a — comparison report only; no independent review was requested or performed.

## 9. Implementation handoff

n/a — no product code, persisted state, migration or configuration was changed.

## 10. Tổng kết

Baseline mô tả chính xác hợp đồng action/reducer và các guard của Explorer tại SHA nghiên cứu; Feature Research bổ sung 11 nhóm capability nhưng không phải cam kết triển khai hoặc runtime evidence. F1 đã được source hiện tại triển khai một phần cho Views, Functions và Triggers, nên tài liệu table-only đã cũ. Hướng tiếp theo an toàn là hoàn thiện filter semantics, refresh trust và metadata preview trước khi mở rộng object coverage hoặc thêm mutation actions.