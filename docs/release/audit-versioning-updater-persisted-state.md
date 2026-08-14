# Release Prep Audit — Versioning, Updater, Persisted-State Compatibility

> Baseline SHA: `a2cf4a3`
> Issue: #127
> Source: `tauri.conf.json`, `Cargo.toml`, `package.json`, `frontend/package.json`, `workspace.store.ts`, `workspace-migrations.ts`, `lib.rs`, `meta/store.rs`

## Version metadata matrix

| Manifest | Field | Value | Source of truth? |
|---|---|---|---|
| `crates/tauri-app/tauri.conf.json` | `version` | `0.1.0` | **Yes** — Tauri build reads this |
| `crates/tauri-app/tauri.conf.json` | `productName` | `DB Pro` | Yes |
| `crates/tauri-app/tauri.conf.json` | `identifier` | `com.dbpro.app` | Yes — bundle ID / app-data path key |
| `Cargo.toml` (workspace) | `version` | not set at workspace level | Each crate has own version |
| `crates/tauri-app/Cargo.toml` | `version` | `0.1.0` | Must match tauri.conf.json |
| `package.json` (root) | `version` | `0.1.0` | Informational only |
| `frontend/package.json` | `version` | `0.1.0` | Informational only |

**Finding F1 [P2]**: Version `0.1.0` is manually duplicated across 4 manifests. No automated sync or validation. A mismatch between `tauri.conf.json` and `Cargo.toml` would cause Tauri build warnings or inconsistent metadata.

**Recommendation**: ACCEPT RC1. Add a CI check or script to validate version consistency before release.

## Updater configuration

| Component | Present? | Details |
|---|---|---|
| `tauri-plugin-updater` | **No** | Not in any `Cargo.toml` |
| Updater endpoint config | **No** | No `updater` section in `tauri.conf.json` |
| Public key / channel | **No** | — |
| Auto-update logic | **No** | — |

**Finding F2 [P2 — Decision needed]**: No updater system exists. v0.1 distribution is **manual download only**. This is correct for an RC1 — auto-update infrastructure adds complexity that isn't needed for initial release.

**Decision**: `update_delivery_model = manual`. Users download new versions from the release page. No dormant updater config to confuse.

## Persisted state inventory

### Backend (Rust)

| Store | Location | Schema management | Migration? |
|---|---|---|---|
| `meta.DB` (SQLite) | `app_data_dir()/meta.DB` | `SCHEMA` constant + `migration::migrate()` | Yes — versioned migrations in `meta/migration.rs` |
| Secrets directory | `app_data_dir()/secrets/` | Flat files per secret | N/A |
| Keyring entries | System keychain (`com.dbpro.app` service) | Keyed by connection ID | Implicit — new key per connection |

**App data path derivation**: `handle.path().app_data_dir()` → platform-specific:
- macOS: `~/Library/Application Support/com.dbpro.app/`
- Windows: `%APPDATA%/com.dbpro.app/`
- Linux: `~/.local/share/com.dbpro.app/`

**Finding F3 [P2]**: If rename (#103) changes `identifier` from `com.dbpro.app`, all persisted data (meta.DB, secrets) becomes orphaned at the old path. No migration path exists.

**Finding F4 [P2]**: The keyring service name `"com.dbpro.app"` is hardcoded in `lib.rs:50`. If the identifier changes, existing keyring entries become inaccessible. The encrypted fallback files in `secrets/` would still work if the path is updated.

### Frontend (Browser localStorage)

| Store | Key | Persistence | Migration? |
|---|---|---|---|
| `useWorkspaceStore` | `workspace-storage` (zustand persist) | localStorage | Yes — `workspaceVersion` = 3, 3 migrations |
| ER diagram positions | `er-pos:{connectionId}:{schema}` | localStorage | No — overwritten on drag |
| `useConnectionStore` | In-memory only | No | N/A |
| `useThemeStore` | `theme-storage` (zustand persist) | localStorage | No — simple key/value |
| `useQueryHistoryStore` | `query-history` (zustand persist) | localStorage | No — append-only log |
| `useRecentStore` | `recent-storage` (zustand persist) | localStorage | No — bounded list |
| `useCommandStore` | `command-storage` (zustand persist) | localStorage | No — ephemeral |
| `useShellStore` | `shell-storage` (zustand persist) | localStorage | No — UI layout |
| `useSettingsStore` | `settings-storage` (zustand persist) | localStorage | No — defaults fill missing |
| `useCloseGuardStore` | In-memory only | No | N/A |
| `useCrashRecoveryStore` | In-memory only | No | N/A |

**Finding F5 [P2]**: Workspace migrations are well-implemented (v0→v1→v2→v3) with `onRehydrateStorage` hook. However, other persisted stores (theme, query-history, recent, command, shell, settings) have **no version field and no migration path**. If their schema changes in a future version, old data could cause deserialization errors.

**Recommendation**: ACCEPT RC1. The zustand persist middleware uses JSON.parse which is lenient — missing fields get defaults. But add version fields to stores before v0.2.

**Finding F6 [P2]**: ER diagram position cache has no version or size limit. Over time with many connections/schemas, localStorage could accumulate stale entries. Low priority for v0.1.

## Persisted-state compatibility policy

**Decision record**:
1. **Update delivery**: Manual download. No auto-updater.
2. **Persisted-state compatibility**: Best-effort. Workspace has versioned migrations. Other stores rely on JSON leniency.
3. **Legacy beta/dev state**: May be reset. No migration guarantee for pre-v0.1 installs.
4. **Version source of truth**: `tauri.conf.json` `version` field. All other manifests should match.
5. **Bundle identifier stability**: `com.dbpro.app` is the stable identifier. Changes require a data migration plan.

## Summary

| Severity | Count | Findings |
|---|---|---|
| P1 | 0 | — |
| P2 | 6 | F1-F6 (all ACCEPT RC1) |

**Conclusion**: v0.1 has no updater (correct for RC1), version is consistently `0.1.0` across 4 manifests (manually), workspace persistence has proper versioned migrations (v0-v3), but 6 other localStorage stores lack version fields. The identifier `com.dbpro.app` is the anchor for app-data paths and keyring — any rename (#103) must include a data migration plan.
