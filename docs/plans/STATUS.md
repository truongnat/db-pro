# Schema Runtime Verification — Status

Canonical lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`.

| Feature | Branch | PR | State | Notes |
|---|---|---|---|---|
| S1 Columns | main | 741a18d | RUNTIME_VERIFY | implementation complete; PG introspection via CI; UI evidence pending |
| S2 Indexes | main | aa77ece | RUNTIME_VERIFY | PR merged; PG introspection via CI; UI evidence pending |
| S3 Relations | feature/schema-relations-runtime | #7 | RUNTIME_VERIFY | source + CI clean, P0=0 P1=0; MERGEABLE; PG FK introspection via CI |
| S4 Triggers | feature/schema-triggers-runtime | #8 | RUNTIME_VERIFY | all impl + tests done; PG + SQLite runtime evidence via CI; MERGEABLE |
| S5 DDL | feature/schema-triggers-runtime | #8 | REVIEW | all Cubic CR1–CR5 FIXED; CI passing; MERGEABLE |
| S6 ER Diagram | feature/schema-triggers-runtime | #8 | REVIEW | composite FK grouping + edge identity + Cubic CR1/CR2 FIXED; CI green |
| S7 Full Schema Regression | feature/schema-triggers-runtime | #8 | IMPLEMENTING | regression matrix complete; 3 PG gap-filling tests added; readiness report done |

## Rules

- A plan with pending runtime/provider evidence stays under `docs/plans/active/`.
- `COMPLETED` requires P0=0, P1=0 and all applicable provider/UI runtime evidence.
- PostgreSQL and SQLite are verified independently; one provider never proves the other.
