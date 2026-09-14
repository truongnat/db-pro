# Platform Native Prerequisites — v0.1 (native `db-pro-native`)

- Baseline SHA: `main @ 5866f09` (the tree this inventory was read against)
- Issue: **#134** ([RC1][Platform] Inventory packaged-runtime native prerequisites and OS-specific
  dependency risks)
- Scope: **audit and documentation only** — no packaging behavior, workflow, artifact, checksum or
  toolchain change.
- Authoritative sources: `.github/workflows/release.yml` (build + package jobs and the exact
  `apt-get install` lists), `scripts/release/package-{macos,linux}.sh`,
  `scripts/release/README-INSTALL.txt` (the consumer-facing counterpart), `Cargo.toml` feature
  flags, `Cargo.lock` (resolved keyring backends), and
  `docs/release/evidence/v01-06/08-post-fix-quality-gates.txt` (measured `minos`).

> **Historical note.** An earlier revision of this document described the Tauri/WebView era
> (`crates/tauri-app`, React frontend, `.dmg`/`.deb`/`.rpm`/MSI installers, WebKitGTK, WebView2).
> None of that is in the v0.1 delivery path: the shipped artifact is the native `db-pro-native`
> (eframe/egui) archive. Tauri-era rows are kept at the end as historical reference only.

## 1. What v0.1 actually ships

