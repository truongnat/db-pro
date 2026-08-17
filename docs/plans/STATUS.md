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
| IT0-101 BIGINT Precision & Staged State | main | #11 (e5c4c9b) | COMPLETED | merged; i64 lossless IPC contract, staged-changes close guard, preview promotion, SQLite metadata; 1483 FE + 20 Rust tests PASS |

## P3 — UI Foundation & Scale Hardening

Pre-release hardening program. Blocks v0.1.

| Sub-program | State | Notes |
|---|---|---|
| P3.1 Design Token Contract | REVIEW | shadcn tokens alias --app-* canonical layer; drift check passes |
| P3.2 shadcn Integration Safety | REVIEW | `npm run check:tokens` detects drift; contract in globals.css |
| P3.3 ER Diagram Algorithm | REVIEW | Pre-indexed maps; benchmark 500t=16ms, 1000t=26ms |
| P3.4 ER Diagram Duplicate Layout | REVIEW | Single layoutGraph() in useMemo; edge highlight separated |
| P3.5 ER Diagram Rendering LOD | REVIEW | 3-tier zoom LOD; MiniMap disabled >200 nodes |
| P3.6 ER Diagram Large Schema Mode | REVIEW | Neighborhood BFS for 200+ tables; search-first default needs RC1 QA correction before release |
| P3.7 Performance Budgets | REVIEW | Fixtures at 20/100/500/1000; automated regression test |
| P3.8 Data Grid / Metadata List Audit | REVIEW | Explorer O(S×T) fixed; deeper RC1 QA found additional state/performance issues |
| **P1 Large-Schema ER Architecture** | **IMPLEMENTING** | locked architecture (graph model → layout worker → spatial index → viewport engine → renderer); P1.1 instrumentation + P1.2 culling done on `feature/er-large-schema-scaling` |

## RC1 Full Product QA

Audit baseline: `main@6e0a04ad675eaa85cae08bbe1a066270596a18db`  
Audit branch: `qa/rc1-static-audit`

| Program | State | P0 | P1 | P2 | Notes |
|---|---|---:|---:|---:|---|
| RC1 Full Product QA — Static Audit & Remediation | IMPLEMENTING | 0 | 3 | 25 | release-blocking findings recorded under `docs/plans/active/rc1-full-product-qa/`; W1, W2, W3 remediations complete |

Release rule for this QA program:

- no `v0.1.0` tag while confirmed P1 findings remain open;
- source evidence does not close provider/runtime checks;
- BIGINT precision, staged mutation identity, connection lifecycle, orphan recovery, large-schema ER and provider type mapping are highest-priority P1 waves;
- no new product/Agent/MCP feature work during RC remediation.

## Rules

- A plan with pending runtime/provider evidence stays under `docs/plans/active/`.
- `COMPLETED` requires P0=0, P1=0 and all applicable provider/UI runtime evidence.
- PostgreSQL and SQLite are verified independently; one provider never proves the other.
