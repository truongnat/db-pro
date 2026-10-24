---
schema_version: 1
id: "MEM-PLAN-gallery-layout-collisions-d98c122ddfb3"
title: "Plan archive: Gallery Layout Collisions"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, gallery-layout-collisions]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d98c122ddfb32bdc70f92e4d26ac575c77fb893340143c6be6c0edf07b482f1d"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/gallery-layout-collisions/PLAN.md"
  - "docs/plans/active/gallery-layout-collisions/FINDINGS.md"
  - "docs/plans/active/gallery-layout-collisions/VERIFICATION.md"
  - "docs/plans/active/gallery-layout-collisions/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/gallery-layout-collisions/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Gallery & component layout collisions

Baseline SHA: `78d914887642dd400c7e2048db9a24d0403619c8` (main, working tree).

## Problem

Native capture reproduction (1280×800, `db-pro-native --capture` harness)
confirmed four layout collisions on the Component Gallery surfaces, matching the
owner-reported screenshots:

1. **Explain plan stat overlap** — `ExplainPlanTree::render_node`
   (`crates/ui/src/components/explain/ui.rs`) anchors the per-node stat text with
   `Layout::right_to_left` filling the row's remaining width. When the remaining
   width is smaller than the stat text, egui places the label so its left edge
   slides under the icon/title/badges. Used in production via
   `crates/ui/src/query_output_actions_view.rs`.
2. **Connection card column overflow** — `ConnectionCard::show`
   (`crates/ui/src/components/database/ui.rs`) reserves `SPACE_XS` before the SSL
   badge, but egui additionally inserts `item_spacing.x` (8pt) between
   `add_sized` and `allocate_exact_size`. The row claims 8pt more than the inner
   width, so the card's `min_rect` extends into the `ui.columns(2)` gutter and
   under column 2's content. Measured: column `562..899`, card `562..907`.
3. **StatusBar left/right collision** — `StatusBar::show`
   (`crates/ui/src/components/workspace/ui.rs`) paints left items forward from
   the left edge and right items backward from the right edge with no bound, so
   `READ COMMITTED` (left) overlaps `Ln 42, Col 18` (right) at gallery width.
4. **Light-theme glyph samples invisible** — `draw_rendering_text_samples`
   (`crates/ui/src/component_gallery_rendering.rs`) hardcodes
   `Color32::from_rgba_unmultiplied(224, 228, 235, alpha)` — a light gray chosen
   for dark surfaces — making every alpha row nearly invisible in light theme.

## Severity

P2 — non-blocking UX/polish, no data or correctness impact. The explain-plan
overlap also affects the production query output surface.

## Scope

Presentation-only fixes in `crates/ui`; no runtime/provider behavior changes.
The production `QueryStatusBar` (`query_status_bar_surface_view.rs`) lays out
right-then-left inside a bounded scope and was not observed colliding — out of
scope.

## Acceptance

- `render_node` stat region is allocated at least as wide as its text; narrow
  rows widen the ScrollArea content instead of overlapping.
- `ConnectionCard` response width ≤ its column width (verify at 1280×800).
- `StatusBar` never paints left items across the right block; clipped items are
  indicated with "…" and remain discoverable via hover.
- Text-rasterization samples derive their base color from `theme.text_primary`.
- Runtime captures: gallery `database-shell` + `devtools` + `rendering`
  sections at 1280×800, dark + light.


## Source: `FINDINGS.md`

# Findings

Baseline SHA: `78d914887642dd400c7e2048db9a24d0403619c8`.
All runtime evidence collected with `db-pro-native` built `--features capture`,
`DB_PRO_WINDOW_SIZE=1280x800` (2× → 2560×1600 physical), dark + light.

## Confirmed defects and root causes

### F1 — Explain plan stat text slides under badges (P2, user-reported)

`crates/ui/src/components/explain/ui.rs` — `render_node` anchored the per-node
cost/time/rows string via `ui.with_layout(Layout::right_to_left)` filling the
row's remaining width. When remaining width < measured text width, egui places
the label so its left edge underlaps the icon/title/bar/badges → the overlap on
`Seq Scan on users` in the owner's screenshot.

Fix: measure `stat_str` with `FontId::monospace(FONT_SIZE_CAPTION)` and allocate
`available_width().max(stat_width)` via `allocate_ui_with_layout`. When the stat
doesn't fit, the row widens inside the existing `ScrollArea::both()` instead of
overlapping; the label lands flush after the last badge. Header labels
(`Total Exec`/`Planning`) got `Label::truncate()` — same hazard, bounded region.

### F2 — ConnectionCard claims column gutter + 8pt of the next column (P2, user-reported)

`crates/ui/src/components/database/ui.rs` — the host row reserved
`SPACE_XS` before the SSL badge, but `add_sized` + `allocate_exact_size` make
egui also insert `item_spacing.x` (8pt) between them. Reservation counted the
gap once; reality charged it twice → the row's `min_rect` grew 8pt past the
column edge. Measured: column `562..899`, card response `562..907`.

