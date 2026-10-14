# Runtime Capture Evidence — 2026-10-13

- Binary: `cargo build -p db-pro-native --features capture` (dev profile, built 2026-10-13 01:18).
- Driver: `crates/native-app/src/capture.rs` (`DB_PRO_CAPTURE_TO` + `DB_PRO_WINDOW_SIZE`).
- Script: `docs/plans/_audit-2026-10-13/capture.sh` — reproducible, `cap <name> <env...>`.
- Output: `docs/plans/_audit-2026-10-13/captures/` — **102 PNGs, 34 surfaces × 3 requested sizes**.

## Effective dimensions (retina 2× framebuffer)

| Requested | PNG pixels | Effective logical | Note |
|---|---|---|---|
| 1280×800 | 2560×1600 | 1280×800 | exact |
| 1440×900 | 2880×1676 | 1440×**838** | macOS display height clamp |
| 1920×1080 | 3840×1676 | 1920×**838** | macOS display height clamp |

Full-height 900/1080 evidence still requires a larger display — unchanged from prior runs.

## Known process quirk (not a defect of the captures)

~60% of capture processes end in `Abort trap: 6` (AppKit NSTouchBar NSException) **after**
writing the PNG. The PNG is always complete before the crash. Matches the pre-recorded
STATUS.md note; cosmetic for evidence purposes. All 34 surfaces produced valid images at
all 3 sizes — **0 capture failures**.

## Surfaces captured (34)

welcome, welcome-light, loading, new-connection, new-connection-light, connection-error,
connection-test-pending, edit-connection, query, query-light, query-row-limit, results,
results-light, table, table-light, table-ddl, table-structure, table-indexes, table-profile,
table-foreign-keys, table-constraints, table-dependencies, table-completion, table-record,
diagram, settings, agent, gallery, explorer-filter, history, compare, quick-open,
quick-open-light, quick-open-empty.

## Visual review notes (inspector pass on rendered PNGs)

- Results dock renders seeded rows correctly — confirms `294ab089` blank-grid fix visually.
- ER diagram: 10 seeded tables, orthogonal FK edges, minimap, zoom readout 48%.
- Record view: fixed left rail + Field/Value single-record layout as designed.
- Schema compare: **SAFETY LOCK** banner correctly fires on divergent plan target
  (recorded prod-pg vs active local-pg), destructive ops marked, confirm-gate checkbox
  present — spec-09 safety behavior visible.
- Connection error dialog: required-field red outlines, inline messages, bottom toast —
  all legible, no layout breakage.
- Light theme (table-light, query-light, quick-open-light, welcome-light,
  new-connection-light): correct light palette throughout, no dark remnants.
- Minor cosmetic: compare summary diff-count row concatenates counts without spacing
  (`2 total difference(s)1 Added1 Changed0 Removed`) — cosmetic P2.
- Explorer fixture counts (Tables/Views/Functions/Triggers = 0) are fixture-seeded, not
  a rendering defect.

## What the captures do NOT cover

- Interactive states: hover, drag, scroll, keyboard focus, IME.
- 900/1080 full heights (display clamped at 838 logical px).

## Live provider runs (same session, 2026-10-13)

- **PostgreSQL 18.6** (Homebrew, localhost): fixture `dbpro_fixture` loaded from
  `fixtures/postgres/001_schema.sql` + `002_seed.sql`.
  `DATABASE_URL=postgres://dbpro:dbpro_test@localhost:5432/dbpro_fixture cargo test -p
  db-pro-infrastructure --test pg_integration -- --ignored` → **28 passed / 0 failed**.
- **MySQL** (localhost:3306): fixture `dbpro_fixture` loaded from
  `fixtures/mysql/001_schema.sql` + `002_seed.sql` (needed
  `SET GLOBAL log_bin_trust_function_creators=1` for the trigger/function DDL).
  `DATABASE_URL=mysql://dbpro:dbpro_test@localhost:3306/dbpro_fixture cargo test -p
  db-pro-infrastructure --test mysql_integration -- --ignored` → **8 passed / 0 failed**.
- **SQL Server**: no server on :1433 — `sqlserver_integration` remains BLOCKED.
- **SSH backup suite**: `DB_PRO_SSH_*` fixtures unavailable — remains BLOCKED.

## Verification after audit fixes

- Refreshed compare layout capture: `captures/compare-spacing_1280x800.png`.
  Explicit `SPACE_XS` gaps now separate the diff badge from its heading and bullet
  from its object name; visual capture confirms legibility. The compare summary
  counts were already spaced by the existing `SPACE_SM` horizontal layout.
- New column ALTER UI capture: `captures/table-structure-alter_1280x800.png`;
  Structure rows expose an `Alter` action for each column.
- PostgreSQL trigger operation smoke: in a transaction, `ALTER TABLE
  public.products DISABLE TRIGGER products_updated_at` produced catalog mode
  `D`; ENABLE restored mode `O`; transaction rolled back. This proves PG syntax
  and catalog state, not the full native UI→runtime path.
- PostgreSQL column ALTER smoke: transaction-created temporary table; combined
  TYPE + DROP NOT NULL + SET DEFAULT succeeded and catalog reported `text`, `YES`,
  `'new'::text`; transaction rolled back.
- MySQL column ALTER smoke: temporary `VARCHAR(20) NOT NULL DEFAULT 'old'` changed
  to `VARCHAR(320) NULL DEFAULT 'new'` successfully. `TEXT DEFAULT 'new'` was
  rejected by the local server (MariaDB 26.7.0); arbitrary type/default
  compatibility remains provider/version-dependent and surfaces as a DB error.
