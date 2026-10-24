---
schema_version: 1
id: "MEM-SRC-2c63565b61fb4a2d"
title: "Source mirror: docs/plans/active/native-feature-ui-cleanup/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/native-feature-ui-cleanup/FINDINGS.md"
source_sha256: "2c63565b61fb4a2dc519a81de8b083f304360cdb979f227a00ae1e5ca6e0e384"
mirror_body_sha256: "2c63565b61fb4a2dc519a81de8b083f304360cdb979f227a00ae1e5ca6e0e384"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
