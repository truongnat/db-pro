# Central Release Risk, Decision & Blocker Register — v0.1

> One canonical release-risk and decision ledger for DB Pro v0.1.
> Current HEAD under assessment: `85a7fa3cc0a84c56ac2a5049ce08130db06e0a20` (2026-09-14).
> **Release pipeline verified end to end — final run `34860902181`.** Dispatched with
> `workflow_dispatch` on `main` for the candidate `85a7fa3`; every job green (`Resolve release
> candidate`, `Pre-flight checks` — fmt/clippy/tests on the pinned 1.95.0 toolchain, `Build`
> macOS/Windows/Linux, `Package` macOS/Windows/Linux, `Assemble SHA256SUMS`). Its three archives
> were downloaded and independently re-hashed on this host: sizes and SHA-256 values reproduced
> byte-for-byte against the run's `SHA256SUMS.txt`, archive member lists matching the contract,
> provenance recording `candidate_sha 85a7fa3cc0a84c56ac2a5049ce08130db06e0a20`,
> `short_sha 85a7fa3`, `rustc 1.95.0 (59807616e 2026-04-14)`, `version 0.1.0`,
> `source_kind commit`, `event workflow_dispatch`. Those are the **final release-candidate
> values** and are recorded in §4, `0.1.0-readiness.md` and `0.1.0-handoff.md` §3. The earlier
> green run `34859158012` (`1a0c186`) is cited below only where it is history.
> **Post-candidate history:** the packaged-app startup blocker `R-STATE-DIR` was reproduced on
> candidate `fbf9fdab9100f08f12e29434983f32c18f14ac2f` and fixed in code commit `543b526`
> (`fix(native): resolve the app state directory outside the working directory`); the release
> pre-flight blocker chain (toolchain drift → Linux D-Bus → flaky ER worker test) was closed by
> `fbf9fda`, `e22a498` and `1a0c186`. Evidence:
> `docs/release/evidence/v01-06/09-toolchain-pinning.txt`, `11-linux-build-dependency-fix.txt`,
> `12-state-dir-blocker-fix.txt`, `13-flaky-er-worker-test.txt`, `14-install-smoke.txt`. See §4.
> Companion docs: `docs/release/0.1.0-readiness.md`, `docs/release/0.1.0-handoff.md`,
> `docs/notes/PRODUCT_CAPABILITY_MATRIX.md`, `docs/release/known-limitations.md`
> Evidence base: `docs/release/evidence/v01-06/*`
> **Runtime evidence update (2026-09-14, `v01-runtime` session):** live PostgreSQL 16.15
> integration **18/18 PASS** (`providers/07`), a deterministic SQLite runtime fixture
> (`providers/02`–`05`), CLI-level `pg_dump`/`pg_restore` verification with a version-skew caveat
> (`providers/10`–`14`), cancellation capability gating re-verified correct (`providers/15`),
> keyring stall reproduced and classified (`providers/16`, `providers/22`), launch error-log audit
> (`providers/19`), and findings `F1`–`F3` (`providers/23`). **No GUI evidence, no screenshots**
> (`providers/21`). New entries below: `F1`, `F2`; `R-KEYRING-STALL` and `R-PROV` updated. No
> production code changed in that session.

## Record format

Every open risk below carries the six required fields:

| Field | Meaning |
|---|---|
| `ID` | stable identifier |
| `Description` | what the risk is, in one or two sentences |
| `Severity` | `P0` release stops · `P1` blocks the affected gate/claim · `P2` fix/accept/defer before freeze · `P3` backlog |
| `Owner / Decision` | who must decide, or the recorded decision |
| `Status` | `OPEN` · `FIXED` · `ACCEPTED` · `DEFERRED` · `BLOCKING` |
| `Release disposition` | one of exactly **`FIXED` / `ACCEPTED` / `DEFERRED` / `BLOCKING`** |

`BLOCKING` means: the affected claim or distribution mode must not be made until the item is
resolved. It does not necessarily block the internal release candidate; each record names what it
blocks.

---

## 1. v0.1 release register

### R-LICENSE — no license selected

| Field | Value |
|---|---|
| ID | `R-LICENSE` |
| Description | There is no `LICENSE` file anywhere in the tree and no `license`/`license-file` key in `Cargo.toml` or any `crates/*/Cargo.toml`. No license has been chosen. Confirmed 2026-09-14 by `git ls-files \| grep -iE '(^\|/)(LICENSE\|COPYING\|NOTICE\|UNLICENSE)'` (no matches) and by grep over every manifest. The only license text in the repo is `crates/ui/assets/fonts/OFL.txt` (SIL OFL for the bundled Inter font — a third-party asset license, not a project license). |
| Severity | `P1` (governance) |
| Owner / Decision | **User/project owner decision required.** Not chosen, not added, not implied by this pass. |
| Status | `OPEN` |
| Release disposition | **`BLOCKING`** — blocks **public distribution only**. The internal/private release candidate may be built and evaluated; the artifacts must not be represented as licensed for public use, and the archives deliberately contain no `LICENSE` file. |

### R003 — code signing / notarization not configured

| Field | Value |
|---|---|
| ID | `R003` |
| Description | No Developer ID / certificate exists for any platform. Measured on the release binary: `Signature=adhoc`, `flags=0x20002(adhoc,linker-signed)`, `TeamIdentifier=not set`, `Info.plist=not bound`; `spctl -a -vvv -t execute` → **rejected** (exit 3). No `notarytool`/`stapler` step and no `signtool` step exists in `.github/workflows/release.yml`; no signing secrets exist in the repository and none were added. |
| Severity | `P2` (trust/UX) |
| Owner / Decision | Decision: ship **`UNSIGNED`** for v0.1 and state the consequence. Consequence documented: macOS shows *"…cannot be opened because the developer cannot be verified"* (right-click → Open / Privacy & Security → Open Anyway); Windows SmartScreen shows *"Windows protected your PC"*; Linux artifacts are simply unsigned. |
| Status | `ACCEPTED` |
| Release disposition | `ACCEPTED` — signed installers must never be claimed. If the project later markets trust/polish, this must be revisited. |

### R001 — brand / naming unresolved

