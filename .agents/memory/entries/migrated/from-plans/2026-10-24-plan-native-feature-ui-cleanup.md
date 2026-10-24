---
schema_version: 1
id: "MEM-PLAN-native-feature-ui-cleanup-f4210b9d287c"
title: "Plan archive: Native Feature Ui Cleanup"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, native-feature-ui-cleanup]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "f4210b9d287ce192f302b11f4d7ed685c1c16f0971242622a5f07d2743008b16"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/native-feature-ui-cleanup/PLAN.md"
  - "docs/plans/active/native-feature-ui-cleanup/FINDINGS.md"
  - "docs/plans/active/native-feature-ui-cleanup/VERIFICATION.md"
  - "docs/plans/active/native-feature-ui-cleanup/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/native-feature-ui-cleanup/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Native Feature UI Cleanup

State: IMPLEMENTING. Work directly on main under the owner override.
Baseline HEAD: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; existing uncommitted Quick Open/theme/shell work is preserved.

## Goal and scope

Owner requests research and implementation to make native feature UI clean, attractive and compact. Audit active native feature families, then fix concrete presentation defects and shared sources of repeated chrome. Preserve every action and data-safety boundary.

- Shared panel/toolbar surfaces: quiet fill-based grouping, no nested decorative boxes; preserve input, focus, validation and data-grid boundaries.
- Welcome/Query/Agent/Settings: compact, readable hierarchy with no content hidden underneath side panels.
- Table/Grid/Schema/DDL/ER/Tools/Monitoring/Transfers/Security: audit shared frame/spacing and actual overflow; implement evidence-backed fixes, not speculative restyling.
- Reuse current DbProTheme, tokens, components, fonts and capture engine. No framework/dependency/palette changes.

## Design contract

Database developers/analysts, keyboard/mouse desktop use, native Rust egui 0.36.2. Prioritize data over chrome. Clear primary action, neutral secondary actions, visible focus/hover, stable layout, 4/8 spacing rhythm and independent Light/Dark review. Keep dense desktop controls; do not apply phone-sized targets or 16px mobile body requirements indiscriminately.

ui-ux-pro-max research: explicit UX search confirms visible keyboard focus. Initial design-system result was an unrelated documentation landing/OLED-only recommendation, rejected. Narrow style search returned Minimalism & Swiss Style, applicable for restrained enterprise/tool hierarchy and both themes. Existing project tokens remain authoritative; no suggested raw palette/font changes are adopted.

## Non-goals

No domain/provider/SQL/runtime changes, no action deletion, no archived frontend, no wholesale view rewrites, no theme palette replacement. Existing sample SQLite and Clippy failures remain outside this presentation task.

## Acceptance

- Audit records concrete findings, owners, severity and provider impact (PostgreSQL n/a; SQLite n/a).
- Shared surfaces are consistent across their native feature callers.
- Observed clipping/overflow fixed at its owner; actions remain accessible.
- Focus, hover, selected, disabled and validation states retain semantic feedback.
- Run affected package tests, final fmt/check/architecture/release gates; report existing workspace failures without suppression.
- Capture and inspect affected Light/Dark surfaces at requested 1280×800, 1440×900 and 1920×1080; record actual clamped sizes and fixture limits.
- Lifecycle remains RUNTIME_VERIFY while exact-height/native-pointer/independent-review evidence is incomplete.


## Source: `FINDINGS.md`

# Findings

