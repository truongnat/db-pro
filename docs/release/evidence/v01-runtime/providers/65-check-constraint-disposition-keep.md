# CHECK constraint disposition — KEEP for v0.1 (#68)

- Session: 2026-09-15
- Tree: `main @ 776f58ee` (audited before the follow-up #69 commit)
- Decision: **KEEP v0.1**
- Activates: **#69**, **#70**
- Closes as not planned: **#71**

## 1. Source inventory (exact paths)

| Layer | Path | Behavior |
|---|---|---|
| Domain | `crates/core/src/domain/schema.rs` `CheckConstraint` + `IntrospectResult.check_constraints` | Typed field exists |
| PostgreSQL | `crates/infrastructure/src/postgres/introspect.rs:682` | `pg_constraint` + `pg_get_constraintdef`, ordered |
| SQLite | `crates/infrastructure/src/sqlite/introspect.rs:323` + `parse_check_constraint_definitions` | CREATE TABLE SQL scan with quote/comment-aware depth |
| MySQL | `crates/infrastructure/src/mysql/introspect.rs:304` | Always empty (recorded PARTIAL) |
| Schema service | `crates/core/src/application/schema_service.rs:128` | Filters per-table into `TableInfo` |
| Native translate | `crates/native-app/src/translate.rs:823` | Maps to `UiCheckConstraint` |
| Native UI | `crates/ui/src/table_metadata_view.rs:41,736` | Renders CHECK rows in the constraints panel |
| Tauri DTO | `crates/tauri-app/src/dto.rs` | Serializes `checkConstraints` on introspect + TableInfo (completed in #70) |

## 2. Current behavior table

| Provider | Introspected | Shown in native UI | Risk |
|---|---|---|---|
| PostgreSQL | Yes (catalog) | Yes when `TableInfo` is loaded | Low — catalog-backed |
| SQLite | Yes (parser) | Yes when loaded | Medium — parser edge cases (#69) |
| MySQL | No (empty) | N/A | Documented PARTIAL |

## 3. Why KEEP (not DEFER)

1. The **shipping native UI already consumes CHECK** (`table_metadata_view.rs`). DEFER (#71) would require removing a live user-visible surface and rewriting docs to claim absence while the code path exists.
2. PostgreSQL path is catalog-backed and already release-grade.
3. SQLite residual risk is bounded and owned by **#69** (parser fixtures), not a reason to strip the feature.
4. Completing native exposure end-to-end (filter per table, empty-state honesty, any missing translate fields) is **#70** — smaller than removing the feature.

## 4. What this decision does *not* claim

- MySQL CHECK introspection (stays empty / PARTIAL).
- Perfect SQLite parsing of every dialect extension — #69 hardens the listed cases.
- Tauri/React CHECK DTO — frontend archived; native path is the shipping contract.

## 5. Docs updates in the same change

- `docs/release/known-limitations.md` LIM-011 → kept with residual SQLite parser caveat pointing at #69/#70.
- `docs/release/provider-capability-matrix.md` CHECK row → disposition KEEP.