| Field | Value |
|---|---|
| ID | `R001` (legacy `R001`) |
| Description | The product ships as "DB Pro" (`with_title("DB Pro")` in `crates/native-app/src/main.rs`; three literals total) and the name collides with an existing dbpro.app product. Related naming facts from the metadata audit: `crates/native-app/` contains **no** icon resource (the only icons belong to the retired Tauri host), there is **no `Info.plist` in-repo** (the packaging script generates a minimal one at build time), and the version is duplicated across six manifests. |
| Severity | `P2` (legal/branding if published) |
| Owner / Decision | **Deferred to the owner's brand decision.** Per project policy the application is **not renamed** in this pass, and no artifact name was tied to a new brand; the `db-pro-*` artifact names match the current internal name and are the correct holding pattern. |
| Status | `DEFERRED` |
| Release disposition | `DEFERRED` — blocks any *public* marketing/naming claims; does not block the internal RC. |

### R009 — SSH tunnels not end-to-end qualified (cross-platform)

| Field | Value |
|---|---|
| ID | `R009` |
| Description | SSH tunnels shell out to the system `ssh` binary (`crates/infrastructure/src/ssh/tunnel.rs`) with host-key verification; the single automated runtime test (`ssh_backup_runtime_verification.rs`) is `#[ignore]`d and requires nine `DB_PRO_SSH_*` variables, so on HEAD it runs as **0 passed / 1 ignored**. The `v01-runtime` session confirmed all nine variables are unset in this environment, so the suite was **`BLOCKED`, not run** (`docs/release/evidence/v01-runtime/providers/07`, `09`). No cross-platform E2E qualification exists (no Windows/Linux host). |
| Severity | `P2` |
| Owner / Decision | Decision: keep SSH out of the qualified feature set for 0.1.0; document the `ssh`-on-`PATH` requirement (LIM-006). |
| Status | `DEFERRED` |
| Release disposition | `DEFERRED` — do not market SSH tunnels as a qualified capability. |

### R-PKG-DEFER — packaging deferrals

| Field | Value |
|---|---|
| ID | `R-PKG-DEFER` |
| Description | The v0.1 contract is portable archives + `SHA256SUMS.txt` only. Deliberately not produced: macOS `.dmg`; Windows `.msi` and NSIS installers; Linux `.deb`/`.rpm`/AppImage; macOS x86_64 artifact; universal binary; auto-update. Rationale: installers multiply unverifiable (and unsigned) artifact surface across five packaging ecosystems without resolving any actual blocker; macOS x86_64 has no runner in the matrix; auto-update is out of scope for a first release. Archives are also **not bit-reproducible** (mtimes are embedded by the archivers). |
| Severity | `P2` (platform coverage) |
| Owner / Decision | Decision: Option A + macOS `.app` wrapper (recorded in `docs/release/0.1.0-packaging.md` and `docs/release/evidence/v01-06/05-workflow-audit.md` §3). |
| Status | `DEFERRED` |
| Release disposition | `DEFERRED` — release notes and README state the deferral explicitly. |

### R-WINLINUX — Windows/Linux build-verified in CI, runtime-unverified

| Field | Value |
|---|---|
| ID | `R-WINLINUX` |
| Description | The release matrix builds `windows-latest` (x86_64 MSVC) and `ubuntu-latest` (x86_64 GNU). In the final run `34860902181` (and in run `34859158012` before it) both **compile and package successfully in CI** — `Build (Windows)` ✔, `Build (Linux)` ✔, `Package (Windows)` ✔, `Package (Linux)` ✔ — so they are **`BUILD_VERIFIED`** (and `PACKAGE_VERIFIED`), and the produced archives were checked against the contract member list and re-hashed locally. They remain **`RUNTIME_UNVERIFIED`**: **no Windows or Linux host exists in this project**, so no process has ever been launched from those archives. The two states are deliberately not blurred: "the pipeline can build and package them" is not "they run". Additionally, Linux credential storage requires a D-Bus Secret Service provider (the shipping crate uses `sync-secret-service`), so headless/minimal installs will fail credential writes. |
| Severity | `P1` (for any cross-platform *runtime* claim) |
| Owner / Decision | Decision: ship the matrix as contract; record `BUILD_VERIFIED` (CI, run `34860902181`) / `RUNTIME_UNVERIFIED` (no host), and never claim Linux/Windows runtime quality. Final artifact values for the candidate `85a7fa3` are in §4. |
| Status | `OPEN` (build/package verified; runtime not verifiable here) |
| Release disposition | `ACCEPTED` for the internal macOS-ARM64-only RC; **`BLOCKING`** for any cross-platform runtime-quality claim. |

### R-AGENT — Agent Preview not verified live

| Field | Value |
|---|---|
| ID | `R-AGENT` |
| Description | The Agent panel ships labelled Preview (`crates/ui/src/agent_view.rs` renders a `Preview` badge) and the tool path is confirmation-gated with anti-TOCTOU re-classification. Its runtime claims rest on automated tests only; **no live provider run is recorded** in the repository, and `query-editor-intelligence/VERIFICATION.md` still records that live AI verification is pending because no provider key was configured. |
| Severity | `P2` |
| Owner / Decision | Decision: keep the panel in 0.1.0 **as Preview**, with no autonomy claims; provider is optional and inert without a key. |
| Status | `ACCEPTED` |
| Release disposition | `ACCEPTED` — release notes list Agent under Preview, and "advanced AI / autonomous execution" under not-included. |

### R-PROV — provider limitations

