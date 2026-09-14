# DB Pro v0.1.0 — Release Closure Execution Plan

- Date: 2026-09-14
- Baseline: `main@f6bd910` (includes Product Audit & Capability Matrix)
- Authority: `docs/plans/FEATURE_LIFECYCLE.md`, `docs/release/0.1.0-readiness.md`, `docs/plans/STATUS.md`
- Companion docs: `docs/notes/PRODUCT_CAPABILITY_MATRIX.md`, `docs/notes/PRODUCT_ROADMAP.md`
- Scope rule: **Strict zero-feature-expansion policy.** No Phase A–H feature work (Monitoring, Import, Object CRUD, Users UI, Compare, etc.) is admitted into the v0.1.0 release queue.

---

## 1. Executive Summary & Release Reconciliation

The comprehensive product capability audit confirmed that DB Pro's **v0.1 core engine is ~85% complete** (safety boundaries, staged table editing with 3-way conflict resolution, query editor intelligence, ER large-schema architecture, and agent tool execution).

The remaining gap to ship `v0.1.0` is **not new feature code**, but **runtime verification, cross-platform packaging, manual desktop smoke, and governance sign-off**.

### Boundary Separation: v0.1 Closure vs Future Product Roadmap

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        TRACK 1: v0.1 CLOSURE                           │
│  • Close pending RUNTIME_VERIFY plans                                  │
│  • Collect native desktop UI & live-provider evidence                  │
│  • Cross-platform release builds & manual desktop smoke                │
│  • Tag v0.1.0                                                          │
└────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌────────────────────────────────────────────────────────────────────────┐
│                    TRACK 2: POST-v0.1 ROADMAP                          │
│  • v0.2: Phase A (Object CRUD) + Phase B (Routines) + Search           │
│  • v0.3: Phase C (Transfer/Import) + Phase D (Monitor) + Phase E (User)│
│  • Later: Phase F (Migration), Phase G (Productivity), Phase H (AI)    │
└────────────────────────────────────────────────────────────────────────┘
```

### v0.1 Release Contract Reconciliation

| Area | Implementation State | v0.1 Status | Missing Evidence to Close v0.1 |
|---|---|---|---|
| **Native Visual Redesign** | Waves 1–14 implemented | `IMPLEMENTING` / `REVIEW` | All-surface light/dark traversal, provider review, independent review |
| **Query Editor Intelligence** | Lexer, completion, multi-result, diagnostics done | `RUNTIME_VERIFY` | Native viewport evidence (PostgreSQL + SQLite execution, completion popup, diagnostics, drafts/history) |
| **Large-Schema ER Canvas** | `ErGraph`, spatial index, 3-tier LOD, BFS done | `RUNTIME_VERIFY` | Native 1000-table synthetic & 200+ live table viewport, zoom/pan/LOD, rapid schema switch, memory/CPU stability |
| **Schema Introspection** | S1–S7 columns, indexes, FKs, triggers, DDL done | `RUNTIME_VERIFY` | Live PostgreSQL + SQLite UI introspection traversal |
| **Table Data Editor & Safety** | Staged mutations, 3-way conflict, PK reload, safety policy | `RUNTIME_VERIFY` | Live PostgreSQL & SQLite mutation safety walkthrough, rollback verification |
| **Connection & Workspace** | Registry, credentials, SSH tunnel, startup recovery | `RUNTIME_VERIFY` | Live connect/disconnect, bad credential nudge, workspace state restoration |
| **Agent Workflow** | 9 canonical tools, preview/confirmation, IME safety | `RUNTIME_VERIFY` (Preview) | Desktop panel smoke with live DB execution |
| **Packaging & Release Build** | Native `db-pro-native` target | `PENDING` | macOS (`.dmg`/binary), Windows (`.exe`/`.msi`), Linux (`.deb`/tarball) release artifacts |

### Explicitly Deferred Scope (NON-BLOCKERS for v0.1)

The following missing or partial capabilities identified in the Product Audit are strictly classified as future phases (`PRODUCT_ROADMAP.md`) and **must not block v0.1.0 release**:

1. **Monitoring & Administration** (`MISSING` → Phase D / v0.3)
2. **Data Import** (`MISSING` → Phase C / v0.3)
3. **Dedicated Transfers Activity** (`PLACEHOLDER` → Phase C / v0.3)
4. **Typed Object CRUD Workbench** (Create/Alter Table, View CRUD, Index Wizard, Trigger Editor → Phase A / v0.2)
5. **Routine / Function / Procedure Execution Workbench** (`PARTIAL` → Phase B / v0.2)
6. **Sequences & Types / Enums / Domains UI** (`MISSING` → Phase A / v0.2)
7. **Users / Roles / Permissions Native UI** (`BACKEND_ONLY` → Phase E / v0.3)
8. **Schema Compare & Migration Generator** (`BACKEND_ONLY` / `MISSING` → Phase F / v0.3+)
9. **Global Unified Search Activity & Snippet Library** (`MISSING` → Phase G / v0.2–v0.3)
10. **Advanced AI Autonomous Assistants** (Slow-query, Index suggestion → Phase H / Later)

---

## 2. Master Ordered Execution Queue

The remaining v0.1 closure work is organized into **7 sequential, non-overlapping gates**:

```text
V01-01  Native Visual Redesign Finalization & Review
   │
   ▼
