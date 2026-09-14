# Version/updater/persisted-state compatibility audit (#127)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 28e8795` (clean at the start of the change); commit recorded in `LEDGER.md`
- Issue: **#127** ([RC1][Release Prep] Audit app versioning, updater configuration, and
  persisted-state compatibility across upgrades)
- Deliverable: `docs/release/0.1.0-versioning-updater-persistence.md`

## What the audit produced

1. **Version matrix** — `crates/native-app/Cargo.toml` is the release source of truth
   (`release.yml:136` parses it, `:144-145` cross-checks the tag); the other five crate manifests
   and the two archived manifests (`tauri.conf.json`, `_archive/frontend/package.json`) are
   manually duplicated `0.1.0` with no synchronization check.
2. **Updater state** — no updater config, plugin, endpoint, public key or channel exists
   (`grep updater` over the live tree → no matches); the no-auto-update decision is already in
   release truth (`risk-register.md:110`, LIM-017, `release-notes.md:143`). Requirement "no
   dormant updater config presented as working" is met by absence.
3. **Persisted-state reality** — meta store `meta.db` with `schema_version` table and
   `LATEST_VERSION = 2` (`meta/migration.rs:21`), idempotent apply, self-healing partially-applied
   v2 (`:73-97`), and an explicit downgrade refusal (`current > LATEST_VERSION` → error, `:61-64`).
   Eframe `dbpro.native.*` keys (`app.rs:445-471`) are **not persisted in v0.1** (LIM-016;
   `NativeOptions` sets no persistence). Data-dir precedence (`main.rs:137-168`) already reuses a
   legacy `.db-pro-data` when present, and the precedence is tested (`main.rs:350-392`).
4. **Decision record** — D1 (manual updates) and D4 (version source of truth) are recorded from
   existing tree decisions; D2 (persisted-state policy: append-only migrations, downgrade refused,
   JSON persistence follows the brand inventory) is proposed with the supporting code rows; D3
   (legacy beta/dev state: migrate-and-keep / backup / reset) and D5 (bundle id stability) are
   recorded with options and left **PENDING owner decision**, as the task requires — not decided
   here.
5. **Correction made in the same change** — `docs/release/brand-rename-inventory.md` row K12 now
   states that eframe persistence is off in v0.1, so the `dbpro.native.*` keys are compatibility
   anchors for the future, not live state today.

## Verification

Docs-only change. `cargo check --workspace` re-run green after the commit. Cited line numbers were
re-opened during the audit (see the document for each pointer).
