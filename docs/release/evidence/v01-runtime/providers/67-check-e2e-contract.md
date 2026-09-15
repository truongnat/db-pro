# CHECK introspection end-to-end contract (#70)

## Claim

CHECK constraints travel provider → domain → Tauri DTO / native bridge → inspection UI without being dropped at the IPC or translate boundary.

## Mapping

| Layer | Location | Behavior |
|---|---|---|
| PostgreSQL | `crates/infrastructure/src/postgres/introspect.rs` `introspect_check_constraints` | Catalog rows → `CheckConstraint` |
| SQLite | `crates/infrastructure/src/sqlite/introspect.rs` (hardened in #69) | CREATE TABLE parser → `CheckConstraint` |
| Domain | `crates/core/src/domain/schema.rs` | `IntrospectResult.check_constraints`, `TableInfo.check_constraints` |
| Schema service | `crates/core/src/application/schema_service.rs` | Filters CHECK rows onto `TableInfo` |
| Tauri DTO | `crates/tauri-app/src/dto.rs` | `IntrospectResultDto.checkConstraints`, `TableInfoDto.checkConstraints` |
| Contract fixture | `crates/tauri-app/tests/fixtures/introspect-contract.json` | Includes representative CHECK row |
| Native bridge | `crates/native-app/src/translate.rs` `map_table_info` | → `UiCheckConstraint` |
| Native UI | `crates/ui/src/table_metadata_view.rs` | Constraints panel lists CHECK rows |

Archived React frontend is not part of the shipping contract; native egui is the consumer.

## Tests executed

```bash
cargo test -p db-pro-tauri introspect_result_dto --lib
cargo test -p db-pro-tauri table_info_dto_serializes_check_constraints --lib
cargo test -p db-pro-native map_table_info_preserves_check_constraints_for_native_ui
cargo test -p db-pro-infrastructure parse_check_constraints --lib
```

## Provider notes

- PostgreSQL / SQLite: SUPPORTED + QUALIFIED (matrix).
- MySQL: CHECK introspection still empty (PARTIAL); out of #70 scope.

## Residual

MySQL CHECK catalog support remains deferred; LIM-011 records that residual only.