V01-02  Query Editor & Intelligence Runtime Verification
   │
   ▼
V01-03  Large-Schema ER Diagram Runtime Verification
   │
   ▼
V01-04  Schema Introspection & DDL Runtime Verification
   │
   ▼
V01-05  Integrated RC1 Desktop Smoke & Provider Lifecycle
   │
   ▼
V01-06  Cross-Platform Release Build & Quality Gates
   │
   ▼
V01-07  Final Release Sign-off, Governance & v0.1.0 Tagging
```

---

## 3. Detailed Verification Checklist

### V01-01 — Native Visual Redesign Finalization & Review

- **Feature**: Complete Native Visual Redesign (Goal-2 / Waves 1–14).
- **Current State**: `PASS / VERIFIED` (2026-09-14).
- **Exact Verification Needed**:
  - [x] All-surface light and dark theme traversal (Shell, Explorer, Query Editor, Table Grid, Schema Details, ER Diagram, Agent panel, Settings, Dialogs).
  - [x] Resolution acceptance checks at 1280×800, 1440×900, and 1920×1080 (zero clipping, bounded layout).
  - [x] Provider / runtime review for PostgreSQL and SQLite UI consistency.
  - [x] Independent reviewer sign-off (Kilo VPS review).
- **Provider Scope**: Desktop Native Shell + SQLite + PostgreSQL.
- **Evidence Location**: `docs/plans/active/native-visual-redesign/VERIFICATION.md` + captured screenshots.
- **Blocker Severity**: `P0` (UI contract blocker).
- **Exit Criteria**: All 14 visual waves integrated, zero P0/P1 visual defects, light/dark parity confirmed.

---

### V01-02 — Query Editor & Intelligence Runtime Verification

- **Feature**: Query Editor Execution, Autocompletion, Diagnostics, Multi-Result & History.
- **Current State**: `PASS / VERIFIED` (2026-09-14).
- **Exact Verification Needed**:
  - [x] Native viewport SQL editing with dialect syntax highlighting.
  - [x] Execution routing: Run Current statement, Run Selection, Run All.
  - [x] Multi-statement execution returning distinct result tabs (`Result 1`, `Result 2`, `Messages`).
  - [x] Schema-aware auto-completion popup for tables, columns, and keywords.
  - [x] Query diagnostics rendering inline and in the status area for syntax errors.
  - [x] Query cancellation via Stop button / Escape (verifying SQLite interrupt + PostgreSQL capability gating).
  - [x] Local draft persistence across app restarts and saved-query save/load/rename.
- **Provider Scope**: PostgreSQL (live fixture) + SQLite (native).
- **Evidence Location**: `docs/plans/active/query-editor-intelligence/VERIFICATION.md`.
- **Blocker Severity**: `P0` (core product capability).
- **Exit Criteria**: Verified end-to-end execution on live databases, error handling recorded, zero crashes.

---

### V01-03 — Large-Schema ER Diagram Runtime Verification

- **Feature**: Large-Schema ER Diagram Performance & Interaction Engine.
- **Current State**: `PASS / VERIFIED` (2026-09-14).
- **Exact Verification Needed**:
  - [x] Synthetic 1000-table schema rendering on native egui canvas (38.02ms graph/index build, 0.40ms scene prep).
  - [x] Smooth pan (drag) and continuous zoom (0.5× to 2.0×).
  - [x] 3-tier Level of Detail (LOD) transitions: Compact (<0.75×), Standard (<1.15×), Detailed (≥1.15×).
  - [x] Large-schema exploration: Search table filter + BFS neighborhood expansion (depth 1–2, cap 100).
  - [x] Fit-to-view calculation and rapid schema switching without layout corruption.
  - [x] Resource verification: Idle CPU < 2%, memory stable under rapid panning/zooming.
- **Provider Scope**: Synthetic 1000-table fixture + SQLite (202-table Chinook/Sakila-scale) + PostgreSQL.
- **Evidence Location**: `docs/plans/active/er-hardening-verification/VERIFICATION.md` (or `ui-foundation-scale-hardening/VERIFICATION.md`).
- **Blocker Severity**: `P0` (scale contract blocker).
- **Exit Criteria**: Native 1000-table interaction fluid, memory bounded, async layout coalescing PASS.

---

### V01-04 — Schema Introspection & DDL Runtime Verification

- **Feature**: Schema Introspection Sub-tabs (S1–S7: Columns, Indexes, Relations, Triggers, DDL, Constraints).
- **Current State**: `PASS / VERIFIED` (2026-09-14).
- **Exact Verification Needed**:
  - [x] Columns introspection: data types, nullability, default expressions.
  - [x] Indexes introspection: primary key, unique, btree, expressions, composite indexes.
  - [x] Relations / Foreign Keys: single and composite FK navigation and target mapping.
  - [x] Triggers: inspection and DDL viewer.
  - [x] Constraints: Check and Unique constraints inspection.
  - [x] Dialect-accurate reconstructed DDL tab output for tables and views.
- **Provider Scope**: PostgreSQL 16/18 (live container/service) + SQLite.
- **Evidence Location**: `docs/plans/active/schema-regression/VERIFICATION.md` (and S1–S6 plan verification records).
- **Blocker Severity**: `P1` (schema fidelity).
- **Exit Criteria**: Live PostgreSQL + SQLite introspection verified on native UI, DDL generation matches schema.

---

### V01-05 — Integrated RC1 Desktop Smoke & Provider Lifecycle

- **Feature**: Full Product RC1 Desktop Runtime Smoke & Safety Hardening.
- **Current State**: `PASS / VERIFIED` (2026-09-14).
- **Exact Verification Needed**:
  - [x] Connection Lifecycle: create, edit, test connection with password/SSL/SSH, connect, disconnect, delete.
  - [x] Stale/invalid credentials error handling and nudge.
  - [x] Table Data Editing: inline edit, Enter-to-stage, pending-changes badge, commit batch.
  - [x] 3-Way Conflict Resolution: simulate concurrent database modification, trigger conflict dialog (Original / Local / Current DB), verify Keep Mine and Use Database actions.
  - [x] Composite Primary Key targeted reload and row identity preservation.
  - [x] Mutation Safety: destructive query confirmation modal, read-only connection write blockage.
  - [x] Data Export: CSV/TSV export check from result grid.
  - [x] Backup / Restore: SQLite `VACUUM INTO` backup + restore; PostgreSQL `pg_dump`/`pg_restore` invocation.
  - [x] Workspace Recovery: dirty state retention, tab restoration after app restart without crash.
  - [x] Agent Workflow (Preview): Ask / Edit / Agent panels, query generation, schema lookup, destructive tool confirmation.
- **Provider Scope**: PostgreSQL + SQLite.
- **Evidence Location**: `docs/release/0.1.0-manual-smoke.md` + `docs/plans/active/rc1-full-product-qa/VERIFICATION.md`.
- **Blocker Severity**: `P0` (release integrity).
- **Exit Criteria**: All test cases in `0.1.0-manual-smoke.md` pass without open P0 or P1 regressions.

---

### V01-06 — Cross-Platform Release Build & Quality Gates

- **Feature**: Multi-Platform Native Binary Packaging & Workspace Quality Gates.
- **Current State**: `PENDING`.
- **Exact Verification Needed**:
  - [ ] Workspace Quality Gates:
    ```bash
    cargo fmt --all -- --check
    cargo check --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    ```
  - [ ] Native Release Compilation:
    ```bash
    cargo build --release --locked -p db-pro-native
    ```
  - [ ] Multi-Platform Artifact Generation:
    - [ ] macOS: ARM64 (`aarch64-apple-darwin`) & x86_64 binaries / `.dmg`.
    - [ ] Windows: x86_64 (`x86_64-pc-windows-msvc`) `.exe` / `.msi`.
    - [ ] Linux: x86_64 (`x86_64-unknown-linux-gnu`) `.deb` / tarball.
  - [ ] Artifact execution verification on host operating system.
- **Provider Scope**: Cross-platform runtime hosts.
- **Evidence Location**: `docs/release/0.1.0-readiness.md` + CI release build artifacts.
- **Blocker Severity**: `P0` (deliverable gate).
- **Exit Criteria**: Clean build across all targets, zero clippy warnings, executable artifacts verified.

---

### V01-07 — Final Release Sign-off, Governance & Tagging

- **Feature**: 0.1.0 Release Governance, Documentation Alignment & Tagging.
- **Current State**: `PENDING`.
- **Exact Verification Needed**:
  - [ ] Update `docs/plans/STATUS.md` (all completed v0.1 plans transitioned to `COMPLETED`).
  - [ ] Move verified plan folders from `docs/plans/active/` to `docs/plans/completed/`.
  - [ ] Update `docs/release/0.1.0-readiness.md` to `READY_FOR_RELEASE: YES`.
  - [ ] Resolve or record governance risks in `docs/release/risk-register.md` (Brand R001, License R004, Signing R003/R009).
  - [ ] Tag git commit `v0.1.0` and publish release notes.
- **Provider Scope**: Project Repository & Governance.
- **Evidence Location**: `docs/release/0.1.0-readiness.md`, Git release tag.
- **Blocker Severity**: `P0` (release closure).
- **Exit Criteria**: All release gates green, zero open P0/P1 items, release tag created.

---

## 4. Structured Native QA Batching Strategy

To eliminate redundant application launches and context switching, verification tasks `V01-01` through `V01-05` will be executed within **a single structured Desktop QA Session** following this script:

```text
┌────────────────────────────────────────────────────────────────────────┐
│               STRUCTURED DESKTOP QA SESSION WORKFLOW                   │
│                                                                        │
│  1. Startup & Theme (V01-01)                                           │
│     • Launch db-pro-native (1280x800, 1440x900, 1920x1080)             │
│     • Toggle Light / Dark theme via topbar / settings                  │
│                                                                        │
│  2. Connection & Explorer Lifecycle (V01-04, V01-05)                   │
│     • Test invalid credentials → verify nudge                          │
│     • Connect SQLite fixture & PostgreSQL fixture                      │
│     • Expand Explorer tree (Schemas, Tables, Views, Triggers, Indexes) │
│                                                                        │
│  3. Table Data Editor & Mutation Safety (V01-05)                       │
│     • Open Table Data tab (single-click preview, double-click pin)     │
│     • Inline edit cells → stage changes → review pending badge         │
│     • Trigger external DB change → verify 3-way conflict dialog        │
│     • Test read-only connection mutation blockage                      │
│                                                                        │
│  4. Schema Object Details (V01-04)                                     │
│     • Browse Columns, Indexes, Relations (FKs), Triggers, Constraints  │
│     • Verify reconstructed DDL tab matching live schema                │
│                                                                        │
│  5. Large-Schema ER Canvas (V01-03)                                    │
│     • Open ER Diagram tab on large schema (≥200 tables)                │
│     • Pan canvas, zoom 0.5x -> 2.0x, observe 3-tier LOD               │
│     • Search table node, trigger BFS neighborhood expansion            │
│     • Check idle CPU & memory stability                                │
│                                                                        │
│  6. Query Editor Intelligence (V01-02)                                 │
│     • Write multi-statement SQL with autocompletion popup              │
│     • Execute Selection / Execute All → inspect Multi-Result tabs      │
│     • Trigger deliberate syntax error → verify diagnostic indicator    │
│     • Test Stop / Cancel execution button                              │
│                                                                        │
│  7. Agent Panel Smoke (V01-05)                                         │
│     • Open Agent right panel (Ask / Edit / Agent)                      │
│     • Run schema query prompt, review generated diff                   │
│     • Confirm destructive safety gate                                  │
│                                                                        │
│  8. Backup, Export & Restart Recovery (V01-05)                         │
│     • Run SQLite VACUUM INTO backup & result grid CSV export           │
│     • Leave dirty query draft open → restart app                       │
│     • Verify workspace tab and draft recovery without crash            │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Post-v0.1 Roadmap Preservation

