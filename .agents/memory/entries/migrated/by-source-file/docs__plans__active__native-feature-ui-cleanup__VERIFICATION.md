---
schema_version: 1
id: "MEM-SRC-a479c2ebb9f096eb"
title: "Source mirror: docs/plans/active/native-feature-ui-cleanup/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/native-feature-ui-cleanup/VERIFICATION.md"
source_sha256: "a479c2ebb9f096eb792f4b8df4ac3badf42e4ec809154ea539e8c69be8df1291"
mirror_body_sha256: "a479c2ebb9f096eb792f4b8df4ac3badf42e4ec809154ea539e8c69be8df1291"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification

Baseline HEAD: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`. Candidate is uncommitted on main.

Skill searches executed successfully; output fit/rejection recorded in FINDINGS.md.
Implementation-specific checks and runtime captures: pending, not claimed as passing.

Known earlier-session gate results (not this candidate's final gates): full-workspace tests failed at native SQLite sample seeding; strict Clippy failed in untouched core/UI paths. Exact-height captures are capped to 838 logical pixels at larger widths. See ../quick-open-action-visuals/VERIFICATION.md for commands, source snapshots and original artifacts.

## 2026-10-19 Transfers sidebar clip

Source SHA remains `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; the layout fix is uncommitted.

Evidence-backed defect: at 1280×800 the Transfers sidebar cut copy labels and harness buttons (`Target`/`Format` shown as `Tar`/`For`, `Run then cancel` shown as `Ru`). The 260px CentralPanel test did not catch it. A wider-than-clip max rect was not measured, so that is not claimed as the cause.

Fix, presentation only: constrain the sidebar scroll body and the Transfers draw root to `min(available_width, clip_rect.width)`; request each copy-field group's real width; wrap the synthetic input and action rows.

Checked:

- `cargo test -p db-pro-app transfer_choice_groups_and_masking_actions_reflow_within_sidebar` — passed
- `cargo test -p db-pro-app session_actions_stay_inside_the_sidebar` — passed
- `cargo build --locked -p db-pro-native --features capture` — built
- Native captures, both themes. Requested 1280×800 rendered 2560×1600; 1440×900 rendered 2880×1676; 1920×1080 rendered 3840×1676. Height stays clamped at 838 logical pixels, not the requested 900/1080. Files: `evidence/transfers-{light,dark}-{1280x800,1440x900,1920x1080}.png`.
- Inspected at 1280, both themes: Source, Target, Tables, Format, Conflict, and Run then cancel stay inside the sidebar. Inspected at 1440 dark and 1920 light/dark: those fields plus Suggest cols, Preview sample, Demo masked CSV export, Run synthetic harness, and CSV export harness also stay inside. JSONL and later actions sit below the clamped viewport and scroll.
- Not inspected at 1440/1920: the synthetic seed card. `capture_focus_once` scrolls the sidebar to masking, so that card is above the viewport. Do not treat those captures as a pass for synthetic inputs/actions.

Not claimed: shared quiet-frame sweep, empty/loading/error states, workspace `cargo test`/`clippy`, independent review. Transfers width clips are inspected at the three requested widths; height remains clamped at 838 logical pixels.

## 2026-10-19 recheck gates

- `cargo test -p db-pro-app welcome_cards_fit_both_sides_of_the_grid_breakpoint` — passed
- `bash scripts/check-ui-architecture.sh` — PASS
- Welcome, Schema Object, Security, and Settings General captures at requested 1280×800 (rendered 2560×1600) inspected. No new layout change. See FINDINGS.md recheck.

## 2026-10-19 monitor header overlap

`evidence/monitoring-dark-1280x800.png` before the wrap showed `Connected` painted under `Auto-refresh`. After reserving the checkbox width and dropping the right-to-left row, both themes at requested 1280×800 (rendered 2560×1600) show `Disconnected` and `Auto-refresh` on separate lines with intact labels: `evidence/monitoring-dark-1280x800.png`, `evidence/monitoring-light-1280x800.png`. The fixture was disconnected, so the Refresh button was not on screen. `connected_status_does_not_share_a_row` passed. `cargo build --release --locked -p db-pro-native` passed again after this header edit.

Historical limitation at the time of the 2026-10-19 capture: status bar reported `database "fullstack_starter" does not exist`. The local loopback PostgreSQL fixture was provisioned in the 2026-10-20 review pass below; the 2026-10-19 capture itself remains SQLite-only.

## 2026-10-20 owner-requested layout and PostgreSQL demo pass

Source SHA: `cb6e00d633772de538498384a8b14d32d25f7915`; source files unchanged. The capture binary was built from the current worktree with `capture`; the only worktree changes are plan/evidence files.

