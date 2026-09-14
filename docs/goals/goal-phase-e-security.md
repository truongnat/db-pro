# DB Pro — Phase E Goal: Users, Roles & Permissions (PostgreSQL Security Workbench)

- Doc ID: `GOAL-PHASE-E`
- Phase: E — Users / Roles / Permissions
- Release targets: v0.3 (E01–E02), v0.4 (E03)
- Priority: P2 (E01–E02), P3 (E03)
- Authority: `docs/goals/goal-full-product.md` (master goal)
- Depends on: Phase A components (`ObjectTabs`, `PropertyGrid`, `FormEditor`,
  `ConfirmationDialog`), master goal §10 policy producer.
- Status: PLANNING. No implementation has started.
- Companions: `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §11, `docs/release/known-limitations.md`
  (LIM-005), `docs/architecture/security-boundaries.md`.

---

## 1. Problem

The PostgreSQL security backend is complete and unreachable. This is the one phase where most
of the work is **already done and idle**.

What exists today (source-verified):

- **Port**: `UserManager` with `list_users`, `create_role`, `drop_role`, `list_privileges`,
  `grant_privilege`, `revoke_privilege` (`crates/core/src/ports/user_manager.rs:9-22`).
- **Service**: `UserService` (`crates/core/src/application/user_service.rs:10-136`) with a
  PostgreSQL-only guard at `:130-136` that rejects non-PostgreSQL drivers *before* any provider
  call (tested at `:139`).
- **Adapter**: `PostgresUserManager` (`crates/infrastructure/src/postgres/user_manager.rs:62-209`)
  with:
  - `quote_identifier` rejecting empty/NUL identifiers (`:11-19`),
  - `quote_privilege` restricted to an 8-value allowlist
    (`ALL`, `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `TRUNCATE`, `REFERENCES`, `TRIGGER`) at
    `:21-33` — arbitrary privilege text cannot reach SQL,
  - `list_users` from `pg_catalog.pg_roles` (`:63`),
  - `create_role` with `LOGIN|NOLOGIN` (`:88`), `drop_role` (`:101`),
  - `list_privileges` from `information_schema.role_table_grants` filtered by grantee (`:113`),
  - `grant_privilege` (`:137`) / `revoke_privilege` (`:159`).
- **Runtime API**: `UserApi` exposes all six operations (`crates/runtime/src/api.rs:797-861`).
- **Legacy host**: Tauri commands `list_users`/`create_role`/`drop_role`/`list_privileges`/
  `grant_privilege`/`revoke_privilege` (`crates/tauri-app/src/commands/user_management.rs`).

What is missing:

1. **Zero native UI.** Grepping `crates/ui` finds no user/role surface, no `UiCommand`
   variant, and no `RuntimeCommand` variant for any of these operations. The capability matrix
   marks the whole row group `BACKEND_ONLY` and notes the surface is "hidden per LIM-005".
2. **Memberships are missing in the backend.** There is no `grant_role`/`revoke_role`,
   no membership listing, and no role-inheritance view. `pg_auth_members` is never queried.