| Item | Value | Source |
|---|---|---|
| Binary | `db-pro-native` (native eframe/egui; no WebView, no Node) | `crates/native-app`, `README.md:4,159` |
| macOS artifact | `db-pro-v0.1.0-macos-arm64.tar.gz` → `DB Pro.app` + `README-INSTALL.txt` | `package-macos.sh`, `release.yml:14` |
| Windows artifact | `db-pro-v0.1.0-windows-x86_64.zip` → `db-pro-native.exe` + `README-INSTALL.txt` | `package-windows.ps1`, `release.yml:15` |
| Linux artifact | `db-pro-v0.1.0-linux-x86_64.tar.gz` → `db-pro-native` + `README-INSTALL.txt` | `package-linux.sh`, `release.yml:16` |
| Installers | none (no `.dmg`/MSI/NSIS/`.deb`/`.rpm`/AppImage) | `risk-register.md:110` (R-PKG-DEFER), LIM-017 |
| Updater | none | `0.1.0-versioning-updater-persistence.md` §2 (#127) |
| Signing / notarization | none (unsigned archives) | LIM-017, #118 |

## 2. Prerequisite matrix

`build` = needed on the machine that compiles/qualifies the release; `run` = needed on the machine
that executes the packaged binary.

| OS | Arch | Package | Build prerequisites | Runtime prerequisites | Missing-dependency behavior |
|---|---|---|---|---|---|
| macOS | arm64 only | `.tar.gz` with `DB Pro.app` | Xcode command line tools, `rustup` with the pinned toolchain (`rust-toolchain.toml`), `macos-14` runner | macOS **11.0+** (measured `LC_BUILD_VERSION minos 11.0` — `evidence/v01-06/08-post-fix-quality-gates.txt:163`), system Keychain (Security.framework), the OS OpenGL/Metal driver | Gatekeeper blocks an unsigned app on first launch → right-click → Open; Keychain authorization prompt on first credential access |
| Windows | x86_64 | `.zip` | MSVC build tools (present on the `windows-latest` image), `rustup` + pinned toolchain | Windows 10/11 x86_64, Windows Credential Manager, system OpenGL/D3D driver. **No WebView2 runtime is required** — the shipping binary is not a Tauri/WebView host | SmartScreen warns on an unsigned binary ("More info" → "Run anyway") |
| Linux | x86_64 | `.tar.gz` | `libxkbcommon-dev libwayland-dev libx11-dev libgl1-mesa-dev libdbus-1-dev pkg-config` (`release.yml:191-193` preflight, `:397-399` build job) | X11 or Wayland session, a working OpenGL driver, `libdbus-1.so.3` (linked through `dbus-secret-service` → `libdbus-sys`, resolved in `Cargo.lock`), and a **D-Bus Secret Service provider** (`gnome-keyring`, KWallet, KeePassXC) for credential storage | Without a Secret Service provider, credential writes fail — release builds keep secrets in memory for the session only (#142); without an OpenGL driver, window creation fails |

The Linux row is the one to check against the real package list: the workflow's two
`apt-get install` blocks are the build-time truth, and they agree with the runtime list above. No
WebKitGTK/GTK/AppIndicator package is installed or needed for `-p db-pro-native`
(`release.yml:385-392` says this explicitly — the preflight job's larger list exists only because
that job also builds the legacy `crates/tauri-app`).

## 3. Provider and external-tool prerequisites

| Tool | Required for | Bundled? | Notes |
|---|---|---|---|
| `pg_dump` | PostgreSQL backup | **No** | must be on `PATH`; when missing, the backup action fails with the shell-out error and no in-app guidance |
| `pg_restore` | PostgreSQL restore | **No** | must be on `PATH`; **version skew matters** — an 18.x `pg_restore` exits 1 against a 16.x server (`evidence/v01-runtime/providers/23`, finding `F1`/P2) |
| `ssh` | SSH tunnels | **No** | system client, must be on `PATH` |
| `psql`, `sqlite3` | — | Not required | the app speaks the wire protocol through `sqlx`; SQLite runs in-process |
| SQLite library | all SQLite work | **Yes** | `rusqlite` `bundled` feature (`Cargo.toml:28`); no system `libsqlite3` |
| TLS | PostgreSQL TLS | **Yes** | `sqlx` `runtime-tokio-rustls`; no OpenSSL, no system CA-bundle dependency beyond rustls' roots |

This matches the consumer-facing note in `scripts/release/README-INSTALL.txt` ("PostgreSQL backup
and restore shell out to `pg_dump` and `pg_restore`, and SSH tunnels use the system `ssh` client")
and the release notes (`0.1.0-release-notes.md:110-111`). Nothing claims these tools are bundled.

## 4. Packaging risks carried by v0.1

| Risk | Severity | Status |
|---|---|---|
| Unsigned, not-notarized archives trigger Gatekeeper (macOS) / SmartScreen (Windows) | P2 | accepted and recorded (LIM-017, #118); the install note gives the workaround |
| No macOS x86_64 artifact (arm64 only) | P2 | accepted and recorded (LIM-017) |
| Linux credential storage needs a Secret Service provider; minimal/headless installs cannot write credentials | P2 | accepted and recorded in the release notes and install note; #142 removed the plaintext-file fallback from release builds, so the degradation is session-only |
| `pg_dump`/`pg_restore` not bundled and version-skew sensitive | P2 | recorded (findings `F1`/P2, providers/23); prerequisite stated in the install note and release notes |
| Windows and Linux archives are `RUNTIME_UNVERIFIED` (no host available) | P1 for release claims | recorded in the risk register and readiness docs; not a prerequisites defect |
| No installers/package-manager metadata, so no OS-level dependency declaration exists for any platform | informational | by design (portable archives); §2 is the substitute |

**No new release-critical missing dependency was found by this reconciliation:** the Linux build
list in `release.yml` is the complete build set for `-p db-pro-native`, the runtime libraries it
implies are stated in §2, and the external tools are stated in §3. No focused child issue is
spawned.

## 5. Smoke setup checklist (#91-#93)

Record these before a packaged-runtime smoke run starts:

1. **macOS** — Apple Silicon host, macOS 11+, archive unpacked, Gatekeeper override performed,
   unlockable Keychain, `pg_dump`/`pg_restore`/`ssh` on `PATH` if those features are in scope.
2. **Windows** — x86_64 Windows 10/11 host, archive unpacked, SmartScreen override performed,
   Credential Manager available, external tools on `PATH` for the backup/SSH items.
3. **Linux** — x86_64 host with an X11 or Wayland session and a working OpenGL driver,
   `libdbus-1.so.3` present, a running Secret Service provider (or the credential items are
   expected to fail), external tools on `PATH` for the backup/SSH items.
4. **All platforms** — `SHA256SUMS.txt` verified before launch, a writable data directory
   (`DB_PRO_DATA_DIR` or the platform default — see
   `0.1.0-versioning-updater-persistence.md` §3), and the exact archive SHA recorded in the smoke
   worksheet.

The manual-smoke worksheet classifies unexecuted rows as blocked
(`docs/release/0.1.0-manual-smoke.md`, tallies `passed 0 / blocked 165 / failed 0`); this checklist
is what an operator must satisfy before those rows can be unblocked.

## 6. Historical reference (Tauri era — not part of v0.1)

Kept for traceability only; do not use for release claims.

- Tauri-era package targets (`deb`, `appimage`, `rpm`, `dmg`, `msi`, `nsis`) and the Linux
  `deb`/`rpm` `depends` lists lived in `crates/tauri-app/tauri.conf.json`; that bundler path is
  archived with the webview frontend (`_archive/README.md`).
- WebKitGTK 4.1, WebView2 and `libayatana-appindicator3` were required by that host; they are not
  required by `db-pro-native`.
- The old `libsecret`-dependency finding assumed declared package metadata (`deb`/`rpm` `depends`),
  which v0.1 does not have at all; the live equivalent is the D-Bus Secret Service runtime
  requirement in §2.
