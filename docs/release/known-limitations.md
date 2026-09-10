# Known Limitations & Non-Goals — v0.1

> Canonical registry of what DB Pro v0.1 intentionally does not support or has not yet qualified.
> Baseline SHA: `65bbca3`
> Issue: #135
> Supports: #27, #30, #105, #110, #111

## Registry fields

Each entry includes:
- **Title**: short name
- **Category**: connection | schema | query | data-grid | export | er | security | brand | platform | agent
- **Actual behavior**: what the code actually does
- **User-visible impact**: what the user sees
- **Reason**: why this is a limitation
- **Status**: `Accepted v0.1` | `Blocked decision` | `Fix before v0.1` | `Deferred v0.2+`
- **Target issue/release**: tracking issue
- **Safe release-note wording**: what can be said publicly
- **UI/docs locations that must not contradict**: where overclaims could appear
- **Evidence/source**: code references

---

## LIM-001: Public name unresolved

| Field | Value |
|---|---|
| Category | brand |
| Actual behavior | App is named "DB Pro" with identifier `com.dbpro.app`; name collision with existing dbpro.app product |
| User-visible impact | Public release under current name carries legal/branding risk |
| Reason | Name decision pending (#101, #104); competitive research (#137-#141) informs the decision |
| Status | Blocked decision |
| Target issue | #101, #104 |
| Safe release-note wording | "DB Pro (working title)" or omit product name |
| Must not contradict | README, app title bar, installer metadata, website |
| Evidence | #137 dbpro.app teardown; `tauri.conf.json` `productName: "DB Pro"` |

## LIM-002: PostgreSQL + SQLite only

| Field | Value |
|---|---|
| Category | connection |
| Actual behavior | Only PostgreSQL and SQLite drivers are implemented |
| User-visible impact | MySQL, MariaDB, SQL Server, Oracle, etc. are not supported |
| Reason | v0.1 scope decision |
| Status | Accepted v0.1 |
| Target issue | N/A (scope) |
| Safe release-note wording | "Supports PostgreSQL and SQLite" |
| Must not contradict | README, feature list, marketing |
| Evidence | `crates/core/src/domain/connection.rs` DriverType enum |

## LIM-003: Row insertion incomplete

| Field | Value |
|---|---|
| Category | data-grid |
| Actual behavior | Row insert is implemented at the backend level but UI may not expose all column types correctly |
| User-visible impact | Some column types (JSONB, arrays, UUID) may not have proper input widgets |
| Reason | Provider-aware editing is complex; deferred for advanced types |
| Status | Accepted v0.1 |
| Target issue | N/A |
| Safe release-note wording | "Basic row insertion for common types" |
| Must not contradict | Data grid documentation, feature claims |
| Evidence | `DataCapabilities.insert = true` but frontend type handling varies |

## LIM-004: Advanced schema mutation deferred

| Field | Value |
|---|---|
| Category | schema |
| Actual behavior | ALTER COLUMN TYPE is supported for PG but the UI does not expose a schema mutation workflow |
| User-visible impact | Users cannot modify column types, add constraints, or restructure schemas through the UI |
| Reason | Schema mutation UX is complex and error-prone; deferred to v0.2 |
| Status | Deferred v0.2+ |
| Target issue | N/A |
| Safe release-note wording | "Schema browsing and introspection; DDL via query editor" |
| Must not contradict | Schema panel UI, feature descriptions |
| Evidence | `SchemaCapabilities.alter_column_type = true` (PG) / `false` (SQLite) but no schema mutation UI |

## LIM-005: Users/roles administration deferred

| Field | Value |
|---|---|
| Category | connection |
| Actual behavior | PG user_manager port exists but full role administration UI is not implemented |
| User-visible impact | Cannot create/modify database users or roles through the app |
| Reason | Out of v0.1 scope |
| Status | Deferred v0.2+ |
| Target issue | N/A |
| Safe release-note wording | "Database connection management; user administration not included" |
| Must not contradict | Feature list |
| Evidence | `crates/core/src/ports/user_manager.rs` (port only); `FeatureCapabilities.server_sessions` |

## LIM-006: SSH tunnel not E2E qualified

| Field | Value |
|---|---|
| Category | connection |
| Actual behavior | SSH tunnel implementation exists (shells out to `ssh` binary) but has not been end-to-end tested |
| User-visible impact | SSH tunneling may not work reliably; no automated test coverage |
| Reason | E2E qualification requires CI infrastructure for SSH server setup |
| Status | Accepted v0.1 |
| Target issue | N/A |
| Safe release-note wording | "SSH tunnel support is experimental" or omit from release notes |
| Must not contradict | Connection docs, feature claims |
| Evidence | `crates/infrastructure/src/ssh/tunnel.rs`; `FeatureCapabilities.ssh_tunnel = true` (PG) / `false` (SQLite) |

## LIM-007: Agent workspace is Preview

| Field | Value |
|---|---|
| Category | agent |
| Actual behavior | Agent workspace exists as a preview feature; not production autonomous DB execution |
| User-visible impact | Agent features may be unstable or incomplete |
| Reason | Agent is a preview feature for v0.1; full autonomy deferred |
| Status | Accepted v0.1 |
| Target issue | N/A |
| Safe release-note wording | "Agent workspace (Preview)" |
| Must not contradict | Agent documentation, marketing claims |
| Evidence | Agent module in frontend |

## LIM-008: MCP not in v0.1

| Field | Value |
|---|---|
| Category | agent |
| Actual behavior | Model Context Protocol integration is not implemented |
| User-visible impact | No MCP server/client capability |
| Reason | Deferred to v0.2+ |
| Status | Deferred v0.2+ |
| Target issue | N/A |
| Safe release-note wording | Do not mention MCP |
| Must not contradict | Agent docs, feature list |
| Evidence | No MCP code in codebase |

## LIM-009: Signing/notarization absent

| Field | Value |
|---|---|
| Category | security |
| Actual behavior | No code signing or notarization is configured for any platform |
| User-visible impact | macOS Gatekeeper warnings; Windows SmartScreen warnings; Linux no trust indicator |
| Reason | Signing certificate and process not yet established (#118) |
| Status | Blocked decision |
| Target issue | #118 |
| Safe release-note wording | Do not claim signed or notarized |
| Must not contradict | Download page, installation docs |
| Evidence | `tauri.conf.json` has no signing configuration |

## LIM-010: License policy unresolved

| Field | Value |
|---|---|
| Category | brand |
| Actual behavior | No LICENSE file in repository; redistribution policy not decided |
| User-visible impact | Legal ambiguity for public release |
| Reason | License decision pending (#119) |
| Status | Blocked decision |
| Target issue | #119 |
| Safe release-note wording | Do not claim any license |
| Must not contradict | Repository, distribution metadata |
| Evidence | No LICENSE file at repo root |

## LIM-011: CHECK constraint disposition pending

| Field | Value |
|---|---|
| Category | schema |
| Actual behavior | CHECK constraints are introspected but the release disposition (keep/defer/fix) is undecided |
| User-visible impact | May show incorrect CHECK info or miss edge cases |
| Reason | Decision pending (#68) |
| Status | Blocked decision |
| Target issue | #68 |
| Safe release-note wording | "Basic constraint introspection" |
| Must not contradict | Schema panel, introspection docs |
| Evidence | `SchemaCapabilities` does not have a `check_constraints` field |

## LIM-012: Import deferred

| Field | Value |
|---|---|
| Category | export |
| Actual behavior | Data import (CSV, JSON → table) is not implemented |
| User-visible impact | Users cannot import data files into tables |
| Reason | Out of v0.1 scope |
| Status | Deferred v0.2+ |
| Target issue | N/A |
| Safe release-note wording | "Export to CSV and XLSX; import not yet available" |
| Must not contradict | Data grid docs, feature list |
| Evidence | No import code in infrastructure; `BackupEngine` only handles backup/restore |

## LIM-013: SQLite type limitations

| Field | Value |
|---|---|
| Category | schema |
| Actual behavior | SQLite stores UUID as TEXT, JSON as TEXT, has no native array/enum/generated-column support |
| User-visible impact | Type display may differ from PostgreSQL equivalent; no UUID validation, no JSON editing |
| Reason | SQLite's type system is fundamentally different (affinity-based) |
| Status | Accepted v0.1 |
| Target issue | #132 (capability matrix) |
| Safe release-note wording | "SQLite support with type-system limitations" |
| Must not contradict | Provider capability matrix, type documentation |
| Evidence | `DataCapabilities.sqlite()` — `uuid_type: false`, `array_types: false`, `generated_columns: false` |

## LIM-014: No query cancellation for SQLite

| Field | Value |
|---|---|
| Category | query |
| Actual behavior | SQLite queries cannot be cancelled once started |
| User-visible impact | Long-running SQLite queries block the UI until completion |
| Reason | SQLite has no built-in cancel mechanism |
| Status | Accepted v0.1 |
| Target issue | #132 |
| Safe release-note wording | "Query cancellation for PostgreSQL" (not SQLite) |
| Must not contradict | Query editor docs |
| Evidence | `QueryCapabilities.sqlite().cancel = false` |

## LIM-015: pg_dump/pg_restore not bundled

| Field | Value |
|---|---|
| Category | platform |
| Actual behavior | PostgreSQL backup shells out to `pg_dump`/`pg_restore` which must be on PATH |
| User-visible impact | Backup/restore fails silently if pg_dump is not installed |
| Reason | Bundling pg_dump adds significant size; not done in v0.1 |
| Status | Accepted v0.1 |
| Target issue | #134 |
| Safe release-note wording | "PostgreSQL backup requires pg_dump on PATH" |
| Must not contradict | Backup docs, platform prerequisites |
| Evidence | `crates/infrastructure/src/backup/pg_dump.rs` |

---

## Summary by status

| Status | Count | IDs |
|---|---|---|
| Accepted v0.1 | 8 | LIM-002, LIM-003, LIM-005, LIM-006, LIM-007, LIM-013, LIM-014, LIM-015 |
| Blocked decision | 4 | LIM-001, LIM-009, LIM-010, LIM-011 |
| Deferred v0.2+ | 3 | LIM-004, LIM-008, LIM-012 |
| Fix before v0.1 | 0 | — |

## Rules

- This registry tracks decisions dynamically; do not freeze stale assumptions.
- If a limitation is fixed before release, mark superseded with evidence instead of deleting history.
- Any limitation that actually constitutes P0/P1 correctness/security must not be hidden as a harmless known limitation.
- Differentiate unsupported from unqualified/experimental.
