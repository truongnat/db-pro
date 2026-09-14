# DB Pro — Product Roadmap Notes: Sidebar & DBeaver-Parity

Updated: 2026-09-14

Purpose: keep a single durable note for the long-term product direction so work on hardening/verification does not make us forget the broader database-client feature surface.

# 0. Audit-grounded companions (2026-09-14)

This note is the original product vision and is retained as-is. It is now
grounded by a code-evidence audit on `feature/product-audit-roadmap`:

- `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` — master capability matrix
  (Area × Feature × PostgreSQL × SQLite × Backend × Native UI × Safety ×
  Tests × Runtime Evidence × Status × Priority × Target Phase), including
  §16 evidence conflicts where code and release docs disagree.
- `docs/notes/PRODUCT_ROADMAP.md` — adjusted phase plan (A–H), release
  boundaries (v0.1/v0.2/v0.3/Later), FINAL sidebar proposal, 20-milestone
  implementation order, and tech-debt list.

## Authority and goal documents (added 2026-09-14)

**`docs/goals/goal-full-product.md` is the authority for long-term product direction.** It owns
the vision and binding non-goals, the final Information Architecture / Activity Bar, the
architecture principles (including the shared UI component contract), feature areas A–L with
full field sets, the release roadmap, the 39-milestone breakdown, the provider capability matrix,
the safety model, the testing/verification strategy, and exit criteria. This parity note remains
the **vision and rationale record** — it explains *why* DB Pro is shaped this way; the master goal
decides *what* ships and *when*. Where the two differ, the master goal wins.

Execution goals (implementation-ready, one per phase):

- `docs/goals/goal-phase-a-object-crud.md`
- `docs/goals/goal-phase-b-routines.md`
- `docs/goals/goal-phase-c-transfer.md`
- `docs/goals/goal-phase-d-monitoring.md`
- `docs/goals/goal-phase-e-security.md`
- `docs/goals/goal-phase-f-compare-migration.md`
- `docs/goals/goal-phase-g-productivity.md`
- `docs/goals/goal-phase-h-ai.md`

The sections below keep their original wording as the vision record. Two points where the goals
work superseded this note's wording:

- §3.4's component/pattern names are realized by the contract in the master goal §4.5 (real
  implementations today: `components/diff.rs::DiffViewer`, `components/explain.rs::ExplainPlanTree`,
  `components/tree.rs::DatabaseTreeNode`, `components/form.rs::FormField`,
  `components/input.rs::SearchInput`, `components/chrome.rs::EmptyState`,
  `components/dialog.rs::Dialog`, `components/agent_primitives.rs::{ExecutionApproval, RiskLevel}`,
  `result_grid.rs`/`result_grid_view.rs`, `palette_view.rs`).
- §5's long-term surface and §1's Activity Bar are implemented exactly as listed in the master
  goal §3.1, with two additions the audit justified: **Security** stays an Explorer node rather
  than a rail icon, and **Data** enters the rail only when Phase C gives it real content.


## Product direction

DB Pro should evolve toward:

> **DBeaver-class database client + Codex/VS Code-style native UX + AI-native workflow.**

The current implementation is already strong in the engine/core areas: Query Editor, Table Data Editor, Agent Workflow, ER Diagram, schema inspection, connections, and native UI foundation. The remaining gap is mostly broader database-client feature coverage, administration tooling, object CRUD, productivity, and release/runtime verification.

## Current core status

- Core Safety Hardening — RUNTIME_VERIFY
- Table Data Editor Hardening — RUNTIME_VERIFY
- Query Editor Intelligence Layer — RUNTIME_VERIFY
- Agent Workflow — RUNTIME_VERIFY
- P1 Large-Schema ER Architecture — RUNTIME_VERIFY
- Native UI Foundation — COMPLETED
- Native IDE Redesign — COMPLETED
- Native Visual Redesign — IMPLEMENTING
- RC1 Full Product QA — RUNTIME_VERIFY, P0=0, P1=0, P2 remaining

This means the database/query/agent engine is largely implemented; future work should not be mistaken for “more core editor work.” The bigger remaining surface is the rest of the database IDE/product.

---

# 1. Sidebar / Activity Bar target

Do not put every database object or feature as a top-level icon. Use a compact Activity Bar plus contextual sidebars, similar to VS Code/Codex.

Recommended top-level Activity Bar:

```text
▣ Explorer
⌕ Search
⌘ Queries
◫ Data
◇ ER
✦ Agent
▥ Monitoring
⇄ Transfer
⚙ Settings
```

Potential later entries if justified:

```text
♙ Security
▶ Tasks / Jobs
★ Favorites
```

## Explorer contextual sidebar

```text
CONNECTIONS

Local PostgreSQL
 ├─ Databases
 │   └─ postgres
 │       ├─ Schemas
 │       │   └─ public
 │       │       ├─ Tables
 │       │       ├─ Views
 │       │       ├─ Materialized Views
 │       │       ├─ Functions
 │       │       ├─ Procedures
 │       │       ├─ Sequences
 │       │       └─ Types
 │       └─ Security
 │           ├─ Users
 │           └─ Roles
```

