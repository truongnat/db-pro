# Brand rename inventory — public surfaces (#102) and persisted identifiers (#121)

- Baseline: `main @ 6c698bc` (the SHA recorded in the ledger for this document)
- Issues: **#102** ([RC1][Brand C1] Inventory every public-facing rename surface before code changes)
  and **#121** ([RC1][Brand Prep] Inventory persisted storage/config identifiers for rename
  compatibility) — both children of workstream **#30** (resolve public identity and complete
  controlled product rename)
- Decision note: **the owner chose to retain `DB Pro` for v0.1.0** on 2026-09-16 (HD-009).
  No replacement rename is authorized in this release. This document remains the occurrence
  inventory + classification + compatibility checklist that #102/#121 ask for; every
  "must rename / keep / migrate" row below describes the work that would be required only if a
  future release authorizes a replacement identity.

## 1. Search patterns used (repository-wide, excluding `target/`)

| Pattern | Raw matches | Notes |
|---|---:|---|
| `DB Pro` | 399 | product display name; also prose in docs |
| `db-pro` | 992 | crate names, artifact names, file names, docs |
| `db_pro` | 638 | Rust crate identifiers |
| `dbpro` | 431 | bundle id `com.dbpro.app`, eframe storage keys |
| `DbPro` | 647 | Rust type names (`DbProApp`, `DbProRuntime`, `DbProTheme`) |

Raw counts include docs and test fixtures; the classified occurrences that matter for a rename are
the rows below. Every row cites the file:line read at the baseline SHA.

## 2. Public-facing rename surfaces (#102)

| # | Surface | Location | Current value | Classification |
|---:|---|---|---|---|
| P1 | Tauri product name | `crates/tauri-app/tauri.conf.json:2` | `DB Pro` | must rename before v0.1 if the product renames (archived bundler path — see §5) |
| P2 | Tauri window titles (2 windows) | `crates/tauri-app/tauri.conf.json:15,25` | `DB Pro` | must rename with P1 |
| P3 | Tauri bundle identifier | `crates/tauri-app/tauri.conf.json:4` | `com.dbpro.app` | **PERSISTED COMPATIBILITY — preserve legacy key** (keyring namespace + macOS identity are derived from it, see §3 rows K1/K3) |
| P4 | Native window title | `crates/native-app/src/main.rs:315` (`with_title`), `:322` (`run_native` app id) | `DB Pro` | must rename before v0.1 if the product renames |
| P5 | In-app product label (navigation bar) | `crates/ui/src/navigation_view.rs:478` | `DB Pro` | must rename (visible product string) |
| P6 | Component gallery subtitle | `crates/ui/src/component_gallery_view.rs:227` | `DB Pro Native` | must rename (visible string) |
| P7 | Palette view subtitle | `crates/ui/src/palette_view.rs:143` | `DB Pro common UI design system` | must rename (visible string) |
| P8 | README title + prose | `README.md:1,5,7,91-93,109,129` | `DB Pro`, `db-pro-*` | must rename (docs) |
| P9 | Repository name / description | GitHub `truongnat/db-pro`; description "A native desktop Database IDE for PostgreSQL and SQLite, built with Tauri 2, Rust, React, and TypeScript." | `db-pro` | must rename before v0.1 (description also still claims Tauri/React, which the native migration invalidated — flag separately) |
| P10 | Release archive names | `.github/workflows/release.yml:14-16` (contract) | `db-pro-v<version>-<platform>.tar.gz/.zip`, contents `DB Pro.app` / `db-pro-native(.exe)` | must rename if identity changes; artifact naming is a **release-contract change** and is explicitly out of scope for v0.1 workstreams (owner call) |
| P11 | CI intermediate artifact names | `.github/workflows/release.yml:421` | `db-pro-native-<arch>-v…-<sha>` | historical/internal — named after the binary deliberately (comment at `:415-418`); keep |
| P12 | Crate/package names | `Cargo.toml`; `crates/*/Cargo.toml` (`db-pro-core`, `db-pro-infrastructure`, `db-pro-runtime`, `db-pro-ui`, `db-pro-native`, `db-pro-tauri`, lib `db_pro_tauri_lib`) | `db-pro-*` / `db_pro` | **INTERNAL TECH ID — keep** unless a rename is ordered: crate renames touch every `use` path, the workspace layout, and CI package selection; renames risk release-path churn for zero user value |
| P13 | Rust type names | `DbProApp` (`crates/ui`), `DbProRuntime` (`crates/runtime`), `DbProTheme`, `DbProError`, etc. | `DbPro*` | INTERNAL TECH ID — keep (not user-visible) |
| P14 | Archived npm package name | `_archive/frontend/package.json:2` | `db-pro-frontend` | historical reference only (archived, not built) |
| P15 | Archived web frontend strings | `_archive/frontend/**` | `DB Pro`, `db-pro` | historical reference only |
| P16 | Icons/splash/about surfaces | Tauri icons: `crates/tauri-app/icons/**`; native app has no icon/splash/about asset in the native path (`crates/native-app` ships the binary + README only, `release.yml:14-16`) | n/a | any asset carrying the old name must be re-exported at rename time; none found under `crates/native-app` |
| P17 | User-agent / telemetry strings | grep for `user-agent`, `User-Agent`, `telemetry` over `crates/**` | none found | no action |
| P18 | Public links / website / domain | repo has no homepage (`gh repo view --json homepageUrl` → empty); no domain strings found in the tree | — | no action |
| P19 | Release notes / version metadata | `docs/release/0.1.0-release-notes.md`, `tauri.conf.json:3` (`version: 0.1.0`) | `DB Pro` prose | rename in the same pass as README |

