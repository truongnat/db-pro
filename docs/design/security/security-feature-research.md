# Security — feature research and recommendation

- Research date: 2026-09-24
- Repository baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Scope: product role, native IA, role/grant/RLS workflows, provider boundaries, operational safety, reference patterns, and roadmap drift. No implementation is included.

## Decision

**Treat Security as connection/database administration in Explorer, not a permanent top-level rail activity.** This follows the product's full-product target and Phase E target placement (`docs/goals/goal-full-product.md:239-258`; `docs/goals/goal-phase-e-security.md:143-158`). The current wired Security activity is useful evidence of implemented behavior, not evidence that the top-level placement is canonical. Preserve its reachable functions during a navigation cutover; do not regress to a placeholder.

**Separate role administration from table-level row security.** Roles, memberships, and grants belong to a connection/database Security context. RLS policy state is table-specific and should be reached from the selected table (with a route back to broader role management). Phase E explicitly excludes RLS, while the current view mixes RLS into Security; revise scope ownership rather than silently treating that mismatch as complete Phase E work.

**Do not call the current administrative workflow complete until the password path is confidential and high-impact changes have deliberate, auditable apply semantics.** PostgreSQL SQL generation is protected against identifier injection and privilege-token injection, and role mutations honor configured read-only mode. Those protections do not cover cleartext password statements, mis-targeted grants, or a missing durable audit record.

## User model and workflow

