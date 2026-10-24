---
schema_version: 1
id: "MEM-AUDIT-fix-trigger-lifecycle-12edd4702e81"
title: "Plan archive: Audit Fix Trigger Lifecycle"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, audit-fix-trigger-lifecycle]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "12edd4702e811da34938c6f8f44e9785a19688c114c31214255504d5965c1eac"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/_audit-2026-10-13/fix-trigger-lifecycle.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ audit slice `docs/plans/_audit-2026-10-13/fix-trigger-lifecycle.md`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

# Fix — Trigger lifecycle (audit slice 01, P1)

Date: 2026-10-13. Adds table-trigger enable/disable to the native schema mutation path and wires introspected state to Explorer rows.

## Implementation

**Core (`db-pro-core`)**
- `application/ddl_builder.rs` — `build_set_trigger_enabled` generates PostgreSQL `ALTER TABLE <qualified table> ENABLE|DISABLE TRIGGER <quoted name>` using the dialect identifier quote builder for schema, table, and trigger name.
- `application/object_mutation_builders.rs` — `(Trigger, Enable|Disable)` now uses the builder.
- `application/object_mutation_service.rs` — SQLite, MySQL, SQL Server, and unknown providers return an explicit unsupported reason before statement generation; PostgreSQL remains supported. Unsupported plans contain no SQL.
- `domain/capabilities.rs`, `domain/capability_presets.rs`, `domain/capability_limitations.rs` — add `TriggerEnableDisable` capability; enabled only for PostgreSQL. SQLite, MySQL, and SQL Server have provider-specific reasons. The new serialized boolean defaults to false for older capability payloads.

**PostgreSQL introspection (`db-pro-infrastructure`)**
- `postgres/introspect.rs` — maps `tgenabled` `D` to disabled and `O`/`R`/`A` to enabled; explicitly excludes `tgisinternal` triggers while retaining the existing system-schema filters.

**Native UI (`db-pro-ui`)**
- `explorer_schema_object_row_view.rs`, `explorer_schema_object_folders_view.rs` — trigger rows display Enabled/Disabled from `UiTriggerSummary.enabled`; context menu offers the inverse action only when the capability is supported, otherwise it shows a disabled reason.
- `explorer_view.rs`, `explorer_surface_view.rs`, `explorer_schema_tree_view.rs`, `explorer_schema_objects_view.rs` — thread the provider capability reason to the trigger row.
- `explorer_folders.rs` — checks capability again when handling the action, builds a typed `ObjectMutationRequest`, plans through `ObjectMutationService`, and opens Schema Workbench preview/apply. DDL execution remains on `UiCommand::ExecuteDdl` through the existing workbench flow; drop confirmation is unchanged.
- Existing `runtime/src/api.rs` and `native-app/src/translate_evt.rs` already propagate `enabled` into `UiTriggerSummary`; the row now consumes that field.
- `app_tests.rs` — asserts PostgreSQL support and unsupported/reason behavior for SQLite, MySQL, and SQL Server.

**Plan/report updates**
- `docs/plans/active/schema-triggers-runtime/PLAN.md` documents exact PG syntax, mode mapping, internal-trigger filtering, and unsupported provider gates.
- `docs/plans/active/schema-triggers-runtime/VERIFICATION.md` now distinguishes source implementation from pending verification and marks historical CI results as predating this change.
- `docs/plans/_audit-2026-10-13/01-schema-safety-providers.md` records the findings as implemented on source with evidence pending.

## Tests added

- `ddl_builder.rs::pg_trigger_enable_disable_quotes_identifiers` covers enable/disable SQL and identifier escaping.
- `object_mutation_service.rs::plans_postgres_trigger_enable_and_disable_with_quoted_identifiers` covers PostgreSQL statements; `sqlite_and_mysql_trigger_toggle_plans_emit_no_sql` proves both unsupported providers return empty statements and clear reasons.
- `postgres/introspect.rs::postgres_trigger_enabled_state_maps_all_catalog_modes` covers disabled `D` and enabled `O`, `R`, `A` states.
- `app_tests.rs::trigger_toggle_capability_is_postgres_only_with_provider_reason` covers supported/unsupported UI capability decisions.
- `explorer_schema_object_folders_view.rs::trigger_toggle_menu_offers_only_supported_inverse_action` covers visible Enable/Disable menu labels and capability gating.

## Verification status

No checks were run by this worker. Focused tests, `cargo check -p db-pro-core -p db-pro-infrastructure -p db-pro-ui`, `cargo fmt --check -p db-pro-core -p db-pro-infrastructure -p db-pro-ui`, and (if runtime changes are later made) the runtime check remain to be run by the integrating agent. No live PostgreSQL fixture or native UI capture was run.

## Remaining limitations

- SQL Server deliberately remains unsupported for table-trigger toggling; its UI explains this and the mutation planner rejects the operation without SQL.
- SQLite and MySQL remain unsupported for individual trigger toggling; the UI does not expose an executable action and the planner emits no SQL.
- Live PostgreSQL behavior (including disabled-state round-trip) and native UI interaction still need fixture/runtime evidence.

