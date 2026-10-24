---
schema_version: 1
id: "MEM-PLAN-theme-core-token-db134231317a"
title: "Plan archive: Theme Core Token"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, theme-core-token]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "db134231317a4b9a065e26564c5374fefaf8c3a9626909a6879feb29fab7defd"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/theme-core-token/PLAN.md"
  - "docs/plans/active/theme-core-token/FINDINGS.md"
  - "docs/plans/active/theme-core-token/VERIFICATION.md"
  - "docs/plans/active/theme-core-token/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/theme-core-token/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Core theme token foundation

## Goal

Turn the native UI token foundation into a clear, maintainable core token
contract with three layers — `Primitive → Semantic → Component` — while keeping
the existing `DbProTheme` field API as a compatibility facade and preserving
every currently rendered value.

## Architecture

| Layer | File | Owns | Consumers |
|---|---|---|---|
| Primitive | `crates/ui/src/tokens/primitive.rs` | light/dark raw palettes (private), spacing/radius/type/stroke/elevation/motion scales | re-exported scales via `crate::tokens`; palettes reachable only by the semantic layer |
| Semantic | `crates/ui/src/tokens/semantic.rs` | purpose-based roles with explicit light+dark treatments, status wash recipe | `DbProTheme` (flattened facade), components via `theme.semantic` |
| Component | `crates/ui/src/tokens/component/{button,input,dialog,table,feedback}.rs` | component-owned sizes, state precedence, role-consumption contracts | the matching components; sizes re-pointed at their contract |
| Adapter | `crates/ui/src/theme.rs` | `DbProTheme` facade + `apply()` mapping to `egui::Visuals`/`egui::Style`; `EGUI_*` constants are private here | all existing callers |

## Scope (this slice)

- Extract raw light/dark palettes out of the `DbProTheme` constructors into the
  primitive layer; rebuild both constructors from semantic roles.
- Define every prescribed semantic role (background, foreground, border,
  accent, status with subtle/fill/border/foreground, syntax) for both themes.
- Define component token contracts for Button, Input, Dialog/Overlay,
  Table/Grid, Feedback/Status: owned values, supported states, explicit
  precedence `disabled → loading → active → focus → hover → default`.
- Privatize renderer adapter constants (`EGUI_*`, `RADIUS_EGUI_*`) inside
  `theme.rs`.
- Fix two measured state-precedence defects: disabled+loading button rendered
  the spinner; disabled input could wear focus/error borders.
- Route alert status frames through `theme.semantic.status.*`.

## Non-goals

- DTCG/JSON generation tooling (one active Rust UI; typed Rust tokens are the
  runtime source).
- Palette or density changes — every rendered value is preserved.
- Migrating every component-local config file; genuinely local geometry stays
  in its component, classified by its config header or the contract doc.
- Editor helper derivations (`editor_*_fill`, line numbers) and badge inline
  opacity multipliers — recorded in FINDINGS as deferred.

## Acceptance criteria

- `DbProTheme::light()` and `DbProTheme::dark()` are built from explicit
  primitive palettes via `SemanticTokens`.
- `DbProTheme::apply()` contains no unowned radius, spacing, stroke, or shadow
  literals; `EGUI_*`/`RADIUS_EGUI_*` are private to the adapter.
- Every semantic role has both light and dark treatment (unit test).
- Button sizing has one source of truth (contract constants; config re-points).
- Existing semantic field access remains valid; aliases documented with one
  authoritative role.
- State precedence is explicit in code and pinned by tests.
- Formatting, targeted compilation, clean-code scan, and `git diff --check`
  pass; runtime capture recorded honestly.


## Source: `FINDINGS.md`

# Findings

## P2 — Core values had multiple owners (slice 1, fixed)

`DbProTheme` owned raw colors and semantic roles in the same constructors.
`tokens.rs` and component config files also owned overlapping button values.
This allowed visual drift without changing a semantic token name.

**Resolution (this slice):** raw colors live in `tokens/primitive.rs`
(palettes), roles in `tokens/semantic.rs`, component-owned sizes in
`tokens/component/*`, and `DbProTheme` is a flat compatibility facade built
from `SemanticTokens`.

## Decision — layering and ownership

