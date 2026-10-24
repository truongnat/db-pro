---
schema_version: 1
id: "MEM-PLAN-welcome-query-editor-polish-1ed1059cb092"
title: "Plan archive: Welcome Query Editor Polish"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, welcome-query-editor-polish]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "1ed1059cb092a7a8ef81f43cbd7c69fefbd6a62769b9223611874609efead095"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/welcome-query-editor-polish/PLAN.md"
  - "docs/plans/active/welcome-query-editor-polish/FINDINGS.md"
  - "docs/plans/active/welcome-query-editor-polish/VERIFICATION.md"
  - "docs/plans/active/welcome-query-editor-polish/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/welcome-query-editor-polish/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Welcome and query editor polish

State: RUNTIME_VERIFY
Branch: `cursor/welcome-query-editor-polish-e577`

## Goal

The first workspace tab (Welcome) and the query editor read as a finished database IDE: a composed start page, and query actions that sit on the editor instead of a 24px status row.

## Evidence / baseline

- Baseline: `d52e752fd1f7becdd5abbcdf6a12662959c08a72`
- Welcome content started 32px from the top of a large empty canvas. Start actions had no surface; Connections sat in a card.
- With an empty catalog the Explorer still painted "Filter objects…" above "No connections".
- Query Run lived only in the status bar. Explain and Format were inside the overflow menu. The current-line wash and inactive line numbers were barely visible on the dark buffer.

## Scope

- Welcome vertical inset and matched Start / Connections cards
- Hide the Explorer filter when the catalog is empty
- Query toolbar: Run, Explain, Format, with the connection chip
- Stronger current-line fill and readable gutter numbers

## Non-goals

- SQL execution, completion, or provider behavior
- Activity-rail icon set
- Pixel-perfect Zed clone

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | unchanged | UI chrome is provider-neutral |
| SQLite | unchanged | UI chrome is provider-neutral |

## Acceptance criteria

- [x] Welcome start block sits in the upper portion, with Start and Connections as paired cards
- [x] Empty Explorer does not show the object filter
- [x] Run, Explain, and Format are on the query toolbar
- [x] Status bar keeps cursor, driver, schema, and transaction metadata
- [x] Capture evidence at 1280×800, 1440×900, and 1920×1080


## Source: `FINDINGS.md`

# Findings

Baseline: `d52e752fd1f7becdd5abbcdf6a12662959c08a72`.

## P2 — Welcome is a short block on an empty canvas

`welcome_surface_view` inserted a fixed 32px top gap and left Start as bare rows beside a Connections card. On a 1440×900 capture the rest of the central panel was empty surface.

Decision: scale the top inset with spare height (clamped 24–96) and put Start in the same card frame as Connections.

## P2 — Empty Explorer still shows a filter

`ExplorerSurfaceContext::draw` always painted the object filter, including when `catalog` is empty. The first screen then showed "Filter objects…" above "No connections".

Decision: draw the toolbar only when the catalog has a connection. The existing border test uses one connection and still paints the field.

## P2 — Query Run is a status-bar control

`draw_status_bar` was the only Run/Stop control. Explain and Format lived in the overflow menu. A query editor without a toolbar reads as unfinished.

Decision: draw Run, Explain, and Format on the context strip. The status bar keeps Ln/Col, driver, schema, transaction, parameters, and diagnostics.

## P2 — Current line and gutter numbers are too faint

Dark current-line fill used alpha 10. Inactive line numbers were rgb(90,90,90) on the gutter.

Decision: raise the wash to alpha 22 (dark) / 16 (light) and inactive dark numbers to rgb(120,120,120). Editor top padding is 12px. Line-number size tracks the editor font.


## Source: `VERIFICATION.md`

# Verification

Baseline: `d52e752fd1f7becdd5abbcdf6a12662959c08a72`.

## Commands

```text
cargo check -p db-pro-ui --offline
  exit 0

cargo test -p db-pro-ui --offline --lib welcome_top_inset
  1 passed / 0 failed / 0 ignored

cargo test -p db-pro-ui --offline --lib the_filter_field_border
  1 passed / 0 failed / 0 ignored

cargo fmt -p db-pro-ui -- --check
  exit 0
```

Not run: `cargo clippy --workspace`, `cargo test --workspace`, `cargo build --release --locked -p db-pro-native` without the capture feature. The capture binary was built:

```text
cargo build --release --locked --offline -p db-pro-native --features capture
  Finished release in 48.90s, exit 0
```

## UI captures

Empty catalog, keyring disabled. Framebuffer is slightly inside the requested window (window chrome).