| Field | Value |
|---|---|
| ID | `R-PROV` |
| Description | Provider capabilities are asymmetric and partly unqualified. **Query cancellation: SQLite supported (VM interrupt + actor acknowledgement), PostgreSQL `Unsupported`/capability-gated** (`capabilities.rs`: `postgres.cancel = false`, `sqlite.cancel = true`; `postgres/connector.rs` exposes no wire-level cancel). Earlier release docs stated the inverse; the code is the authority and the docs are corrected. **The `v01-runtime` session verified this against the live provider and the shipping paths** — `cancel: false` for PostgreSQL, `true` for SQLite, gated in the UI render/activation, the Esc path and the fail-closed lookup, with an explicit connector-level `Unsupported`; the only other cancel-shaped control is gallery-only (`providers/15`, `providers/23`). The same session ran **18/18 live PostgreSQL integration tests** covering introspection (tables/indexes/FKs/triggers/views), transaction/rollback and typed value decoding (`providers/07`) — provider-level evidence only, no UI run. Still open: `sequences:true` / `enum_types:true` are declared in `DatabaseCapabilities::postgres()` with **no catalog implementation** (the capability matrix records both as `MISSING`); CHECK/Unique constraint introspection is unqualified (`R005`); SQLite has no UUID/array/generated-column support; `pg_dump`/`pg_restore` are not bundled (`R006`, and see `F1` for the version-skew behaviour). |
| Severity | `P2` |
| Owner / Decision | Decision: document the real per-provider state; do not infer PostgreSQL support from a SQLite result or vice versa. Flag corrections are recorded here as a follow-up (production code is out of scope for this pass). |
| Status | `ACCEPTED` (documented) / flag fix `DEFERRED` |
| Release disposition | `ACCEPTED` for documented limits; the misleading capability flags are `DEFERRED` to a code pass. |

### R-STATE-DIR — packaged-app working directory / state directory

| Field | Value |
|---|---|
| ID | `R-STATE-DIR` |
| Description | **Reproduced as a real startup blocker and fixed on 2026-09-14 (code commit `543b526`).** `resolve_data_dir()` (formerly `crates/native-app/src/main.rs:127-135`) returned `DB_PRO_DATA_DIR` if set, otherwise `current_dir()/.db-pro-data`. A `.app` launched through LaunchServices inherits `cwd=/`, so it resolved `/.db-pro-data`; `DbProRuntime::new`'s `create_dir_all` failed and `main()` returned `Err`, exiting 1 **without ever opening a window** with the exact error `Error: CreateDataDir(Os { code: 30, kind: ReadOnlyFilesystem, message: "Read-only file system" })`. Because the process dies after `open` returns, "LaunchServices accepted the bundle" (the earlier claim in `03-release-binary.txt` / `08-post-fix-quality-gates.txt`, now annotated) proved nothing by itself. Fixed resolution order: `DB_PRO_DATA_DIR` (set and non-empty) → an **existing** `<cwd>/.db-pro-data` (so the developer checkout and existing installs keep their data in place) → `<platform data dir>/DB Pro` (macOS `$HOME/Library/Application Support/DB Pro`, Windows `%APPDATA%\DB Pro`, other Unix `$XDG_DATA_HOME/db-pro` else `$HOME/.local/share/db-pro`) → `<cwd>/.db-pro-data`, or the relative `.db-pro-data` when the working directory is unavailable (the old `expect(...)` panic is gone). No dependency added; `--locked` builds unaffected. |
| Severity | `P2` |
| Owner / Decision | **Fixed** in `543b526`, with the regression test `platform_dir_is_used_when_no_legacy_dir_exists` (exactly the `cwd=/` + no-legacy-dir + resolvable-platform-dir case) and artifact-level proof in `docs/release/evidence/v01-06/12-state-dir-blocker-fix.txt`. Honest scope of the verification: on macOS the packaged bundle now launches through `open` and stays alive past 30 s, and a direct `cwd=/` run creates a writable per-user state directory containing `meta.db` (baseline schema + migration v2) with no `/.db-pro-data`; **no window was rendered or interacted with**, and the full interactive smoke (create a SQLite connection, run `SELECT 1;`, relaunch for persistence) remains **`NOT VERIFIED`**, owned by the coordinator — the post-fix host smoke is recorded in `docs/release/evidence/v01-06/14-install-smoke.txt` (archive extraction, launch and file-level state reuse verified, including on the CI-produced artifact of the final run; every GUI step is listed as NOT VERIFIED in its §7.7, human runbook in its §8) and the residual gap is tracked separately as `R-GUI-SMOKE`. Not fixed here and recorded separately in `12-…txt` §6: a pre-existing Keychain authorization prompt on first launch of the ad-hoc-signed app can stall an *unattended* launch before data-dir resolution (register entry `R-KEYRING-STALL`, limitation LIM-018). |
| Status | `FIXED` |
| Release disposition | `FIXED` |

### R-INSTALL-NOTE — packaged install note contradicted the fixed state-directory behaviour

| Field | Value |
|---|---|
| ID | `R-INSTALL-NOTE` |
| Description | `scripts/release/README-INSTALL.txt` ships inside all three v0.1 archives. Its "First run" section still described the **pre-`543b526`** behaviour — *"DB Pro keeps its workspace state (connections, saved queries, query history, open tabs, settings) in a `.db-pro-data` directory next to the directory the app is started from. Set the `DB_PRO_DATA_DIR` environment variable to choose a different location."* — so a user reading the shipped note would look in the wrong place. Found while writing `14-install-smoke.txt` (§9: the note in the archive is byte-identical to the repo file, verified with `diff`). |
| Severity | `P3` (shipped documentation defect) |
| Owner / Decision | **Fixed in `85a7fa3`** (`docs(release): correct the packaged install note's state directory description`), a text-only packaging-asset commit (no `.rs`, manifest or workflow change). The new "First run" text documents the resolution order as implemented: (1) `DB_PRO_DATA_DIR` when set and non-empty, used as-is; (2) an existing `.db-pro-data` next to the directory the app is started from; (3) the per-user data directory (macOS `~/Library/Application Support/DB Pro`, Windows `%APPDATA%\DB Pro`, Linux `$XDG_DATA_HOME/db-pro` or `~/.local/share/db-pro`); (4) otherwise a `.db-pro-data` next to the start directory. |
| Status | `FIXED` |
| Release disposition | `FIXED` — **verified inside the shipped archive**: the corrected note was read back from the CI-produced archive of the final run `34860902181`, which is the artifact a distributor would ship. The superseded archives of run `34859158012` (`1a0c186`) were built before this commit and still contain the old wording; they must not be distributed as the release artifact. |

### R-GUI-SMOKE — interactive install smoke not verified

