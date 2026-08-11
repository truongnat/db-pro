# S5 — DDL Normalization Plan

## Goal
Normalize DDL generation, viewing, and editing across all schema object types (tables, views, indexes, foreign keys, triggers) with provider-aware syntax.

## Problem
- View DDL is not available (get_table_ddl only handles tables, not views)
- Backend DDL builder uses hardcoded quote_identifier instead of dialect-aware quoting
- DDL editor missing trigger operations
- No comprehensive DDL test coverage

## Current Behavior
- `get_table_ddl()` generates CREATE TABLE + indexes + FKs + triggers for tables
- Selecting a view and clicking DDL tab calls `get_table_ddl()` which fails (views not in tables list)
- Frontend DDL builder (`ddl-builder.ts`) is dialect-aware, but backend is not
- DDL editor supports: createTable, addColumn, dropColumn, renameTable, dropTable, createView, dropView, createIndex, dropIndex

## Target Behavior
- View DDL shows the CREATE VIEW statement from introspection
- Backend DDL uses dialect-aware quoting where possible
- DDL editor includes trigger enable/disable operations
- All DDL paths have test coverage

## Scope
- Add `get_view_ddl` method to SchemaService
- Add view DDL Tauri command or extend `get_table_ddl` to handle views
- Add dialect parameter to `build_create_table_ddl` (or accept current quoting as compatible)
- Add trigger operations to DDL editor
- Add DDL tests

## Out of Scope
- DDL round-trip fidelity (introspection may not preserve exact original formatting)
- DDL migration between providers
- DDL diff/comparison

## Safety Requirements
- DDL viewer is read-only (no execution risk)
- DDL editor already enforces safety policy (readonly connections, multi-statement rejection)
- All raw SQL fragments must pass validate_raw_fragment

## Test Strategy
- Unit tests for view DDL generation
- Unit tests for dialect-aware quoting
- Frontend tests for DDL editor trigger operations

## Completion Criteria
- View DDL displays correctly for both PostgreSQL and SQLite
- All DDL paths have test coverage
- DDL editor supports trigger operations
- CI passing (Rust + Frontend)