- Primitive palettes are private to the `tokens` module tree: component
  rendering cannot read a raw theme value; colors arrive through
  `DbProTheme`/`theme.semantic`. Numeric scales stay reachable via the
  `crate::tokens` facade so existing `use crate::tokens::*` imports resolve
  unchanged (single-definition re-exports, no duplicated values).
- `DbProTheme` gained one additive field, `semantic: SemanticTokens`, so
  components that need role detail (alert fill/border pairs) stop multiplying
  raw values at the paint site. No struct-literal construction exists outside
  `theme.rs` (verified at `bd787db1e5d75862ecd08c86539e910d4e651ebd`), so the
  added field breaks no caller.
- Renderer adapter constants (`EGUI_ITEM_SPACING`, `EGUI_BUTTON_PADDING`,
  `EGUI_INTERACT_SIZE`, `EGUI_WINDOW_MARGIN`, `EGUI_MENU_MARGIN`,
  `EGUI_INDENT`, `RADIUS_EGUI_*`) moved from `tokens.rs` into private consts
  in `theme.rs` — their only consumer was `theme.apply()` (grep verified).

## P2 — State-precedence defects (fixed, measured)

- **Button `disabled + loading` rendered the spinner.** `Button::show()`
  branched on `loading` before consulting `enabled`, so the gallery's
  `.enabled(false).loading(true)` sample painted loading colors instead of the
  disabled palette. Fixed via `button_contract::shows_loading`
  (`disabled → loading`); observable only in the disabled+loading combination.
- **Disabled input could wear focus/error borders.** `paint_field_chrome`
  evaluated `has_error`/`focused` before `!enabled`, contradicting the
  required precedence. Fixed via `input_contract::resolve_chrome`
  (`disabled → error → focus → hover → default`); enabled-state rendering is
  byte-identical (the hover lerp still ticks on exactly the previous
  condition).

## Decision — compatibility aliases (one authoritative role)

- `surface_2` → authoritative role `background.subtle` (quiet fill: disabled
  button fills, card icon boxes). Value differs from `background.panel` in both
  themes, so it cannot collapse into another role without a visual change.