### PostgreSQL fixture — PASS

- Started `db-pro-ui-demo-pg` from the locally available `postgres:16` image; host binding is `127.0.0.1:5432`, restart policy `unless-stopped`, persistent volume `db-pro-ui-demo-pg-data`. Authentication is `trust` and loopback-only; development fixture, never expose this container beyond localhost.
- Mounted `fixtures/postgres/001_schema.sql`, `002_seed.sql`, `fixtures/lab/001_schema.sql`, `002_load.sh`, `003_expand.sql`, and `seed-map.json` read-only into PostgreSQL initialization.
- `PGHOST=127.0.0.1 PGPORT=5432 PGUSER=postgres PGDATABASE=fullstack_starter psql -v ON_ERROR_STOP=1 -Atc "SELECT table_schema || ':' || count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema') AND table_type='BASE TABLE' GROUP BY table_schema ORDER BY table_schema; SELECT 'views=' || count(*) FROM information_schema.views WHERE table_schema NOT IN ('pg_catalog','information_schema'); SELECT 'rows=' || ((SELECT count(*) FROM lab.customers)+(SELECT count(*) FROM lab.products)+(SELECT count(*) FROM lab.orders)+(SELECT count(*) FROM lab.order_items)+(SELECT count(*) FROM lab.events));"` — PASS, output `lab:7`, `public:11`, `views=2`, `rows=81200`.
- `PGHOST=127.0.0.1 PGPORT=5432 PGUSER=postgres PGDATABASE=fullstack_starter psql -v ON_ERROR_STOP=1 -Atc "SELECT 'base_tables=' || count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema') AND table_type='BASE TABLE'; SELECT 'lab_customers=' || count(*) FROM lab.customers; SELECT 'lab_products=' || count(*) FROM lab.products; SELECT 'lab_orders=' || count(*) FROM lab.orders; SELECT 'lab_order_items=' || count(*) FROM lab.order_items; SELECT 'lab_events=' || count(*) FROM lab.events;"` — PASS, output `base_tables=18`, `lab_customers=8000`, `lab_products=1200`, `lab_orders=20000`, `lab_order_items=40000`, `lab_events=12000`.
- `pg_isready -h 127.0.0.1 -p 5432` — accepting connections.
- Connection profile `Xe Lạc Hồng (PostgreSQL)` is stored in the normal DB Pro data directory and points to `localhost:5432/fullstack_starter` as `postgres`. It pre-existed this pass. The isolated demo data directory `/home/vietis/.local/share/db-pro-ui-demo` now also has a seeded copy; no second profile was added to the user's normal profile list.

### Layout evidence — PARTIAL

- `DB_PRO_DATA_DIR=/home/vietis/.local/share/db-pro-ui-demo DB_PRO_CAPTURE_TO=docs/plans/active/native-feature-ui-cleanup/evidence/postgres-demo/welcome-light-1280x800.png DB_PRO_WINDOW_SIZE=1280x800 RUST_LOG=info cargo run --locked --features capture -p db-pro-native` — PASS, exit 0; native app wrote a 1280×800 framebuffer. Build emitted the existing warning `unused import: db_pro_core::ports::DbConnector` in `crates/infrastructure/src/sqlserver/introspect.rs`.
- Inspected `evidence/postgres-demo/welcome-light-1280x800.png`: Welcome/shell groups and cards fit. The active connection is `Sample E-Commerce (SQLite)`; the PostgreSQL profile appears collapsed. This capture is not PG-connected UI evidence.
- Inspected existing `evidence/query-agent-dark-1280x800.png` and prior findings/recheck. The Query/Agent composition is dense but the captured controls do not overlap or clip; the old `fullstack_starter does not exist` status is stale after fixture creation.
- A regular app run in the isolated state also selected the SQLite sample. The capture driver does not select a saved connection or open a real table. X11 window discovery found no app window; `gnome-screenshot -w -f /tmp/db-pro-ui-demo-window.png` failed because this virtual display has no capturable app window. The app service was stopped after inspection.
- One initial capture used the normal data directory. App startup restored an existing saved `postgres` profile at `localhost:15433` and began schema introspection automatically. No SQL query or mutation was issued; that capture was discarded. Subsequent runs used the isolated data directory.

### Not run / not verified

- No tests, UI architecture scan, release build, 1440×900 or 1920×1080 captures, or PostgreSQL-selected table/grid/query screenshot were run in this pass.
- Native layout for populated PostgreSQL Table/Data, Query results, Schema browser, hover/focus, and resizing is **NOT VERIFIED**. Existing captures and DB counts are not substitutes for that UI evidence.
- No app code, tests, theme tokens, or provider behavior changed. Feature remains `IMPLEMENTING`; do not close the layout review.
