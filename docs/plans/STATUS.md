# Schema Runtime Verification — Status

Canonical lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`.

| Feature | Branch | PR | State | Notes |
|---|---|---|---|---|
| S1 Columns | main | 741a18d | RUNTIME_VERIFY | implementation complete; PG introspection via CI; UI evidence pending |
| S2 Indexes | main | aa77ece | RUNTIME_VERIFY | PR merged; PG introspection via CI; UI evidence pending |
| S3 Relations | main | #7 (7facb95) | RUNTIME_VERIFY | merged; composite FK identity + DDL + UI grouping; CI integrated PASS; PG live + UI pending |
| S4 Triggers | main | #8 (7facb95) | RUNTIME_VERIFY | merged; introspection + lifecycle + DDL via CI; enable/disable not yet exercised in live PG |
| S5 DDL | main | #8 (7facb95) | RUNTIME_VERIFY | merged; view DDL + dialect quoting + trigger DDL ops; CI integrated PASS |
| S6 ER Diagram | main | #9 (89f11a9) | RUNTIME_VERIFY | merged; schema-level workspace tab; explicit schema prop; composite FK edge grouping; position persistence; workspace migration v2→v3 |
| S7 Full Schema Regression | main | #9 (89f11a9) | RUNTIME_VERIFY | merged; regression matrix complete; 39 Rust + 1324 FE tests; CI integrated PASS |

## P3 — UI Foundation & Scale Hardening

Pre-release hardening program. Blocks v0.1.

| Sub-program | State | Notes |
|---|---|---|
| P3.1 Design Token Contract | IMPLEMENTING | shadcn tokens now alias --app-* canonical layer |
| P3.2 shadcn Integration Safety | IMPLEMENTING | Contract documented in globals.css; CI check pending |
| P3.3 ER Diagram Algorithm | IMPLEMENTING | Pre-indexed maps replace O(T×C) filter scans |
| P3.4 ER Diagram Duplicate Layout | IMPLEMENTING | Single layoutGraph() call in useMemo |
| P3.5 ER Diagram Rendering LOD | IMPLEMENTING | 3-tier zoom LOD: name-only / count / full columns |
| P3.6 ER Diagram Large Schema Mode | IMPLEMENTING | Neighborhood mode with 2-hop FK BFS for 200+ tables |
| P3.7 Performance Budgets | PLANNING | No benchmark fixtures or budgets defined |
| P3.8 Data Grid / Metadata List Audit | PLANNING | Audit of O(N) scan patterns pending |

## Rules

- A plan with pending runtime/provider evidence stays under `docs/plans/active/`.
- `COMPLETED` requires P0=0, P1=0 and all applicable provider/UI runtime evidence.
- PostgreSQL and SQLite are verified independently; one provider never proves the other.