Baseline SHA: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` plus existing uncommitted patches.

## Verified visual evidence

- Agent-open 1280×800 native capture shows Welcome's second action-card column clipped by the Agent boundary. Evidence: ../quick-open-action-visuals/evidence/2026-10-19/border-followup/agent-light-1280x800.png and dark counterpart.
- Agent context and action bands still render nested outlined groups although the outer panel is now borderless. Their shared `toolbar_frame` draws a full rectangle stroke; investigate all native consumers rather than restyling only Agent.
- Shared `panel_frame`/`toolbar_frame` use subtle rectangular strokes across grid, table, schema, Agent and Gallery surfaces. A stroke is decoration unless it communicates an input/grid/validation/focus boundary; use fill/spacing for passive grouping.

## Skill research

`python3 /Users/truongdq/.agents/skills/ui-ux-pro-max/scripts/search.py "keyboard focus visible desktop" --domain ux`: Focus States, Platform All. Keep visible focus rings.

`... "database developer tool compact minimal" --design-system -p "DB Pro"`: rejected as off-topic (FAQ/documentation landing and OLED-only advice).

Retry `... "minimalism enterprise dashboard" --domain style`: Minimalism & Swiss Style, enterprise/professional tools, Light/Dark supported. Adopt restrained hierarchy, not its raw colors/CSS/mobile defaults.

## Provider impact

- No production provider code or behavior changed. A loopback-only PostgreSQL fixture is used as review data; no SQLite runtime comparison or mutation was performed.

## Accepted research handoff

Read-only lanes `AuditWorkstation` and `AuditDataFeatures` inspected baseline
`95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` plus uncommitted changes. Their
handoffs do not claim tests, captures, or independent implementation approval.

| Presentation issue | Confidence | Implementation decision |
|---|---|---|
| Welcome requests 620pt at a two-column breakpoint of 572pt; narrow mode forces 280pt | Source invariant + captured cutoff | Cap to remaining workspace width; preserve four actions; scroll when height is short |
| Agent context hidden in horizontal metadata strip | Captured + source | Separate status/detail from wrapped bounded chips; preserve full names in accessible text/tooltip |
| Query controls have no narrow central-width strategy | Source-backed risk | Geometry verification; compact inline actions only when required, keep alternatives in More |
| Settings repeats global/section/GENERAL hierarchy | Source + earlier image | One selected-section heading; retain controls and move diagnostics to Advanced |
| Schema definition wraps a fully framed CodeBlock in another card | Source | Remove redundant card only; keep SQL viewer behavior |
| Repeated monitoring/PG settings records use 12pt card padding plus 8pt gaps | Measured source geometry | Compact operational rows only (8×4 inset), keep global Card/forms unchanged |
| Long Transfers/Security/ER/DDL action groups lack width-aware reflow | Source-backed risk | Verify realistic narrow geometry; wrap complete control groups without deleting actions |

## 2026-10-19 recheck

Welcome beside Agent, current uncommitted tree, requested 1280×800, rendered 2560×1600, both themes: `evidence/welcome-agent-light-1280x800.png`, `evidence/welcome-agent-dark-1280x800.png`. All four action cards are fully visible; the second column is not cut by the Agent panel. Agent context chips wrap inside the panel; the long connection name ellipsizes inside its chip. The older cutoff note is not reproduced on this build. No Welcome layout change in this pass.

Schema Object definition, requested 1280×800, rendered 2560×1600, dark: `evidence/schema-definition-dark-1280x800.png`. One framed SQL viewer, no outer card. The recorded nested-card defect is not in this call path. Security at the same size shows the connect-required empty state inside the sidebar; no role actions are present to reflow. Settings General shows one content heading, not a SETTINGS/General/GENERAL stack: `evidence/settings-general-dark-1280x800.png`.

Shared passive panel/toolbar/activity frames now use fill rather than rectangular
decoration. The redundant table-only `table_band_frame` override is removed and its
four callers use the shared `toolbar_frame`. Card grouping, data-grid boundaries,
input validation and keyboard focus remain distinct.

Manual sidebar and query-output splitter lines are also hidden at rest; hover/drag
feedback and hit targets remain. The shell's rendered-boundary regression and an
actual synthetic sidebar hover/drag scenario protect the difference between decoration
and interactive affordance.

Detected implementation stack is Rust egui/eframe, absent from this skill's 22 stack
datasets. No web/mobile stack output is misrepresented as egui-specific guidance.

## 2026-10-20 owner-requested layout review and PostgreSQL demo

Source baseline: `cb6e00d633772de538498384a8b14d32d25f7915` (source tree unchanged; current uncommitted changes are documentation/evidence only).

| Surface | Observed evidence | Finding |
|---|---|---|
| Welcome / shell, 1280×800 light | `evidence/postgres-demo/welcome-light-1280x800.png`, captured from the native egui app. The center's four action tiles form a balanced 2×2 group; visible shell/sidebar/status contents stay within the frame. The connection list shows the PostgreSQL demo row, but the active connection is the bundled SQLite sample. | No clipping or overlap confirmed on this surface. This is not proof of a PostgreSQL-connected view. |
| Query + Agent, 1280×800 dark | Existing `evidence/query-agent-dark-1280x800.png` (2560×1600 framebuffer for the requested logical size) shows the open-query sidebar, SQL editor and Agent pane sharing the width. Controls are visible; the center editor is comparatively narrow. | Density trade-off worth reassessing with a real PostgreSQL table/query open; no clipping or unusable control is established by this capture. The old bottom-bar error is historical: the demo database did not exist when the image was taken. |
| Welcome + Agent / Schema / Settings | Existing 2026-10-19 two-theme observations at lines 41–43 above show all four cards visible, Agent chips wrapping, and the specific schema/settings framing issues absent on those paths. | Keep those fixes; do not repeat or report their superseded defects as current. |
| PostgreSQL-backed table/editor view | No screenshot or interaction in this pass shows a PostgreSQL-selected table in the app. | NOT VERIFIED; no layout conclusion for the populated grid, query results, or schema browser. |

**Severity:** no new P0/P1/P2 layout defect is confirmed. The narrow Query editor is an observation, not a defect claim; do not change pane sizing until the data-view layout is inspected.

The requested review database now exists in a local-only PostgreSQL container: `db-pro-ui-demo-pg`, bound to `127.0.0.1:5432`, with persistent volume `db-pro-ui-demo-pg-data`. It uses the repo's `fixtures/postgres` schema/seed plus `fixtures/lab` expansion. Verification reports 18 base tables (11 `public`, 7 `lab`), 2 views, and 81,200 rows across `lab.customers`, `products`, `orders`, `order_items`, and `events`. The sample connection is `Xe Lạc Hồng (PostgreSQL)` → `localhost:5432/fullstack_starter`, user `postgres`; this profile already existed in the normal DB Pro data directory. A separate profile was created in `/home/vietis/.local/share/db-pro-ui-demo` so the demo can launch without the other saved profiles.

The isolated startup capture selected the default SQLite sample, not PostgreSQL. The app's native capture path rendered a Welcome screen but cannot interactively select a connection; direct X11 window discovery returned no app window and `gnome-screenshot` failed in this virtual display. Do not imply that populated PostgreSQL views have been visually audited yet.


## Source: `VERIFICATION.md`

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


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — native UI layout review and PostgreSQL demo

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · native UI layout audit |
| Issue(s) | none linked |
| Task state | Blocked — populated PostgreSQL-backed views were not captured interactively |
| Baseline SHA | `cb6e00d633772de538498384a8b14d32d25f7915` — source behavior assertions below refer to this snapshot |
| Branch / PR | `main` / no PR |
| Scope interpretation | Review available native-egui layout evidence, provision a local PostgreSQL demo dataset/profile, and document the visible layout and evidence limits. |
| Out of scope | UI source edits, provider behavior changes, database mutation through the app, and claiming a PG-backed UI review without visual evidence. |

## 2. Progress checkpoint

- Current source HEAD: `cb6e00d633772de538498384a8b14d32d25f7915`; working-tree changes for this pass are plan/evidence files only.
- Completed acceptance rows: [x] provision loopback PostgreSQL fixture and dedicated DB Pro demo profile; [x] inspect available Welcome and Query/Agent screenshots; [x] record observed layout and evidence limits.
- Remaining acceptance rows: [ ] capture/inspect the actual PostgreSQL-connected Table/Data, Query results, and Schema views; [ ] verify the specified Light/Dark states and 1440×900 / 1920×1080 sizes.
- Findings / risks: no confirmed P0/P1/P2 visual defect in the inspected captures. The Query/Agent composition is dense at 1280×800, but no overlap/clipping is visible. PG-backed views are **NOT VERIFIED** because this virtual desktop did not expose an interactable/capturable app window.
- Tests already run: no tests. Runtime capture command below exited 0; it displays the SQLite sample, not the PG fixture. PostgreSQL readiness/data-count commands passed as recorded below.
- Dependency / blocker changes: PostgreSQL fixture is now available on loopback at `127.0.0.1:5432`; interactive table-view inspection remains blocked by the virtual desktop capture/input surface.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `cb6e00d633772de538498384a8b14d32d25f7915` (source snapshot; plan/evidence files are uncommitted) |
| Commit list | none |
| File / surface inventory | `docs/plans/active/native-feature-ui-cleanup/CHECKLIST.md` — recorded fixture completion and outstanding PG-view visual verification; `FINDINGS.md` — added this pass’s layout observations and provider-scope note; `VERIFICATION.md` — recorded DB/capture results and limitations; `evidence/postgres-demo/welcome-light-1280x800.png` — actual native capture showing the Welcome shell with SQLite active; `AGENT_EVIDENCE.md` — this handoff. No application source changed. |
| Acceptance mapping | “Provide substantial PostgreSQL data” → `db-pro-ui-demo-pg` and `fullstack_starter` counts below; “inspect layout” → Welcome screenshot and existing Query/Agent screenshot cited in `FINDINGS.md`; “inspect live PostgreSQL table views” → not met, explicitly remains unchecked in `CHECKLIST.md`. |
| Commands and counts | `pg_isready -h 127.0.0.1 -p 5432` → accepting connections. Exact PostgreSQL `psql` table/view/row-count commands and outputs are in `VERIFICATION.md`. `DB_PRO_DATA_DIR=/home/vietis/.local/share/db-pro-ui-demo DB_PRO_CAPTURE_TO=docs/plans/active/native-feature-ui-cleanup/evidence/postgres-demo/welcome-light-1280x800.png DB_PRO_WINDOW_SIZE=1280x800 RUST_LOG=info cargo run --locked --features capture -p db-pro-native` → exit 0; produced 1280×800 capture. No tests, architecture scan, release build, or other requested resolutions ran in this pass. |
| CI run IDs / status | not run |
| Known limitations | Screenshot starts on `Sample E-Commerce (SQLite)`; no PG-backed table/data/query/schema view captured. Only Welcome and one existing Query/Agent screenshot were visually inspected. No fresh light/dark focus, hover, resize, empty/loading/error, 1440×900, or 1920×1080 evidence. |
| Migrations / config implications | Started persistent local container `db-pro-ui-demo-pg` (`postgres:16`, restart `unless-stopped`) with volume `db-pro-ui-demo-pg-data`, bound only to `127.0.0.1:5432`; `trust` authentication is acceptable only while loopback-bound. Database `fullstack_starter`, user `postgres`. Dedicated app state is `/home/vietis/.local/share/db-pro-ui-demo`; launch with `DB_PRO_DATA_DIR=/home/vietis/.local/share/db-pro-ui-demo cargo run --locked -p db-pro-native`, then select `Xe Lạc Hồng (PostgreSQL)`. The normal profile with that name pre-existed this pass. |
| Out-of-scope changes | no app code, tests, theme tokens, provider paths, or database data mutations through DB Pro. One initial capture against the normal app state restored an existing profile on `localhost:15433` and automatically began schema introspection; no user SQL or mutation was issued, and that capture was discarded. Subsequent app runs used the isolated demo state. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `cb6e00d633772de538498384a8b14d32d25f7915` |
| Verdict | `BLOCK` — PG-backed view evidence and remaining visual gates are outstanding |
| P0 / P1 / P2 counts | introduced by this pass: 0 / 0 / 0 confirmed; inherited: not independently re-audited in this pass |
| Findings | `docs/plans/active/native-feature-ui-cleanup/FINDINGS.md` records the evidence. The dense Query/Agent composition is an observation, not a confirmed defect. Missing PG-backed visual evidence blocks completion. |
| CI disposition | not run; no CI run id |
| Next task(s) unblocked | Capture the app with `Xe Lạc Hồng (PostgreSQL)` selected; inspect representative Table/Data, Query results and Schema layouts, then complete remaining width/theme/state gates and update the plan. |

## 5. Research / audit handoff

- Source date: 2026-10-20.
- Source URLs / references: source snapshot `cb6e00d633772de538498384a8b14d32d25f7915`; `docs/plans/active/native-feature-ui-cleanup/{PLAN.md,CHECKLIST.md,FINDINGS.md,VERIFICATION.md}`; `docs/plans/active/native-feature-ui-cleanup/evidence/postgres-demo/welcome-light-1280x800.png`; `docs/plans/active/native-feature-ui-cleanup/evidence/query-agent-dark-1280x800.png`; `fixtures/postgres/{001_schema.sql,002_seed.sql}`; `fixtures/lab/{001_schema.sql,002_load.sh,003_expand.sql,seed-map.json}`; `crates/infrastructure/src/meta/connection_repo.rs` (connection list is ordered by `created_at` at the baseline SHA); `better-layout`, `ui-design-review`, and project frontend/theme skills.
- Factual findings: PostgreSQL accepts connections on loopback port 5432; `fullstack_starter` has 18 base tables, 2 views, and 81,200 rows across `lab.customers` (8,000), `lab.products` (1,200), `lab.orders` (20,000), `lab.order_items` (40,000), and `lab.events` (12,000). The actual app capture uses SQLite; see `VERIFICATION.md` for exact commands and capture details.
- Inference: Query/Agent’s three-pane composition may constrain editor width at 1280×800; the screenshot does not establish an overlap, broken focus, or clipping defect. No change is recommended without PG-connected and interactive evidence.
- Decision / recommendation: keep the feature in `IMPLEMENTING`; use the prepared local PG data for a targeted interactive layout pass. Do not close the plan based on database readiness or the SQLite Welcome screenshot.
- Unresolved questions: whether the user-reported focus/tab disorder can be reproduced in the actual PG-backed UI; whether the narrow Query/Agent editor is a genuine usability defect at interactive sizes.
- Downstream tasks activated: none; follow-up remains in the active feature checklist.

## 6. Tổng kết (Vietnamese summary)

Đã chuẩn bị PostgreSQL demo chạy local tại `127.0.0.1:5432` với 18 bảng, 2 view và 81.200 dòng dữ liệu mẫu; đã ghi nhận layout từ các ảnh hiện có. Chưa kiểm tra được màn hình Table/Data, Query và Schema khi kết nối PostgreSQL vì môi trường desktop ảo không cho tương tác/chụp cửa sổ app; ảnh mới hiện vẫn là SQLite. Giữ feature ở trạng thái `IMPLEMENTING`, cần chạy app với data dir demo và hoàn tất kiểm chứng trực quan trước khi kết luận.

