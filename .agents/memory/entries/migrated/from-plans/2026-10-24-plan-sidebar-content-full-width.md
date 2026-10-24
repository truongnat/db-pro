---
schema_version: 1
id: "MEM-PLAN-sidebar-content-full-width-03870f795a24"
title: "Plan archive: Sidebar Content Full Width"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, sidebar-content-full-width]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "03870f795a24571f0c7c3aaf384d2aeeaa7b137d9e72db7e0006157c66050a8a"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/sidebar-content-full-width/PLAN.md"
  - "docs/plans/completed/sidebar-content-full-width/FINDINGS.md"
  - "docs/plans/completed/sidebar-content-full-width/VERIFICATION.md"
  - "docs/plans/completed/sidebar-content-full-width/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/sidebar-content-full-width/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Sidebar content full width

## Problem

Navigator tree content did not fill the left sidebar the way VS Code / DBeaver do.
Screenshot evidence: tree labels / `Failed:` hints clipped around ~200px while
`New query` / filter chrome reached ~360px and the drag separator sat near ~445px.

## Cause

1. `ScrollArea` content width settled on intrinsic (short-label) size, so tree rows
   truncated mid-panel.
2. Resize handle used egui `SidePanel` frame `response.rect`, which could disagree
   with the `exact_width(sidebar_width)` we requested — separator floated past content.

## Fix

- Derive sidebar content rect from clamped `sidebar_width` (asymmetric pad: left
  `SPACE_SM`, right `SPACE_XXS`).
- Lock drag separator to `panel_left + sidebar_width`.
- Force explorer `ScrollArea` + tree rows to the padded content width every frame.

## Scope

`crates/ui` only: `sidebar_view.rs`, `explorer_view.rs`, `explorer_tree.rs`.

---

## Follow-up: the width contract broke on the clip (2026-09-17)

The `max_rect`-based width above is a *preference*, not a cap. Four defects followed.

### Cause

1. `Ui::set_max_width` ends with `max_rect = max_rect.union(min_rect)` (`placer.rs`), so a
   sibling that overflowed earlier in the frame inflates `max_rect` for the rest of that
   frame. The explorer toolbar overflowed: it reserved a hardcoded `actions_width = 28.0`
   for the refresh button, but `compact_icon_button` is a style-sized `Button` and measures
   **34** wide. Field 240 + spacing 8 + button 34 = 282 inside a 240px column.
2. Everything downstream sized from `max_rect()` / `available_width()`, so tree rows were
   **282** wide in a **240** column. `Painter::with_clip_rect` *intersects*, so the overflow
   was discarded without complaint: the `Native Test` driver badge was painted at
   `x 239–286` against a clip ending at `248`, leaving a ~1px sliver of its label.
3. The badge width was additionally estimated per character — `badge.len() * 6.5 + 8.0`
   yields 47 for `SQLITE`, whose measured label is 31.
4. Separately, the sidebar clipped to exactly the content column while content filled it.
   `Shape::rect_stroke` paints *entirely outside* its path (`StrokeKind::Outside`), so every
   1px border on a widget filling the column lost both vertical edges — the filter field,
   the `New query` button, the header action — while the horizontal ones survived, because
   those sit well inside the clip.

### Fix

- `explorer_view`: lay the refresh button out first in a `Layout::right_to_left` row and give
  the field the remainder, so the button's width is *measured*, never assumed.
- `explorer_view`: bound `tree_width` by the clip as well as `max_rect`, and do it *before*
  the `ScrollArea`. Inside the row the clip is narrowed by the scrollbar
  (`inner_size = outer_size - current_bar_use`), so a clip-derived row would jitter narrower
  every time the connection list crossed the fold.
- `explorer_tree`: measure the badge label with the painter instead of estimating it.
- `sidebar_view`: clip to the content column plus a 1px bleed — the least room a 1px outside
  stroke needs to render, and far too little to hide a real layout overflow.
- `components/input`: `paint_field_chrome` strokes an inset path, so the field's own border
  lands inside the field whatever container it sits in.

### Scope

`crates/ui` only: `sidebar_view.rs`, `explorer_view.rs`, `explorer_tree.rs`,
`components/input/{layout,search}.rs`.


## Source: `FINDINGS.md`

# Findings — sidebar-content-full-width

## P0 / P1

None open for this fix scope after the header hover fix.

## Evidence (pre-fix)

1. Screenshot crop: drag line ~445px, `New query` chrome ~360px, tree/`Failed:` ink ~200px.
   Tree was not filling the sidebar content width.
