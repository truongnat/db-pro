# V01-06 — Cross-Platform Release Workflow, Packaging & Secrets Audit

- Audited at: HEAD `3e0b0775c2248d37253d7325eddebd952e26d101`, branch `main`
- Audited revision of the workflow: `.github/workflows/release.yml` as of this HEAD
- This audit **changes nothing**. No workflow file, Cargo manifest, README, CHANGELOG, or source file was modified.

Files inspected:

| File | Purpose |
|---|---|
| `.github/workflows/release.yml` | release build workflow (325 lines) |
| `.github/workflows/ci.yml` | CI (fmt/clippy/build/test + release-build of the native app) |
| `.github/workflows/vps-pr-review.yml` | Kilo adversarial PR review (self-hosted VPS) |
| `Cargo.toml` / `crates/native-app/Cargo.toml` / `crates/*/Cargo.toml` | manifests |
| `README.md`, `CHANGELOG.md` | user-facing docs |
| `docs/release/0.1.0-packaging.md`, `docs/release/0.1.0-release-checklist.md`, `docs/release/0.1.0-readiness.md`, `docs/release/risk-register.md`, `docs/release/platform-prerequisites.md` | release docs |
| `docker-compose.yml`, `fixtures/smoke/*` | fixture/tooling |

---

## 1. (a) Truthful platform matrix

The `build` job in `.github/workflows/release.yml:235-325` defines a 3-entry matrix (`release.yml:244-256`). **No `--target` is passed** — each job runs `cargo build --release --locked -p db-pro-native` (`release.yml:307`) and therefore builds the **runner's own default target**.

| Platform | Runner | Rust target actually built | Binary artifact path | Packaging format | Artifact retention | Checksum | Signing | COMPILE | ARTIFACT | INSTALLER | SIGNING |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **macOS ARM64** | `macos-latest` | `aarch64-apple-darwin` (runner default) | `target/release/db-pro-native` | **raw Mach-O executable** uploaded as-is — no `.app` bundle, no `.dmg` | GitHub default for `actions/upload-artifact@v4` (not pinned in the workflow) | **none** | **none** | YES | YES | NO | NO |
| **macOS x64** | — | `x86_64-apple-darwin` | — | — | — | — | — | **NO — not built** | NO | NO | NO |
| **Windows x86_64 MSVC** | `windows-latest` | `x86_64-pc-windows-msvc` (runner default) | `target/release/db-pro-native.exe` | **raw PE executable** — no `.msi`, no NSIS `.exe` installer | as above | **none** | **none** | YES | YES | NO | NO |
| **Linux x86_64 GNU** | `ubuntu-latest` | `x86_64-unknown-linux-gnu` (runner default) | `target/release/db-pro-native` | **raw ELF executable** — no `.tar.gz`, `.deb`, `.rpm`, AppImage | as above | **none** | **none** | YES | YES | NO | NO |

**Facts that must not be smoothed over:**

1. **`macos-latest` builds the runner's own architecture, which is arm64.** The workflow does not pass `--target`, and does not build an x86_64 slice. **macOS x64 is NOT built by CI.** There is also no `lipo`/`cargo-lipo` universal-binary step. Consequence: an Intel Mac user has no artifact.
2. **`macos-latest` is a moving label.** Because no runner version is pinned (`macos-14` vs `macos-15`), the produced architecture could change under the project without a commit. This is a reproducibility defect, not just a documentation gap.
3. **COMPILE and ARTIFACT are `YES` for three platforms, but this is a code-measured claim only for macOS.** I compiled and ran only macOS ARM64 on this host (see `01-toolchain.txt` / `03-release-binary.txt`: only `aarch64-apple-darwin` is installed, no cross targets). The Windows and Linux columns are read from the workflow definition — they are **`CANDIDATE` (workflow says it builds them), not `BUILD_VERIFIED`**. Per goal-3 §8 the correct status for Windows/Linux in this run is `BUILD_UNVERIFIED_HERE` / `RUNTIME_UNVERIFIED`.
4. **The `resolve`/`preflight` jobs are good.** `release.yml:30-150` resolves an arbitrary ref to an exact 40-char SHA, re-checks the checkout SHA, and fails on mismatch. `release.yml:131-137` validates that a `v*` tag matches the version in `crates/native-app/Cargo.toml`. `release.yml:197-233` emits and uploads `release-provenance.json`. This is a genuine strength worth preserving.
5. **`permissions: contents: read` (`release.yml:22-23`).** The workflow cannot create or edit a GitHub Release, and there is no release-assembly job. Artifacts are only downloadable from the Actions run page.
6. **Two build paths exist.** `ci.yml:131` also runs `cargo build --release --locked -p db-pro-native` on every push/PR, but uploads nothing. Only `release.yml` uploads.

