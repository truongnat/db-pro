# Known Limitations & Non-Goals — v0.1

> Canonical registry of what DB Pro v0.1 intentionally does not support or has not yet qualified.
> Baseline SHA: `65bbca3` (historical); **corrected 2026-09-14 for candidate `fbf9fdab9100f08f12e29434983f32c18f14ac2f`**
> Corrections in this revision: LIM-001/LIM-007/LIM-009 evidence updated to the native build, LIM-003 wording aligned with the staged insert that ships, **LIM-014 inversion fixed** (SQLite cancellation is supported; PostgreSQL is not), LIM-016/LIM-017 added. No entry was deleted.
> Further correction (2026-09-14, after the `543b526` state-directory fix): **LIM-018 added** — application state location and the first-launch credential prompt. No entry was deleted or rewritten.
> Risk IDs referenced below (`R003`, `R-LICENSE`, `R001`) are defined in `docs/release/risk-register.md`.
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
| Evidence | #137 dbpro.app teardown; native literals `with_title("DB Pro")` in `crates/native-app/src/main.rs` (three occurrences, no icon resource and no `Info.plist` in-repo — the packaging script generates a minimal plist). Register: `R001` (disposition `DEFERRED`) |

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
| Actual behavior | Staged row insert **ships** in the native Table Data Editor (insert dialog + staged commit, PK-targeted). It is narrower than a full insert workflow because complex column types do not have complete input widgets. |
| User-visible impact | Basic inserts work for common types; JSONB/array/UUID columns may lack a proper input widget |
| Reason | Provider-aware editing is complex; deferred for advanced types |
| Status | Accepted v0.1 |
| Target issue | N/A |
| Safe release-note wording | "Basic row insertion for common types" |
| Must not contradict | Data grid documentation, feature claims |
| Evidence | `DataCapabilities.insert = true`; `crates/ui/src/table_editor_view.rs` insert path; native `ChangeSet` staged insert. Corrected 2026-09-14: the earlier "implemented at the backend level but UI may not expose" wording was the pre-native framing |

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
| Evidence | `crates/ui/src/agent_view.rs` (Ask/Edit/Agent; renders a `Preview` badge); confirmation gate in the agent executor. No live-provider run is recorded — `R-AGENT` (`ACCEPTED`) |

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
| Evidence | Measured on the release binary: `Signature=adhoc`, `flags=0x20002(adhoc,linker-signed)`, `TeamIdentifier=not set`, `Info.plist=not bound`; `spctl -a -vvv -t execute` → **rejected** (exit 3). No `notarytool`/`stapler`/`signtool` step exists; no signing secrets exist and none were added. Register: `R003` (`ACCEPTED`) |

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

## LIM-014: Provider-asymmetric query cancellation

| Field | Value |
|---|---|
| Category | query |
| Actual behavior | **SQLite cancellation works** (the actor interrupts the running VM and waits for acknowledgement). **PostgreSQL cancellation is not implemented**: the connector exposes no wire-level cancel, and the capability is declared `Unsupported` and gates the Stop control off. |
| User-visible impact | Stop/Escape cancels a SQLite query; a long PostgreSQL query cannot be cancelled from the app |
| Reason | PostgreSQL's cancel primitive is not wired through the connector; advertising best-effort task cancellation as provider cancellation would be misleading |
| Status | Accepted v0.1 |
| Target issue | #132 |
| Safe release-note wording | "Query cancellation is supported for SQLite; PostgreSQL cancellation is not available in 0.1.0" |
| Must not contradict | Query editor docs, release notes, provider capability matrix |
| Evidence | `crates/core/src/domain/capabilities.rs`: `postgres.cancel = false`, `sqlite.cancel = true`; `crates/infrastructure/src/postgres/connector.rs:199-203`; `crates/infrastructure/src/sqlite/actor.rs:106-124`. **Corrected 2026-09-14 — the previous entry asserted the exact opposite ("SQLite queries cannot be cancelled", `sqlite.cancel = false`) and contradicted the code.** Register: `R-PROV` |

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

## LIM-016: Workspace/session persistence not implemented

| Field | Value |
|---|---|
| Category | platform |
| Actual behavior | The native build does not persist workspace tabs or settings across restarts (eframe persistence is off); previously active *connections* reconnect on startup, but tabs/drafts are not restored |
| User-visible impact | After quitting, the tab layout and open resources are gone; only the startup connection reconnect is restored |
| Reason | Persistence is deferred; implementing it is feature work outside the v0.1 closure scope |
| Status | Accepted v0.1 |
| Target issue | N/A |
| Safe release-note wording | Do not claim workspace/tab restore after restart |
| Must not contradict | `0.1.0-manual-smoke.md` "Workspace tabs restore" item, README, release notes |
| Evidence | eframe persistence feature not enabled; `docs/release/risk-register.md` `R-015` (`ACCEPTED`) |