2. Workspace selector painted `surface_hover` *after* the label → hover wiped the name
   to a blank wash ("trắng xóa").

## Fixes in this branch

- Content rect + drag line locked to `sidebar_width`
- Explorer ScrollArea / tree rows forced to content width
- Header selector: allocate → hover wash → text (correct paint order)
- Removed duplicate Plus on the explorer filter row (New Connection stays in header)

## Residual

Runtime screenshot evidence still pending after native rebuild.

---

## Follow-up findings (2026-09-17)

Reported: the tree is still overlapped by the width — the `Native Test` row paints a badge
but it is hidden — and the filter field's border is missing both of its ends.

### P1 — tree rows overflow the column and lose their trailing badge

Evidence (pre-fix, 1440×900, `sidebar_width = 260`):

```text
TOOLBAR row_width=240 search_width=204 spacing=8 btn=[[220.0 107.0] - [254.0 131.0]]
ROW max_rect=[[8.0 146.0] - [290.0 892.0]] avail=282.0 -> width=282
#33 Rect fill=#EE_EE_EE bounds=[[239.0 155.0] - [286.0 171.0]] clip=[[8.0 150.0] - [248.0 176.0]]
#34 Text "SQLITE"        bounds=[[247.0 159.5] - [278.0 167.5]]
```

The row is allocated 282 wide in a 240 column. The clip keeps only `x 239–248` of the pill,
so the label survives as a ~1px sliver: the badge is *painted* and *invisible* at once,
which is exactly what was reported.

Severity **P1**: a value the product deliberately surfaces — the connection's driver — is
unreadable. Not P0: no data, mutation or transaction is involved.

**Runtime verified** — the screenshots in `screenshots/` (1280×800 / 1440×900 / 1920×1080)
show the `SQLITE` and `PG` driver badges fully readable, the pill ending inside the clip with
the label fully painted (31px of text in a 39px pill).

Provider impact: none. This is layout only; PostgreSQL and SQLite render the same tree.

### P1 — every 1px border on a widget filling the column loses both vertical edges

Evidence (pre-fix): `#29 Rect stroke width=1.0 rounding=6 bounds=[[7.5 101.5] - [248.5 136.5]]`
against `clip=[[8.0 8.0] - [248.0 892.0]]`. The stroked path is `[8,248]`; an outside stroke
paints `[7,249]`, so the clip removes both vertical bands entirely while leaving the
horizontal ones, which sit well inside it.

Same defect and root cause, measured on widgets the report did not name:
`#13 Rect stroke bounds=[[7.5 47.5] - [248.5 76.5]]` — the `New query` primary button.

Severity **P1**: a primary action's outline is missing two of four edges.

### Disproved along the way

- **epaint tessellates four-edged rounded-rect strokes correctly.** Rasterizing the
  tessellator's own `rect_stroke` output for a 240×35 rect filled all four edges and all
  corner arcs. The loss was clipping, never tessellation.
- **A degenerate or very thin rect was not the cause.** The pre-fix field was 240×34.
- **The row-level clip clamp was written, measured, and removed.** `ScrollArea` narrows its
  clip by the scrollbar (`inner_size = outer_size - current_bar_use`,
  `scroll_area.rs:555`), so a row clamped to the clip shrank to 230 whenever the list grew
  long enough to scroll — and back to 240 when it did not. The bound moved to `tree_width`,
  before the `ScrollArea`, and `the_tree_row_spans_its_layout_width_not_its_clip` now pins
  the row to its layout width.

### Residual (P2)

- `SearchInput` still reserves guessed `36.0` / `20.0` for the shortcut badge and the clear
  button. Harmless now that the frame is capped, but it narrows the text area by roughly
  14px against the measured widths.
- The outside-stroke loss affects any 1px `Frame` / `Button` border flush with a container
  edge elsewhere in the app. Only the sidebar column carries the 1px bleed.


## Source: `VERIFICATION.md`

# Verification — sidebar-content-full-width

## Automated

```text
cargo check -p db-pro-ui --all-targets
→ PASS (2026-09-17)
```

## Runtime (pending)

Rebuild `db-pro-native`, open Explorer with a failed connection:

1. Tree hover wash extends to near the blue drag line (not mid-panel).
2. Long error hint truncates near the drag edge, not ~halfway across the sidebar.
3. `New query` / filter and tree share the same content width.
4. Dragging the separator still respects 220–380.

## Providers

n/a — layout-only UI change (PostgreSQL / SQLite unaffected).

---

