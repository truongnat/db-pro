# Packaged-runtime native prerequisites reconciliation (#134)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 5866f09` (clean at the start of the change); commit recorded in `LEDGER.md`
- Issue: **#134** ([RC1][Platform] Inventory packaged-runtime native prerequisites and OS-specific
  dependency risks) — parent runtime workstream **#29**
- Deliverable: rewritten `docs/release/platform-prerequisites.md`

## The finding

The ledger triage flagged the row `PARTIAL_ON_MAIN` because `docs/release/platform-prerequisites.md`
carried a 2026-09-11 amendment on top of a document that still described the **Tauri/WebView era**:
`.dmg`/`.deb`/`.rpm`/MSI/NSIS targets, WebKitGTK 4.1, WebView2, `libsecret` declared as a `deb`/`rpm`
dependency, and a "missing `libsecret` declaration (P1)" finding. None of that is in the v0.1
delivery path — the shipped artifact is the native `db-pro-native` portable archive
(`release.yml:14-16`, `package-linux.sh` comment "no .deb/.rpm/AppImage is part of the v0.1
contract"). So the document both overclaimed prerequisites that no longer matter and under-described
the runtime dependency it does have.

## The reconciliation

| Aspect | Before | After (grounded in) |
|---|---|---|
| Shipped artifacts | `.dmg`/`deb`/`rpm`/`appimage`/`msi`/`nsis` (Tauri) | three portable archives: `macos-arm64.tar.gz`, `windows-x86_64.zip`, `linux-x86_64.tar.gz` (`scripts/release/package-*.sh`, `release.yml:14-16`) |
| WebView | WebKitGTK 4.1 / WebView2 required | not required — the binary is native eframe/egui (`README.md:4,159`); **no WebView2 runtime prerequisite on Windows** |
| Linux build deps | WebKitGTK/GTK + a disputed `libsecret` P1 | the workflow's actual list: `libxkbcommon-dev libwayland-dev libx11-dev libgl1-mesa-dev libdbus-1-dev pkg-config` (`release.yml:191-193`, `:397-399`), with the reason the preflight job additionally installs WebKitGTK/GTK (it runs `cargo test --workspace`, which builds the legacy `crates/tauri-app`) |
| Linux runtime deps | `libsecret` "packaging bug" | X11/Wayland + OpenGL driver + `libdbus-1.so.3` (verified via `cargo tree --target all -p db-pro-infrastructure`: `keyring → dbus-secret-service → dbus → libdbus-sys`) + a running D-Bus Secret Service provider — the same requirement `README-INSTALL.txt:22-23` and `0.1.0-release-notes.md:108-109` already state |
| macOS minimum OS | "10.15+ (Tauri 2 default)" | **11.0**, measured from the binary's `LC_BUILD_VERSION minos` (`evidence/v01-06/08-post-fix-quality-gates.txt:163`; `package-macos.sh` derives `LSMinimumSystemVersion` from exactly that and fails hard if it cannot) |
| External tools | already correct | kept, plus the version-skew finding: an 18.x `pg_restore` exits 1 against a 16.x server (`evidence/v01-runtime/providers/23`, `F1`/P2) |
| Acceptance item "Linux package claims match actual declared native dependencies" | could not be evaluated | satisfied vacuously and recorded: v0.1 has **no** package metadata, and §2 is the substitute declaration |
| Acceptance item "smoke #91-#93 has an environment prerequisite checklist" | prose only | §5 is the explicit per-platform checklist an operator must satisfy before unblocking smoke rows |

## Verification

- Every cited pointer re-opened during the audit: `release.yml:14-16`, `:191-193`, `:385-392`,
  `:397-399`, `scripts/release/package-linux.sh`, `package-macos.sh`, `README-INSTALL.txt:22-23,43-45`,
  `risk-register.md:105` (`R-PKG-DEFER`), `0.1.0-release-notes.md:108-111`,
  `evidence/v01-06/08-post-fix-quality-gates.txt:163`, `evidence/v01-runtime/providers/23`.
- `cargo tree -p db-pro-infrastructure -e normal --target all -f "{p} {f}"` printed
  `dbus-secret-service v4.1.0` → `dbus v0.9.12` → `libdbus-sys v0.2.7 default,pkg-config` under
  `keyring v3.6.3 apple-native,linux-native,linux-native-sync-persistent,sync-secret-service,windows-native`,
  which is the evidence for the Linux runtime row.
- Docs-only change; `cargo check --workspace` green after the commit.
- **No focused child issue spawned:** no release-critical dependency declaration is missing — the
  workflow's build list is complete for `-p db-pro-native` and the runtime requirements are now
  stated (§2, §3). The remaining risks are already-recorded accepted deferrals (§4).
