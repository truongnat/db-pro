# RC1 P2-F — cross-cutting i18n / accessibility / error-state audit (#80)

- Session: issue-queue pass 3, 2026-09-15
- Issue: **#80** ([RC1][P2-F] Audit cross-cutting i18n/accessibility/error-state release polish) —
  parent **#27**
- **Base head:** `main @ 2a2d09e` (worktree clean at session start)
- **Dispositions:** `docs/release/rc1-p2-release-dispositions.md` §F
- **Child issue:** **#241** (the six defects fixed in this audit)

## 1. What was audited

The nine scope bullets of the issue, against the native UI (`crates/ui`, `crates/native-app`). The
domain holds exactly one live `QA-P2-*` row — `QA-P2-06`, "significant Agent/Connection UI strings
bypass i18n" — and the rows in adjacent domains that touch these bullets (`QA-P2-02/04/05/23` in §A,
`QA-P2-10` in §B, `QA-P2-15/16/17/19` in §C) are cross-referenced rather than re-counted, so no row is
dispositioned twice.

## 2. Defects found and fixed (child issue #241)

| # | Defect (verified in source) | Impact | Fix |
|---|---|---|---|
| 1 | the status bar rendered `runtime_message` only when the text contained `failed`/`Failed`/`error`/`Error` (`navigation_view.rs:233` + `app.rs:723-728`), while the app assigns that field at ~165 sites | every refusal and gate message was invisible, including `"Connect with write access to delete rows"` (`table_editor_view.rs:1741,1746`), `"Table has no primary key; safe row editing is unavailable."` (`:1738`), `"Select a row before deleting"` (`:1755`), `"Table structure is still loading"` (`:654`) | `runtime_status()` (`app.rs:730-746`) returns any non-empty message with the danger colour reserved for errors; the bar renders it (`navigation_view.rs:284`) |
| 2 | the table editor's **Apply DDL** did nothing: `draw_ddl_script_card` returns "pressed" (`table_editor_view.rs:1398`) and the caller stored the buffer back (`table_ddl_view.rs:58-61`); `ddl_execute_confirmation` is assigned `false` in all eight sites and never `true`, so the confirmation card and `submit_ddl`'s only non-test caller were unreachable — while the caption claimed "execution is confirmation-gated" | a destructive action advertised as working and confirmation-gated did nothing at all | the control is disabled with the reason and the caption names the documented path (`table_editor_view.rs:1400-1425`, `DDL_APPLY_ENABLED`), consistent with `known-limitations.md:87` ("DDL via query editor") |
| 3 | `Button::accessible_name()` fell back to the literal `"Button"` when an icon-only control had a tooltip but no text (`components/button.rs:155-160`), and AccessKit is shipped (`crates/ui/Cargo.toml:8-9`) | all seven main-toolbar controls, the tab-strip `+` and the palette/dialog buttons announced as "Button" to a screen reader | the tooltip is the last fallback before `"Button"` (`components/button.rs:155-163`) |
| 4 | `is_context_menu_triggered` required `hovered()` or a pointer position inside the rect (`components/overlay.rs:791`), although `Sense::click()` is focusable in egui 0.29 (`egui-0.29.1/src/sense.rs:65-69`) | `Shift+F10` could not open a context menu on a *focused* widget — keyboard-only users had no context-menu path | focus counts as a target (`components/overlay.rs:789-794`) |
| 5 | five tooltips spelled `Cmd/Ctrl+…` instead of using the platform helper (`table_editor_view.rs:177,183`, `result_grid_view.rs:730,736`) and the agent tooltip advertised a hardcoded `⌘I` on every platform for a key no handler binds (`navigation_view.rs:141,143`) — `Key::I` appears in no handler | wrong shortcut text (both the modifier and, for the agent, the existence of the binding) | the helper is used (`primary_modifier_label()`), and the unbound shortcut claim is removed |
| 6 | eight hardcoded RGB literals duplicated theme tokens of the *opposite* theme (`explorer_view.rs:332,336,338,554,688`, `explorer_tree.rs:372,374`, `explorer_details.rs:464`) | connection/schema/column status colours did not follow the theme | `theme.success` / `danger` / `text_muted` / `warning` / `info` |

## 3. Measurements

### 3.1 Contrast (WCAG 2.1 relative luminance, computed from the literals in `crates/ui/src/theme.rs`)