## 3. Persisted storage/config identifiers (#121)

| # | Identifier | Location | Classification | Migration/notes |
|---:|---|---|---|---|
| K1 | Keyring service name | `crates/runtime/src/lib.rs:62` (`KEYRING_SERVICE = "com.dbpro.app"`), duplicated in `crates/native-app/src/main.rs:198` | **PERSISTED COMPATIBILITY — preserve legacy key** | entries already written (Groq key under `agent/groq_api_key`, connection secrets) must stay readable; a renamed product keeps reading the old service, or adds read-old/write-new. Do NOT blind-replace |
| K2 | Keyring key names | `crates/native-app/src/main.rs:200` (`agent/groq_api_key`); connection credential entries in `crates/infrastructure/src/secret/keyring_vault.rs` | INTERNAL TECH ID — keep | not brand-specific |
| K3 | macOS app-data dir | `crates/native-app/src/main.rs:130` (`PLATFORM_APP_DIR_NAME = "DB Pro"` → `~/Library/Application Support/DB Pro`) | **MIGRATE WITH FALLBACK** if renamed | resolve new dir; if absent and legacy dir exists, copy/migrate `meta.db` + `secrets/` once; keep a one-time marker (e.g. a `migrated` file) and never delete the legacy dir in the same release |
| K4 | Windows app-data dir | same constant → `%APPDATA%\DB Pro` | MIGRATE WITH FALLBACK (as K3) | same strategy |
| K5 | Linux data dir | `crates/native-app/src/main.rs:187-194` → `$XDG_DATA_HOME|~/.local/share/db-pro` | MIGRATE WITH FALLBACK (as K3) | already lowercase/hyphenated; rename implies a directory move with the same fallback rule |
| K6 | Legacy working-dir data dir | `crates/native-app/src/main.rs:128` (`LEGACY_DATA_DIR_NAME = ".db-pro-data"`), resolution precedence at `:137-168` (`DB_PRO_DATA_DIR` → existing `.db-pro-data` → platform dir) | **PERSISTED COMPATIBILITY — preserve legacy key** | the precedence rule itself must keep checking the old name for existing installs; the lookup is already tested (`main.rs:350-392`) |
| K7 | State override env var | `crates/native-app/src/main.rs:141` (`DB_PRO_DATA_DIR`) | PERSISTED COMPATIBILITY — preserve legacy key (support both names if a new one is added) | users with the var set in shell profiles must not lose their data dir |
| K8 | Secret-store fallback dir + file | `crates/runtime/src/lib.rs:118-119` (`data_dir/secrets`), `crates/infrastructure/src/secret/keyring_vault.rs:87,342` (`secrets.json`) | brand-neutral (path derived from data dir) | moves only as a consequence of K3-K5 |
| K9 | Meta store file | `crates/runtime/src/lib.rs:121` (`meta.db`) | brand-neutral | same |
| K10 | Secret fallback env var | `crates/runtime/src/lib.rs:68` (`DB_PRO_ALLOW_FILE_SECRET_FALLBACK`) | PERSISTED COMPATIBILITY — preserve legacy key (accept both) | release-safety switch |
| K11 | Agent endpoint/model env vars | `crates/runtime/src/agent.rs:123-134` (`DB_PRO_CODEX_ENDPOINT`, `DB_PRO_CODEX_MODEL`, `DB_PRO_GROQ_ENDPOINT`, `DB_PRO_GROQ_MODEL`) | PERSISTED COMPATIBILITY — preserve legacy key (accept both old and new) | users configure these in shells/launchd |
| K12 | eframe window/state storage keys | `crates/ui/src/app.rs:445-471` (`dbpro.native.grid-layouts`, `dbpro.native.grid-widths`, `dbpro.native.grid-widths-customized`, `dbpro.native.query-documents`, `dbpro.native.query-history-v1`, `dbpro.native.theme-version`, `dbpro.native.dark-mode`, `dbpro.native.reduce-motion`, `dbpro.native.prediction-mode`, `dbpro.native.sidebar-width`, `dbpro.native.agent-width`) | **PERSISTED COMPATIBILITY — preserve legacy key** | eframe persistence is OFF in v0.1 (`NativeOptions` at `main.rs:313-319` sets no persistence; LIM-016), so these keys are written to in-memory storage only. If persistence is enabled later, eframe persists them per app-id; renaming the app id (`run_native` arg, `main.rs:322`) would then silently orphan all layouts/theme/history unless the keys are migrated or the app id is kept. Keep the app id as the compatibility anchor, or add read-old/write-new |
| K13 | Backup temp files | `crates/infrastructure/src/backup/sqlite_backup.rs:20,553,578` (`.db-pro-<uuid>.tmp` + the cleanup filter) | INTERNAL TECH ID — keep | transient, self-cleaning; a rename is cosmetic and risks touching the cleanup matcher |
| K14 | Temp test dirs | `crates/runtime/src/lib.rs:341` (`db-pro-runtime-secrets-<pid>`) | INTERNAL TECH ID — keep | test-only |
| K15 | RUST_LOG default filter | `crates/native-app/src/main.rs:123` (`db_pro_runtime=info`) | INTERNAL TECH ID — keep | log filter, not brand surface; renaming would break existing user log configs |
| K16 | Updater channel/endpoint ids | grep `updater` over `crates/**` → none | no action | there is no updater in v0.1 (see #127) |

**Migration rules recorded for the MIGRATE WITH FALLBACK rows (K3-K5):** read-old/write-new — on
first launch of a renamed build, resolve the new directory; if it does not exist and the legacy
directory does, migrate its contents (copy, verify, then keep the legacy directory untouched for one
release) and write a one-time marker file inside the new directory so the migration never re-runs.
Rollback = relaunch the previous build, which still reads the legacy directory. Cleanup = a later
release may remove the legacy directory after a migration-marker grace period. Tests needed:
directory-resolution precedence, migration-once, marker present, legacy untouched.

## 4. Classification summary

| Class | Count | Rows |
|---|---:|---|
| must rename before v0.1 (conditional on the owner's identity decision) | 10 | P1, P2, P4, P5, P6, P7, P8, P9, P10, P19 |
| PERSISTED COMPATIBILITY — preserve legacy key | 7 | P3, K1, K6, K7, K10, K11, K12 |
| MIGRATE WITH FALLBACK | 3 | K3, K4, K5 |
| INTERNAL TECH ID — keep | 8 | K2, K13, K14, K15, P11, P12, P13, K16-note |
| historical reference only | 2 | P14, P15 |
| no action / not present | 2 | P17, P18 |

## 5. Implementation checklist handed to #103

1. Do NOT global search/replace `db-pro`/`db_pro`/`dbpro` — rows marked PERSISTED COMPATIBILITY and
   INTERNAL TECH ID must not move in the same pass as the display rename.
2. Order of work: owner decides the identity (#101/#30) → display surfaces (P1-P9, P19) → persisted
   identifiers with fallback (K3-K5) → grace-period cleanup.
3. The keyring namespace (K1) and the eframe app id (K12) are the two identifiers where a blind
   rename loses user data with no error message — both are flagged "preserve legacy key".
4. The Tauri bundler path is archived (`_archive/README.md`; release workflow ships `db-pro-native`
   only, `release.yml:3`), so P1-P3 matter only if the archived bundler is ever revived.
5. #121's acceptance ("no blind search/replace of persisted identifiers") is satisfied by §3: every
   persisted occurrence has a classification and the two high-risk rows carry an explicit migration
   rule.