- `text_muted` → authoritative role `foreground.muted`. Light coincides with
  `foreground.tertiary` (#8a8a8a); dark deliberately differs (#8c96a0 vs
  #828c97), so both roles stay (pinned by test).
- `border.separator` is defined per the prescribed role list with the value of
  `border.default` and currently has no consumer — documented in the role so
  future separator work has an owner instead of an inline literal.
- `StatusRole.foreground` equals `solid` in the shipped palettes (alert icons
  consume it); it exists so on-status text can diverge without touching
  consumers.

## Decision — button dimensions: 32/38 is authoritative

At baseline `bd787db1`, `tokens.rs` carried `BUTTON_HEIGHT_DEFAULT = 36.0` and
`BUTTON_HEIGHT_LG = 42.0` with **no consumers**, while
`components/button/config.rs` owned the rendered literals 32.0/38.0. Slice 1
re-pointed config at the tokens and corrected the tokens to 32.0/38.0, so
rendered dimensions never changed. The Button contract now owns
28/32/38 (pinned by `dimensions_preserve_the_existing_rendered_values`).

## Notes — classification kept out of this slice (P2, deferred)

- `DIALOG_RADIUS = 16.0` (product dialog frame, `components/dialog/config.rs`)
  vs `tokens::RADIUS_DIALOG = 6.0` (egui window rounding via the adapter) are
  different surfaces with confusingly similar names; documented in the
  Dialog/Overlay contract instead of renamed.
- `TABLE_ROW_HEIGHT_COMPACT` is consumed by `components/select` menu rows, not
  by the table component; moved under the Table contract as its name implies,
  with the quirk recorded here.
- Pre-existing dead constants kept (not introduced by this slice):
  `TREE_ROW_HEIGHT`, `SIDEBAR_ROW_HEIGHT`, `TABLE_ROW_HEIGHT_DEFAULT`,
  `TABLE_HEADER_HEIGHT`, and the `BUTTON_ICON_SIZE_*` hit-box vectors (the
  icon hit-box/glyph separation the naming rules ask for — hit-box sizes now
  sit next to glyph sizes in the Button contract).
- Editor helpers (`editor_gutter_fill`, `editor_current_line_fill`,
  `editor_selection_fill`, `editor_line_number`) still own raw values in
  `theme.rs`; they are derivation helpers outside the five representative
  contracts and deferred to a follow-up editor-role slice.
- Badge borders still apply inline `linear_multiply(0.4/0.5)` multipliers
  (`components/badge/handler.rs`); they differ per variant (0.4 success, 0.5
  others) and were left as component-local rather than silently unified.
- Clean-code scan warnings accepted with reason: 6 `as` casts are the
  verbatim `soft_tint` recipe moved into `semantic::subtle_wash` (changing
  them to `try_from` is impossible on floats and would risk value drift);
  `paint_field_chrome` (5 params) and `paint_focus_ring` (4 params) are
  inherited signatures; `SemanticTokens::light/dark` (54 lines) stay as one
  constructor per theme so a reviewer can diff them 1:1 against the baseline
  constructors.


## Source: `VERIFICATION.md`

# Verification

## Source evidence

- Baseline: `bd787db1e5d75862ecd08c86539e910d4e651ebd` (main).
- Branch: `feature/theme-core-token` (created successfully this session — this
  corrects the previous record claiming `.git` was read-only and the branch
  creation was blocked; `.git` is writable in this environment).
- Source slice commits: `8a188e84` (core contract) and
  `f7cf6541bf0462544d8e9c2c041457337b68043e` (state-precedence fix).

## Commands (executed this session, results recorded verbatim)

| Command | Result |
|---|---|
| `git status --short --branch` / `git rev-parse HEAD` | PASS — branch `feature/theme-core-token`, source fix HEAD `f7cf6541…` |
| `cargo fmt --all -- --check` | PASS (exit 0), run after the final edit |
| `CARGO_TARGET_DIR=/tmp/db-pro-target cargo check --workspace` | PASS (exit 0) — extra guard: no consumer outside `db-pro-ui` uses the moved tokens |
| `CARGO_TARGET_DIR=/tmp/db-pro-target cargo clippy --workspace --all-targets -- -D warnings` | PASS (exit 0) |
| `CARGO_TARGET_DIR=/tmp/db-pro-release-target cargo build --release --locked -p db-pro-native` | PASS (exit 0) |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 13 ✓ / 3 ⚠ / 0 ✗, exit 0; warnings explained in FINDINGS |
| `git diff --check` | PASS (exit 0) |
| `CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/db-pro-target cargo test --workspace` | PASS — 928 passed / 0 failed / 0 ignored, exit 0 |

## Static contract checks

| Check | Result |
|---|---|
| No new raw color/radius/spacing/stroke in component rendering | PASS — component diffs only replace literals with named roles/consts (`STROKE_THICK`, `border.focus`, `theme.semantic.status.*`); alert's two opacity literals moved to `semantic::STATUS_*` |
| Every semantic role has light and dark treatment | PASS — `every_role_group_has_light_and_dark_treatment` |
| State precedence explicit (`disabled → loading → active → focus → hover → default`) | PASS — `component::STATE_PRECEDENCE` + `button::shows_loading` + `input::resolve_chrome`, each pinned by tests |
| Existing button dimensions and theme values unchanged | PASS — `dimensions_preserve_the_existing_rendered_values`, `light_tokens_follow_warm_minimalism_surface_contract`, `dark_tokens_follow_database_workstation_surface_contract`, `flat_facade_fields_are_built_from_the_semantic_roles` |
| All imported symbols exist; no dead compatibility alias introduced | PASS — `cargo check --workspace` clean; the `FieldChromeState` re-export in `input/layout.rs` is consumed by password/text/textarea/search |

## Runtime evidence

**NOT VERIFIED** — no UI runtime session (screenshot/screen recording at
1280×800, 1440×900, 1920×1080) was captured in this slice. Source inspection
and unit tests are not runtime evidence. The two precedence fixes
(disabled+loading button, disabled input chrome) are observable in the
component gallery and must be checked in a runtime pass before this feature
moves to `RUNTIME_VERIFY`/`COMPLETED`.

## Tổng kết bằng tiếng Việt

Đã xác minh lại toàn bộ bằng chứng cho slice hợp đồng token ba lớp: `.git` có
thể ghi được nên branch `feature/theme-core-token` đã được tạo (sửa lại ghi chú
 sai của lần trước), mã nguồn được commit tại `f7cf6541`. Các gate bắt buộc
đều PASS (fmt, workspace check, clippy, release build, clean-code scan,
`git diff --check`), và 928 test toàn workspace chạy xanh
100%. Các kiểm tra tĩnh (không thêm giá trị thô, đủ role light/dark, thứ tự
state precedence, kích thước button giữ nguyên, không import chết) đều đạt.
Bằng chứng runtime giao diện vẫn ở trạng thái **NOT VERIFIED** — cần một phiên
chụp màn hình gallery ở 3 độ phân giải trước khi đưa feature sang `RUNTIME_VERIFY`.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — core token contract (Primitive → Semantic → Component)

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Buffy · Codebuff coding agent · implementation lane |
| Issue(s) | n/a — driven by task prompt `/tmp/db-pro-core-token-contract-agent-prompt.md` |
| Task state | `Review` (implementation + self-review done; independent review pending) |
| Baseline SHA | `bd787db1e5d75862ecd08c86539e910d4e651ebd` — everything below is asserted against this tree |
| Branch / PR | `feature/theme-core-token` · PR none (not opened) |
| Scope interpretation | Build the three-layer core token contract for the native egui UI with five representative component contracts, keeping `DbProTheme` as compatibility facade and every rendered value unchanged |
| Out of scope | DTCG/JSON tooling; palette or density changes; migrating the other component configs; editor-derivation and badge-opacity roles (deferred in FINDINGS); archived frontend |

## 2. Progress checkpoint

- Source HEAD: `f7cf6541bf0462544d8e9c2c041457337b68043e` (source fix commit; this handoff docs commit follows)
- Completed acceptance rows: `[x]` primitive palettes extracted and private, `[x]` semantic roles with light+dark per role, `[x]` status roles carry solid/subtle/fill/border/foreground and are consumed by alert, `[x]` five component contracts with explicit precedence, `[x]` `EGUI_*`/`RADIUS_EGUI_*` private to the adapter, `[x]` button single source of truth, `[x]` facade fields + documented aliases, `[x]` four gates + targeted tests green, `[x]` static checks green
- Remaining acceptance rows: `[ ]` runtime UI capture at 1280×800 / 1440×900 / 1920×1080 — **NOT VERIFIED**
- Findings / risks: no P0/P1 introduced. P2 deferred items listed in `FINDINGS.md` at `f7cf6541` (editor helper raw values, badge inline multipliers, `border.separator` unconsumed, dead size constants kept, `DIALOG_RADIUS`/`RADIUS_DIALOG` naming, `TABLE_ROW_HEIGHT_COMPACT` consumed by select)
- Tests already run: `CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/db-pro-target cargo test --workspace` → 928 passed / 0 failed, exit 0
- Dependency / blocker changes: previous record claimed `.git` was read-only and blocked branch creation; corrected — `.git` is writable, branch `feature/theme-core-token` created from `bd787db1`

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `f7cf6541bf0462544d8e9c2c041457337b68043e` |
| Commit list | `8a188e84` feat(ui): build core token contract (primitive → semantic → component); `f7cf6541` fix(ui): enforce button token state precedence |
| File / surface inventory | **new** `crates/ui/src/tokens/primitive.rs` (private light/dark palettes + moved scales), `crates/ui/src/tokens/semantic.rs` (role structs, `SemanticTokens::light/dark`, status wash recipe, 3 tests), `crates/ui/src/tokens/component.rs` (mandatory `STATE_PRECEDENCE` + test), `crates/ui/src/tokens/component/{button,input,dialog,table,feedback}.rs` (owned sizes, `shows_loading`, `FieldChromeState`/`resolve_chrome`, contracts + tests); **rewritten** `crates/ui/src/tokens.rs` (facade: `mod primitive` private, `pub mod semantic/component`, single-definition scale re-exports, shell chrome sizes); **modified** `crates/ui/src/theme.rs` (constructors rebuilt from `SemanticTokens`, additive `semantic` + `border_focus` fields with alias docs, adapter consts privatized, `soft_tint` delegates to `semantic::subtle_wash`, `*_soft()` reads status roles, 2 new tests), `components/button/ui.rs` (disabled→loading gate), `components/button/config.rs` (imports its contract), `components/input/layout.rs` (contract-owned state + `resolve_chrome`, focus uses `border.focus`), `components/interact.rs` (focus ring = `border.focus` × `STROKE_THICK`), `components/alert/{handler,config}.rs` (status frames from `theme.semantic.status.*`, opacities moved to semantic), `components/select/{config,handler,ui/option}.rs` + `sidebar_chrome_view.rs` (import re-pointing only) |
| Acceptance mapping | "primitive palettes" → `tokens/primitive.rs` + `DbProTheme::light/dark`; "each semantic role light+dark" → `semantic.rs::every_role_group_has_light_and_dark_treatment`; "state precedence explicit" → `component::STATE_PRECEDENCE` + `button::shows_loading`/`input::resolve_chrome` tests; "button dimensions unchanged" → `component::button::dimensions_preserve_the_existing_rendered_values` + inherited theme value tests; "adapter constants private" → `theme.rs` private `EGUI_*`/`RADIUS_EGUI_*` (grep: no consumer outside `theme.rs`) |
| Commands and counts | `cargo fmt --all -- --check` → exit 0; `CARGO_TARGET_DIR=/tmp/db-pro-target cargo check --workspace` → exit 0; `CARGO_TARGET_DIR=/tmp/db-pro-target cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `CARGO_TARGET_DIR=/tmp/db-pro-release-target cargo build --release --locked -p db-pro-native` → exit 0; `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → exit 0; `git diff --check` → exit 0; `CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/db-pro-target cargo test --workspace` → 928 passed / 0 failed / 0 ignored, exit 0 |
| CI run IDs / status | not run (no CI trigger in this session) |
| Known limitations | runtime UI capture **NOT VERIFIED**; only the five representative contracts migrated — other components still read local config literals; `border.separator` role defined per the prescribed list but unconsumed; clean-code warnings accepted with reasons in FINDINGS |
| Migrations / config implications | none — no persisted state, env vars, or keys; `DbProTheme` change is additive (one field), no struct-literal construction exists outside `theme.rs` (verified at baseline) |
| Out-of-scope changes | `none` — all source changes belong to the token contract; plan docs updated under `docs/plans/active/theme-core-token/` |

## 4. Review outcome

n/a for this pass: implementation self-reviewed (architecture, correctness,
coverage) but not independently reviewed; the read-only external reviewer has
not run against `f7cf6541`. Self-review is not independent approval — P0/P1
findings from the external reviewer must be resolved before merge.

## 5. Research / audit handoff

n/a (implementation lane). Source date for all claims: 2026-09-30, tree at
`bd787db1` (baseline) through `f7cf6541` (source fix commit), repo paths listed in
the file inventory above.

## 6. Tổng kết (Vietnamese summary)

Đã hoàn thành hợp đồng token ba lớp cho UI egui native trên branch
`feature/theme-core-token`: primitive (bảng màu light/dark riêng tư + scale số),
semantic (đủ role light/dark, status có solid/subtle/fill/border/foreground),
và 5 contract component (button, input, dialog/overlay, table, feedback) với
thứ tự state precedence tường minh `disabled → loading → active → focus →
hover → default`. `DbProTheme` vẫn là facade tương thích (thêm field
`semantic` và `border_focus`, mọi field cũ giữ nguyên, bí danh được ghi chú
role chủ sở hữu); hằng số adapter `EGUI_*` đã chuyển vào riêng `theme.rs`.
Hai lỗi precedence được sửa: nút `disabled + loading` không còn hiện spinner,
input disabled không còn viền focus/error. Mọi giá trị render được giữ nguyên
và ghim bằng test — 928 test workspace xanh, các gate fmt, check, clippy,
release build, clean-code
scan 0 fail, `git diff --check`), check toàn workspace PASS. Việc chưa làm:
chụp runtime UI (NOT VERIFIED), migrate các component còn lại,/editor helper
và badge opacity (ghi trong FINDINGS). Commit mã
nguồn `f7cf6541`; runtime UI còn NOT VERIFIED, bàn giao cho reviewer độc lập
trước khi merge.