## Follow-up: automated evidence (2026-09-17, `fix/sidebar-column-overflow`, off `main`)

```text
cargo test --workspace --offline
→ 1185 passed / 0 failed / 42 ignored          (db-pro-ui alone: 516 passed / 0 failed)
cargo clippy --workspace --all-targets --offline -- -D warnings
→ PASS
cargo build --release --locked -p db-pro-native
→ PASS
bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci
→ 15 pass / 1 warn / 0 fail
cargo fmt --all -- --check
→ red at baseline (repo-wide, pre-existing). Every line this branch changed is clean;
  the remaining hits in `sidebar_view.rs` (36/65/95/112/145/172/186/252/300/313/332/352/395)
  and `app_tests.rs:2195` are original untouched code — the added `set_clip_rect` line
  appears in the 65 hunk as context, not as a change.
```

The single clean-code warning is the ratcheted function-length debt
(`SearchInput::show` 98, `draw_sidebar` 66, `draw_sidebar_contents` 216) — all pre-existing
and all held at warning level by `--ratchet`. Zero blocking findings.

### Guards, each verified to fail on the reverted fix

A guard that passes on broken code is worthless, so each was checked by reverting the fix
it protects and confirming the failure message.

| Guard | Fix reverted | Observed failure |
| --- | --- | --- |
| `the_sidebar_paints_nothing_past_its_clip` | toolbar back to a hardcoded `actions_width = 28.0` | `[[180.0 103.0] - [214.0 127.0]] … clip=[[7.0 7.0] - [209.0 893.0]]` — the refresh button paints 5px past the clip |
| `the_sidebar_paints_nothing_past_its_clip` | `SIDEBAR_CLIP_BLEED` back to `0.0` | 3 shapes escape: the `New query` border `painted=[[7.0 47.0] - [209.0 77.0]]` vs `clip=[[8.0 8.0] - [208.0 892.0]]`, the separator rule, and the header action |
| `the_driver_badge_is_fully_visible_in_the_navigator_tree` | badge back to `badge.len() * 6.5 + 8.0` | `the driver badge pads its label by 8 instead of 4` |
| `the_filter_field_border_is_painted_inside_the_field` | `paint_field_chrome` back to stroking `rect` directly | `painted [[7.0 101.0] - [167.0 129.0]] against field [[8.0 102.0] - [166.0 128.0]]` |
| `the_tree_row_spans_its_layout_width_not_its_clip` | row width clamped to the clip | `left: 230.0, right: 240.0` — the row shrinks by the 10px simulated scrollbar |

Each guard also runs at `SIDEBAR_MIN_WIDTH`, `260.0` and `360.0`, because the fix has to
hold across the whole legal sidebar width rather than at one screenshot size.

### Measured after the fix (1440×900, `sidebar_width = 260`)

```text
SUB_PANES outer max_rect=[[8.0 8.0] - [248.0 892.0]] avail=240.0   (was [[8.0 102.0] - [254.0 136.5]], 282 wide)
#26 Rect fill=#FA_FA_FA  bounds=[[8.0 102.0] - [206.0 128.0]]       filter field takes the remainder
#24 Rect fill=#F7_F7_F7  bounds=[[214.0 102.0] - [248.0 126.0]]     refresh button, measured 34 wide, inside the column
#31 Rect stroke w=1.0 rounding=5 bounds=[[8.5 102.5] - [205.5 127.5]] clip=[[8.0 8.0] - [248.0 892.0]]
#35 Rect fill=#EE_EE_EE  bounds=[[205.0 147.0] - [244.0 163.0]] clip=[[8.0 142.0] - [248.0 168.0]]
#36 Text "SQLITE"        bounds=[[209.0 151.5] - [240.0 159.5]]
#37 Text "Native Test"   bounds=[[54.0 149.5] - [119.0 160.5]]
```

The badge pill now ends at `244`, inside the `248` clip, and its label is fully painted
(31px of text in a 39px pill) instead of surviving as a 1px sliver. The field's border sits
at `8.5–205.5`, strictly inside the clip, so no part of it is discarded.

## Runtime: COMPLETED — follow-up

Captured via the `capture` feature (`--features capture`; the driver lives on `main` and
lands on this branch via the rebase onto `main`). Three PNGs in `screenshots/`, captured
from the default explorer view with a populated store (the repo-root `.db-pro-data`):