### Installer vs binary — explicitly separated

The repo's own docs already state the position (`release.yml:7-9`):
> *"this workflow produces the platform binaries only. Installer/packaging formats (DMG, MSI/NSIS, DEB/RPM/AppImage) and code signing are NOT yet implemented for the native app"*

and `docs/release/0.1.0-packaging.md:9`:
> *"Packaging status: native binary build wired; installer formats + signing NOT IMPLEMENTED"*

**However, `docs/release/0.1.0-packaging.md:46-53` still presents a "Bundle Targets" table listing `.dmg`, `.msi`, NSIS `.exe`, `.deb`, `.rpm` each as `PASS`** citing Tauri-era run `#31476095697`. That table is **stale and misleading**: those bundles came from the retired Tauri bundler, they are not produced for `db-pro-native`, and the table is not marked historical in place (only the document's top amendment says Tauri details are historical). This must be corrected in the docs pass — a reader who skims will believe DMG/MSI/DEB ship.

---

## 2. (b) Workflow gap list for V01-06 (do NOT edit in this run)

Goal-3 §10 requires artefacts named `db-pro-v0.1.0-<platform>-<arch>.<ext>` with archive packaging and a `SHA256SUMS.txt`.

Current artifact name (`release.yml:322`):
```
db-pro-${{ matrix.artifact_suffix }}-v${{ needs.resolve.outputs.app_version }}-${{ needs.resolve.outputs.short_sha }}
# e.g. db-pro-macos-v0.1.0-3e0b0775c224
```
This is a **bare artifact name for a single raw file**, not a filename. It contains no architecture, no file extension, and includes the short SHA (which goal-3 §10's contract shape does not).

| # | Gap | Missing job/step | Notes |
|---|---|---|---|
| G-1 | **No architecture in the artifact identity** | matrix entry needs an `arch` key; name/step must use it | `macos-latest` is arm64 — calling the artifact plain `macos` is wrong once x64 is added |
| G-2 | **No archive packaging (zip/tar.gz)** | new per-platform *package* step after `release.yml:317` | macOS/Linux: `tar -czf`; Windows: `Compress-Archive`. Must not exist today — `release.yml:319-324` uploads `matrix.binary_path` directly |
| G-3 | **No `SHA256SUMS.txt`** | new checksum step + final assembly job | `shasum -a 256` (macOS/Linux) / `Get-FileHash` (Windows). Nothing in the workflow computes hashes today. |
| G-4 | **No release-assembly job** | new job with `needs: build` | Would merge per-platform checksums into one `SHA256SUMS.txt` and (only if authorised) attach to a Release. Blocked by `permissions: contents: read` (`release.yml:22`). |
| G-5 | **No `README`/install note inside the archive** | part of G-2 | Goal-3 §10: bundle an install note. `LICENSE` only once a license is chosen (§4 below) |
| G-6 | **No macOS `.app` bundle / `Info.plist`** | new macOS step | Without a bundle, `CFBundleIdentifier`/`CFBundleShortVersionString`/`CFBundleVersion` cannot be set, Gatekeeper behaves worse, and — importantly — the repo's own automated capture harness cannot observe the app (see `03-release-binary.txt` §B). This is the single highest-value packaging change. |
| G-7 | **No explicit artifact retention** | extend `upload-artifact@v4` with `retention-days` | Leaving it implicit means a repo-setting change silently alters the release evidence lifetime |
| G-8 | **No signing/notarization steps** | deferred by decision (§5) | Must remain absent until credentials exist. Never add secrets to the repo. |
| G-9 | **No smoke/startup check on the produced artifact** | new step per platform | Even a headless `--version`-style check would upgrade Windows/Linux from `BUILD_UNVERIFIED` to at least `ARTIFACT_PRESENT`. Note: the binary currently has no CLI/`--version` path, so this needs a small, contract-appropriate change. |
| G-10 | **No `--target` pinning / no pinned runner version** | pass `--target` and pin `macos-14`-style labels | Reproducibility (§6). |

**Deliberately NOT on this list** (avoid scope creep): `.dmg`, `.msi`, `.deb`, `.rpm`, AppImage, notarization, universal binaries. See §3.

---

## 3. (c) Recommended v0.1 packaging contract

**Recommendation: Option A — portable native binaries only, wrapped in goal-3 §10's archive + checksum contract.**

Concretely: `db-pro-v0.1.0-<platform>-<arch>.tar.gz` for macOS ARM64 and Linux x86_64, `db-pro-v0.1.0-windows-x86_64.zip` for Windows, each containing the executable plus a short install note, plus one assembled `SHA256SUMS.txt`.

### Evaluation against the stated criteria

| Criterion | Option A (portable binaries) | Option B (full installers) | Option C (mixed .app/.dmg + .exe + tar.gz) |
|---|---|---|---|
| **Reliable** | High. One `cargo build` → one file → `tar`. Almost no failure surface. | Low. Five packaging ecosystems (`dmg`, `msi`, `nsis`, `deb`, `rpm`), each with its own failure modes; the repo has *zero* current tooling for any of them. | Medium. `.app` bundling is well-understood and local; `.dmg` needs `hdiutil` and signing to behave; Windows side reduces to "zip a .exe", same as A. |
| **Reproducible** | High. Same inputs → byte-comparable tarball. | Low. `hdiutil`/WiX/NSIS embed timestamps and GUIDs by default. | Medium (the `.dmg` drags it down). |
| **CI-friendly** | Highest. No extra toolchain installs; the existing matrix already produces the binaries. | Lowest. Needs WiX on Windows, `rpmbuild`/`dpkg-deb` on Linux, `hdiutil` on macOS. | Medium. Needs `hdiutil` on the macOS runner only. |
| **Native app launchable** | Yes. A Mach-O arm64 executable launches on the host (proven in `03-release-binary.txt`). Caveat: unsigned + no bundle → Gatekeeper warning and no `CFBundleIdentifier`. | Yes, but only after signing; unsigned MSI/DMG is arguably *worse* than a plain binary because users trust installers more. | Yes, and a `.app` bundle is the best un-signed experience on macOS (still warns, but the window/permissions model is correct). |
| **Appropriate for v0.1** | **Yes.** No license decision, no signing identity, no branding decision (`risk-register.md` R001), and no runtime verification on Windows/Linux yet. Shipping binaries is honest about the maturity level. | **No.** Would claim distribution-grade polish that the underlying evidence does not support, and multiplies unverifiable artifacts by ~5. | **Partly.** The `.app` half is genuinely valuable; the `.dmg` half is not needed. |

### Why not B, in one line

`risk-register.md` shows the four open release blockers are **license (R004), brand name (R001), signing (R003)** and unresolved legacy gates — none of them is "we lack an installer". Installers add unsigned-artifact surface without resolving any actual blocker. `docs/release/0.1.0-readiness.md:192` already records *"Installer formats for the native binary — NOT IMPLEMENTED"*; the honest move is to keep that true and say so.

### Justified refinement to Option A (the one thing worth adding)

**Ship a macOS `.app` bundle, and drop `.dmg`.** Rationale:
- It costs one directory tree (`Contents/MacOS/`, `Contents/Resources/`, `Contents/Info.plist`) produced by a shell step — no new toolchain, no signing required.
- It is the only way to set `CFBundleIdentifier` / `CFBundleShortVersionString` / `CFBundleVersion` (goal-3 §7's explicit checklist).
- It is a **prerequisite for automated visual evidence on macOS**: an un-bundled binary is not discoverable by the project's own capture harness — I reproduced this in `03-release-binary.txt` §B, and the repo already recorded the same limitation at `query-editor-intelligence/VERIFICATION.md:93-94`. Without a bundle, V01-01-style visual verification cannot be re-run at all.

So the practical contract is: **A + macOS `.app` wrapper** = `db-pro-v0.1.0-macos-aarch64.tar.gz` containing `DB Pro.app`, plus plain-binary tarball/zip for Linux/Windows.

### Intentional deferrals (record explicitly)

| Deferred | Why |
|---|---|
| macOS `.dmg` | Needs signing to be non-hostile; adds `hdiutil` nondeterminism; not required to launch the app |
| Windows `.msi` / NSIS | WiX/NSIS toolchain; unsigned installers raise SmartScreen just like a zip; no Windows runtime evidence yet |
| Linux `.deb` / `.rpm` / AppImage | distro-specific dependency declarations (esp. the unresolved `libsecret` question in `risk-register.md` R002), plus AppImage/FUSE complications |
| macOS x86_64 artifact | No Intel runner in the matrix today; decide separately (§7). Do not claim it. |
| Universal binary | Needs both slices built |
| Code signing / notarization | No Developer ID / certificate; user decision |
| Auto-update | Out of scope for v0.1 |

---

## 4. (d) Native app metadata audit

### Version consistency — `EVIDENCED` (clean)

| Location | Value | Verdict |
|---|---|---|
| `crates/core/Cargo.toml` | `0.1.0` | consistent |
| `crates/infrastructure/Cargo.toml` | `0.1.0` | consistent |
| `crates/ui/Cargo.toml` | `0.1.0` | consistent |
| `crates/runtime/Cargo.toml` | `0.1.0` | consistent |
| `crates/native-app/Cargo.toml` | `0.1.0` | consistent — **this is the authoritative one**; `release.yml:125` reads it |
| `crates/tauri-app/Cargo.toml` | `0.1.0` | consistent (legacy host) |
| `crates/tauri-app/tauri.conf.json` | `"version": "0.1.0"` | consistent (legacy host) |
| `README.md:11,13` | describes `0.1.0 Release Candidate` | consistent |
| `CHANGELOG.md:129` | `## [0.1.0]` section exists, plus `## [Unreleased]` at line 3 | consistent |

**No version inconsistency found. No `0.1.0-rc.1` representation exists anywhere.** The workspace `Cargo.toml` has no `[workspace.package] version`, so each crate declares `0.1.0` independently — six places to keep in sync (a maintenance risk, not a current defect). `release.yml:131-137` guards the tag↔manifest agreement for `v*` tags, which mitigates it.

### Metadata that is MISSING or WRONG — every inconsistency found

| # | Field | Finding | Evidence |
|---|---|---|---|
| M-1 | **`license`** | **Absent from every single Cargo manifest.** No `license` or `license-file` key exists in `Cargo.toml` or any `crates/*/Cargo.toml`. | `grep -n "^license" Cargo.toml crates/*/Cargo.toml` → no output |
| M-2 | **`description`** | Absent from every manifest. Released metadata would have no project description. | same grep (`^description`) → no output |
| M-3 | **`authors`** | Absent from every manifest. | same grep (`^authors`) → no output |
| M-4 | **`repository` / `homepage`** | Absent from every manifest, though the repo is `https://github.com/truongnat/db-pro.git`. | same grep → no output |
| M-5 | **Icons / resources for the native app** | `crates/native-app/` contains **only** `Cargo.toml` and `src/` — no icon, no `.icns`, no `.ico`, no resource directory. The only icons in the tree are `crates/tauri-app/icons/*` (`icon.icns`, `icon.ico`, PNGs) which belong to the **retired Tauri host**. `crates/ui/assets/` contains only `fonts/`. | `find crates -iname '*.icns' -o -iname '*.ico'` → only `crates/tauri-app/icons/…` |
| M-6 | **No `Info.plist` / bundle identifier for the shipped artifact** | `com.dbpro.app` appears in exactly two places: `crates/tauri-app/tauri.conf.json:4` (legacy Tauri) and `crates/native-app/src/main.rs:138` where it is the **keyring service name** (`KEYRING_SERVICE`). There is no `Info.plist`, so the shipped binary has no `CFBundleIdentifier`, no `CFBundleShortVersionString`, no `CFBundleVersion`. | `03-release-binary.txt`: `Info.plist=not bound` |
| M-7 | **`README.md` vs current status** | `README.md:11` says *"0.1.0 Release Candidate — not yet release-signed-off"* — accurate. But the readme does not mention that V01-01..05 are claimed PASS while `STATUS.md` says the runtime smoke is pending (see `04-v01-01-05-evidence-audit.md`). Not a contradiction invented here; just not reconciled. | `README.md:11-20` |
| M-8 | **`CHANGELOG.md` has no dated 0.1.0 release entry** | `CHANGELOG.md:3` `## [Unreleased]`, `:28` `### 0.1.0 Release Candidate`, `:129` `## [0.1.0]` — the released section exists but carries no date and the Unreleased block has not been folded into it. Goal-3 §21 wants `## [0.1.0] - YYYY-MM-DD`. | `CHANGELOG.md:3,28,129` |
| M-9 | **Stale `Monaco editor` metadata** | `docs/release/0.1.0-release-checklist.md` lists `- [x] Monaco editor` as a satisfied P1 Query item, and `docs/release/0.1.0-manual-smoke.md:93` instructs the tester to confirm *"Monaco editor loads"*. Monaco belonged to the archived React UI; the native editor is egui-based. | those two lines |
| M-10 | **Window title / app name are hardcoded and duplicated** | `crates/native-app/src/main.rs:226` `.with_title("DB Pro")`, `:233` `"DB Pro"`, `crates/ui/src/navigation_view.rs:478` `"DB Pro"`. Three literals; no constant, no `CARGO_PKG_NAME`. Changing the product name (which R001 says may be required) means editing them by hand in ≥3 places. | those lines |

**Version is not to be changed** (goal-3 §6). Everything above is either additive metadata or a doc correction.

### Branding note (goal-3 §12)

`risk-register.md` R001 is `P1 / OPEN` — *"Public name 'DB Pro' collides with existing dbpro.app product"*, blocking README/release notes/public release. Since a naming decision is unresolved, **do not invent artifact names tied to a new brand**, and do not rename anything. `db-pro-*` artifact names are consistent with the current internal name, which is the correct holding pattern. Disposition for the register: `DEFER` to the user's brand decision (the register already says `Pending brand decision`).

---

## 5. (e)+(f) License and signing

### `R-LICENSE` — governance blocker, CONFIRMED OPEN

| Question | Answer |
|---|---|
| Does a `LICENSE` file exist? | **No.** `git ls-files \| grep -iE '(^|/)(LICENSE\|COPYING\|NOTICE\|UNLICENSE)'` → **no matches**. Nothing on disk either. |
| Does any Cargo manifest declare a license? | **No** (M-1). |
| What is the only license text in the repo? | `crates/ui/assets/fonts/OFL.txt` — the **SIL Open Font License for the bundled Inter font**. That is a third-party asset license, not a project license. |
| What do old status docs claim? | `docs/release/0.1.0-readiness.md:221`: *"Project license is not yet defined."* — `README.md:227`: *"Project license is not yet defined."* — `CHANGELOG.md:115`: *"Project license is not yet defined."* — `docs/release/risk-register.md` **R004** (`P1 / governance / OPEN`): *"No LICENSE file; redistribution policy unresolved. Impact: Cannot legally distribute; blocks public release."* |

**Current state is unchanged from those claims: `R-LICENSE` REMAINS OPEN and is a public-release blocker.**

Disposition: **`BLOCKING` for public release; not chosen here.** Per goal-3 §11 I have not selected a license. Consequence to record plainly: until a license is chosen, `0.1.0` artifacts may be built and smoke-tested but **must not be represented as publicly licensed**, and the archive must **not** contain a `LICENSE` file (goal-3 §10 explicitly ties `LICENSE` inclusion to the license decision existing). This matches `risk-register.md` R004 already listing public release as blocked.

### Code signing / notarization — `UNSIGNED`, confirmed by measurement

| Platform | Status | Measured evidence |
|---|---|---|
| **macOS signing** | **NOT CONFIGURED** | `codesign -dv --verbose=2` on the release binary reports `Signature=adhoc`, `flags=0x20002(adhoc,linker-signed)`, `TeamIdentifier=not set`, `Info.plist=not bound`. This is the linker's automatic ad-hoc signature on Apple Silicon — **not** a Developer ID signature. |
| **macOS notarization** | **NOT CONFIGURED** | No `notarytool`/`stapler` step exists in `release.yml`; no notarization secrets referenced (workflow uses no secrets at all except `GITHUB_TOKEN` in the review workflow). |
| **macOS Gatekeeper** | **WOULD BLOCK a downloaded copy** | `spctl -a -vvv -t execute <binary>` → **`rejected`, exit 3**. |
| **Windows signing** | **NOT CONFIGURED** | No `signtool` step, no certificate reference in `release.yml`. Expected user-facing effect: SmartScreen *"Windows protected your PC"* on an unsigned `.exe`. |
| **Linux signing** | **NOT CONFIGURED** | No GPG signing of artifacts. (No distro packages are produced either, so nothing to sign yet.) |

Expected user-facing warnings if 0.1.0 ships unsigned:
- **macOS:** *"db-pro-native cannot be opened because the developer cannot be verified."* (newer macOS: *"Apple could not verify … is free of malware"*). A downloaded copy also carries `com.apple.quarantine`; the user must right-click → Open or use System Settings → Privacy & Security → Open Anyway. If a `.app` bundle without a signature is shipped, macOS Sequoia+ additionally requires the user to approve it in Privacy & Security.
- **Windows:** SmartScreen blocks the first run behind *"Run anyway"*.
- **Linux:** no OS-level warning; the artifact is simply unsigned.

**Never add secrets.** Confirmed: `release.yml` declares no `secrets.*`; the only secret-typed usage in the workflows is `secrets.GITHUB_TOKEN` in `vps-pr-review.yml:27`, which is provided automatically by GitHub.

Risk-register cross-reference: **R003** (`P2 / Trust / OPEN`) states *"No code signing or notarization configured for any platform… release can proceed unsigned, but with degraded UX"*. That is consistent with what I measured. Note R003's evidence line cites `tauri.conf.json` having no signing block — now stale/superseded by the native binary; the finding itself stands.

---

## 6. (g) Reproducibility audit

| Requirement | Status | Evidence |
|---|---|---|
| `Cargo.lock` present | **PASS** | `Cargo.lock` exists (219,639 bytes) and is **tracked** (`git ls-files --error-unmatch Cargo.lock` succeeds) |
| `--locked` used for the release build | **PASS** | `release.yml:307` and `ci.yml:131` both use `cargo build --release --locked -p db-pro-native` |
| `--locked` used for *all* gates | **GAP** | `release.yml:193-195` runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` **without `--locked`**. A preflight could pass against an un-locked resolution while the build fails. Low severity, trivially fixable. |
| No floating git dependencies | **PASS** | `grep -n 'source = "git' Cargo.lock` → **no matches**. Every dependency resolves from crates.io. |
| No network-downloaded runtime assets at launch | **PASS** | Fonts are compiled in via `include_bytes!` (`crates/ui/src/theme.rs:3-6`, 4 Inter TTFs). SQLite is compiled in (`rusqlite` feature `bundled`, `Cargo.toml:33`). TLS is rustls (`sqlx` feature `runtime-tokio-rustls`) — no OpenSSL. The only `reqwest` use is `crates/runtime/src/agent.rs:8`, i.e. the Agent provider's API calls, which is intended network use, not asset fetch. |
| No local absolute paths in source/config | **PASS (source)** | `grep -rn "/Users/\|/home/[a-z]" crates/ --include='*.rs'` (excluding tests) → **no matches**; no `/Users/truongdev` in any `.toml`/`.json`/`.yml`/`.sh` outside `target/`. |
| Local absolute paths **in the built binary** | **INFO** | `strings` on the release binary shows absolute **Cargo registry** build paths (`/Users/truongdev/.cargo/registry/src/index.crates.io-…/winit-0.30.13/…`, `…/hyper-util-0.1.20/…`) leaked from Rust `Location` panic metadata. Not a credential and not a runtime dependency, but it does disclose the builder's home directory and crate versions. Avoidable with `--remap-path-prefix` if desired. |
| Embedded assets | **PASS** | 4 Inter TTFs + their OFL license are compiled into the binary; nothing is read from disk at launch. |
| Fresh checkout can build | **PASS (macOS ARM64, measured)** | Full workspace check/clippy/test/release-build all succeeded from this checkout on this host (`02-quality-gates.txt`). |

### Required native system packages per platform

**Build-time (as declared in the workflows):**

| Platform | Packages | Source |
|---|---|---|
| macOS | Xcode command line tools | `docs/release/platform-prerequisites.md:10` |
| Windows | MSVC build tools | `docs/release/platform-prerequisites.md:12` |
| Linux | `libxkbcommon-dev libwayland-dev libx11-dev libgl1-mesa-dev` | `release.yml:302-304` (build job) |

**Build-time, whole-workspace (CI/preflight only):** `ci.yml:46-48` and `release.yml:175-177` additionally install `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev`. These are needed because `crates/tauri-app` is **still a workspace member** (`Cargo.toml:7`) even though it is a retired host. Consequence: **any full-workspace gate on Linux drags in the GTK/WebKit stack**, and the release `preflight` job therefore installs WebKit/GTK headers it does not need for the artifact. Worth reviewing when the legacy host is removed.

**Runtime dependencies of the shipped artifact:**

| Platform | Runtime requirement | Status | Evidence |
|---|---|---|---|
| macOS ARM64 | none beyond the OS | **PASS** | `otool -L` on the release binary lists **only** `/System/...` frameworks and `/usr/lib` dylibs — no Homebrew, no third-party dylib. Minimum OS `minos 11.0`. |
| Linux x86_64 | X11 or Wayland + OpenGL + a working GL driver | **CANDIDATE — not verified here** | `eframe` is configured with `features = ["glow", "wayland", "x11", "accesskit"]` (`crates/native-app/Cargo.toml`); no Linux host was available. Headless CI cannot run it. |
| Linux x86_64 | **Credential storage needs a D-Bus Secret Service provider** | **CANDIDATE — risk** | `crates/native-app/Cargo.toml` overrides the workspace `keyring` features with `["apple-native", "windows-native", "sync-secret-service"]`. The workspace declares `linux-native-sync-persistent` (kernel keyutils) but the shipping crate does **not** use it. So on Linux the app requires gnome-keyring/KWallet on D-Bus; on a minimal or headless install, credential writes fail. This is the same class as `risk-register.md` **R002** (libsecret) and **R011** (keyring features), which predate this feature combination. Needs a decision, not a repo change in this run. |
| Windows x86_64 | MSVC runtime (`vcruntime140.dll`) normally present; `keyring` → Credential Manager | **CANDIDATE — not verified here** | `windows-native` keyring feature is enabled; no Windows host available. |
| **All** | `pg_dump` / `pg_restore` on `PATH` for PostgreSQL backup/restore; system `ssh` for SSH tunnels | **KNOWN GAP** | `risk-register.md` R006 (P2, OPEN) and R009 (P2, ACCEPT RC1); `docs/release/platform-prerequisites.md` calls them out as not bundled. Nothing in the release contract installs or documents them. |

**Reproducibility verdict: strong for a clean local macOS build; the weak points are the un-pinned `macos-latest` runner, the missing `--target`, the missing `--locked` on the gate jobs, and the unverified Linux keyring requirement.**

---

## 7. Open decisions the user/coordinator must make (not decided here)

1. **License** (`R-LICENSE`) — blocks public release. I did not choose one.
2. **Brand name** (`R001`) — blocks README/release notes/public naming. Do not rename anything.
3. **Whether macOS x86_64 is intentionally supported** (goal-3 §4). If yes, CI needs an `macos-13`/x86_64 runner and an `arch` matrix key. If no, the release notes must say "Apple Silicon only" explicitly.
4. **Whether to ship unsigned.** Technically permissible; must be stated as `UNSIGNED` with the Gatekeeper/SmartScreen consequence in the release notes.
5. **Windows/Linux runtime verification.** These remain `BUILD_UNVERIFIED_HERE` — but note they are also **`RUNTIME_UNVERIFIED` even in principle** today, because no Windows/Linux host exists in this project. The honest status words are `BUILD_VERIFIED_VIA_CI` (once a CI run exists) and `RUNTIME_NOT_VERIFIED`.

---

## 8. (Step 7) Secrets / credential release audit

Method: `git grep -nIE` over **tracked** files with high-signal credential regexes (private-key headers, `sk-…`, `ghp_…`/`gho_…`, `AKIA…`, `xox…`, `AIza…`), plus heuristic `password|secret|api_key|token = "…"` matching, plus inspection of the local runtime state directory and `strings` of the release binary. **Values are redacted below.**

| # | Location / class | Finding | Verdict |
|---|---|---|---|
| S-1 | `crates/ui/src/runtime.rs:45-49` — **`local-connection-metadata` + fixture credential in PRODUCTION default** | `impl Default for UiConnectionDraft` (production code, not `#[cfg(test)]`) hardcodes a developer preset: name `"Xe Lạc Hồng (PostgreSQL)"`, `host: "localhost"`, `port: "5432"`, `database: "fullstack_starter"`, `username: "postgres"`, `password: "postgres"`. This is compiled into the shipped binary. | **FINDING (low severity, release hygiene).** The credential itself is fixture-grade and non-sensitive, and `CHANGELOG.md` records it as an intentional *"Developer convenience, not a product default"*. But goal-3 §16 says do not package *local connection metadata*: a released build pre-fills users' New-Connection dialog with a private database name and connection label. Recommend neutralizing for release builds (gate on `cfg(debug_assertions)`) — **not done in this run.** |
| S-2 | `crates/native-app/src/main.rs:177-202` — `seed_default_connection` | Production startup path seeds that same developer connection into the runtime on launch (surfacing it in the user's connection list). | Same finding as S-1; same recommendation. |
| S-3 | Release binary, `local-connection-metadata` | `strings` on `target/release/db-pro-native` contains the preset (`…ng (PostgreSQL)localhostfullstack_starter`) — confirming S-1/S-2 are actually present in the distributable artifact, not tree-shaken away. | **CONFIRMED shipped.** |
| S-4 | Release binary, `build-path-disclosure` | Contains absolute builder paths under `/Users/truongdev/.cargo/registry/src/…` (winit, hyper-util, accesskit, …) from Rust panic `Location` metadata. | **INFO**, not a credential. Optional `--remap-path-prefix` hardening. |
| S-5 | Release binary, `env-var-names` | Contains the *names* `GROQ_API_KEY`, `OPENAI_API_KEY`, `DB_PRO_GROQ_ENDPOINT`, `DB_PRO_GROQ_MODEL`, `DB_PRO_CODEX_ENDPOINT`, `DB_PRO_CODEX_MODEL`. | **ACCEPTABLE** — names only; **no key values** are embedded. No `sk-…`-shaped value matched anywhere. |
| S-6 | `.db-pro-data/secrets/secrets.json` — `user-credential-store` | A real local credential store: one entry `connection/<uuid>/password`, value length 140 chars, file mode `0600`. **Value redacted.** | **NOT TRACKED — acceptable.** `.gitignore:34` ignores `/.db-pro-data/`; `git check-ignore -v` confirms. **Caveat that must be honoured by the packaging step:** any packaging/publish step must build from a clean checkout or `git archive`, never by copying the working tree, or it would ship this file. |
| S-7 | `.db-pro-data/meta.db` — `user-db-file` | 626 KB SQLite workspace DB with tables `connections, query_history, saved_queries, saved_query_folders, workspaces, settings, introspection_cache, run_configs`. Connection rows store `config` + a `secret_ref` (an **indirection**, not the raw password). | **NOT TRACKED — acceptable**, and the secret/metadata separation is good practice (corroborates `STATUS.md:7`). Same packaging caveat as S-6. |
| S-8 | `fixtures/smoke/**`, `docker-compose.yml`, `.github/workflows/ci.yml` — `fixture-credential` | `dbpro` / `dbpro_test` on `localhost:5432/dbpro_fixture`; CI also generates ephemeral SSH keys at runtime into `$RUNNER_TEMP` (`ci.yml:74-112`), never committed. | **ACCEPTABLE — clearly test-only**, confined to fixtures/CI/local compose, well documented as such. |
| S-9 | `crates/native-app/src/main.rs:259` — `test-sentinel` | `password: "should-not-cross-boundary"` — inside `#[cfg(test)] mod tests` (line 250), asserting SQLite drafts drop PG-only credentials. | **ACCEPTABLE — test-only.** |
| S-10 | `docs/plans/completed/native-ui-foundation/VERIFICATION.md:23` — `local-connection-metadata` | Records a real local PostgreSQL endpoint `127.0.0.1:15433` with `postgres/postgres` and internal BSN tenant schema names (`master`, `tenant1`, `tenant2`). | **INFO (low).** No live secret (`postgres/postgres` is a local default), but it discloses the developer's fixture topology. Historical plan record; do not alter, but avoid repeating in public release docs. |
| S-11 | Tracked files, `high-signal-credential scan` | **Zero matches** for private-key headers, AWS keys, GitHub tokens, Slack tokens, Google API keys, or `sk-`-shaped secrets. No `.env`, `.pem`, `id_rsa`, or credentials file is tracked (`git ls-files` filtered → only `crates/*/src/secret/*.rs` source modules). | **CLEAN.** |
| S-12 | Workflow secrets | `release.yml` and `ci.yml` reference **no** `secrets.*`. `vps-pr-review.yml:27` uses only auto-provided `secrets.GITHUB_TOKEN`. | **CLEAN** — no secret provisioning to add, and none to leak. |

### Could a generated artifact package user config / DB files / credentials / provider keys?

**Yes — but only if packaging is done incorrectly.** Nothing in the current repository marks these paths for exclusion because *no packaging step exists yet*. When G-2/G-6 add archive creation, they must explicitly package **only** the release executable (and the `.app` wrapper), sourced from `target/release/` — never the repo root. Concretely, the following must be in the exclude list of any future packaging step:

```
.db-pro-data/            # user workspace DB + credential store (S-6, S-7)
target/                  # build output incl. debug binaries, incremental artifacts
.claude/ .workbuddy-ai/ .qoder/ .jules/ .kilo/   # tool/session state
fixtures/                # test credentials and fixtures
docker-compose.yml       # test credentials
.env*                    # ignored but must never be globbed in
_archive/                # archived React frontend + benchmarks
```

Goal-3 §10 already forbids debug binaries, incremental artifacts, API keys, test DB credentials and local settings in the artifact; this list is the concrete mapping onto this repo.

---

## 9. Summary of blockers this audit surfaces for the *later* V01-06 steps

| ID | Blocker | Severity | Blocks |
|---|---|---|---|
| B-1 | `R-LICENSE` — no LICENSE file, no license metadata in any manifest | **governance blocker** | public release; `LICENSE` in archives |
| B-2 | All artifacts `UNSIGNED`; macOS `spctl` **rejects** the binary | accepted-if-stated | public distribution UX |
| B-3 | macOS x64 not built; `macos-latest` implicitly arm64 and un-pinned | medium | platform-coverage claims |
| B-4 | No `.app` bundle → no bundle ID, and the project's own capture harness **cannot observe the app** | medium-high | automated visual evidence (V01-01 re-verification) |
| B-5 | No archive packaging, no `SHA256SUMS.txt`, no arch in artifact names, no assembly job | required by goal-3 §10 | artifact/checksum deliverables in later runs |
| B-6 | Windows/Linux artifacts are `BUILD_UNVERIFIED_HERE` and will remain `RUNTIME_UNVERIFIED` | medium | "cross-platform release" claim |
| B-7 | Linux credential storage requires a D-Bus Secret Service provider (`sync-secret-service` in the shipping crate) | medium | Linux runtime correctness |
| B-8 | Dev connection preset (`fullstack_starter`) ships in the release binary | low | release hygiene |
| B-9 | `pg_dump`/`pg_restore`/`ssh` not bundled and not documented for users | low-medium | backup/restore + SSH claims |
| B-10 | `docs/release/0.1.0-packaging.md:46-53` still presents Tauri-era `.dmg`/`.msi`/`.deb`/`.rpm` as `PASS` | low (but actively misleading) | doc consistency |

No workflow file, manifest, or source file was changed to produce this audit.