## Monitoring sidebar

```text
MONITORING

Overview
Active Queries
Sessions
Locks
Transactions
Database Size
Table Statistics
```

## Transfer sidebar

```text
DATA TRANSFER

Import
Export
Backup
Restore
Schema Compare
Data Compare
Migrations
```

## Queries sidebar

```text
QUERIES

Open Queries
Saved Queries
Query History
Snippets
Scratch SQL
Favorites
```

## Agent sidebar

```text
AGENT

Ask
Edit
Agent mode
Recent Sessions
Tool Activity
Pending Changes / Confirmations
```

---

# 2. Sidebar feature inventory

| Sidebar / Area | Current direction | Remaining / later work |
|---|---|---|
| Connections | Strong foundation | folders/tags, duplicate, import/export connections, richer favorites |
| Database Explorer | Implemented foundation | richer context actions, favorites, global object search |
| SQL Editor | Strong | snippets/templates, script manager, dedicated saved-query UX |
| Tables | Strong | complete schema/table mutation surface |
| Views | Mostly inspection | create/edit/drop view, materialized-view support |
| Indexes | Inspection/introspection strong | create/edit/drop wizard |
| Relations / FK | Inspection/introspection strong | create/edit/drop relations |
| Triggers | Lifecycle/DDL exists | native trigger editor and fuller provider/runtime qualification |
| Functions / Procedures | Major gap | browser, source editor, create/edit/drop, execute/debug-oriented UX |
| Sequences | Gap | inspect/create/edit/drop |
| Types / Enums / Domains | Gap | PostgreSQL type management |
| Users & Roles | Partial | role editor, grants, memberships, permission matrix |
| ER Diagram | Strong | optional future edit/design mode |
| Query History | Foundation exists | first-class sidebar, filters/search/favorites |
| Saved Queries | Foundation exists | folders/tags/favorites/workspaces |
| AI Agent | Strong foundation | session/history/context management, advanced workflows |
| Database Tools | Partial | backup/restore/import/export/maintenance center |
| Monitoring | Major gap | sessions, locks, active queries, transactions, sizes/stats |
| Jobs / Tasks | Gap | scheduled SQL/scripts/tasks |

---

# 3. DBeaver-parity feature groups

## 3.1 Database Object Management

This is one of the largest remaining product gaps.

```text
Tables
├─ Columns
├─ Constraints
│  ├─ Primary Key
│  ├─ Foreign Key
│  ├─ Unique
│  └─ Check
├─ Indexes
├─ Triggers
└─ DDL

Views
Materialized Views
Functions
Procedures
Sequences
Types
Enums
Domains
```

Target capability for each supported object:

- Inspect metadata
- View DDL
- Create
- Edit / alter
- Drop with safety confirmation
- Refresh targeted metadata
- Open dependencies/references
- Generate SQL instead of always executing directly

Provider capability must be explicit. PostgreSQL and SQLite should never be assumed equivalent.

## 3.2 Database Administration / Monitoring

A major difference between a powerful SQL client and a real database IDE/admin tool.

```text
Administration
├─ Sessions
├─ Active Queries
├─ Locks
├─ Transactions
├─ Cancel Query
├─ Kill Session
├─ Database Size
├─ Table Size
├─ Statistics
├─ Vacuum / Analyze
└─ Server Information
```

Potential future provider-specific sections:

- PostgreSQL replication/status
- connection pool/session inspection
- extension inventory
- long-running query diagnostics
- blocking/blocked session graph

## 3.3 Import / Export / Transfer

First-class data-transfer UX:

```text
Data Transfer
├─ CSV → Table
├─ Excel → Table
├─ JSON → Table
├─ Table → CSV
├─ Table → Excel
├─ Table → JSON
├─ Table → SQL INSERT
├─ Table → SQL COPY
└─ Database → Database
```

Later:

```text
Schema Compare
Data Compare
Migration Preview
Generate Migration SQL
Apply Migration
```

Important requirements:

- preview before import
- column mapping
- type mapping
- null/default handling
- transaction/rollback policy
- conflict strategy
- progress/cancellation
- bounded memory/streaming for large datasets

## 3.4 Security / Users / Roles

```text
Security
├─ Users
├─ Roles
├─ Memberships
├─ Object Grants
├─ Schema Permissions
├─ Table Permissions
└─ Role DDL
```

Need provider-specific capability gating and safe SQL generation.

## 3.5 Productivity Layer

This is where DB Pro can feel better than traditional database clients.

```text
Workspace
├─ Favorites
├─ Recent
├─ Saved Queries
├─ Query History
├─ Snippets
├─ Scratch SQL
├─ Pinned Objects
└─ Global Search
```

### Global Search / Command Palette

A first-class `⌘K` / command-palette style surface should search:

- connections
- databases/schemas
- tables
- columns
- views
- functions/procedures
- saved queries
- query history
- commands/actions
- Agent actions

Example commands:

```text
> Open table users
> New query on production
> Explain current SQL
> Search column customer_id
> Export current result
> Open active queries
```

## 3.6 AI-Native Layer

DB Pro should not stop at “chat that writes SQL.”

```text
AI
├─ Ask Database
├─ Generate SQL
├─ Explain SQL
├─ Fix SQL
├─ Optimize SQL
├─ Explain Query Plan
├─ Find Schema
├─ Generate Migration
├─ Compare Schemas
├─ Analyze Slow Query
├─ Analyze Data
└─ Agent
```

Future Agent workflows can reuse the existing typed tool architecture rather than adding ad-hoc chat features.

Potential future tools:

- inspect active sessions/locks
- inspect query plan
- compare schema
- generate migration patch
- inspect sampled data
- analyze slow queries
- generate indexes with explicit confirmation

No autonomous destructive operations.

---

# 4. Feature groups to prioritize after current verification work

Recommended order after Native Visual Redesign and pending runtime verification are closed:

## Phase A — Database Object CRUD

1. Tables/columns/constraints full create/edit/drop
2. Views + Materialized Views
3. Index create/edit/drop
4. Relation/FK create/edit/drop
5. Trigger editor
6. Sequences
7. Types / Enums / Domains

## Phase B — Functions & Procedures

High priority because this is a major DBeaver-class capability gap.

- Function/procedure explorer
- source/DDL view
- create/edit
- parameter metadata
- execute with argument form
- result/output rendering
- dependencies
- Agent-aware explain/edit flow

## Phase C — Data Transfer

- CSV import/export
- Excel import/export
- JSON import/export
- SQL INSERT/COPY generation
- import mapping/preview
- progress + cancellation

## Phase D — Monitoring / Administration

- active sessions
- running queries
- locks
- transactions
- cancel/kill
- DB/table sizes
- statistics
- vacuum/analyze

## Phase E — Users / Roles / Permissions

- users
- roles
- memberships
- grants
- schema/table privileges

## Phase F — Compare / Migration

- schema compare
- DDL diff
- migration preview
- migration SQL generation
- safe apply
- later: data compare

## Phase G — Productivity

- global search / command palette
- saved-query folders/tags
- snippets
- scratch SQL
- favorites/recent/pinned objects

## Phase H — Advanced AI

Build only on top of stable typed tools:

- optimize slow query
- suggest indexes
- schema migration assistant
- schema compare assistant
- administration assistant
- data analysis assistant

---

# 5. Suggested long-term product surface

```text
DB Pro
│
├─ Explorer
│  ├─ Connections
│  ├─ Schemas
│  ├─ Tables
│  ├─ Views
│  ├─ Functions
│  ├─ Procedures
│  ├─ Sequences
│  ├─ Types
│  └─ Security
│
├─ Search
│  └─ Global object / command search
│
├─ Queries
│  ├─ Open Queries
│  ├─ Saved Queries
│  ├─ History
│  ├─ Snippets
│  └─ Scratch SQL
│
├─ Data
│  ├─ Table Data Editor
│  ├─ Results
│  └─ Data Transfer
│
├─ ER
│  ├─ Diagram
│  ├─ Search / Neighborhood
│  └─ Future Design Mode
│
├─ Agent
│  ├─ Ask
│  ├─ Edit
│  ├─ Agent
│  └─ Tool Activity
│
├─ Monitoring
│  ├─ Sessions
│  ├─ Queries
│  ├─ Locks
│  ├─ Transactions
│  └─ Stats / Size
│
├─ Transfer
│  ├─ Import
│  ├─ Export
│  ├─ Backup
│  ├─ Restore
│  ├─ Schema Compare
│  └─ Migration
│
└─ Settings
   ├─ Connections
   ├─ Editor
   ├─ AI Providers
   ├─ Appearance
   ├─ Keybindings
   └─ Advanced
```

---

# 6. What not to do

- Do not turn the Activity Bar into a list of 20 database object icons.
- Do not rebuild separate one-off UI systems for each object type; reuse native workbench patterns.
- Do not couple provider-specific functionality directly to renderer/UI logic.
- Do not bypass existing safety/confirmation/query runtime for Agent-generated operations.
- Do not add large new feature waves while RC/runtime verification is incomplete unless explicitly reprioritized.
- Do not try to achieve literal DBeaver parity in one release; prioritize the feature clusters that make DB Pro feel like a complete database IDE.

---

# 7. Product positioning checkpoint

Current DB Pro is strongest in:

- native database workspace
- SQL/query intelligence
- table data editing
- AI Agent orchestration
- ER visualization/scaling
- database safety architecture

Major remaining gaps versus mature tools such as DBeaver:

1. broader database-object CRUD
2. functions/procedures
3. administration/monitoring
4. data transfer/import workflows
5. users/roles/permissions
6. compare/migration tooling
7. productivity surfaces (global search/snippets/favorites)
8. packaging/runtime qualification and final release polish

The goal is not to copy DBeaver UI. The goal is to reach similar database capability coverage while keeping DB Pro faster, more focused, native, and AI-first.
