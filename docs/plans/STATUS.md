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

## Rules

- A plan with pending runtime/provider evidence stays under `docs/plans/active/`.
- `COMPLETED` requires P0=0, P1=0 and all applicable provider/UI runtime evidence.
- PostgreSQL and SQLite are verified independently; one provider never proves the other.