Fix: `ssl_gap = ui.spacing().item_spacing.x + SPACE_XS` when SSL is on.

### F3 — StatusBar left/right item collision (P2, found during capture)

`crates/ui/src/components/workspace/ui.rs` — left items painted forward from
`rect.left()+SPACE_MD`, right items backward from `rect.right()-SPACE_MD`, with
no bound between them → `UTF-8 · READ COMMITTED` overlapped `Ln 42, Col 18`.

Fix: measure the right block first (`right_status_block_width`), stop left items
at `left_status_items_limit` (block left edge − `SPACE_MD`), and collapse the
rest into a `…` indicator whose hover lists the hidden items. Right items
additionally stop before crossing the inner left edge.

### F4 — Glyph alpha samples invisible on light theme (P2, user-reported)

`crates/ui/src/component_gallery_rendering.rs` —
`Color32::from_rgba_unmultiplied(224, 228, 235, alpha)` is a light gray tuned
for dark surfaces; on light surfaces all alpha rows were near-invisible.

Fix: base color = `theme.text_primary`, alpha still modulated — the diagnostic
intent (alpha on identical glyphs) is preserved for both themes.

### F5 — Duplicate egui widget Ids inside `ui.columns` (P2, found during capture)

Debug builds paint "First/Second use of widget/ScrollArea ID" callouts.
`Ui::columns` gives every column child `stable_id = parent.with("child")`
(egui 0.29 `Ui::new_child` default), so unsalted `ScrollArea`s in same-shaped
column content share scroll state — captured pair: explain card `ScrollArea`
vs `TerminalBlock` scroll (`FA5A`), plus the scroll-area drag interact
(`481B`).

Fix: salted each component scroll area —
`explain` uses the existing unique `tree_id`; `DiffViewer` and `TerminalBlock`
use `ui.auto_id_with("…_scroll")`. Verified overlays gone in recapture.

The same latent hazard exists anywhere `ui.id().with(name)` appears inside
`columns()`; systemic audit is out of scope (recorded as follow-up).

## Testability notes

- Pure helpers added/tested: `right_status_block_width`, `left_status_items_limit`.
- Visual acceptance: see `VERIFICATION.md` + `evidence/`.


## Source: `VERIFICATION.md`

# Verification

Baseline: `78d914887642dd400c7e2048db9a24d0403619c8` + working-tree fix.

## Automated gates (executed)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --offline` | PASS |
| `cargo clippy -p db-pro-ui --all-targets --offline -- -D warnings` | PASS |
| `cargo test -p db-pro-ui --offline` | 949 passed / 0 failed / 0 ignored |
| `cargo build -p db-pro-native --features capture` (debug) | PASS |
| `cargo build -p db-pro-native` (release, via perf scan) | PASS, 32.1MB |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | 14 pass / 2 warn / 0 fail — warnings: `StatusBar::show` 51 lines (>50 heuristic) and `paint_status_overflow` 4 params, matching sibling `paint_status_item` signature style |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | PASS (partial) — 4 executed checks pass; benchmark/db/er sections not executed (need window server / live DB) |

Not run: `cargo test --workspace` in full (focused `db-pro-ui` run only); no
runtime/provider behavior changed, all touched code is presentation layer.

## Runtime evidence (native captures)

`DB_PRO_CAPTURE_COMPONENT_GALLERY=1`, `DB_PRO_CAPTURE_GALLERY_SECTION`,
`DB_PRO_WINDOW_SIZE`, `DB_PRO_CAPTURE_GALLERY_SCROLL` (new harness env), files in
`evidence/`:

| File | Surface | Result |
|---|---|---|
| `database-shell-dark-1280x800.png` | card columns + status bar | card inside col, no bleed into "Connection Health"; status bar shows `…` for clipped left items, no overlap |
| `database-shell-light-1280x800.png` | same, light | same; readable on light surface |
| `database-shell-dark-1440x838-clamped.png` | wider viewport | all left items incl. `UTF-8 · READ COMMITTED` fit; clear gap to `Ln 42, Col 18` |
| `database-shell-dark-1920x838-clamped.png` | widest | same (display clamps height to 838) |
| `devtools-explain-dark-1280x800.png` | explain card @scroll 900 | `Hash Join`/`Seq Scan` stats right-aligned or clipped at scroll viewport — no overlap; no ID-collision callouts |
| `devtools-explain-light-1280x800.png` | same, light | same |
| `rendering-text-light-1280x800.png` | text rasterization grid | all `Query 0x7F3A` alpha rows readable on light bg |
| `rendering-dark-1280x800.png` | rendering section, dark | unchanged behavior on dark |

## Harness additions (capture-only)

- `DB_PRO_CAPTURE_GALLERY_SECTION=rendering` arm in `workspace_actions.rs`.
- `DB_PRO_CAPTURE_GALLERY_SCROLL=<f32>` → `vertical_scroll_offset` on the gallery
  detail `ScrollArea` (`component_gallery_view.rs`). Both inert in normal runs.