| Surface | Requested | File |
|---|---|---|
| Welcome | 1280×800 | `/opt/cursor/artifacts/welcome_1280x800.png` (1267×792) |
| Welcome | 1440×900 | `/opt/cursor/artifacts/welcome_1440x900.png` (1425×891) |
| Welcome | 1920×1080 | `/opt/cursor/artifacts/welcome_1920x1080.png` (1900×1069) |
| Query | 1280×800 | `/opt/cursor/artifacts/query_1280x800.png` (1267×792) |
| Query | 1440×900 | `/opt/cursor/artifacts/query_1440x900.png` (1425×891) |
| Query | 1920×1080 | `/opt/cursor/artifacts/query_1920x1080.png` (1900×1069) |

Accent pixels for the Run control on the 1440 query capture sit on rows 72–95, in the toolbar under the tab strip, not on the status bar.

## Provider matrix

| Provider | Automated | Live | Notes |
|---|---|---|---|
| PostgreSQL | N/A | N/A | No SQL or connection behavior changed |
| SQLite | N/A | N/A | No SQL or connection behavior changed |

## Lab database

`docker-compose.lab.yml` on host port 55433, database `dbpro_lab`, user `dbpro`. Seed mapping: `fixtures/lab/seed-map.json`, expanded by `003_expand.sql`.

```text
sudo docker compose -f docker-compose.lab.yml up -d
PGPASSWORD=dbpro_test psql -h 127.0.0.1 -p 55433 -U dbpro -d dbpro_lab
```

Measured 2026-09-24 after init (`INSERT 0 1` then `DO` in the container log):

| Table | Rows |
|---|---|
| lab.products | 1200 |
| lab.customers | 8000 |
| lab.orders | 20000 |
| lab.order_items | 40000 |
| lab.events | 12000 |

`SELECT status, count(*) FROM lab.orders GROUP BY status` returned six statuses, 3333–3334 each. The Welcome starter joining customers to orders returned rows (top spend 6309.90).

Welcome capture after the composer redesign: `/opt/cursor/artifacts/welcome_lab_1440.png`.

## Remaining

- P0: 0
- P1: 0
- P2: workspace clippy and the full test suite were not run in this pass
- Independent review is still required; this plan stays under `docs/plans/active/`

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/02-query-editor-workspace.md` verdict: **STALE-DOC** —
the toolbar/filter claims hold at HEAD (Run/Explain/Format on the editor toolbar
`query_context_view.rs:94,101`; explorer object filter gated
`explorer_surface_view.rs:53,83`), but the Welcome surface has since been redesigned
(`3ea9e249`..`c6a033da`): no separate "Connections card", and the cited test
`welcome_top_inset_*` no longer exists. The `welcome` captures re-collected in this
audit (`_audit-2026-10-13/captures/welcome{,-light}_*.png`, 3 sizes) document the
current surface, not the composer this plan verified. Re-run of the workspace suite
+ clippy remains outstanding from the original pass. State stays RUNTIME_VERIFY.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — welcome and query editor polish

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | cloud agent |
| Issue(s) | n/a |
| Task state | `Review` |
| Baseline SHA | `d52e752fd1f7becdd5abbcdf6a12662959c08a72` |
| Branch / PR | `cursor/welcome-query-editor-polish-e577` · PR draft |
| Scope interpretation | Compose the Welcome tab and put Run / Explain / Format on the query toolbar. |
| Out of scope | SQL execution, completion, activity-rail icons, provider behavior |

## 2. Progress checkpoint

- Current HEAD: `06c6df6f4bced9032c446c9a038911005750226c`
- Completed acceptance rows: Welcome inset and cards, empty Explorer filter, query toolbar, gutter contrast, three-size captures
- Remaining acceptance rows: independent review; full workspace clippy/test suite
- Findings / risks: P2 only, see FINDINGS.md. No P0/P1
- Tests already run: `cargo test -p db-pro-ui --offline --lib welcome_top_inset` → 1 passed / 0 failed; `the_filter_field_border` → 1 passed / 0 failed
- Blockers: none

## 3. Handoff

- Branch: `cursor/welcome-query-editor-polish-e577`
- Plan: `docs/plans/active/welcome-query-editor-polish/`
- What changed: Welcome composition, empty Explorer toolbar, query chrome actions, editor line treatment
- How to verify: capture feature with `DB_PRO_CAPTURE_QUERY=1` and the default Welcome surface
- Known limits: full workspace gates not run

## 4. Review

n/a — implementer self-review only. Verdict is not an independent ACCEPT.

## 5. Task state

`Review`

## 6. Summary

Implementation is on this branch. UI captures exist for Welcome and Query at the three gate sizes. Workspace clippy and the full test suite were not run.

### Tổng kết bằng tiếng Việt

Tab Welcome được đẩy lên phần trên và hai cột Start / Connections dùng cùng một thẻ. Query editor có Run, Explain và Format trên thanh công cụ; thanh trạng thái chỉ còn thông tin vị trí và kết nối. Chưa chạy clippy toàn workspace.

