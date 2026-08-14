# Platform Native Prerequisites — v0.1

> Source: `tauri.conf.json`, `Cargo.toml`, `capabilities/default.json`, infrastructure code.
> Baseline SHA: `65bbca3`
> Issue: #134

## Build configuration summary

| Setting | Value | Source |
|---|---|---|
| Product name | DB Pro | `tauri.conf.json` `productName` |
| Version | 0.1.0 | `tauri.conf.json` `version` |
| Identifier | com.dbpro.app | `tauri.conf.json` `identifier` |
| Bundle targets | deb, appimage, rpm, dmg, msi, nsis | `tauri.conf.json` `bundle.targets` |
| Rust edition | 2021 | `Cargo.toml` workspace |
| Min Rust version | 1.77.2 | `Cargo.toml` `rust-version` |
| Tauri version | 2 | `tauri-app/Cargo.toml` |
| Window min size | 1024×640 | `tauri.conf.json` |
| Frontend dist | `frontend/dist` | `tauri.conf.json` `build.frontendDist` |

## Native dependencies

### Rust crates with native bindings

| Crate | Version | Feature flags | Native requirement |
|---|---|---|---|
| `sqlx` | 0.8 | `postgres`, `runtime-tokio-rustls`, `chrono`, `uuid`, `json` | TLS via rustls (no OpenSSL needed) |
| `rusqlite` | 0.32 | `bundled`, `column_decltype` | SQLite compiled from bundled source (no system libsqlite3) |
| `keyring` | 3 | — | Platform credential storage (see per-OS below) |
| `aes-gcm` | 0.10 | — | Pure Rust (no native dep) |
| `argon2` | 0.5 | — | Pure Rust (no native dep) |

### Tauri plugins

| Plugin | Version | Purpose |
|---|---|---|
| `tauri-plugin-dialog` | 2 | File picker (SQLite database selection) |

### Capabilities (permissions)

| Permission | Purpose |
|---|---|
| `core:default` | Standard Tauri core permissions |
| `dialog:allow-open` | Open file dialog for SQLite file picker |

## Platform matrix

### macOS