3. **Grant depth is narrow.** Only **table** privileges, only filtered by *grantee*
   (`list_privileges` takes a role name, not an object). There is no schema-level grant, no
   database-level grant, no column-level grant, and no reverse lookup ("who can read this
   table?").
4. **No role detail.** Attributes visible in `pg_roles` beyond a login flag are not surfaced:
   `SUPERUSER`, `CREATEDB`, `CREATEROLE`, `REPLICATION`, `BYPASSRLS`, `CONNECTION LIMIT`,
   `VALID UNTIL`, role-level settings, ownership of objects.
5. **No DDL view.** A user cannot see what SQL the UI would run — the single most important
   safety affordance for privilege changes.
6. **No password management.** `ALTER ROLE … PASSWORD` is not implemented, and the product has
   no path to rotate a credential.
7. **No read-only / environment enforcement at the UI level** and no audit record for
   privilege changes, even though the backend rejects writes on read-only connections
   (`user_service.rs:33`).

Why this matters: role and grant administration is the part of PostgreSQL operations where
mistakes are most damaging and least visible (a wrong `GRANT ALL` on `public` schema, a
superuser flag left on, a role dropped while it owns objects). The value of this phase is not
just "expose the backend" — it is to expose it with a **DDL-preview, allowlist-constrained,
fully audited** surface.

## 2. Scope

### 2.1 In scope

| Group | Content | Milestone |
|---|---|---|
| Roles list | All roles from `pg_roles` with attributes, login ability, ownership summary, connection limit, validity | E01 |
| Role details | Attributes, memberships (in/out), owned objects summary, role-level settings, comment | E01 |
| Role lifecycle | Create role (with attribute selection), alter attributes, drop role (with dependency warning) | E01 |
| Role DDL view | `CREATE ROLE` / `ALTER ROLE` / `DROP ROLE` statements the UI would run, copyable, never auto-executed | E01 |
| Privilege matrix | Table and schema grants: a matrix of grants × objects, filterable, with the grantee/privilege columns | E02 |
| Grant / revoke | Table and schema privileges from the allowlisted set, with preview | E02 |
| Reverse lookup | "Who can access this table/schema?" — grantee list per object | E02 |
| Memberships | List, grant, revoke role memberships; membership tree; inherited-privilege explanation | E03 |
| Password management | `ALTER ROLE … PASSWORD` with a typed confirmation, no echo, no logging | E03 |
| Role rename | `ALTER ROLE … RENAME TO` with dependent-object warning | E03 |

### 2.2 Out of scope (later or never in this roadmap)

- Row-level security policy authoring.
- Column-level and database-level grants (recorded as a later extension; the port is designed
  so they are additive).
- Default privileges (`ALTER DEFAULT PRIVILEGES`).
- Ownership transfer (`ALTER … OWNER TO`) beyond what appears in role details as information.
- LDAP/AD/GSSAPI/SCRAM configuration, `pg_hba.conf` editing.
- Audit/log dashboards for logins or failed authentication.
- Cloud IAM or any external identity provider.

## 3. Non-goals

- **No raw SQL input for privileges.** Every privilege value comes from the allowlist
  (`user_manager.rs:21-33`); the UI offers a fixed set of checkboxes/dropdown values. A
  free-text field for "privilege" is a review-blocking defect.
- **No unsafe identifier interpolation.** All identifiers go through `quote_identifier`
  (or its extension), and identifiers are never built from concatenated user text without
  validation.
- **No password value ever rendered, logged, stored in the meta store, or returned to the UI.**
  After a password change, the UI shows only success/failure.
- **No `GRANT ALL ON ALL TABLES` convenience action** — a bulk grant must be explicit
  (select the tables and privileges), because the blast radius is invisible.
- **No automatic role creation** for convenience (e.g. "create an app role for this
  connection").
- **No SQLite security surface.** SQLite has no users; the node must render disabled with a
  reason and the service must reject before any provider call (already true — keep it).
- **No superuser-by-default.** Creating a superuser requires an explicit, separately confirmed
  action with a warning; it can never be the default state of a checkbox.

## 4. Current implementation (code-verified baseline)

| Capability | Backend | Native UI | Evidence |
|---|---|---|---|
| Users list | DONE (PG) | **MISSING** | `user_manager.rs:63`; `user_service.rs:53`; `runtime/src/api.rs:806` |
| Roles create/drop | DONE (PG) | **MISSING** | `user_manager.rs:88, 101`; `user_service.rs:62, 71` |
| Role attributes (beyond login) | **MISSING** | MISSING | no query for `rolsuper`/`rolcreatedb`/… beyond the name+login shape |
| Role DDL view | **MISSING** | MISSING | no statement reconstruction |
| Memberships | **MISSING** | MISSING | `pg_auth_members` never queried |
| Table grants | DONE (PG, 8-privilege allowlist) | **MISSING** | `user_manager.rs:113, 137, 159` |
| Schema privileges | **MISSING** | MISSING | — |
| Column / database grants | MISSING | MISSING | — |
| Reverse lookup (object → grantees) | **MISSING** | MISSING | `list_privileges` is grantee-scoped only |
| Password rotation | **MISSING** | MISSING | no `ALTER ROLE … PASSWORD` |
| Read-only gate | DONE | — | `user_service.rs:33` |
| SQLite rejection | DONE, tested | — | `user_service.rs:130-136`, test at `:139` |
| Secret redaction | DONE | — | `domain/secret.rs`; `RedactedSecret` |
| Audit records | **MISSING** | MISSING | no audit entity exists yet (master goal §10.5 introduces it) |

Runtime/UI gap evidence: `crates/ui` contains no reference to `UserApi`, `list_users`,
`create_role`, `grant_privilege`, or any role-related `UiCommand`/`UiEvent` variant; the
`Activity` enum has no Security variant and the Explorer tree has no Security node
(`crates/ui/src/explorer_view.rs` renders Tables/Views/Functions/Triggers folders only).

## 5. UX

### 5.1 Placement (per master goal §3.1)

Security is **not** a rail icon. It lives in two places:

```text
Explorer
  └─ <connection>
      └─ <database>
          └─ Security
              ├─ Users            (roles with LOGIN)
              └─ Roles            (all roles)
```

and as workspace tabs opened from those nodes (`Role: app_writer`). This mirrors how the
master goal defers a top-level Security activity: the object tree is the discovery surface, and
role detail is a workspace tab like any other object.

### 5.2 Roles list

```
Users / Roles                                          [ filter… ]  [ + Create role ]

 Role            Login  Super  CreateDB  CreateRole  Repl  BypassRLS  Conn limit  Memberships
 app_writer      ●      –      –         –           –     –          20          2 roles
 reporting       ●      –      –         –           –     –          –           1 role
 postgres        ●      ●      ●         ●           ●     ●          –           –
 deploy          –      –      –         –           –     –          –           1 role
```

- Columns are sortable; dangerous attributes (`SUPERUSER`, `BYPASSRLS`, `REPLICATION`) render
  with a warning-tinted badge so they are visible at a glance.
- The filter matches name and membership.
- Context menu: Open, Create role like this…, Alter attributes…, Grant/Revoke privileges…,
  Manage memberships…, Generate DDL, Copy name, Drop…, Add to favorites (Phase G).
- The role the app is connected as is marked and protected from accidental drop (drop requires
  typing the name even outside Production).

### 5.3 Role detail tab

```text
Role: app_writer                                    [environment badge] [read-only badge]

Tabs:  Properties | Memberships | Privileges | Owned Objects | DDL

Properties
  Name              app_writer
  Login             yes
  Superuser         no
  Create databases  no            Memberships (in)   app_readers, deploy
  Create roles      no            Memberships (out)  —
  Replication       no            Connection limit   20
  Bypass RLS        no            Valid until        2026-12-31 00:00
  Comment           application writer role
  Role settings     statement_timeout = 30s

  [ Alter attributes… ]   [ Manage memberships… ]   [ Change password… ]   [ Drop role… ]
```

- **Memberships** shows both directions with a tree (`app_writer` inherits from
  `app_readers` which inherits from `analytics_ro`), and a plain-language explanation of what is
  inherited: "app_writer can read tables granted to app_readers."
- **Privileges** is the per-role projection of the E02 matrix (grants to this role, expandable
  by inheritance with an explicit `via <role>` marker).
- **Owned Objects** lists objects the role owns — needed before a drop, because
  `DROP ROLE` fails while the role owns objects.
- **DDL** shows the exact statements the UI would run for the pending change (never a stale
  reconstruction presented as current server state; it is labeled "statements for the pending
  change" or, when nothing is pending, "statements to recreate this role's attributes").

### 5.4 Privilege matrix (E02)

```
Privileges                    Scope: [ Table ▾ ]  Schema: [ public ▾ ]   [ filter… ]

 Object                     Grantee          SELECT INSERT UPDATE DELETE TRUNCATE REFERENCES TRIGGER
 public.orders              app_writer         ✓      ✓      ✓      –      –          –          –
 public.orders              reporting          ✓      –      –      –      –          –          –
 public.customers           app_readers        ✓      –      –      –      –          –          –
 public.audit_log           (none)             –      –      –      –      –          –          –

 [ + Grant… ]   [ Revoke selected… ]   [ Generate SQL ]   [ Who can access this object? ]
```

- Cell-level editing is **not** offered (a click does not grant). Grant/revoke go through the
  form + preview + confirmation flow so the SQL is always visible.
- `Who can access this object?` switches to the reverse view (the missing capability today):
  one row per (object, grantee, privilege) with the granting role noted
  (`via app_readers`).
- Schema-scope mode lists `schema` privileges (`USAGE`, `CREATE`) per (schema, grantee).
- Inherited grants are shown with a distinct marker and cannot be revoked directly — the UI
  says which role to revoke from instead of silently failing.

### 5.5 Create / alter role form

```
Create role
  Name             [ app_service                    ]
  [x] Can log in          [ ] Superuser     ⚠ grants unrestricted access
  [ ] Create databases    [ ] Create roles
  [ ] Replication         [ ] Bypass RLS
  Connection limit [ 20        ]     Valid until [ 2026-12-31 ] (optional)
  Password         ( ) none  (•) set now  ( ) no password (login-only via other means)
  Memberships      [ + add role… ]  app_readers ×
  Comment          [ application service role        ]

  Preview SQL                                 [ Cancel ]  [ Create role ]
```

Rules:

- The form never defaults to dangerous attributes.
- Selecting `Superuser` shows an inline warning and requires a separate confirmation before
  apply (in addition to the normal confirmation).
- "Set password now" uses a password field with confirm-entry, no echo, no clipboard helper,
  and no strength meter that logs anything; the value is passed through as a parameter to the
  composed statement (see §11) and dropped immediately after.
- Membership additions may reference roles that need `ADMIN OPTION`; the resulting failure must
  be reported with the reason, and the form states the requirement up front.

### 5.6 Error, empty, and disabled states

- **Empty**: "No roles besides the default ones" is practically impossible, but an empty
  filter result shows a filter-aware empty state.
- **Permission denied**: creating a role without `CREATEROLE` returns the structured
  permission error with the required privilege named; the UI does not hide the action
  (visibility is fine; the server decides).
- **Read-only connection**: every mutating action disabled with "Connection is read-only",
  enforced in the service as well.
- **SQLite**: the Security node is present but disabled with the reason
  "SQLite has no users or roles"; opening it shows an explanatory empty state. The service
  already rejects before any provider call — the UI must not be the only gate.
- **Production**: role and privilege mutations carry the environment banner; `DROP ROLE` and
  any `SUPERUSER` grant require the typed confirmation of §10.3 of the master goal.

## 6. Architecture

### 6.1 Flow

```text
UI (Security node → Role tab / Privileges matrix)
  → UiCommand::SecurityListRoles   { request_id, connection_id }
  → UiCommand::SecurityRoleDetail  { request_id, connection_id, role }
  → UiCommand::SecurityPrivileges  { request_id, connection_id, scope, schema, object?, grantee? }
  → UiCommand::SecurityPlanChange  { request_id, connection_id, change: SecurityChange }
        → returns statements + class + warnings (no execution)
  → UiCommand::SecurityApplyChange { request_id, connection_id, change, fingerprint }
  → RuntimeCommand::* mirrors
  → SecurityService → UserManager port (extended) → PostgresUserManager
  → UiEvent::SecurityRolesLoaded / RoleDetailLoaded / PrivilegesLoaded / ChangePlanned / ChangeApplied
```

Two-step plan/apply mirrors Phase A (master goal §4.4): the preview is a first-class payload,
not a UI-side string build. `SecurityChange` is a typed enum (see §7) and the service is the
only place that turns it into SQL.

### 6.2 Layering rules

- `SecurityService` (renamed/extended from `UserService`, or a new service delegating to it)
  lives in `crates/core/src/application/` and depends only on the `UserManager` port plus the
  policy inputs. It never touches a connector.
- Statement composition lives in the adapter (`PostgresUserManager`) **or** in a dedicated
  `security_ddl.rs` builder in `crates/core/src/application/` used by the adapter. Choose one in
  E01 and document it; do not split statement building between service and adapter.
- The UI never sees or composes SQL; it receives a planned statement list for display only.
- Capability gating: `role_management` flag (introduced in Phase D's split) gates the surface;
  the service rejects non-PostgreSQL before the port call (existing behavior, retained).

### 6.3 Why not reuse the raw DDL path

`SchemaService::execute_ddl` accepts arbitrary SQL. Role administration must **not** be routed
through it: privilege statements have a smaller, allowlisted vocabulary and a different
confirmation model. The security service owns its own statement construction, and its
statements still pass the standard policy check (`read_only`, `allow_ddl`,
`allow_destructive`) before execution.

## 7. Domain model

New module `crates/core/src/domain/security.rs`:

```rust
pub struct Role {
    pub name: String,
    pub can_login: bool,
    pub superuser: bool,
    pub create_db: bool,
    pub create_role: bool,
    pub inherit: bool,           // rolinherit
    pub replication: bool,
    pub bypass_rls: bool,
    pub connection_limit: Option<i32>,
    pub valid_until: Option<OffsetDateTime>,
    pub comment: Option<String>,
    pub role_settings: Vec<(String, String)>,   // rolconfig
    pub is_current_user: bool,
}

pub struct Membership { pub role: String, pub member: String, pub admin_option: bool, pub inherit_option: bool, pub set_option: bool }

pub enum PrivilegeScope { Table, Schema, Database }   // Database reserved for a later extension

pub struct TableGrant { pub schema: String, pub table: String, pub grantee: String,
                        pub privilege: TablePrivilege, pub grantable: bool, pub granted_via: Option<String> }

pub enum TablePrivilege { Select, Insert, Update, Delete, Truncate, References, Trigger, All }
pub enum SchemaPrivilege { Usage, Create }

pub struct OwnedObject { pub kind: ObjectKind, pub schema: Option<String>, pub name: String }

pub enum SecurityChange {
    CreateRole(RoleDefinition),
    AlterRole { role: String, changes: RoleAttributeChanges },
    RenameRole { role: String, new_name: String },
    DropRole { role: String, reassign_owned_to: Option<String> },
    SetPassword { role: String, password: SecretString },       // never Debug-printed
    GrantMembership { role: String, member: String, admin_option: bool },
    RevokeMembership { role: String, member: String },
    GrantTable { schema: String, table: String, grantee: String, privileges: Vec<TablePrivilege>, with_grant_option: bool },
    RevokeTable { schema: String, table: String, grantee: String, privileges: Vec<TablePrivilege> },
    GrantSchema { schema: String, grantee: String, privileges: Vec<SchemaPrivilege> },
    RevokeSchema { schema: String, grantee: String, privileges: Vec<SchemaPrivilege> },
}

pub struct SecurityChangePlan {
    pub statements: Vec<String>,
    pub safety: StatementSafety,
    pub warnings: Vec<String>,        // e.g. "role owns 3 objects", "SECURITY: grants unrestricted access"
    pub requires_extra_confirmation: bool,
    pub fingerprint: String,
}
```

`SecretString` is a wrapper with a redacted `Debug`, no `Clone` beyond what is required, and a
zeroizing drop if the workspace already has such a facility; if not, implement a minimal version
in the domain and document it. **The password must never enter a `String` in the UI, an event, a
log, or the meta store** — it travels from the password widget to the command and into the
parameterized statement, then is dropped.

## 8. APIs, services, ports

### 8.1 Port extension

```rust
#[async_trait]
pub trait UserManager: Send + Sync {
    // existing
    async fn list_users(&self, handle: ConnectionHandle) -> Result<Vec<DatabaseUser>, DbError>;
    async fn create_role(&self, handle: ConnectionHandle, name: &str, login: bool) -> Result<(), DbError>;
    async fn drop_role(&self, handle: ConnectionHandle, name: &str) -> Result<(), DbError>;
    async fn list_privileges(&self, handle: ConnectionHandle, role: &str) -> Result<Vec<Privilege>, DbError>;
    async fn grant_privilege(&self, handle: ConnectionHandle, role: &str, schema: &str, table: &str, privilege: &str) -> Result<(), DbError>;
    async fn revoke_privilege(&self, handle: ConnectionHandle, role: &str, schema: &str, table: &str, privilege: &str) -> Result<(), DbError>;

    // added in Phase E
    async fn list_roles(&self, handle: ConnectionHandle) -> Result<Vec<Role>, DbError>;
    async fn role_detail(&self, handle: ConnectionHandle, role: &str) -> Result<RoleDetail, DbError>;
    async fn memberships(&self, handle: ConnectionHandle, role: Option<&str>) -> Result<Vec<Membership>, DbError>;
    async fn owned_objects(&self, handle: ConnectionHandle, role: &str) -> Result<Vec<OwnedObject>, DbError>;
    async fn table_grants(&self, handle: ConnectionHandle, schema: Option<&str>, object: Option<&str>, grantee: Option<&str>) -> Result<Vec<TableGrant>, DbError>;
    async fn schema_grants(&self, handle: ConnectionHandle, schema: Option<&str>, grantee: Option<&str>) -> Result<Vec<SchemaGrant>, DbError>;
    async fn apply_change(&self, handle: ConnectionHandle, change: &SecurityChange, password: Option<SecretString>) -> Result<u64, DbError>;
    fn plan_change(&self, change: &SecurityChange) -> Result<SecurityChangePlan, DbError>;   // pure, no handle
}
```

`plan_change` is deliberately pure (no handle) so the statement text, safety class, and
warnings are unit-testable and identical to what `apply_change` executes. Password is passed
separately from `SecurityChange` so it never needs to be stored inside a cloneable enum.

### 8.2 Service

```rust
impl SecurityService {
    pub async fn roles(&self, connection_id: ConnectionId) -> Result<Vec<Role>, DbError>;
    pub async fn role_detail(&self, connection_id: ConnectionId, role: &str) -> Result<RoleDetail, DbError>;
    pub async fn memberships(&self, connection_id: ConnectionId, role: Option<&str>) -> Result<MembershipTree, DbError>;
    pub async fn privileges(&self, connection_id: ConnectionId, filter: PrivilegeFilter) -> Result<PrivilegeView, DbError>;
    pub async fn who_can_access(&self, connection_id: ConnectionId, object: ObjectRef) -> Result<Vec<TableGrant>, DbError>;
    pub async fn plan(&self, connection_id: ConnectionId, change: SecurityChange) -> Result<SecurityChangePlan, DbError>;
    pub async fn apply(&self, connection_id: ConnectionId, change: SecurityChange, password: Option<SecretString>, fingerprint: &str)
        -> Result<SecurityApplyResult, DbError>;
}
```

Guards inside the service (all enforced, not only in the UI):

1. Driver must be PostgreSQL (existing `ensure_server_sessions_config`, renamed to
   `ensure_role_management_supported` — keep the behavior and its test).
2. Connection must not be read-only (existing check at `user_service.rs:33`).
3. `plan` and `apply` must agree on the fingerprint.
4. Dropping the currently connected role is refused.
5. Dropping a role that owns objects returns a plan whose statements fail gracefully; the UI
   warns first and offers reassignment (E03) rather than letting the user see a raw error.
6. Every apply writes an audit record (master goal §10.5).

### 8.3 Runtime/UI wiring

```text
UiCommand::SecurityRoles        { request_id, connection_id }
UiCommand::SecurityRoleDetail   { request_id, connection_id, role }
UiCommand::SecurityMemberships  { request_id, connection_id, role: Option<String> }
UiCommand::SecurityPrivileges   { request_id, connection_id, filter }
UiCommand::SecurityWhoCanAccess { request_id, connection_id, object }
UiCommand::SecurityPlanChange   { request_id, connection_id, change }
UiCommand::SecurityApplyChange  { request_id, connection_id, change, password, fingerprint }
UiEvent::SecurityRolesLoaded / RoleDetailLoaded / MembershipsLoaded / PrivilegesLoaded /
         WhoCanAccessLoaded / SecurityChangePlanned / SecurityChangeApplied
RuntimeCommand / RuntimeEvent mirror these; `translate.rs` maps them.
```

`SecurityChange` crosses the boundary as a typed DTO. The password crosses as an opaque value
that is never serialized into an event or a persisted structure.

## 9. Provider behavior

### 9.1 PostgreSQL

| Operation | Catalog / statement | Notes |
|---|---|---|
| Roles list | `pg_catalog.pg_roles` (existing) extended with `rolsuper`, `rolcreatedb`, `rolcreaterole`, `rolinherit`, `rolreplication`, `rolbypassrls`, `rolconnlimit`, `rolvaliduntil`, `rolconfig`, `shobj_description(oid,'pg_authid')` | `rolpassword` is **never** selected |
| Role detail | plus `pg_auth_members` for both directions, `pg_class.relowner`/`pg_namespace.nspowner`/`pg_proc.proowner` for owned objects | Ownership counts are informative, not blocking |
| Create | `CREATE ROLE … WITH LOGIN/NOLOGIN SUPERUSER/NOSUPERUSER CREATEDB/NOCREATEDB CREATEROLE/NOCREATEROLE REPLICATION/NOREPLICATION BYPASSRLS/NOBYPASSRLS CONNECTION LIMIT n VALID UNTIL '…' PASSWORD $1` | Password is a **bound parameter** (see §11.2) |
| Alter | `ALTER ROLE … WITH …` for attributes; `ALTER ROLE … RENAME TO …`; `ALTER ROLE … SET key = value` | Renaming a role does not change object ownership (ownership is by OID) but breaks app configs — warn |
| Drop | `DROP ROLE name` | Fails if the role owns objects or has privileges; the UI pre-checks ownership and offers reassignment (E03) |
| Memberships | `pg_auth_members` (`roleid`, `member`, `admin_option`, `inherit_option`, `set_option` on PG16+; older versions expose `admin_option` only) | Version-gate `inherit_option`/`set_option` and degrade gracefully |
| Table grants | `information_schema.role_table_grants` (existing) plus a reverse query by object (`table_schema`, `table_name` filters) | The current implementation is grantee-scoped; add object-scoped |
| Applying privileges | `GRANT <priv> ON TABLE schema.table TO role [WITH GRANT OPTION]` / `REVOKE …` | `<priv>` only from the allowlist |
| Schema grants | `GRANT USAGE|CREATE ON SCHEMA s TO role` / `REVOKE …` | New in E02 |
| Inherited grants | Determined from `pg_auth_members` traversal, not from a single query | The service builds the inheritance explanation |

### 9.2 SQLite

| Operation | Behavior |
|---|---|
| Everything | `DbError::Unsupported { capability: "role_management", provider: SQLite, reason: "SQLite has no users or roles" }` before any provider call (existing behavior) |
| UI | Security node present, disabled, with the reason; opening shows an explanatory empty state |

## 10. UI structure

### 10.1 Components

| Need | Component | Status |
|---|---|---|
| Roles list | `ObjectList` (master goal §4.5) | new, shared with A/B/D |
| Role attributes | `PropertyGrid` | new, shared |
| Tabs | `ObjectTabs` | reuse |
| Forms (create/alter role) | `FormEditor` + `Select`/`Checkbox`/`Input` | reuse |
| Membership tree | `ObjectTree` | reuse |
| Privilege matrix | `DataGrid` with check-mark cells (read-only display) | reuse |
| DDL preview | `DDLViewer` | reuse |
| Confirmation | `ConfirmationDialog` (with extra confirmation for `SUPERUSER`, `DROP ROLE`, Production) | reuse |
| Password entry | `PasswordInput` (`components/input.rs`) | reuse |
| Audit | `ActivityLog` | promote from gallery |

### 10.2 Screens

1. **Explorer Security node** — Users and Roles folders with a count badge each.
2. **Roles list** (workspace or sidebar list, consistent with the Routines decision in Phase B).
3. **Role detail tab** (`Properties | Memberships | Privileges | Owned Objects | DDL`).
4. **Create role form** and **Alter attributes form**.
5. **Membership manager** (add/remove with `ADMIN OPTION` where supported).
6. **Privilege matrix** with table/schema scope switch and the reverse lookup view.
7. **Grant/Revoke forms** with preview.
8. **Password change dialog** (typed confirmation; no echo; no clipboard).
9. **Drop role dialog** with ownership pre-check and reassignment offer.

### 10.3 Interaction rules

- Memberships and privileges are presented as **inheritance-aware**: a grant that exists only
  because of a parent role cannot be revoked from the child; the UI names the parent.
- `SUPERUSER`, `BYPASSRLS`, `REPLICATION`, and `CREATEROLE` grants always carry a warning
  affordance explaining the blast radius in one sentence.
- The matrix view is read-only display; all edits route through forms so the preview is
  guaranteed.
- Applying a change refreshes only the affected lists (roles list, role detail, matrix) via
  targeted events, not a full schema re-introspection.
- The DDL tab is always available, including before any change, showing "statements to
  recreate these attributes for review".

## 11. Safety model

### 11.1 Classification

| Operation | Class |
|---|---|
| List roles / details / memberships / grants | `ReadOnly` |
| Create role (without superuser/bypassrls) | `Administrative` |
| Create/alter role with `SUPERUSER` / `BYPASSRLS` / `REPLICATION` | `Administrative` + extra confirmation |
| Alter attributes (non-privilege-escalating) | `Administrative` |
| Rename role | `Administrative` + warning about application configs |
| Grant / revoke table or schema privileges | `Administrative` |
| Grant `ALL` privileges | `Administrative` + warning listing the expanded privilege set |
| Membership grant/revoke | `Administrative` |
| Password change | `Administrative` + typed confirmation |
| Drop role | `Destructive` (owns objects? → warning/reassignment; currently connected role? → refused) |

### 11.2 Injection safety (non-negotiable)

1. **Identifiers** (role names, schemas, tables) are validated and quoted through
   `quote_identifier` semantics; the existing implementation rejects empty and NUL-containing
   input (`user_manager.rs:11-19`) — extend it to reject oversized identifiers and to state
   clearly that it does not attempt Unicode normalization.
2. **Privileges** come only from the allowlist (`user_manager.rs:21-33`), extended with
   `USAGE`/`CREATE` for schemas. A privilege string that is not in the allowlist is a typed
   validation error.
3. **Passwords are bound parameters, never literals.** `ALTER ROLE … PASSWORD $1` is required;
   constructing `PASSWORD '...'` by string concatenation is forbidden even with escaping,
   because the password would then exist in a statement string that can be logged or displayed.
   The preview for a password change shows `PASSWORD ****` and never the value.
4. **No statement ever contains a value derived from a password field** other than the
   placeholder.
5. The preview statement list is built by the same function that builds the executed
   statements (single source), and the password placeholder is substituted at bind time only.

### 11.3 Additional rules

- Read-only connections block every mutating security operation (service-enforced).
- Every apply is audited: actor (the connected role), target role/object, class, statement
  digest (not the full statement when it contains a password placeholder), outcome.
- No privilege change is ever applied automatically from a suggestion (including any future AI
  suggestion — master goal §10.6).
- The product never renders `rolpassword` or any password hash, even as a placeholder value.

## 12. Testing strategy

### 12.1 Unit

- Statement composition (`plan_change`) for every `SecurityChange` variant: exact SQL, quoting
  for reserved/odd names, allowlist enforcement (unknown privilege rejected), `ALL` expansion
  listing, `WITH GRANT OPTION` handling.
- **Injection attempts**: role names like `x"; DROP ROLE postgres; --`, schema/table names with
  quotes and semicolons, privilege strings not in the allowlist — all must be rejected or
  correctly quoted, and a test must assert the composed statement contains no unquoted user
  text.
- Password handling: assert the plan for a password change contains no password value and that
  `SecretString` debug output is redacted; assert the password is not present in any serialized
  `SecurityChange`.
- Classification per §11.1 and the extra-confirmation flags.
- Inheritance explanation: given a membership graph, the inherited-grant projection is correct
  and marks `granted_via`.

### 12.2 Service / integration

- PostgreSQL live (ignored suite): create role with attributes → verify in `pg_roles`; grant a
  table privilege → verify in `information_schema.role_table_grants`; revoke; grant schema
  `USAGE`; add/remove a membership and verify inherited access by attempting a query as that
  role (via `SET ROLE` in a separate session); `ALTER ROLE … PASSWORD` then authenticate with
  the new password from a fresh connection; drop the role.
- Guard tests: read-only connection rejects every mutating change; non-PostgreSQL driver is
  rejected before the port is called (extend the existing test at `user_service.rs:139`).
- Ownership pre-check: create a role that owns a table, attempt drop, assert the plan warns and
  the statement fails with the expected error, and (E03) that reassignment succeeds.
- Version gating: `pg_auth_members.inherit_option`/`set_option` are absent before PG16 —
  assert graceful degradation (query the server version and select the appropriate column set).

### 12.3 UI

- Roles list rendering with attribute badges; sorting and filtering.
- Role detail tabs, inheritance tree, owned-object list.
- Matrix rendering (table and schema scope), inherited-grant marking, reverse lookup.
- Forms: validation, superuser extra confirmation, password entry (no echo, no copy), apply
  flow with preview and confirmation.
- Disabled states: read-only connection; SQLite node; Production confirmation strength.
- Regression: no surface renders a password value (assert on rendered text where feasible, plus
  a review checklist item).

## 13. Runtime verification

Recorded for PostgreSQL; SQLite evidence is the gating test.

**PostgreSQL (E01)**

1. Capture the role list with attributes (including one superuser and one non-login role).
2. Create a role `dbpro_e2e_writer` with `LOGIN`, connection limit, and a comment; capture the
   form, the preview SQL, the confirmation, and the refreshed list.
3. Alter attributes (e.g. add `CREATEDB`, change the connection limit); capture the preview and
   the refreshed detail.
4. Capture the DDL tab for the role.
5. Drop the role; capture the confirmation and the removal.
6. Error path: create a role with a name that already exists; capture the structured error.
7. Permission path: connect as a role without `CREATEROLE` and capture the permission error.

**PostgreSQL (E02)**

8. Capture the privilege matrix for a schema with several tables and roles.
9. Grant `SELECT` on a table to the new role with `WITH GRANT OPTION`; capture preview,
   confirmation, and the refreshed matrix.
10. Revoke it; capture the same.
11. Add a schema `USAGE` grant; capture the schema-scope matrix.
12. Capture `Who can access this object?` for a table with both direct and inherited grants,
    showing the `via …` marker.
13. Attempt to revoke an inherited grant; capture the refusal message naming the parent role.

**PostgreSQL (E03)**

14. Grant a membership; capture the membership tree and the plain-language inheritance
    explanation.
15. Change the role's password; capture the dialog (no echo), the preview showing `****`, and
    prove authentication works by opening a new connection with the new password.
16. Rename a role; capture the warning and the refreshed references.
17. Create an object owned by the role, attempt drop, capture the ownership warning, then
    reassign ownership and complete the drop.

**SQLite**

18. Capture the disabled Security node with its reason and the explanatory empty state; capture
    the automated `Unsupported` results as evidence.

## 14. Milestone order

| Order | ID | Title | Release | Pri | Prerequisites | Rationale |
|---:|---|---|---|---|---|---|
| 1 | E01 | Security Workbench (roles, users) | v0.3 | P2 | Phase A components; `role_management` flag from Phase D's split | The backend exists; this milestone is mostly UI + role detail queries + the DDL view, and it establishes the plan/apply pattern for E02/E03 |
| 2 | E02 | Privilege Matrix (grants) | v0.3 | P2 | E01 | Extends the port with object-scoped and schema-scoped queries; needs the matrix UI and the plan/apply path from E01 |
| 3 | E03 | Membership & Password Management | v0.4 | P3 | E02 | Requires new port methods (`pg_auth_members`, ownership reassignment, password binding) and carries the highest sensitivity (password handling, ownership transfer) |

Ordering rules: E01 must include the plan/apply infrastructure (not just read-only lists);
retrofitting it in E02 would duplicate statement building. E03 is the only milestone in this
phase with a new secret-handling path, so it must not be merged into E02.

## 15. Definition of done

1. **Plan folder** per milestone; `docs/plans/STATUS.md` matches.
2. **P0 = 0, P1 = 0.**
3. **BACKEND_ONLY is eliminated** for the in-scope rows in `PRODUCT_CAPABILITY_MATRIX.md` §11:
   users list, roles create/alter/drop, table grants, schema privileges, memberships, role DDL.
4. **Plan/apply pattern**: every mutating security change has a pure `plan_change` producing
   statements, warnings, and a fingerprint; `apply_change` re-plans and compares fingerprints;
   unit tests assert the executed statements equal the previewed statements.
5. **Injection safety proven**: identifiers quoted through the shared helper, privileges
   allowlisted, and a test suite of hostile inputs asserts no unquoted user text reaches a
   statement.
6. **Password discipline proven**: no password value appears in any plan, event, log, meta-store
   row, or rendered UI string; a password change is verified by an actual re-authentication in
   the PostgreSQL live suite.
7. **Inheritance honesty**: inherited grants are marked and cannot be revoked from the child;
   the refusal names the parent role. Tested.
8. **Ownership pre-check**: dropping a role that owns objects warns before the attempt, and the
   reassignment path (E03) is verified end-to-end.
9. **Read-only and driver guards** are enforced in the service (not only the UI), with tests.
10. **SQLite is gated** with a reason in the UI, a rejection in the service, and no provider
    call; the existing test is retained and extended.
11. **Audit records** exist for every applied change with a statement digest and no secrets.
12. **Quality gates executed and recorded** (fmt, clippy with stated tauri scope,
    `cargo test --workspace`, SQLite suite, PostgreSQL live suite including role lifecycle,
    membership inheritance, and password re-authentication).
13. **Docs updated**: capability matrix §11 rows, master goal §9.6, and `LIM-005` recorded as
    superseded (the limitation said role administration was not implemented; after E01/E02 the
    UI exists).
14. **Non-goals respected**: no RLS authoring, no column/database grants, no default
    privileges, no bulk `GRANT ALL` shortcut, no superuser-by-default, no external identity
    integration.

Phase E is complete when a PostgreSQL administrator can see every role with its attributes,
inspect and edit memberships with inheritance explained, grant and revoke table and schema
privileges through a previewed, allowlisted, audited flow, answer "who can access this object?",
rename a role, rotate a password with proof of re-authentication, and drop a role safely after
reassigning its objects — while a SQLite user sees an explicit, tested unavailability reason.