| Field | Value |
|---|---|
| ID | `R-GUI-SMOKE` |
| Description | No window has ever been rendered, inspected or interacted with on a packaged build. Specifically **NOT VERIFIED** (`14-install-smoke.txt` §7.7): that the window renders at all; Settings navigation and the light/dark switch; creating a SQLite connection through the UI (driver cards, file-path field, Test/Save); running `SELECT 1;` and reading the result grid; running `SELECT * FROM items;` and counting 3 rows; quitting with a clean window-close / ⌘Q; and the interactive persistence check (a saved connection surviving relaunch). What **is** verified on the packaged archive: the contract layout and checksum, the shipped executable being byte-identical to the qualified build, LaunchServices launch and survival from the extracted path, the state directory resolving to `~/Library/Application Support/DB Pro` with no `/.db-pro-data`, and file-level state reuse across relaunch (`14-install-smoke.txt` §1–§5). The 2026-09-14 attempt could not reach the GUI at all, for four separately recorded environment reasons: the Orca computer-use helper returned `runtime_unavailable` (and later lost its runtime metadata entirely), AppleScript fallback hit a `-1743` TCC denial, `screencapture` returned "could not create image from display", and no accessibility tree was available (`14-install-smoke.txt` §7.1–§7.6). These are host/authorization limits, not app failures — and precisely because they could not be lifted, no app-side GUI claim can be made either way. The human runbook that closes the gap is `14-install-smoke.txt` §8, in executable form `docs/release/0.1.0-interactive-verification-runbook.md`. The `v01-runtime` session re-verified all four blockers live and captured no screenshot (`providers/21`, `screenshots/README.md`). |
| Severity | `P1` (blocks any "installable and usable" runtime claim) |
| Owner / Decision | Decision: record as an open, disclosed gap and keep every release document from claiming interactive or visual verification. `docs/release/0.1.0-ui-visual-description.md` is a **code-derived** description of the shipped surfaces that does **not** close this gap. Closing it requires a desktop session and the §8 runbook. |
| Status | `OPEN` |
| Release disposition | `ACCEPTED` for the internal RC **with the gap stated**; **`BLOCKING`** for any public claim that the packaged app is interactive/visually verified. |

### R-KEYRING-STALL — pre-existing keyring prompt can stall an unattended first launch