## Residuals

- P2 follow-up: audit other `ui.id().with(…)` / unsalted `ScrollArea` sites inside
  `ui.columns` children repo-wide (egui quirk documented in FINDINGS §F5).
- Stat text on very narrow plan rows is clipped by the scroll viewport rather
  than overlapping — expected; horizontal scrollbar exposes it.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — gallery layout collisions fix

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Devin (interactive CLI session) |
| Issue(s) | user report "UI hỏng" + 3 screenshots (no issue #) |
| Task state | In Progress → Review (handoff) |
| Baseline SHA | `78d914887642dd400c7e2048db9a24d0403619c8` — all assertions vs this tree + the fix diff below |
| Branch / PR | `main` (owner workflow override; no worktree, no PR created) |
| Scope interpretation | Fix four confirmed egui layout/visibility collisions on Component Gallery + production-shared components (`ExplainPlanTree` also used by `query_output_actions_view.rs`) |
| Out of scope | systemic `ui.columns`/`id().with` audit beyond the captured collisions; production `QueryStatusBar` layout (separate implementation, no observed collision); archived frontend |

## 2. Progress checkpoint

- Current HEAD: `78d914887642dd400c7e2048db9a24d0403619c8` (fix committed as new commit on top)
- Acceptance rows: all done — see `CHECKLIST.md`
- Findings: P2 only — `FINDINGS.md` §F1–F5
- Tests run: `cargo test -p db-pro-ui --offline` → **949 passed / 0 failed / 0 ignored** (exit 0)

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | top commit on `main` — `fix(ui): bound layout collisions across gallery surfaces` (baseline `78d91488`; a commit cannot embed its own SHA) |
| Commit list | 1 commit — `fix(ui): bound layout collisions across gallery surfaces` |
| File / surface inventory | `components/explain/ui.rs` (stat region ≥ text width, header labels truncate, ScrollArea `id_salt(tree_id)`) · `components/database/ui.rs` (SSL gap = `item_spacing.x + SPACE_XS`) · `components/workspace/ui.rs` + `handler.rs` (right-block measurement, left bound, `…` overflow, tests) · `components/diff/ui.rs`, `components/dev_tools/ui.rs` (scroll `id_salt`s) · `component_gallery_rendering.rs` (theme-based alpha glyphs) · `component_gallery_view.rs`, `workspace_actions.rs` (capture-only `SCROLL`/`rendering` envs) |
| Acceptance mapping | F1→explain ui.rs; F2→database ui.rs; F3→workspace ui.rs/handler.rs + tests; F4→rendering; F5→id_salt trio |
| Commands and counts | fmt PASS · `cargo check --workspace --offline` PASS · clippy db-pro-ui `-D warnings` PASS · tests 949/0/0 · clean-code 14/2 warn/0 fail · perf scan PASS partial (4/4 executed) |
| CI run IDs / status | not run — local-only branch, no PR |
| Known limitations | `render_node`/`ConnectionCard::show` exceed the 50-line heuristic (pre-existing ratchet warnings); captures above 1280×800 logical clamp height to 838 (display limit) |
| Migrations / config implications | none — two new capture env vars are opt-in |
| Out-of-scope changes | none |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `n/a` — implementer handoff; external review not performed |
| Verdict | n/a |
| P0 / P1 / P2 counts | 0 / 0 / 0 introduced; 4 P2 fixed + 1 P2 defect discovered-and-fixed |
| Findings | n/a |
| CI disposition | not run |
| Next task(s) unblocked | none |

## 5. Research / audit handoff

- Source date: 2026-10-02 session
- References: `crates/ui/src/components/{explain,database,workspace,diff,dev_tools}/`, `component_gallery_{surfaces,rendering,view}.rs`, egui 0.29.1 `ui.rs` (`columns_dyn`, `new_child`) + `scroll_area.rs:526` (`Id::new("scroll_area")`)
- Factual findings + inference separation: FINDINGS.md
- Unresolved questions: full inventory of `id().with()` collisions across other gallery sections (P2 follow-up)
- Downstream tasks activated: none

## 6. Tổng kết (Vietnamese summary)

Đã sửa 4 lỗi UI xác nhận bằng capture thực tế: text chồng nhau trong Execution Plan (vùng stat được cấp tối thiểu bằng độ rộng text thay vì tràn sang trái), card kết nối tràn 8pt sang cột kế (do `item_spacing.x` bị tính hai lần), StatusBar trái/phải đè nhau (đo block phải trước, bên trái dừng sớm với dấu `…`), và hàng mẫu glyph vô hình ở light theme (màu lấy từ `theme.text_primary`). Phát hiện và sửa thêm va chạm widget Id của egui giữa các `ScrollArea` không `id_salt` trong `ui.columns`. Gates: fmt/check/clippy/test (949/0) pass; clean-code 0 fail; perf scan PASS. Bằng chứng capture: `docs/plans/active/gallery-layout-collisions/evidence/`.

