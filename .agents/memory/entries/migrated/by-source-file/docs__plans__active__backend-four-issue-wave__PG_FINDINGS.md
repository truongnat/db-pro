---
schema_version: 1
id: "MEM-SRC-b15d4593c79c7adc"
title: "Source mirror: docs/plans/active/backend-four-issue-wave/PG_FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/backend-four-issue-wave/PG_FINDINGS.md"
source_sha256: "b15d4593c79c7adc5fd16871fdb02489306adddda9d2592768dc0008cfd3395f"
mirror_body_sha256: "b15d4593c79c7adc5fd16871fdb02489306adddda9d2592768dc0008cfd3395f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# PostgreSQL findings — #499 / #508

Baseline source: `bceb735727f81180dad678defee1000bc547d97b`. Candidate changes are uncommitted work on that baseline; runtime/gate execution belongs to Main and is **NOT RUN** by PgBuilderDependencies. No ticket closure is claimed here.

## #499 — zero-argument event-trigger functions

Baseline `crates/core/src/domain/event_trigger.rs:59-70,87-106` validates a character whitelist rather than components, allowing `func(,)`, `func()()`, digit-leading components and empty dotted components. It pastes a bare accepted name without the mandatory `()`.

Official PostgreSQL 17 grammar is `EXECUTE { FUNCTION | PROCEDURE } function_name()`; the referenced function takes no arguments and returns `event_trigger`. Only superusers may create event triggers. PostgreSQL folds unquoted identifiers to lowercase. Sources read 2026-10-16:

- https://www.postgresql.org/docs/17/sql-createeventtrigger.html
- https://www.postgresql.org/docs/17/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS

Candidate: strip at most one optional terminal `()`, validate one or two components with the existing `validate_ident`, lowercase accepted unquoted components, quote with existing `quote_ident`, and always emit one `()`. No parser/dependency added. Names remain the existing simple ASCII identifier subset; quoted/Unicode/dollar-sign function input is not newly supported. Each component is limited to 63 bytes, including the valid two-63-byte-components-plus-`()` case. Event/tag handling, trigger-name case behavior, and create/alter/drop confirmation boundaries are unchanged.

Consumer trace at baseline: app `tools/event_trigger_activity_view.rs:59,73` invokes the core preview; infrastructure `postgres/event_trigger.rs:87-103` invokes the same builder only after confirmation. No caller migration is necessary.

Permanent validation tests: `accepts_simple_zero_argument_function_references`, `rejects_arguments_and_malformed_function_references`, `rejects_overlong_function_identifier_components` under `db_pro_core::domain::event_trigger::tests`. Existing SQL-preview expectation is updated only for the necessary quoting change; the real-provider test, not a new SQL-string assertion, proves SQL validity.

Real-provider selector: `pg_event_trigger_function_refs_create_fire_and_introspect` in `crates/infrastructure/tests/pg_object_identity.rs`. Six references cover bare, bare `()`, qualified, qualified `()`, visible public function, non-visible schema function, and uppercase unquoted components. The public service creates each trigger, introspects its chosen function signature/event/tags/enabled status, creates a table, and verifies one recorded execution per probe. Unconfirmed create/alter/drop remain validation errors.

Exact public smoke calls:

1. `PostgresEventTriggerPort::new(Arc<PostgresConnector>)` using a connected handle.
2. `create_event_trigger(&handle, trigger, "ddl_command_end", function_ref, "CREATE TABLE", true)` → `Ok(())` for the six supported forms; `false` → `DbError::Validation`.
3. `inventory(&handle, connection_id)` → trigger has selected function's `regprocedure` signature ending `()`, event `ddl_command_end`, enabled `O`, tags `["CREATE TABLE"]`.
4. `DbConnector::execute(&handle, "CREATE TABLE <fixture-schema>.probe_N (id integer)", &[])` → function inserts exactly one matching object identity into fixture hits.
5. `drop_event_trigger(&handle, trigger, true)` → `Ok(())`; malformed refs fail preview/service validation before DDL.

## #508 — relation identity, not display-name patterns

Baseline `crates/infrastructure/src/postgres/cross_connection.rs:10-45` accesses `(d.objid::regclass).relnamespace` as though an OID alias were a catalog row, and filters displayed source names with `LIKE schema.object || '.%'`. Display names may omit schema; `%`, `_`, dots and quoting change matching. These are source findings, **not an observed runtime failure**; Main owns before/after probes against the exported baseline.