| Item | Detail |
|---|---|
| Package format | `.dmg` |
| Architecture | Determined by build host (x64 or aarch64) |
| Minimum OS | macOS 10.15+ (Tauri 2 default; WebView2 equivalent via WebKit) |
| WebView | System WebKit (bundled with macOS) — no additional install |
| Keyring | macOS Keychain (via `keyring` crate → Security.framework) |
| TLS | rustls (bundled, no system OpenSSL) |
| SQLite | Bundled (rusqlite `bundled` feature) |
| Signing | Not configured in v0.1 (#118) |
| Notarization | Not configured in v0.1 (#118) |
| Unsigned behavior | Gatekeeper may block; user must right-click → Open or allow in System Settings |
| File picker | `NSSavePanel`/`NSOpenPanel` via Tauri dialog plugin |
| System prompts | Keychain access prompts on first use; no special entitlements needed |

**Risks:**
- Unsigned builds will show "damaged" or "can't be opened" on macOS Sequoia+ without user override.
- No universal binary — separate x64 and aarch64 builds needed for full coverage.

### Windows

| Item | Detail |
|---|---|
| Package formats | `.msi` (Windows Installer), `.nsis` (NSIS installer) |
| Architecture | x64 (primary); x86 possible but not tested |
| WebView | WebView2 Runtime — **must be present on target system** |
| WebView2 bundling | Tauri 2 does NOT bundle WebView2 by default; it's a bootstrapper dependency |
| VC Runtime | Required (MSVC toolchain builds link against `vcruntime140.dll`) |
| Keyring | Windows Credential Manager (via `keyring` crate → `credman`) |
| TLS | rustls (bundled) |
| SQLite | Bundled |
| Code signing | Not configured in v0.1 (#118) |
| SmartScreen | Unsigned `.msi`/`.exe` triggers Windows SmartScreen warning |
| Firewall | No network server opened; outbound connections to PG servers only |
| Certificate store | Not used in v0.1 |

**Risks:**
- WebView2 Runtime is pre-installed on Windows 11 and recent Windows 10 updates, but older systems may need the Evergreen Bootstrapper.
- Unsigned MSIs trigger SmartScreen — users must click "Run anyway."
- No code signing certificate configured.

### Linux

| Item | Detail |
|---|---|
| Package formats | `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL), `.appimage` (universal) |
| Architecture | x86_64 |
| **deb depends** | `libwebkit2gtk-4.1-0`, `libgtk-3-0` |
| **rpm depends** | `webkit2gtk4.1`, `gtk3` |
| WebView | WebKitGTK 4.1 — **must be installed** (declared as package dependency) |
| AppImage | Requires FUSE for direct execution; `--appimage-extract` alternative |
| Keyring | `libsecret` / Secret Service D-Bus API (GNOME Keyring, KWallet, KeePassXC) |
| TLS | rustls (bundled) |
| SQLite | Bundled |
| Signing | Not configured in v0.1 |

**deb declared dependencies** (from `tauri.conf.json`):
```json
"deb": { "depends": ["libwebkit2gtk-4.1-0", "libgtk-3-0"] }
```

**rpm declared dependencies** (from `tauri.conf.json`):
```json
"rpm": { "depends": ["webkit2gtk4.1", "gtk3"] }
```

**Missing dependency declarations:**
- `libsecret` / `libsecret-1-0` — needed by `keyring` crate for credential storage. Not declared in deb/rpm depends. **This is a packaging bug risk.**
- `libssl` — not needed (rustls is bundled), but if any transitive dep requires OpenSSL, it's undeclared.
- `glib2` — pulled in transitively by GTK3 but not explicitly declared.

**Risks:**
- Missing `libsecret` dependency will cause runtime credential storage failure on minimal installs.
- AppImage + FUSE not available on all distros (e.g., RHEL 9+ removed FUSE 2).
- WebKitGTK 4.1 is relatively new; older distros (Ubuntu 20.04, Debian 11) ship 4.0 and cannot run the app.

## Provider tooling

| Tool | Required? | Bundled? | Notes |
|---|---|---|---|
| `psql` | No | No | Not required; PG connection is over TCP via sqlx |
| `pg_dump` | Yes (for backup) | **No** | PG backup shells out to `pg_dump` — must be on PATH |
| `pg_restore` | Yes (for restore) | **No** | PG restore shells out to `pg_restore` — must be on PATH |
| `sqlite3` | No | No | SQLite is handled by rusqlite (bundled) |
| `ssh` | Yes (for SSH tunnel) | **No** | SSH tunnel shells out to system `ssh` — must be on PATH |

**Critical finding:** `pg_dump` and `pg_restore` are NOT bundled and must exist on the user's PATH for backup/restore to work. This is a runtime prerequisite that is not documented in the app or installer.

## Minimum OS support matrix

| OS | Minimum version | WebView | Package | Risk level |
|---|---|---|---|---|
| macOS | 10.15 (Catalina) | System WebKit | .dmg | Medium (unsigned) |
| Windows | 10 1809+ | WebView2 | .msi / .nsis | Medium (unsigned + WebView2) |
| Ubuntu | 22.04+ | WebKitGTK 4.1 | .deb | Low |
| Fedora | 36+ | WebKitGTK 4.1 | .rpm | Low |
| Debian | 12+ | WebKitGTK 4.1 | .deb | Low |
| RHEL/Alma | 9+ | WebKitGTK 4.1 | .rpm | Medium (FUSE for AppImage) |
| Arch | Current | WebKitGTK 4.1 | .pkg / AppImage | Low |

## Actionable findings

1. **[P1] Missing `libsecret` dependency** in deb/rpm package declarations — will break credential storage on minimal Linux installs.
2. **[P2] `pg_dump`/`pg_restore` not bundled** — backup/restore requires these on PATH; no in-app guidance.
3. **[P2] No WebView2 bootstrapper** for Windows — older Windows 10 systems may fail silently.
4. **[P2] No code signing** — all platforms show security warnings to users.
5. **[P3] No universal macOS binary** — separate x64/aarch64 builds needed.

## Source references

- `crates/tauri-app/tauri.conf.json` — bundle config, Linux depends
- `crates/tauri-app/Cargo.toml` — Tauri + plugin dependencies
- `crates/tauri-app/capabilities/default.json` — permission declarations
- `Cargo.toml` (workspace) — sqlx/rusqlite/keyring feature flags
- `crates/infrastructure/src/backup/pg_dump.rs` — pg_dump shell-out
- `crates/infrastructure/src/ssh/tunnel.rs` — ssh shell-out
- `crates/infrastructure/src/secret/keyring_vault.rs` — keyring usage
