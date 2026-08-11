# S3 — Schema Relations Runtime Verification

State: REVIEW
Branch: `feature/schema-relations-runtime`

## Goal

Foreign-key relations must preserve constraint identity, ordered column mappings and database truth across introspection, DDL reconstruction and the relation UI.

```text
database FK constraint
→ provider introspection
→ relation rows with stable constraint identity
→ table info / reconstructed DDL
→ Foreign Keys UI
→ target-table navigation
```

## Baseline evidence

The existing model stores one `ForeignKey` row per column mapping. This is compatible with single-column relations but exposed a composite-FK correctness gap:

- PostgreSQL introspection returns multiple rows with the same real constraint name for a composite FK.
- SQLite previously synthesized the name from `from_column`, so each column mapping of one composite FK received a different identity.
- `build_create_table_ddl()` emitted one `ADD CONSTRAINT` statement per mapping; PostgreSQL composite FKs therefore reconstructed as duplicate constraint names instead of one multi-column constraint.
- `ForeignKeyList` rendered one row per mapping and keyed rows by `fk.name`, producing duplicate keys for PostgreSQL composite FKs.

## Scope

- preserve one stable constraint identity across all column mappings
- preserve composite mapping order
- reconstruct one multi-column FK constraint in generated DDL
- render one relation row in the Foreign Keys UI
- keep target-table navigation
- prove SQLite FK enforcement on a live in-memory provider
- independently account for PostgreSQL runtime evidence

## Non-goals

- create/drop FK editor UI
- automatic SQLite table rebuild for adding a relation
- ER diagram composite-edge presentation; tracked under S6
- ON UPDATE / ON DELETE action editing
- deferrable constraint editing

## Implementation

### SQLite introspection

Use `PRAGMA foreign_key_list` `id` as the shared synthetic constraint identity and preserve the provider's `seq` row order. Composite mappings now share `table_fk_<id>`.

### DDL reconstruction

Group introspected FK rows by constraint identity + target table before generating DDL. Emit one statement:

```sql
ALTER TABLE ...
ADD CONSTRAINT ...
FOREIGN KEY (col_a, col_b)
REFERENCES ... (target_a, target_b);
```

### Frontend relation view

Normalize scalar mapping rows with `groupForeignKeys()` and render joined ordered columns as one relation row.

## Provider matrix

| Provider | Introspection source | Automated evidence | Live/runtime evidence |
|---|---|---|---|
| PostgreSQL | real constraint name + `src.ord` ordering | source/unit coverage | PENDING |
| SQLite | PRAGMA `id` + `seq` | test added, execution pending | PENDING until command executes |

## Acceptance criteria

- [x] composite SQLite mappings share one stable constraint identity in source
- [x] DDL reconstruction groups composite mappings into one constraint in source
- [x] Foreign Keys UI groups composite mappings into one relation row in source
- [x] regression tests added for grouping and DDL reconstruction
- [x] SQLite runtime test added for composite introspection + FK enforcement
- [ ] Rust/frontend quality gates actually executed
- [ ] PostgreSQL composite FK runtime evidence recorded
- [ ] UI relation list/navigation runtime evidence recorded
- [ ] P0 = 0
- [ ] P1 = 0

Until execution/review evidence is available S3 stays in `REVIEW` and must not be marked completed.