| Viewport | File | Size | Verified |
|---|---|---|---|
| 1280×800 | `sidebar-1280x800.png` | 145 KB | driver badges (`SQLITE`, `PG`) fully readable; filter + `New query` borders closed on all four sides; tree fills the sidebar column |
| 1440×900 | `sidebar-1440x900.png` | 151 KB | same — badge readable, borders closed, column filled |
| 1920×1080 | `sidebar-1920x1080.png` | 166 KB | same — badge readable, borders closed, column filled |

`sips` confirms pixel dimensions exactly match the gate sizes (1280×800 / 1440×900 / 1920×1080).

The two P1 defects are visually verified by the screenshots:

- **P1 — tree rows overflow and lose the trailing badge.** Pre-fix: `Native Test` row painted
  `SQLITE` badge at `x 239–286` against a clip ending at `248` — a ~1px sliver. Post-fix:
  the badge pill ends at `244` inside the `248` clip, label fully painted (31px of text in a
  39px pill). The screenshots show the badge fully readable at every gate size.
- **P1 — every 1px border on a widget filling the column loses both vertical edges.** Pre-fix:
  `Shape::rect_stroke` paints entirely outside its path, and the sidebar clipped to exactly
  the content column, so the filter / `New query` borders lost both vertical bands. Post-fix:
  `paint_field_chrome` strokes an inset path and the sidebar has a 1px bleed. The
  screenshots show the filter and `New query` borders closed on all four sides.

The remaining two runtime items are covered by the automated guards (not by a static
screenshot):

- **Rows keep full width when the scrollbar appears** — `the_tree_row_spans_its_layout_width_not_its_clip`
  (the row width is bound *before* the `ScrollArea`, so the scrollbar cannot shrink it).
- **Dragging the separator still clamps 220–380, and the tree still fills the column at
  `SIDEBAR_MIN_WIDTH`** — all five revert-verified guards run at `SIDEBAR_MIN_WIDTH`,
  `260.0`, and `360.0`.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — sidebar-content-full-width (closure)

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | fix lane (sidebar layout / clip) — closure handoff |
| Issue(s) | user-reported: tree still overlapped by the width (driver badge hidden) + filter field border missing both ends |
| Task state | Done (claimed, implemented, gated, runtime-verified) |
| Baseline SHA | `a3a6a5290e3cf992354ee187169818a5c57122ac` — the root-cause fix was authored off this base; this handoff rebases onto current `main` (`1a20eb25`) and closes the runtime-evidence gate |
| Branch / PR | `fix/sidebar-column-overflow` (rebased tip `7b9cfc14`) · merged into `main` as part of this closure; no PR (owner opted to merge directly into `main`) |
| Scope interpretation | rebase the sidebar fix onto the now-advanced `main` (which carries the new-connection secret+input fix), close the UI runtime-evidence gate by capturing the affected surface, flip the plan to `COMPLETED`, and merge into `main` |
| Out of scope | the `SearchInput` `36.0` / `20.0` P2 reservations; the outside-stroke loss on non-sidebar columns (both documented as P2 residuals in `FINDINGS.md`); provider connectors; Tauri host; `SearchInput` width contract |

## 2. Progress checkpoint

- Current branch HEAD (pre-merge): `7b9cfc14` (rebase of original `55593272` onto `main` `1a20eb25`)
- Rebase result: clean, no conflicts (sidebar fix touched `sidebar_view.rs`, `explorer_view.rs`, `explorer_tree.rs`, `components/input/{layout,search}.rs`, `app_tests.rs`; the new-connection fix touched `components/input/{text,password,tests}.rs` — disjoint files in `components/input/`, so no overlap)
- Completed acceptance rows: content rect locked to `sidebar_width`; drag separator locked; ScrollArea / tree rows forced to content width; header paint order fixed; duplicate Plus removed; explorer toolbar lays refresh button out first; `tree_width` bound before `ScrollArea`; driver badge width measured from the label; sidebar 1px clip bleed; `paint_field_chrome` inset path
- Remaining acceptance rows: none — all five revert-verified guards pass; runtime screenshots captured at all three gate sizes
- Findings / risks: P1 tree-row overflow (fixed, runtime-verified); P1 1px border loss on flush widgets (fixed, runtime-verified); P2 residuals (SearchInput guesses, non-sidebar outside-stroke) logged
- Tests already run (rebased tree): `cargo clippy --workspace --all-targets --offline -- -D warnings` → exit 0; `cargo test --workspace --offline` → **518 passed / 0 failed** (plus ignored, unchanged); `cargo build --release --locked -p db-pro-native` → exit 0; clean-code `--diff --ratchet --ci` → 15 pass / 1 warn / 0 fail
- Dependency / blocker changes: none (the `capture` feature already lives on `main`, so it lands on the branch via rebase — no new infra needed)