## LIM-017: macOS ARM64 only; no installers or auto-update

| Field | Value |
|---|---|
| Category | platform |
| Actual behavior | The v0.1 matrix produces a macOS **arm64** archive (Apple Silicon only), a Windows x86_64 `.zip` and a Linux x86_64 `.tar.gz`. No macOS x86_64 artifact, no universal binary, no `.dmg`/MSI/NSIS/`.deb`/`.rpm`/AppImage, no auto-update. All artifacts are unsigned. |
| User-visible impact | Intel Macs have no artifact; there is no installer or update mechanism; macOS/Windows show OS security warnings on first run |
| Reason | Recorded deferral decision (Option A packaging contract); signing identity and installer toolchains do not exist |
| Status | Accepted v0.1 |
| Target issue | N/A |
| Safe release-note wording | "Apple Silicon macOS, Windows x86_64 and Linux x86_64 portable archives; unsigned; no installers" |
| Must not contradict | `0.1.0-packaging.md`, release notes, readiness |
| Evidence | `.github/workflows/release.yml` (matrix pins `macos-14`), `docs/release/0.1.0-packaging.md`, `docs/release/risk-register.md` `R-PKG-DEFER` / `R003` |

## LIM-018: Application state location and first-launch credential prompt

| Field | Value |
|---|---|
| Category | platform |
| Actual behavior | State (`meta.db`, `secrets/`, query history, settings) is stored in the per-user platform data directory — macOS `~/Library/Application Support/DB Pro`, Windows `%APPDATA%\DB Pro`, other Unix `$XDG_DATA_HOME/db-pro` (fallback `~/.local/share/db-pro`) — **unless** `<cwd>/.db-pro-data` already exists, which keeps being used; `DB_PRO_DATA_DIR` overrides both. The build is unsigned, so on macOS a first launch can show a Keychain authorization prompt for the app's own stored provider key before the window appears, and an unattended launch waits there. |
| User-visible impact | On a fresh install the state directory is no longer created next to the working directory; an existing `.db-pro-data` continues to be used, so nothing migrates and no data is orphaned. A first launch can pause on an OS credential prompt. |
| Reason | The previous fallback (`current_dir()/.db-pro-data`) made a Finder/`open`-launched `.app` (which inherits `cwd=/`) resolve `/.db-pro-data`, fail with `CreateDataDir(ReadOnlyFilesystem)` and exit before opening a window; fixed in `543b526`. The prompt follows from shipping unsigned (`R003`): an ad-hoc-signed binary cannot read an existing keyring item without an interactive grant. |
| Status | Accepted v0.1 |
| Target issue | N/A |
| Safe release-note wording | "Application state is stored in your user profile; an existing `.db-pro-data` beside the app is still used. An unsigned build can ask for keychain access on first launch." |
| Must not contradict | `0.1.0-handoff.md` §7, `0.1.0-release-notes.md`, `0.1.0-packaging.md` §4, `README.md` |
| Evidence | `crates/native-app/src/main.rs` (`choose_data_dir`, `platform_data_dir`; tests `platform_dir_is_used_when_no_legacy_dir_exists` and the override/legacy/cwd cases); `docs/release/evidence/v01-06/12-state-dir-blocker-fix.txt` §5–§6; register `R-STATE-DIR` (`FIXED`), `R003` (`ACCEPTED`), `R011` |

---

## Summary by status

| Status | Count | IDs |
|---|---|---|
| Accepted v0.1 | 11 | LIM-002, LIM-003, LIM-005, LIM-006, LIM-007, LIM-013, LIM-014, LIM-015, LIM-016, LIM-017, LIM-018 |
| Blocked decision | 4 | LIM-001, LIM-009, LIM-010, LIM-011 |
| Deferred v0.2+ | 3 | LIM-004, LIM-008, LIM-012 |
| Fix before v0.1 | 0 | — |

## Rules

- This registry tracks decisions dynamically; do not freeze stale assumptions.
- If a limitation is fixed before release, mark superseded with evidence instead of deleting history.
- Any limitation that actually constitutes P0/P1 correctness/security must not be hidden as a harmless known limitation.
- Differentiate unsupported from unqualified/experimental.
