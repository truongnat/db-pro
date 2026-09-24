# Security — source baseline

- Source date: 2026-09-24
- Baseline SHA: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Scope: native Rust/egui Security activity, UI/runtime/service/provider paths, role and privilege safety, RLS, and Phase E/product-goal drift. Source claims refer to this SHA.
- No source code, tests, plans/status, or runtime state were changed or exercised for this research.

## Finding

Security is a real PostgreSQL-only native activity, not a backend-only placeholder. It currently includes role listing/create/drop, a subset of role attributes, password update, memberships, object privileges, and table RLS configuration. The service enforces PostgreSQL-only routing and stored read-only policy for role mutations; the PostgreSQL adapter quotes identifiers and allowlists privilege tokens. The clearest P1 source risk is password confidentiality: the password is embedded in SQL text sent to PostgreSQL, despite being correctly dollar-quoted for SQL syntax. PostgreSQL explicitly warns that this form transmits cleartext and may be logged. The source baseline does not establish that a leak occurred.

## Surface and call path

- `crates/ui/src/activity_bar_view.rs:85-96` puts Security on the top-level activity rail. `sidebar_view.rs:64-74,105-121` routes it to `draw_security_activity` and supplies connection/driver context. The surface reports disconnected and non-PostgreSQL states rather than inventing a SQLite role model (`security_surface_view.rs:23-59`).
- The current page combines Roles, selected-role details, drop confirmation, and RLS (`security_surface_view.rs:23-59`). The role list shows name and four badges and offers Create/Drop (`security_roles_view.rs:9-75`). Details expose LOGIN, CREATEDB, CREATEROLE, password, memberships, and Table/Schema/Database/Sequence grants (`security_role_details_view.rs:46-244`). UI does not expose SUPERUSER mutation, although the domain/provider can represent it.
- Create, alter attributes, password, membership and privilege operations dispatch directly as `UiCommand`s; only Drop has a simple yes/no modal. There is no role/grant SQL preview or typed/environment-bound confirmation in these paths (`security_activity_view.rs:71-169`; `security_confirmation_view.rs:15-40`). RLS is different: it presents planned SQL and requires an explicit acknowledgement checkbox before dispatching generic DDL (`security_activity_view.rs:184-303`; `security_rls_view.rs:2-92`).
- Commands cross the typed UI/runtime protocol and runtime worker to `UserApi`/`UserService` and `UserManager` (`runtime_protocol.rs:320-391`; `runtime/src/worker.rs:2344-2365`; `runtime/src/api.rs:872-979`; `core/src/application/user_service.rs:29-185`). Mutation completion clears the drop/password state and refreshes the role list and selected-role details (`ui/src/operation_events.rs:267-287`).
- RLS state is read via `RlsService`; UI builds RLS DDL and uses `SchemaService::execute_ddl`, which applies the common SQL-safety/read-only policy and rejects multiple statements (`core/src/application/rls_service.rs:20-61`; `ui/src/security_rls.rs:1-98`; `core/src/application/schema_service.rs:217-235`). RLS user expressions are SQL expressions by design; identifiers and policy command choice are constrained by the builder.

## Provider and safety contract

| Area | PostgreSQL | SQLite | MySQL / SQL Server |
|---|---|---|---|
| Roles | Source path lists `pg_roles`; create/drop/alter and password operations; list and mutate memberships; grants/revokes for table, schema, database, sequence (`core/application/user_service.rs:41-176`; `infrastructure/postgres/user_manager.rs:114-405`). | `UserService` rejects non-PostgreSQL before provider mutation; no local role model. | Same explicit PostgreSQL-only service rejection; no source provider implementation exposed by this API. |
| Role model | Lists superuser, createdb, createrole, login. Editable role flags are LOGIN/SUPERUSER/CREATEDB/CREATEROLE in the service/domain; UI only offers LOGIN/CREATEDB/CREATEROLE toggles. Membership model carries role/member/admin_option but query returns memberships for a single member (`domain/user.rs:5-62`; `postgres/user_manager.rs:115-219`; `security_role_details_view.rs:46-244`). | Unsupported. | Unsupported. |
| Privileges | Per-kind allowlists and quoted identifiers; supports the UI's individual table/schema/database/sequence targets. Secondary schema/sequence and database-privilege query failures are ignored with `if let Ok`, so the returned list can be partial without an error (`postgres/user_manager.rs:21-71,239-320,324-405`). | Unsupported. | Unsupported. |
| RLS | RLS state read and preview/apply path exist for PostgreSQL. Apply routes through the common DDL safety/read-only check (`core/application/rls_service.rs:20-61`; `core/application/object_mutation_service.rs:71-98`; `core/application/schema_service.rs:217-235`). | Explicitly unsupported by RLS/object-mutation services. | Not supported by this API. |

`UserService` checks PostgreSQL routing and rejects mutations for configured read-only connections (`core/application/user_service.rs:29-39,178-185`). This is an important backend boundary. The role-drop path has no service-level check against the active connection role; the UI asks only “Drop role `{role}`? This cannot be undone.” (`core/application/user_service.rs:74-78`; `ui/src/security_confirmation_view.rs:20-34`). Do not infer that the database would accept a self-drop; no provider runtime was exercised.

The adapter safely quotes identifiers (rejects empty/NUL and doubles embedded quotes) and restricts each object-kind privilege token before composing SQL (`postgres/user_manager.rs:11-71`). The password helper also chooses a random dollar-quote delimiter, which prevents quote termination/injection; this does not provide secrecy. `update_password` places the password in an `ALTER ROLE ... PASSWORD ...` SQL string and calls `connector.execute` with an empty parameter list (`postgres/user_manager.rs:73-89,184-190`; `postgres/connector.rs:175-196`). The UI masks the field and worker comments avoid putting the value in result events, but the command protocol carries a `String` and the server receives the secret in statement text (`runtime_protocol.rs:5-6,320-391`; `runtime/src/worker.rs:2344-2365`). PostgreSQL's `CREATE ROLE` documentation warns that an unencrypted password is transmitted in cleartext and might be logged by the client or server: [PostgreSQL CREATE ROLE](https://www.postgresql.org/docs/current/sql-createrole.html). No actual log exposure was observed.

No app-level audit-write call was found in the inspected `UserService`/`UserApi` mutation path. The UI's Audit surface is a separate reader and is not evidence of Security mutation audit records (`runtime/src/api.rs:872-979,1073-1087`; `core/src/application/user_service.rs:10-20,62-176`).

## Source-document drift

- `docs/goals/goal-phase-e-security.md:1-19,21-62,105-140` describes PLANNING/backend-only/no UI, but a native Security activity and role/RLS UI are present at the baseline SHA.
- Phase E excludes RLS (`goal-phase-e-security.md:87-96,691-693`), whereas the Security surface currently includes RLS. Its target IA places security under Explorer rather than a top-level rail (`:143-158`).
- `docs/goals/goal-full-product.md:239-258` and `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §11 describe deferred/not-yet-native Security. These current-state claims are stale; target placement still provides an Explorer-oriented direction.
- The source baseline is not a lifecycle completion claim. No Phase E plan/status transition was made.

## Verification

- Source-only inspection at SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`.
- Official PostgreSQL `CREATE ROLE` and `GRANT` docs read; reference links are in `security-feature-research.md`.
- No tests/builds run; no native UI launched; no PostgreSQL or SQLite runtime scenario exercised. Security/provider behavior remains runtime-unverified.
