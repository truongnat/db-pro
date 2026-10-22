# Findings ledger — persistent audit memory

Status values: `open`, `in-progress`, `fixed <sha>`, `false-positive`, `deferred <reason>`.
Never reuse an ID. New findings get the next free number. Evidence paths refer to the
audit round directory (`/tmp/ui-audit/...`); a fix without a re-capture stays
`in-progress`.

Wave 3 (2026-10-22): fixes below are implemented in the working tree on top of
`89095cc7` and re-captured in `/tmp/ui-audit/wave2/`; rows read `fixed (uncommitted)`
until the owner commits and the SHA can be filled in.

| ID | Sev | Finding | Anchor | Evidence | Status |
|---|---|---|---|---|---|
| UI-01 | P1 | Explorer Refresh Schema and Filter buttons occupy the same slot; refresh hitbox 88% covered | `crates/app/src/explorer/explorer_toolbar_view.rs:21-53` | 14 WID dumps, `/tmp/ov_crop.png`; fix: `wave2/png/explorer-filter-*` | fixed (uncommitted) |
| UI-02 | P1 | Agent composer content reversed: label at bottom, send button at top | `crates/app/src/agent/agent_surface_view.rs:141-153` | `/tmp/composer_crop.png`; fix: `wave2/png/agent-*` | fixed (uncommitted) |
| UI-03 | P1 | Query schema picker list is not scrollable; later schemas unreachable on multi-tenant connections | `crates/app/src/query_workbench/query_context_picker_view.rs:70-85` | source + 280 px min width; fix: `ScrollArea(180)` + Escape | fixed (uncommitted) |
| UI-04 | P2 | Select wraps itself in `top_down(Min)`; top-aligns in rows and left-anchors in RTL parents | `crates/ui/src/components/select/ui/select.rs:170` | query toolbar 11 px offset; history 76 px void; fix: `wave2/png/query-*,history-*` | fixed (uncommitted) |
| UI-05 | P2 | Select defaults to the light theme; engine select forgets `.theme` and renders white on dark | `select.rs:83`, `crates/app/src/connection/view.rs:247-252` | `new-connection-1440.png`, `newconn_engine_crop.png`; fix: `wave2/png/new-connection-*` | fixed (uncommitted) |
| UI-06 | P2 | Select has no disabled visual; `add_enabled_ui` around agent mode looks enabled | `crates/app/src/agent/agent_header_view.rs:45-63` | source; fix paints `surface_2`/`border_subtle` | fixed (uncommitted) |
| UI-07 | P2 | Menu rows fixed at 28 px while labels wrap; long options overflow | `crates/ui/src/components/select/ui/option.rs:29-32,75-83` | source; fix: `option_label_job` 1-row ellipsis + handler test | fixed (uncommitted) |
| UI-08 | P2 | Menu options expose no widget info; arrow keys mutate selection and Escape does not revert | `select.rs:332-342`, `option.rs` | source; fix: `WidgetInfo::selected` + `MenuKeyAction` tests | fixed (uncommitted) |
| UI-09 | P2 | Select label (medium 12) differs from form Label (12 strong + required) | `select.rs:174-183`, `components/form/ui.rs:37-68` | source; fix: shared `Label` + `LABEL_HELPER_GAP`; scanner contract findings = 0 | fixed (uncommitted) |
| UI-10 | P2 | Form/sidebar selects mix 28/38 heights, 10 literal widths, Ghost/Outline | `schema_workbench_form.rs`, `sidebar_constants.rs:56` | scanner matrix; fix: `SelectSize::Default` in forms, `FILTER_*_WIDTH` tokens | fixed (uncommitted) |
| UI-11 | P2 | Query row caps hardcoded English while table page limits are localized | `query_context_view.rs:154-158` | source; fix: `query.row_limit_*` keys ×5 locales | fixed (uncommitted) |
| UI-12 | P2 | Query context chip is 50 px tall vs 28 px buttons; row-limit select top-aligned 11 px | `query_context_view.rs:56-104,208-273` | `query-light.png`, WID; fix: `wave2/png/query-*` (menu open) | fixed (uncommitted) |
| UI-13 | P2 | Chart selects use non-wrapping `ui.horizontal`; overflow on narrow panes | `query_output_panes_view.rs:106-188` | source; fix: `horizontal_wrapped` + fixed widths | fixed (uncommitted) |
| UI-14 | P2 | New Connection dialog overflows 800 px with hidden scrollbar | `crates/ui/src/components/dialog/frame.rs:20` | `new-connection-1280.png`; fix: `wave2/png/new-connection-*`, `VisibleWhenNeeded` | fixed (uncommitted) |
| UI-15 | P2 | Required host/username lack inline errors; toast omits them | `form_fields.rs:191,248`, `workspace_actions.rs:343` | `connection-error-1440.png`; fix: `wave2/png/connection-error-*` (dark+light) | fixed (uncommitted) |
| UI-16 | P2 | Settings content offsets 80 px to the right only | `settings_surface_view.rs:98` | `settings-1440.png`; fix: `wave2/png/settings-*` symmetric insets | fixed (uncommitted) |
| UI-17 | P2 | Results toolbar mixes 38/28/26/18 px controls in one row | `result_grid_toolbar_view.rs:58-78,103-110` | `results-light-1440.png`; fix: `wave2/png/results-*` | fixed (uncommitted) |
| UI-18 | P2 | History rail reserves 96 px dead space; Copy SQL button clipped | `sidebar_history_view.rs:209-240` | `history-1440.png`; fix: `wave2/png/history-*` | fixed (uncommitted) |
| UI-19 | P2 | Schema Compare has six 0 px control pairs and a 464 px content column | `schema_compare` surface | JSON ×6; fix: `wave2/png/compare-*` | fixed (uncommitted) |
| UI-20 | P2 | 13 one-off dialog widths; destructive dialog uses body buttons and wrong order | `query_dialog_surface_view.rs:21-92`, `delete_dialog.rs:35-69` | source | deferred (dialog width wave) |
| UI-21 | P2 | Status bar truncates text with no tooltip | `shell_statusbar_view.rs:118-123` | `results-light-1440.png`; fix: soft ellipsis + tooltip | fixed (uncommitted) |
| UI-22 | P2 | Quick Open empty/footer hardcoded English; ~417 hardcoded UI strings | `palette_surface_view.rs:80-84,111` | `quick-open-light.png` | deferred (i18n sweep) |
| UI-23 | P2 | Type scale drift: `font_page_title` (24) unused; 16/18/hardcoded titles | `primitive.rs:67-114`, `dialog/ui.rs:294` | source; fix: token removed, font helpers in sheet/empty state | fixed (uncommitted) |
| UI-24 | P2 | Dark `text_muted` on `surface_floating` = 4.09:1 (< 4.5 AA) | `semantic.rs:182-290` | computed; fix: #999999 + WCAG test | fixed (uncommitted) |
| UI-25 | P2 | Diagram zoom % label, canvas and minimap lack accessible names | `diagram_view.rs:1171`, `diagram_canvas_view.rs:97,552` | `diagram.json`; fix: `WidgetInfo` names, `wave2/png/diagram-*` | fixed (uncommitted) |

## Rejected / false positives

| Claim | Why rejected |
|---|---|
| Welcome cards not keyboard accessible | `Sense::click()` includes focus; Enter/Space + focus ring exist |
| Zero-size nodes in dialogs | intentional hidden focus anchors (`dialog/ui.rs:247-272`) |
| Broken fixture screenshots | deliberate scanner fixtures |
| Common Select has no keyboard support | it handles Arrow/Enter/Escape (`select.rs:323-342`); only the bespoke schema picker lacks it |

## Evidence gaps (round 1 + 2)

- Menu-open capture now works: `Select` open state lives in temp data, and the harness
  writes that flag; `query-1440x900-dark.png` shows the row-limit menu open.
- Table workspace and some fixtures returned `unsupported`/Welcome; not visually confirmed.
- No vi/ja locale captures; no 800×600 or scale-factor captures.
- Light-mode hooks exist only for query/results/new-connection/quick-open/welcome/files/
  tools; settings/compare/history/diagram/explorer/agent remain dark-only.
- No screen-reader session for the new accessible names; plumbing only.
- Residual designer-form width literals (120/140/180/220/260) are a P3 follow-up, not
  shared-component drift.
