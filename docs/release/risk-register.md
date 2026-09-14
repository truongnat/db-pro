# Central Release Risk, Decision & Blocker Register — v0.1

> One canonical release-risk and decision ledger for DB Pro v0.1.
> Candidate SHA under assessment: `fbf9fdab9100f08f12e29434983f32c18f14ac2f` (2026-09-14)
> **Post-candidate fix:** the packaged-app startup blocker `R-STATE-DIR` was reproduced on that
> candidate and fixed afterwards in code commit `543b526`
> (`fix(native): resolve the app state directory outside the working directory`); evidence:
> `docs/release/evidence/v01-06/12-state-dir-blocker-fix.txt`. See §4.
> Companion docs: `docs/release/0.1.0-readiness.md`, `docs/release/0.1.0-handoff.md`,
> `docs/notes/PRODUCT_CAPABILITY_MATRIX.md`, `docs/release/known-limitations.md`
> Evidence base: `docs/release/evidence/v01-06/*`

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
| Description | There is no `LICENSE` file anywhere in the tree and no `license`/`license-file` key in `Cargo.toml` or any `crates/*/Cargo.toml`. No license has been chosen. Confirmed 2026-09-14 by `git ls-files | grep -iE '(^|/)(LICENSE|COPYING|NOTICE|UNLICENSE)'` (no matches) and by grep over every manifest. The only license text in the repo is `crates/ui/assets/fonts/OFL.txt` (SIL OFL for the bundled Inter font — a third-party asset license, not a project license). |
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
| Description | SSH tunnels shell out to the system `ssh` binary (`crates/infrastructure/src/ssh/tunnel.rs`) with host-key verification; the single automated runtime test (`ssh_backup_runtime_verification.rs`) is `#[ignore]`d and requires nine `DB_PRO_SSH_*` variables, so on HEAD it runs as **0 passed / 1 ignored**. No cross-platform E2E qualification exists (no Windows/Linux host). |
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

### R-WINLINUX — Windows/Linux artifacts build-unverified, runtime-unverified