Actual consumer at baseline: `PostgresApi::object_dependencies` (`crates/runtime/src/api.rs:1211-1222`) forwards `(schema, object_name)` unchanged through `PostgresConnector::get_object_dependencies` (`connector.rs:1002-1017`) to the catalog helper. The helper selects **outgoing** dependencies of the requested `pg_class` relation. It is not reverse-impact traversal; view rewrite rules, foreign-key constraint rows and table-trigger rows in other source catalogs are intentionally not added. No UI callsite of this API was found by exact symbol scan. `ObjectDependency`'s four string fields remain unchanged.

Candidate: select `pg_class`/`pg_namespace` by two separately bound exact names; join `pg_depend` on relation OID plus `d.classid = 'pg_catalog.pg_class'::regclass`; retain `deptype IN ('n','a')`. Names never become raw SQL. Missing schema/relation returns `Ok(vec![])`; empty names do not acquire search-path meaning. Existing relation/function target display remains `regclass`/`regproc`; other target classes use PostgreSQL's native `pg_identify_object(...).identity` instead of meaningless numeric OID text, with no invalid cast into a different catalog. Result ordering adds target keys/subobject IDs for stable rows.

Official sources read 2026-10-16:

- https://www.postgresql.org/docs/17/catalog-pg-depend.html — `classid/objid` identify the dependent, `refclassid/refobjid` identify its target, and `n`/`a` keep existing semantics.
- https://www.postgresql.org/docs/17/functions-info.html#FUNCTIONS-INFO-OBJECT-TABLE — `pg_identify_object` supplies machine-readable, nontranslated identity for catalog addresses.
- https://raw.githubusercontent.com/postgres/postgres/REL_17_STABLE/src/backend/catalog/objectaddress.c — type identities are force-qualified; namespace identities quote when necessary. This informed consumer-visible expected fixture rows, not string-pinning tests.

Real-provider selector: `pg_relation_dependencies_use_exact_catalog_identity` in `pg_object_identity.rs`. Fixture: three fresh schemas (one containing `%`, `_`, quote and dot), six relation names (`same_name`, `percent%name`, `under_score`, `quote"name`, `dot.name`, `x';--`), same-named decoys in other schemas, wildcard/prefix decoys, an enum column, reverse index/view dependents, and an owned sequence. A single-connection SQLx pool alternates active schemas deterministically. Every selected table must return exactly its schema and enum-type dependencies, not any decoy/reverse source. The public connector is independently checked against exact full four-field rows. Owned sequence proves AUTO relation-target dependencies remain; missing schema/table returns an empty list.

Exact public smoke call: `PostgresConnector::get_object_dependencies(&handle, schema_name, relation_name)` (or runtime `PostgresApi::object_dependencies(connection_id, schema_name, relation_name)`). For `CREATE TYPE s.marker AS ENUM ('value'); CREATE TABLE s.t (id integer, value s.marker);` with `s` outside the search path, expect exactly:

| object_type | object_name | depends_on_type | depends_on_name |
|---|---|---|---|
| pg_class | s.t | pg_namespace | s |
| pg_class | s.t | pg_type | s.marker |

With `search_path = s, public`, the source display becomes `t`, but membership/target identities stay identical. Actual weird names are quoted only for display, never parsed as request syntax. For `CREATE SEQUENCE s.owned_seq OWNED BY s.t.id`, its outgoing rows include target `pg_class` / `s.t`. Calling with missing relation/schema returns no rows, never fuzzy matches.

## Fixture requirements / parent gates

Use a disposable database with local `postgres://user:password@host:port/database` URL, plain fixture credentials, public schema visible in the default search path, PL/pgSQL available, and superuser for the event-trigger test. No fixture files, schema/role changes, or persisted application config are required. Names use random UUID suffixes, successful runs drop every created object; failed runs may leave uniquely named objects, so discard the database afterward. The catalog test alone needs CREATE SCHEMA/TYPE/TABLE/VIEW/SEQUENCE permissions, not superuser. Tests are opt-in `#[ignore]`, matching provider conventions.

All following gates: **NOT RUN** by this agent (no counts or exit statuses claimed):

- `cargo test -p db-pro-core domain::event_trigger::tests`
- `DATABASE_URL=... cargo test -p db-pro-infrastructure --test pg_object_identity -- --ignored`
- formatting, check, Clippy, workspace tests, release build, baseline/candidate runtime probes, CI.

Main must run the same candidate provider tests/probes against the exact exported baseline `bceb735727f81180dad678defee1000bc547d97b` where applicable, then integrate/gate, commit issues independently, publish and close them only after runtime evidence is observed.