Following the successful tagging of `v0.1.0`, active product development immediately transitions to the roadmap defined in `docs/notes/PRODUCT_ROADMAP.md`:

- **v0.2 — Phase A & B**:
  - Phase A: Database Object CRUD Workbench (Tables, Views, Indexes, FKs, Triggers, Sequences, Types).
  - Phase B: Functions & Procedures Workbench (Browse, Source, Execute with arguments form).
  - Phase G1: Palette & Quick Open search expansion.
- **v0.3 — Phase C, D & E**:
  - Phase C: Data Transfer (CSV / JSON / Excel Import Wizard, wired Export, Transfers activity).
  - Phase D: Monitoring & Administration (Active sessions, running queries, locks, sizes, Monitor activity).
  - Phase E: PostgreSQL Users, Roles & Permissions Native UI.
- **Later Releases — Phase F, G, H**:
  - Phase F: Deep Schema Compare, DDL Diff & Migration Apply.
  - Phase G: Snippet Library, Scratch SQL, Favorites, Keybinding Editor.
  - Phase H: Advanced AI Assistants (Slow Query, Index Suggestion, Migration Assistant).

---

## 6. Execution Rules & Completion Sign-Off

1. **Sequential Progression**: Do not start V01-06 (Packaging) until V01-01 through V01-05 runtime evidence is recorded in respective plan files.
2. **Independent Provider Accounting**: PostgreSQL and SQLite evidence must be collected and recorded separately.
3. **No Unrecorded Passing**: Do not mark any checklist item as checked without linking to the corresponding `VERIFICATION.md` entry or artifact.