| Field | Value |
|---|---|
| ID | `R-WINLINUX` |
| Description | The release matrix builds `windows-latest` (x86_64 MSVC) and `ubuntu-latest` (x86_64 GNU), but only macOS ARM64 was built on a real host in this pass. Windows/Linux are therefore `BUILD_UNVERIFIED` here, and they will remain `RUNTIME_NOT_VERIFIED`: **no Windows or Linux host exists in this project.** Artifact names/checksums from the in-flight run are `PENDING_CI_RUN_RE_DISPATCH`. Additionally, Linux credential storage requires a D-Bus Secret Service provider (the shipping crate uses `sync-secret-service`), so headless/minimal installs will fail credential writes. |
| Severity | `P1` (for any cross-platform claim) |
| Owner / Decision | Decision: ship the matrix as contract, state `BUILD_UNVERIFIED` / `RUNTIME_NOT_VERIFIED` until the CI run completes, and never claim Linux/Windows runtime quality. |
| Status | `BLOCKING` (until the CI run completes) |
| Release disposition | `BLOCKING` for cross-platform release claims; `ACCEPTED` for the internal macOS-ARM64-only RC. |

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
| Description | Provider capabilities are asymmetric and partly unqualified. **Query cancellation: SQLite supported (VM interrupt + actor acknowledgement), PostgreSQL `Unsupported`/capability-gated** (`capabilities.rs`: `postgres.cancel = false`, `sqlite.cancel = true`; `postgres/connector.rs` exposes no wire-level cancel). Earlier release docs stated the inverse; the code is the authority and the docs are corrected. Also: `sequences:true` / `enum_types:true` are declared in `DatabaseCapabilities::postgres()` with **no catalog implementation** (the capability matrix records both as `MISSING`); CHECK/Unique constraint introspection is unqualified (`R005`); SQLite has no UUID/array/generated-column support; `pg_dump`/`pg_restore` are not bundled (`R006`). |
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
| Owner / Decision | **Fixed** in `543b526`, with the regression test `platform_dir_is_used_when_no_legacy_dir_exists` (exactly the `cwd=/` + no-legacy-dir + resolvable-platform-dir case) and artifact-level proof in `docs/release/evidence/v01-06/12-state-dir-blocker-fix.txt`. Honest scope of the verification: on macOS the packaged bundle now launches through `open` and stays alive past 30 s, and a direct `cwd=/` run creates a writable per-user state directory containing `meta.db` (baseline schema + migration v2) with no `/.db-pro-data`; **no window was rendered or interacted with**, and the full interactive smoke (create a SQLite connection, run `SELECT 1;`, relaunch for persistence) remains `PENDING`, owned by the coordinator. Not fixed here and recorded separately in `12-…txt` §6: a pre-existing Keychain authorization prompt on first launch of the ad-hoc-signed app can stall an *unattended* launch before data-dir resolution. |
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
| Description | The push CI workflow fails on `main` because of an escaped-quote bug in a Python one-liner inside the `Configure live SSH backup fixture` step. This predates the candidate and is unrelated to the release artifact; the release workflow's own gates pass. It is recorded so nobody misreads a red `main` as a release blocker — and so nobody "fixes" it inside a release commit. |
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
| `B-5` | No archive packaging, no `SHA256SUMS.txt`, no arch in artifact names, no assembly job | P1 (deliverable) | **`FIXED`** — `release.yml` `package` + `checksums` jobs and `scripts/release/*` implement the contract; artifact values `PENDING_CI_RUN_RE_DISPATCH` |
| `B-6` | Windows/Linux `BUILD_UNVERIFIED` and permanently `RUNTIME_NOT_VERIFIED` | P1 (for the claim) | `BLOCKING` for cross-platform claims — see `R-WINLINUX` |
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
| Description | PostgreSQL backup/restore shells out to `pg_dump`/`pg_restore` (and restore through `psql`), which must be on `PATH`. Silent-failure risk if missing, with no in-app guidance beyond documentation. |
| Severity | `P2` |
| Owner / Decision | Decision: accept for 0.1.0 with documentation; bundling is a post-0.1 decision (LIM-015). |
| Status | `ACCEPTED` |
| Release disposition | `ACCEPTED` |

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
| `R-WINLINUX` | P1 | BLOCKING | **BLOCKING** | cross-platform claims (until CI run lands) |
| `R-015` | P1 | ACCEPTED | `ACCEPTED` | tab-restore claim |
| `R003` | P2 | ACCEPTED | `ACCEPTED` | public trust UX |
| `R001` | P2 | DEFERRED | `DEFERRED` | public naming/marketing |
| `R009` | P2 | DEFERRED | `DEFERRED` | SSH qualification |
| `R-PKG-DEFER` | P2 | DEFERRED | `DEFERRED` | installer/platform-coverage claims |
| `R-AGENT` | P2 | ACCEPTED | `ACCEPTED` | autonomy claims |
| `R-PROV` | P2 | ACCEPTED / DEFERRED | `ACCEPTED` (+`DEFERRED` flag fix) | provider qualification |
| `R-STATE-DIR` | P2 | FIXED | **`FIXED`** | — (was: packaged-app install smoke; fixed in `543b526`) |
| `R-CI-MAIN-RED` | P2 | DEFERRED | `DEFERRED` | nothing release-specific |
| `R-CLIPPY-198` | P2 | DEFERRED | `DEFERRED` | future 1.98 toolchain bump |
| `R-MINOS` | P3 | ACCEPTED | `ACCEPTED` | — |
| `R-RC1-P2` | P2 | DEFERRED / FIXED | `DEFERRED` + `ACCEPTED` + `FIXED` | native P2 re-verification |
| `B-1`…`B-10` | see table | — | `FIXED` (B-4, B-5, B-8, B-10), `BLOCKING` (B-1, B-6), `ACCEPTED` (B-2, B-7, B-9), `DEFERRED` (B-3) | see table |
| `R005` | P2 | DEFERRED | `DEFERRED` | constraint-introspection claims |
| `R006` | P2 | ACCEPTED | `ACCEPTED` | backup UX |
| `R011` | P2 | FIXED | `FIXED` | — |
| `R-STATE-MIGRATION` | P2 | ACCEPTED | `ACCEPTED` | — |

**Blockers to public release:** `R-LICENSE`, `R-WINLINUX` (until artifacts exist), unsigned
artifacts (accepted with disclosure), and the open V01-01…V01-05 runtime evidence gaps.

**P0 count: 0.**

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
(`12-state-dir-blocker-fix.txt`). Consequence: any release-CI contribution for the deferred V01-06
steps must be re-dispatched for the new HEAD (`PENDING_CI_RUN_RE_DISPATCH`), not for `fbf9fda`.
No tag exists for either SHA.

## 5. Completion criteria

This register closes when: P0 count = 0; no `BLOCKING` disposition remains; every `OPEN`/`DEFERRED`
item has either a decision or a scheduled owner; and the release notes/capability matrix state the
same facts as this register.
