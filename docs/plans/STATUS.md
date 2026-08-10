# Schema Runtime Verification — Status

Canonical lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`.

| Feature | Branch | PR | State | Notes |
|---|---|---|---|---|
| S1 Columns | main | 741a18d | RUNTIME_VERIFY | implementation complete; PG + UI runtime evidence pending |
| S2 Indexes | main | aa77ece | RUNTIME_VERIFY | PR merged; PG + UI runtime evidence pending |
| S3 Relations | feature/schema-relations-runtime | #7 | RUNTIME_VERIFY | source + CI clean, P0=0 P1=0; MERGEABLE; PG + UI runtime pending |
| S4 Triggers | — | — | PLANNING | plan documents in progress |
| S5 DDL | — | — | BACKLOG | — |
| S6 ER Diagram | — | — | BACKLOG | existing implementation needs runtime normalization |
| S7 Full Schema Regression | — | — | BACKLOG | run after S1–S6 evidence closure |

## Rules

- A plan with pending runtime/provider evidence stays under `docs/plans/active/`.
- `COMPLETED` requires P0=0, P1=0 and all applicable provider/UI runtime evidence.
- PostgreSQL and SQLite are verified independently; one provider never proves the other.