| Field | Value |
|---|---|
| ID | `R-KEYRING-STALL` |
| Description | `main()` calls `seed_groq_api_key_from_keyring()` (`crates/native-app/src/main.rs`) **before** the state directory is resolved; when a `com.dbpro.app` item exists in the Keychain, that call blocks on a Keychain authorization request for the stored item, so an unattended launch waits there before it ever reaches the data-directory code. Recorded as `LIM-018`; first characterised in `12-state-dir-blocker-fix.txt` §6, then bypassed in the host smoke with a placeholder `GROQ_API_KEY` (`14-install-smoke.txt` §3 — which is why that smoke does **not** evidence a plain double-click launch on a machine that has a stored key). **Reproduced and classified in the `v01-runtime` session (2026-09-14):** on the packaged CI artifact, launch A (real HOME, no `GROQ_API_KEY`) stalled with **4762/4762 samples** in one stack — `main → keyring::Entry::get_password → SecKeychainFindGenericPassword → CSSM_DecryptDataFinal → SecurityServer::ClientSession::decrypt → mach_msg`, waiting on `securityd` — with `meta.db` unchanged and zero bytes of output; the block is unbounded (no timeout on the call). Control launch B (empty fake HOME, no keychain item) passed the keyring instantly and reached `-[NSApplication run]`. Classification: **environment + keyring-backend behaviour, not an app-logic error**, with one real design weakness (the read is unbounded and on the startup path before the data directory is resolved). Practical exposure: **a returning user or any unattended launch on a machine where the app has already stored a keychain item** — a brand-new user is unaffected. Evidence: `providers/16`, `providers/22`, `providers/23` (`F3`). No code was changed and no placeholder key was used to paper over the stall in the reproduction. |
| Severity | `P2` |
| Owner / Decision | Decision: keep as a documented known limitation for 0.1.0 (a fix would be code work, out of scope for this documentation pass). It is a consequence of shipping unsigned/ad-hoc-signed (`R003`): an ad-hoc-signed binary cannot read an existing keyring item without an interactive grant; answering the prompt is the only user-side workaround. **Escalation condition (recorded, owner's call):** if v0.1 is ever deployed to an unattended/headless context, or if the prompt is found to recur on every launch after "Always Allow" (plausible under ad-hoc signing, since the code identity changes across builds), this item must be re-classified `P1` and fixed by making the keyring read **bounded and off the pre-data-dir critical path** (`providers/22` §5). |
| Status | `OPEN` (pre-existing, disclosed) |
| Release disposition | `ACCEPTED` — stated in `known-limitations.md` (LIM-018), the release notes and the handoff. |

### R-CI-PREFLIGHT — release pre-flight blocker chain (toolchain drift → Linux D-Bus → flaky ER worker test)

| Field | Value |
|---|---|
| ID | `R-CI-PREFLIGHT` |
| Description | Three successive, independent blockers stopped the release workflow before run `34859158012`, and all three are now closed. **(1) Toolchain drift:** CI's floating `stable` had moved to rustc 1.98.0, where `clippy::result_large_err` fires at `crates/core/src/application/table_data_service.rs:222` (recorded separately as `R-CLIPPY-198`) — fixed by pinning the release toolchain to the qualified 1.95.0 in `fbf9fda` (`rust-toolchain.toml`, with both release jobs verifying the active compiler; `09-toolchain-pinning.txt`). **(2) Linux build failure:** in run `34847235273`, job `Build (Linux)` died because `libdbus-sys`'s build script could not find `dbus-1.pc` — fixed in `e22a498` by declaring `libdbus-1-dev`/`pkg-config` explicitly in the build job (and naming them in preflight too), removing the reliance on a transitive pull through `libgtk-3-dev` (`11-linux-build-dependency-fix.txt`). **(3) Flaky test:** in run `34851704292`, job `Pre-flight checks` failed on the Linux runner with `diagram::tests::worker_coalescing_latest_result_wins` panicking `left: 5, right: 6` — a test that asserted on the *first* asynchronous layout result while the worker legitimately emits a superseded one first. Fixed in `1a0c186`, tests only (the ER layout worker's runtime behaviour is unchanged), with a deterministic reproduction of the CI signature and a negative control proving the rewrite still catches a worker that loses the newest request (`13-flaky-er-worker-test.txt`). |
| Severity | `P1` (release gate) |
| Owner / Decision | **Closed.** Chain fixed by `fbf9fda`, `e22a498`, `1a0c186`; the release pipeline then ran green end to end in **run `34859158012`**, and again — for the candidate — in the **final run `34860902181`** (`Resolve release candidate` ✔, `Pre-flight checks` ✔ on the pinned 1.95.0 toolchain, `Build` ×3 ✔, `Package` ×3 ✔, `Assemble SHA256SUMS` ✔). `R-CLIPPY-198` remains a recorded follow-up for a future 1.98 toolchain bump, and the wall-clock performance-budget tests in the same diagram test file are named in `13-…txt` §9 as a candidate for a separate determinism review — neither blocks 0.1.0. |
| Status | `FIXED` |
| Release disposition | `FIXED` |

### R-015 — workspace/session persistence not implemented

| Field | Value |
|---|---|
| ID | `R-015` |
| Description | The native build does not persist workspace tabs or settings (eframe persistence is off). The manual-smoke checklist includes "Workspace tabs restore without blank/orphan crash", which therefore cannot pass as written, and release notes must not claim restart recovery of tabs. |
| Severity | `P1` (product truth) |
| Owner / Decision | Decision: document as a known limitation for 0.1.0 (implementing persistence is feature work, out of scope for closure). |
| Status | `ACCEPTED` |
| Release disposition | `ACCEPTED` — stated in `known-limitations.md`, README and release notes. |

### R-CI-MAIN-RED — `ci.yml` red on `main` for a pre-existing, unrelated reason

| Field | Value |
|---|---|
| ID | `R-CI-MAIN-RED` |
| Description | The push CI workflow fails on `main` because of an escaped-quote bug in a Python one-liner inside the `Configure live SSH backup fixture` step. This predates the candidate and is unrelated to the release artifact. **Still open, but no longer a release-gate concern:** the **release** workflow is green end to end (run `34859158012`), so nothing in the release path is blocked by `ci.yml` being red. It is recorded so nobody misreads a red `main` as a release blocker — and so nobody "fixes" it inside a release commit. |
| Severity | `P2` |
| Owner / Decision | Decision: out of scope for the release closure pass; fix separately with a `ci(...)`-scoped commit. |
| Status | `DEFERRED` |
| Release disposition | `DEFERRED` |

### R-CLIPPY-198 — real clippy finding deferred behind the toolchain pin

| Field | Value |
|---|---|
| ID | `R-CLIPPY-198` |
| Description | clippy 1.98 flags `clippy::result_large_err` at `crates/core/src/application/table_data_service.rs:222` (the `Result<u64, TransactionFailure>` return of `apply_mutations_detailed`). With the release toolchain pinned to 1.95.0 this does not fire locally, but it is a **real** finding, not noise: CI on floating `stable` failed on it (run 34845148946). No `#[allow]` was added and no error was boxed. |
| Severity | `P2` (maintenance) |
| Owner / Decision | Decision: pin the toolchain (done, `fbf9fda`) and keep this as a recorded follow-up. Code change deferred out of the documentation pass. |
| Status | `DEFERRED` |
| Release disposition | `DEFERRED` |

### R-MINOS — macOS minimum system version 11.0

| Field | Value |
|---|---|
| ID | `R-MINOS` |
| Description | The macOS binary's `LC_BUILD_VERSION` reports `minos 11.0`; the packaging script derives `LSMinimumSystemVersion` from the binary (currently 11.0). macOS 11 or newer is therefore required, and the bundle metadata cannot silently drift from what was compiled. |
| Severity | `P3` (informational) |
| Owner / Decision | Decision: accept and document; no compatibility claim beyond macOS 11. |
| Status | `ACCEPTED` |
| Release disposition | `ACCEPTED` |

### RC1 P2 dispositions (25 findings) and QA-D1

| Field | Value |
|---|---|
| ID | `R-RC1-P2` |
| Description | The RC1 programme tracked 25 P2 findings, all marked `FIXED` against files that were subsequently **archived**: 24 are `OBSOLETE` for their finding identity (their `frontend/…` targets now live under `_archive/frontend/`), 1 is `DEFERRED` (QA-P2-06: the React-era i18n requirement is silently absent — the native UI has no i18n framework). No finding met the crash/data-loss/credential-leak/destructive-SQL-misclassification bar, so **none was promoted to P0/P1**. The release-relevant column is the native carry-over: **20 items are `CARRIED_OVER_UNVERIFIED`** (no evidence either way in the shipped egui UI), 1 is `SATISFIED_IN_NATIVE` (QA-P2-04 Preview badge), 3 are `NOT_APPLICABLE_IN_NATIVE`/`NOT_CARRIED`. Separately, `QA-D1` ("saved query rename is delete-then-save", data-loss class) is **`FIXED` in the shipping code path** with evidence: `rename()` in `crates/core/src/ports/saved_query_repository.rs`, atomic `UPDATE … SET name` in `crates/infrastructure/src/meta/saved_query_repo.rs`, plus `rename_rejects_missing_saved_query`. The programme's own P2 gate checkbox ("P2 accepted/fixed/deferred explicitly") is **still unchecked**, so that gate is formally unsatisfied. |
| Severity | `P2` |
| Owner / Decision | Decision: do not mass-close the 20 carry-overs; re-verify them in the same native runtime pass that would re-establish V01-01/V01-05 evidence. Give QA-P2-06 an explicit English-only decision. |
| Status | `DEFERRED` (carry-overs) / `FIXED` (QA-D1) |
| Release disposition | `DEFERRED` + `ACCEPTED` (English-only) + `FIXED` (QA-D1) |

### Step-1 blockers B-1…B-10 (from the workflow/packaging audit)

| ID | Description | Severity | Disposition |
|---|---|---|---|
| `B-1` | `R-LICENSE`: no LICENSE file / no manifest metadata | governance | `BLOCKING` (public distribution) — see `R-LICENSE` |
| `B-2` | All artifacts `UNSIGNED`; macOS `spctl` rejects | P2 | `ACCEPTED` if stated — see `R003` |
| `B-3` | macOS x64 not built; runner architecture historically implicit, now pinned to `macos-14` | P2 | `DEFERRED` — see `R-PKG-DEFER`; the pin is fixed, the x64 slice is not offered |
| `B-4` | No `.app` bundle originally → no bundle ID, capture harness cannot see the app | P2 | **`FIXED`** — `scripts/release/package-macos.sh` builds a minimal `DB Pro.app` with a generated `Info.plist` |
| `B-5` | No archive packaging, no `SHA256SUMS.txt`, no arch in artifact names, no assembly job | P1 (deliverable) | **`FIXED`** — `release.yml` `package` + `checksums` jobs and `scripts/release/*` implement the contract; all three archives + `SHA256SUMS.txt` were produced and independently verified in the final run `34860902181` (values in §4) |
| `B-6` | Windows/Linux build+package verified in CI, permanently `RUNTIME_UNVERIFIED` | P1 (for the claim) | `BUILD_VERIFIED` in run `34859158012`; `BLOCKING` for cross-platform *runtime* claims — see `R-WINLINUX` |
| `B-7` | Linux credential storage requires a D-Bus Secret Service provider | P2 | `ACCEPTED` (documented) / `DEFERRED` (hardening) |
| `B-8` | Developer connection preset shipped in the release binary | P2 (hygiene) | **`FIXED`** — `c682552`: preset gated to debug builds; residual single "Xe Lạc Hồng" string is component-gallery demo text with no host/db/user/password, and the gallery label was removed in `7794196` |
| `B-9` | `pg_dump`/`pg_restore`/`ssh` not bundled and not documented for users | P2 | `ACCEPTED` — documented as a limitation (LIM-006, LIM-015) |
| `B-10` | `0.1.0-packaging.md` presented Tauri-era `.dmg`/`.msi`/`.deb`/`.rpm` as `PASS` | P3 (misleading) | **`FIXED`** — packaging doc rewritten to the implemented contract |

### R005 — CHECK/Unique constraint introspection unqualified

| Field | Value |
|---|---|
| ID | `R005` |
| Description | No dedicated CHECK/Unique constraint test exists; the capability matrix records `Unique` as `PARTIAL` (single-column derived, multi-column reported as an index) and CHECK as `INSPECT`/`PARTIAL` with LIM-011's disposition still pending. |
| Severity | `P2` |
| Owner / Decision | Decision: keep out of qualified claims for 0.1.0; revisit with a dedicated introspection pass. |
| Status | `DEFERRED` |
| Release disposition | `DEFERRED` |

### R006 — `pg_dump` / `pg_restore` not bundled

| Field | Value |
|---|---|
| ID | `R006` |
| Description | PostgreSQL backup/restore shells out to `pg_dump`/`pg_restore` (and restore through `psql`), which must be on `PATH`. Silent-failure risk if missing, with no in-app guidance beyond documentation. The `v01-runtime` session verified the dependency at CLI level against a live fixture — dump exit 0, restore complete — and found the client/server **version-skew** behaviour recorded separately as `F1`. |
| Severity | `P2` |
| Owner / Decision | Decision: accept for 0.1.0 with documentation; bundling is a post-0.1 decision (LIM-015). |
| Status | `ACCEPTED` |
| Release disposition | `ACCEPTED` |

### F1 — `pg_restore` reports failure under client/server major-version skew, after a complete restore

| Field | Value |
|---|---|
| ID | `F1` (raised by the `v01-runtime` session, 2026-09-14) |
| Description | With PostgreSQL **client tools 18.4** on `PATH` against the fixture **server 16.15**, a custom-format restore **restores everything correctly but exits 1**: `pg_restore: error: could not execute query: ERROR: unrecognized configuration parameter "transaction_timeout"` / `Command was: SET transaction_timeout = 0;` / `warning: errors ignored on restore: 1`. `transaction_timeout` was introduced in PostgreSQL 17. The restore is nevertheless **complete**: 10 tables / 2 views / 17 indexes / 8 sequences, identical row counts, identical per-table MD5 fingerprints, enum/trigger/functions/view present, and `fixtures/postgres/003_verify.sql` PASS against the restored database. Root cause isolated by control: version-matched 16.15→16.15 tools give **exit 0** with identical data; a 16.15 `pg_restore` cannot even read an 18.4-written archive (`unsupported version (1.16) in file header`). The app surfaces this because `crates/infrastructure/src/backup/pg_dump.rs:167-170` treats **any** nonzero `pg_restore` exit as `restore failed: …`. The plain-format path (`psql -f`, `pg_dump.rs:143-147`) is unaffected: exit 0 on the same skew. Evidence: `docs/release/evidence/v01-runtime/providers/10`–`14`, `providers/23`. |
| Severity | `P2` — the reported *status* is wrong, no data is lost and nothing is written to the wrong target. Deliberately **not** `P1`. |
| Owner / Decision | **Owner decision required (`HD-006`)** on the disposition: state the client/server version expectation in `platform-prerequisites.md`, and/or detect the version pair before a custom-format restore. **The nonzero-exit check was deliberately not weakened**: failing on a nonzero `pg_restore` exit is the safe default, and relaxing it is a production change to the credential/backup path — out of scope for a documentation-only pass and not requested. |
| Status | `OPEN` (recorded, not chased) |
| Release disposition | `ACCEPTED` with disclosure for the internal RC — `pg_dump`/`pg_restore` remain "must be on `PATH`; not bundled" and the version-skew behaviour is stated in `known-limitations`/readiness. **`BLOCKING` for any claim that PostgreSQL restore is qualified across client/server versions.** |

### F2 — `epaint` font-atlas glyph fallback warning (cosmetic)

| Field | Value |
|---|---|
| ID | `F2` (raised by the `v01-runtime` session, 2026-09-14) |
| Description | With `RUST_LOG=info` the packaged binary emits 13 identical lines: `WARN epaint::text::font: Failed to find replacement characters '◻' or '?'. Will use empty glyph.` A glyph requested by the bundled icon font (`◻`, U+25FB) has neither a glyph in the font nor a resolvable replacement, so egui draws an empty glyph. Source is `egui`/`epaint`, not DB Pro code. Emitted only when `RUST_LOG` is set; with `RUST_LOG` unset, every launch in the session produced **zero bytes** of output. Whether it is visible on screen is **not verifiable here** (no window server), so no visual claim is made either way. Evidence: `docs/release/evidence/v01-runtime/providers/19`, `providers/23`. |
| Severity | `P3` (cosmetic) |
| Owner / Decision | Decision: record only; no code change. If a future visual pass shows a missing glyph in the UI, treat it as a packaging/icon-font issue, not a UI-logic bug. |
| Status | `OPEN` (recorded, not chased) |
| Release disposition | `ACCEPTED` (cosmetic, log-only, `RUST_LOG`-gated) |

### R011 — keyring feature coverage (historical)

| Field | Value |
|---|---|
| ID | `R011` |
| Description | Historically the keyring crate was compiled without platform credential-store features and production bootstrap used a weakly-keyed encrypted-file fallback. The shipping crate now selects `apple-native`/`windows-native`/`sync-secret-service`; on Linux that means credentials need a D-Bus Secret Service (see `B-7`). The encrypted fallback remains available and is no longer the production default. |
| Severity | `P2` |
| Owner / Decision | Decision: FIXED for the original defect; Linux Secret Service requirement accepted/documented. |
| Status | `FIXED` |
| Release disposition | `FIXED` |

### R-STATE-MIGRATION — state layout and migration safety (v0.1 install/update)

| Field | Value |
|---|---|
| ID | `R-STATE-MIGRATION` |
| Description | The runtime state directory holds `meta.db` (SQLite workspace store: connections, saved queries, query history, workspace, settings, introspection cache, run configs) plus `secrets/`. A **versioned migration mechanism does exist**: `crates/infrastructure/src/meta/migration.rs` keeps an ordered `MIGRATIONS` registry with `LATEST_VERSION = 2`, records applied versions in `schema_version`, skips already-applied migrations, **fails closed on a malformed version row**, and **rejects a database newer than `LATEST_VERSION`**. Residual risk: there is no automated backup/downgrade path if a future migration is wrong; the state directory now resolves to a per-user location (platform data dir) unless an existing working-directory-local `.db-pro-data` is present — see `R-STATE-DIR` (`FIXED`). |
| Severity | `P2` |
| Owner / Decision | Decision: accept for 0.1.0; state layout and migration behaviour documented in the packaging/startup documentation. |
| Status | `ACCEPTED` |
| Release disposition | `ACCEPTED` |

## 2. Summary

| ID | Severity | Status | Release disposition | Blocks |
|---|---|---|---|---|
| `R-LICENSE` | P1 | OPEN | **BLOCKING** | public distribution |
| `R-WINLINUX` | P1 | OPEN (build+package verified in CI, run `34860902181`) | `ACCEPTED` (internal RC) / **`BLOCKING`** | cross-platform *runtime* claims |
| `R-GUI-SMOKE` | P1 | OPEN | `ACCEPTED` (internal RC, disclosed) / **`BLOCKING`** | interactive/visual runtime claims |
| `R-015` | P1 | ACCEPTED | `ACCEPTED` | tab-restore claim |
| `R003` | P2 | ACCEPTED | `ACCEPTED` | public trust UX |
| `R001` | P2 | DEFERRED | `DEFERRED` | public naming/marketing |
| `R009` | P2 | DEFERRED | `DEFERRED` | SSH qualification |
| `R-PKG-DEFER` | P2 | DEFERRED | `DEFERRED` | installer/platform-coverage claims |
| `R-AGENT` | P2 | ACCEPTED | `ACCEPTED` | autonomy claims |
| `R-PROV` | P2 | ACCEPTED / DEFERRED | `ACCEPTED` (+`DEFERRED` flag fix) | provider qualification |
| `R-STATE-DIR` | P2 | FIXED | **`FIXED`** | — (was: packaged-app startup; fixed in `543b526`) |
| `R-INSTALL-NOTE` | P3 | FIXED | **`FIXED`** | — (was: shipped install note; fixed in `85a7fa3`, re-verified inside the final run's archive) |
| `R-KEYRING-STALL` | P2 | OPEN | `ACCEPTED` | unattended first-launch behaviour |
| `R-CI-PREFLIGHT` | P1 | FIXED | **`FIXED`** | — (toolchain pin `fbf9fda`, Linux D-Bus `e22a498`, flaky test `1a0c186`; run `34859158012` green) |
| `R-CI-MAIN-RED` | P2 | DEFERRED | `DEFERRED` | nothing release-specific (release workflow is green) |
| `R-CLIPPY-198` | P2 | DEFERRED | `DEFERRED` | future 1.98 toolchain bump |
| `R-MINOS` | P3 | ACCEPTED | `ACCEPTED` | — |
| `R-RC1-P2` | P2 | DEFERRED / FIXED | `DEFERRED` + `ACCEPTED` + `FIXED` | native P2 re-verification |
| `B-1`…`B-10` | see table | — | `FIXED` (B-4, B-5, B-8, B-10), `BLOCKING` (B-1), `ACCEPTED` (B-2, B-7, B-9), `DEFERRED` (B-3), B-6 = `BUILD_VERIFIED` + `BLOCKING` for runtime claims | see table |
| `R005` | P2 | DEFERRED | `DEFERRED` | constraint-introspection claims |
| `R006` | P2 | ACCEPTED | `ACCEPTED` | backup UX |
| `F1` (`pg_restore` version-skew exit code) | P2 | OPEN (recorded) | `ACCEPTED` with disclosure / **`BLOCKING`** for cross-version restore claims | restore-status reporting; owner decision `HD-006` |
| `F2` (glyph-fallback log warning) | P3 | OPEN (recorded) | `ACCEPTED` (cosmetic) | — |
| `R011` | P2 | FIXED | `FIXED` | — |
| `R-STATE-MIGRATION` | P2 | ACCEPTED | `ACCEPTED` | — |

**Blockers to public release:** `R-LICENSE` (no license chosen — the binding reason), the
interactive GUI install-smoke gap (`R-GUI-SMOKE`), unsigned artifacts (accepted with disclosure),
the open V01-01…V01-05 runtime evidence gaps, and Windows/Linux runtime being unverified.
**Not** a blocker any more: the packaging/build pipeline and the final artifacts — run
`34860902181` produced, packaged and checksum-verified all three platforms for the candidate
`85a7fa3` (final values in §4).

**P0 count: 0.** The `v01-runtime` session (2026-09-14) found **no `P0` and no `P1`**; its three
findings (`F1` `P2`, `F2` `P3`, `F3` = the already-tracked `R-KEYRING-STALL`) are recorded with
dispositions in `providers/23` and above.

## 3. Decision records

```
DECISION D001  (historical, still in force)
Question: Should v0.1 support MySQL or other providers?
Decision: PostgreSQL + SQLite only for v0.1
Rationale: Scope management; these two providers cover the target audience
```

```
DECISION D004  (2026-09-14)
Question: What is the v0.1 packaging contract?
Decision: portable archives + SHA256SUMS.txt (macOS ARM64 .tar.gz carrying a minimal
          "DB Pro.app", Windows .zip, Linux .tar.gz). No installers, no signing.
Rationale: installers would multiply unverifiable/unsigned artifact surface without
           resolving a real blocker; a .app bundle is the cheapest way to get a bundle ID
           and make the app observable to the capture harness.
Supersedes: the Tauri bundler contract (DMG/MSI/DEB/RPM), which no longer exists in the pipeline.
```

```
DECISION D005  (2026-09-14)
Question: Ship unsigned, or block on signing?
Decision: ship UNSIGNED and state the OS warnings explicitly.
Rationale: no certificate exists; an unsigned artifact is usable (with warnings), a claimed-signed
           artifact would be a lie. Revisit before any public distribution push.
```

```
DECISION D006  (2026-09-14)
Question: Does 0.1.0 ship the Agent?
Decision: the Agent **panel** ships, labelled Preview; production/autonomous execution does not.
Rationale: the preview is useful and confirmation-gated; autonomy is unverified and out of scope.
```

## 4. Candidate invalidation log

**One event recorded (2026-09-14).** Candidate `fbf9fdab9100f08f12e29434983f32c18f14ac2f` is
superseded for artifact purposes: the packaged macOS artifact produced from that tree could not
start when launched the normal way. `resolve_data_dir()` resolved `/.db-pro-data` under
LaunchServices' `cwd=/`, `DbProRuntime::new` failed with
`CreateDataDir(Os { code: 30, kind: ReadOnlyFilesystem, message: "Read-only file system" })` and the
app exited 1 without opening a window (`R-STATE-DIR`, reproduced by the coordinator and re-verified
here). Fixed in code commit `543b526` with a regression test and artifact-level evidence
(`12-state-dir-blocker-fix.txt`). No tag exists for `fbf9fda` or for the SHA that superseded it.

**Resolution (2026-09-14).** The re-dispatch this log required happened: run **`34859158012`** was
dispatched for SHA **`1a0c186`** (which contains `543b526`, `e22a498`, `fbf9fda` and the flaky-test
fix) and finished **fully green** — `Resolve` ✔, `Pre-flight checks` ✔ (fmt/clippy/tests on the
pinned 1.95.0 toolchain), `Build (macOS)` ✔, `Build (Windows)` ✔, `Build (Linux)` ✔,
`Package (macOS|Windows|Linux)` ✔, `Assemble SHA256SUMS` ✔. Its artifacts were downloaded and
verified independently: the three archives' sizes and SHA-256 values reproduced byte-for-byte with
`shasum -a 256` against the run's `SHA256SUMS.txt`, the archive member lists matched the contract
exactly (`DB Pro.app/Contents/{Info.plist,MacOS/db-pro-native}` + `README-INSTALL.txt`;
`db-pro-native` + note; `db-pro-native.exe` + note), and the provenance record named
`candidate_sha 1a0c186…`, `rustc 1.95.0 (59807616e 2026-04-14)`, `version 0.1.0`.

**Final values — run `34860902181` (candidate `85a7fa3cc0a84c56ac2a5049ce08130db06e0a20`).** The
final release run was green end to end and its artifacts were downloaded and independently
re-hashed on this host; the three values below reproduced byte-for-byte against the run's
`SHA256SUMS.txt` (298 bytes, exactly these three lines).

| Archive (run `34860902181`, candidate `85a7fa3`) | Size (bytes) | SHA-256 |
|---|---|---|
| `db-pro-v0.1.0-macos-arm64.tar.gz` | 10,045,965 | `8141d6b7b7ecd98399dd9c2ca23c345da96df496abe0cd0169f9ab967bf2a83a` |
| `db-pro-v0.1.0-linux-x86_64.tar.gz` | 15,168,133 | `3fecfc171dfae339c2c81d9f79be4616eb4168255116137d59e795ef09194970` |
| `db-pro-v0.1.0-windows-x86_64.zip` | 10,076,835 | `3b0ba8ebe8b7aa86d51a53eb1ddc7dd6f5adccaad57e6adc5d26536de4580452` |

Provenance for the same run: `candidate_sha 85a7fa3cc0a84c56ac2a5049ce08130db06e0a20`,
`short_sha 85a7fa3`, `rustc 1.95.0 (59807616e 2026-04-14)`, `version 0.1.0`, `source_kind commit`,
`event workflow_dispatch`.

Superseded values, kept as history (run `34859158012`, SHA `1a0c186`) — do not ship these:

| Archive (run `34859158012`, SHA `1a0c186`) | Size (bytes) | SHA-256 |
|---|---|---|
| `db-pro-v0.1.0-macos-arm64.tar.gz` | 10,045,739 | `911335d0ad8623ea44cf6a1f011c227baf6979eeebce6a9ac4c92ddaa2d5f817` |
| `db-pro-v0.1.0-linux-x86_64.tar.gz` | 15,167,954 | `d28bbf3a55c56f927f283edf4cc47e90867f6c08d285f6dc7d7ea7b7424c54a8` |
| `db-pro-v0.1.0-windows-x86_64.zip` | 10,076,654 | `62870d9a78f8edfaa7bcd800cf903e031da465ec1624c421d199aedcc01029fd` |

A second invalidation, if any, must be recorded here rather than silently re-tagging.

## 5. Completion criteria

This register closes when: P0 count = 0; no `BLOCKING` disposition remains; every `OPEN`/`DEFERRED`
item has either a decision or a scheduled owner; and the release notes/capability matrix state the
same facts as this register.