1. **Know the scope before changing access.** Show connection, database, role, provider support, and environment context before role/grant changes.
2. **Understand effective role access.** Distinguish direct object grants, inherited membership, membership options, ownership, PUBLIC grants, and provider limitations. PostgreSQL `GRANT` treats object privileges and role membership as distinct forms, and membership has `SET`, `INHERIT`, and `ADMIN` options ([PostgreSQL GRANT](https://www.postgresql.org/docs/current/sql-grant.html)). Current UI is a narrower direct-grant view, not an effective-permission analyzer.
3. **Make a reviewed change.** Stage a change set with target role/object, before/after effect, generated SQL, and exact environment; show a diff and require apply confirmation for privilege changes, role deletion, and sensitive attributes. RLS already has a distinct SQL preview/acknowledgement path; role and grant operations do not.
4. **Verify and audit.** After apply, refresh source-of-truth role/membership/privilege state, report partial visibility or failure, and preserve an app-level audit record without secrets. Current mutation completion refreshes role and selected-role details, but the inspected service/API path has no audit writer (`security-baseline.md`).

## Recommended structure

```text
Explorer
  Connection / Database
    Security
      Roles                 search, provider scope, safe create/drop
      Role detail           attributes, memberships, effective/direct grants
      Change set            preview, scope, apply confirmation, audit result
  Table
    Security / RLS          enabled/forced state, policies, expression preview
```

Keep this as a proposal, not as a claim that current navigation already works this way. Current `Activity::Security` sits on the rail and composes roles plus RLS (`crates/ui/src/activity_bar_view.rs:85-96`; `security_surface_view.rs:23-59`). The official DBeaver PostgreSQL guide lists Roles among database objects and identifies PostgreSQL administration tools in the selected connection context, supporting object-context discoverability rather than a global user-only role page: [DBeaver PostgreSQL guide](https://dbeaver.com/docs/dbeaver/Database-driver-PostgreSQL/). This is a workflow reference, not evidence about DB Pro behavior.

## Provider contract

| Workflow | PostgreSQL | SQLite | MySQL / SQL Server |
|---|---|---|---|
| Role/user administration | Source implementation exists for create/drop/selected attributes/password, memberships, and object privileges. Visible fields/actions are narrower than PostgreSQL's role model. | Explicitly unsupported by `UserService`; do not show an empty role list as success. Explain that SQLite has no server-wide PostgreSQL role model. | Unsupported by this API. Avoid dispatching PostgreSQL-only actions. |
| Object grants and memberships | Individual Table/Schema/Database/Sequence targets supported in source. Current membership query and privilege display are partial; no effective-privilege calculation or default-privilege editor. PostgreSQL has separate object-grant and role-membership semantics. | Unsupported. | Unsupported. |
| RLS | State and preview/apply source paths exist. Keep table-scoped. | Explicitly unsupported. | Unsupported by this API. |

Runtime support is pending independently for PostgreSQL and SQLite; source routing is not provider runtime evidence. No MySQL/SQL Server runtime evidence was collected.

## Prioritized open work

1. **P1 — remove password from statement text.** `PostgresUserManager::update_password` interpolates the plaintext password into an `ALTER ROLE` statement and executes with no bind arguments (`infrastructure/src/postgres/user_manager.rs:184-190`). PostgreSQL warns that an unencrypted password may be transmitted in cleartext and appear in server logs ([CREATE ROLE](https://www.postgresql.org/docs/current/sql-createrole.html)). Dollar quoting prevents SQL syntax injection but not logging or statement-text exposure. Replace this path with a mechanism that avoids sending a plaintext password as ordinary statement text where supported; redact debug/error paths and verify the actual provider protocol/configuration. The UI statement “never logged” is not a guarantee against server logging. No exposure was runtime-observed.
2. **P2 — add reviewed apply semantics for access mutations.** Create/alter, password, membership, and privilege actions dispatch directly; Drop only has a yes/no irreversible warning; there is no environment-bound target confirmation, SQL/change preview, or service-level active-connection-role drop guard (`ui/security_activity_view.rs:71-169`; `security_confirmation_view.rs:20-34`; `core/application/user_service.rs:74-78`). Preserve existing PostgreSQL/read-only checks and allowlists while strengthening the UI and service contract. Treat `DROP ROLE` rejection behavior as unverified, not as an established provider guarantee.
3. **P2 — represent privilege completeness and errors.** `list_privileges` ignores failures of secondary schema/sequence and database queries, allowing a partial list to look complete (`postgres/user_manager.rs:239-320`). Return a typed partial/error state or fail the list visibly. Include exact known categories/row scope and a refresh path before users make a grant decision.
4. **P2 — model actual PostgreSQL role and membership semantics.** The UI omits SUPERUSER mutation and does not represent INHERIT, REPLICATION, BYPASSRLS, connection limits, validity, ownership, membership direction/options, or effective access. PostgreSQL's current role syntax includes these attributes and its membership grant includes ADMIN/INHERIT/SET options ([CREATE ROLE](https://www.postgresql.org/docs/current/sql-createrole.html); [GRANT](https://www.postgresql.org/docs/current/sql-grant.html)). Add only those workflows the product explicitly intends; keep superuser/replication/BYPASSRLS actions clearly high-risk and privilege-gated.
5. **P2 — add auditable outcome and secret-safe result states.** No durable app audit write is wired through the inspected role mutation service/API path; Audit page presence does not close this gap. Record actor, target connection/database, operation, target role/object, result, and timestamp, but never password or secret-bearing SQL. Show permission-denied, unsupported, partial visibility, and execution failure distinctly.
6. **P2 — reconcile product scope and current-state docs.** Phase E says backend-only/no UI and excludes RLS, while current source contains a rail activity with roles, grants, password and RLS. Full-product placement points to Explorer. Update current-state and ownership statements before lifecycle status changes; keep Security runtime gates distinct from source wiring.

## Recommended decision boundary

Align the destination IA with Explorer-scoped Security and table-scoped RLS. First fix password statement confidentiality, then define review/apply/audit policy for role/grant mutations. Keep PostgreSQL the only supported provider in this feature unless a provider-specific role/authorization model is deliberately designed; SQLite should show a clear unsupported explanation. Do not infer completion from source code: this assessment has no native UI or provider runtime evidence.

## Research limits

No source code, tests, plans/status, or persistent data changed. No build/test/native launch, destructive operation, or provider scenario was executed. P1/P2 findings are source-derived and not reproduced incidents. PostgreSQL documentation and DBeaver's PostgreSQL guide describe provider/product contracts, not DB Pro runtime behavior.