| Theme | Pair | Ratio |
|---|---|---:|
| light | `text_primary` #0d0d0d on `surface_app` #ffffff | 19.68:1 |
| light | `text_secondary` #5f5f5f on #ffffff | 6.39:1 |
| light | **`text_tertiary` #8a8a8a on #ffffff** | **3.45:1** |
| light | `text_tertiary` on `surface_panel` #f7f7f7 | 3.22:1 |
| light | **`warning` #d97706 on #ffffff** | **3.19:1** |
| light | `danger` #dc2626 on #ffffff | 4.83:1 |
| light | `text_disabled` #b3b3b3 on `surface_panel` | 1.96:1 (disabled text is exempt from WCAG 1.4.3) |
| dark | `text_secondary` #b9b9b9 on `surface_app` #212121 | 8.21:1 |
| dark | `text_tertiary` #8d8d8d on #212121 | 4.85:1 |
| dark | `warning` #f59e0b on #212121 | 7.50:1 |
| dark | `danger` #ef4444 on #212121 | 4.28:1 |
| both | white on `accent` (light #0285ff / dark #339cff) | 3.62:1 / 2.86:1 — the pairing `accent_foreground` exists for |

The two bold rows are below the 4.5:1 AA threshold for normal-size text and are **accepted, not
claimed as passing**: the caption colour is used ~68 times in the light theme, and a compliant
candidate (#767676 → 4.54:1) would move it within 0.08 of `text_secondary` (#5f5f5f → 6.39:1), i.e. it
flattens the text hierarchy. That is a design decision needing visual verification, which this
environment cannot produce. Recorded in §F with the numbers.

### 3.2 Negatives verified by search (each is an explicit "does not exist")

| Claim | Evidence |
|---|---|
| no localization layer | `grep -rniE '\bi18n\b\|fluent\|gettext\|translat\|locale\|language' crates/ui crates/native-app` → only `small_translate` (`components/animation.rs:64`), `translate.rs` IPC marshalling, `CodeBlock::language("sql")`; no `.po`/`.ftl`/locale files anywhere; no i18n crate in the workspace; Settings has appearance + data files only (`navigation_view.rs:940-987`) |
| `Response::labelled_by` never used | `grep -rn labelled_by crates/` → 0 hits |
| no modal focus management | `grep -n focus crates/ui/src/components/dialog.rs` → 0 hits; `focus_lock_filter` → 0 hits; no `request_focus` in any close path |
| `Shift+F10` needed hover | `overlay.rs:784-791` before the fix |
| staged-row deletion is unconfirmed | `grep -rn "data_delete_confirmation = true" crates/` → 0 hits (all assignments are `= false`) |
| DDL confirmation unreachable | `grep -rn "ddl_execute_confirmation" crates/ui/src/` → 8 assignments, all `= false`; `submit_ddl` non-test caller only at `table_editor_view.rs:1459`, inside the unreachable card |
| only one production spinner | `ui.spinner()` in `crates/ui/src` → `explorer_view.rs:754` only, against the stated policy at `navigation_view.rs:982` |
| no Tab-navigation at app level | 5 `Key::Tab` sites, all local widget behaviour (`query_view.rs:1135`, `result_grid_view.rs:372,594`, `editor/renderer.rs:241`, gallery badge data) |

## 4. Tests and falsification

Three new tests: `components::button::tests::accessible_name_prefers_access_label_then_label_then_tooltip`,
`…::a_button_with_no_name_at_all_still_has_one`, and `app::tests::every_runtime_message_reaches_the_status_bar`.

Falsified with two probes on copies before being trusted:

| Probe | Result |
|---|---|
| `runtime_status()` restored to the old "only if it contains failed/error" condition | `every_runtime_message_reaches_the_status_bar` **FAILED** (`2 failed; 373 passed`) |
| tooltip fallback removed from `accessible_name()` | `accessible_name_prefers_access_label_then_label_then_tooltip` **FAILED** |

Restored → `cargo test -p db-pro-ui --lib` **375 passed / 0 failed**. The other four fixes are UI
affordance/wording changes with no test harness that can drive a frame here (no window server); they are
stated as source-verified, **not** as interactively verified.

## 5. Gates

| Gate | Result | Delta vs 875/0/27 |
|---|---|---|
| `cargo fmt --all -- --check` | exit 0 | — |
| `cargo check --workspace` | exit 0 | — |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, 0 warnings | — |
| `cargo test --workspace` | **878 passed / 0 failed / 27 ignored** | **+3** |
| `cargo build --release --locked -p db-pro-native` | exit 0 | — |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | `PASS (partial)` — 4 passed / 0 warnings / 0 failed / 4 not executed, qualified `+dirty(12)` | new status wording (#240) |

A first gate pass **failed** and is recorded rather than hidden: the new code triggered three clippy
warnings (two now-unused `Color32` imports and a collapsible `if`), which `-D warnings` turned into
errors, and the new perf-scan semantics reported `Passed: 3 / Warnings: 1` → `WARN` → **exit 2**. The
warnings were fixed, the imports trimmed and the `if` collapsed; the re-run above is the clean one.
That is the #240 fix doing its job on the very next code change.

No Rust behaviour outside the UI crate changed; the `--include-ignored` CI-mirroring leg is unaffected
by this change (no provider or migration code touched).

## 6. Not claimed

- **No interactive verification.** No window server exists here (`R-GUI-SMOKE`), so nothing in this
  audit claims a visual or screen-reader pass. The a11y statements are about what the code hands to
  AccessKit, not about what a screen reader did.
- No design-level approval of the deferred items: the modal focus lifecycle and the ~30 glyph-only
  controls are deferred with their sites named, not silently dropped.
- The `LIM-*` registry (18 entries) is unchanged: it is tied to the frozen candidate
  `evidence-manifest.json`, so English-only UI and the deferred a11y items are recorded in §F instead
  of being added to a registry that describes the accepted candidate.