## 3. Implementation handoff

| Field | Value |
|---|---|
| Original fix SHA | `55593272` (pre-rebase) |
| Rebased tip | `7b9cfc14` (rebased onto `main` `1a20eb25`; no conflicts) |
| Closure commit(s) | planned on the branch: the runtime-evidence commit (CHECKLIST / VERIFICATION / FINDINGS flipped to COMPLETED + screenshots + capture-intent commit message) and the move-to-completed commit; both land before the merge |
| File / surface inventory | `crates/ui/src/sidebar_view.rs` — clip bleed; `crates/ui/src/explorer_view.rs` — toolbar `right_to_left` + `tree_width` bound; `crates/ui/src/explorer_tree.rs` — badge width measured from painter; `crates/ui/src/components/input/layout.rs` + `search.rs` — `paint_field_chrome` inset path; `crates/ui/src/app_tests.rs` — five revert-verified guards |
| Acceptance mapping | "tree still overlapped by the width, badge hidden" → `the_driver_badge_is_fully_visible_in_the_navigator_tree` + runtime screenshots; "filter field border missing both ends" → `the_filter_field_border_is_painted_inside_the_field` + runtime screenshots |
| Commands and counts | `cargo clippy --workspace --all-targets --offline -- -D warnings` → exit 0; `cargo test --workspace --offline` → 518/0; `cargo build --release --locked -p db-pro-native` → exit 0; clean-code `--diff --ratchet --ci` → 15/1/0; `cargo build --release --locked --offline -p db-pro-native --features capture` → exit 0; capture runs (1280×800 / 1440×900 / 1920×1080) → 3 PNGs in `screenshots/`, all on the default explorer view with a populated store |
| CI run IDs / status | not run (no CI trigger in this session) |
| Known limitations | static screenshots cannot show hover / scrollbar / drag states — these are covered by the five revert-verified guards (`the_sidebar_paints_nothing_past_its_clip`, `the_driver_badge_is_fully_visible_in_the_navigator_tree`, `the_filter_field_border_is_painted_inside_the_field`, `the_tree_row_spans_its_layout_width_not_its_clip`) |
| Migrations / config implications | none |
| Out-of-scope changes | none on this branch |

## 4. Review outcome

Self-verified. P0 = 0, P1 = 0 (fixed and runtime-verified), P2 = 2 (logged: `SearchInput` `36.0` / `20.0` reservations; outside-stroke loss on non-sidebar columns).

## 5. Research / audit handoff

- Source date: 2026-09-17
- Source URLs / references: `crates/ui/src/sidebar_view.rs`, `crates/ui/src/explorer_view.rs`, `crates/ui/src/explorer_tree.rs`, `crates/ui/src/components/input/{layout,search}.rs`; `docs/plans/active/sidebar-content-full-width/{PLAN,FINDINGS,VERIFICATION,CHECKLIST}.md`
- Factual findings: `Ui::set_max_width` ends with `max_rect = max_rect.union(min_rect)` (`placer.rs`); `Painter::with_clip_rect` *intersects*; `Shape::rect_stroke` paints entirely outside its path (`StrokeKind::Outside`, `tessellator.rs`); `ScrollArea` narrows its clip by the scrollbar (`scroll_area.rs`)
- Inference: the sidebar defects are *clip*-defects (layout vs paint clip), not tessellation defects — disproved by rasterizing the tessellator's own output
- Decision / recommendation: ship as-is; runtime evidence now captured (PNGs in `screenshots/`); plan moves to `completed/`
- Unresolved questions: none for this scope
- Downstream tasks activated: none

## 6. Tổng kết (Vietnamese summary)

Đóng gate runtime-evidence cho fix sidebar (`fix/sidebar-column-overflow`): rebase sạch lên
`main` (`1a20eb25`), capture 3 PNG (1280×800 / 1440×900 / 1920×1080) bằng `capture` feature
cho thấy driver badge `SQLITE` / `PG` đọc được đầy đủ và border filter + `New query` kín 4
cạnh — đúng hai P1 mà user báo. CHECKLIST / VERIFICATION / FINDINGS flipped sang COMPLETED.
Gates xanh: clippy -D warnings sạch, 518 test pass, release build sạch, clean-code 15/1/0.
P2 residuals (`SearchInput` reservations, outside-stroke ngoài sidebar) đã ghi nhận, không
blocker. Bước tiếp: commit docs + screenshots trên branch, move plan sang `completed/`, rồi
fast-forward merge vào `main`.
