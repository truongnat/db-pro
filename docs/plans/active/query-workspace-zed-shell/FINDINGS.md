# Findings — query-workspace-zed-shell

## Baseline (pre-change)

`draw_query` stacks: header (ComboBoxes + Run + Builder + More) → transaction →
search row → builder collapsing header → editor → floating completion **and**
inline completion card → snippets → diagnostics list → parameters panel →
output tabs → output pane.

Editor height reserves 140–240px for output even when empty.

Shell also has `draw_output_panel` (`bottom_panel_open`) — a second, thinner
output path that does not auto-open on query complete.

## Design decisions

1. Reuse `bottom_panel_open` / `bottom_panel_height` for the Query dock so
   persistence and statusbar toggle stay consistent.
2. Skip shell `draw_output_panel` while `WorkspaceTab::Query` to avoid duplicate
   chrome; Query draws the rich dock itself.
3. Keep floating completion; stop calling inline `draw_sql_completion`.
4. Run lives on the query status strip; More keeps secondary actions.

## Findings (runtime feedback)

- Completion popup rows overlapped: nested `right_to_left` detail inside `horizontal` painted on top of the label. Fixed with fixed-width `add_sized` columns + truncate/tooltip.
- Query status Run could disappear: same nested layout anti-pattern. Fixed with outer `right_to_left` keeping Run pinned.
- Editor stole Enter/Arrow/Page keys while completion open — skip those when `completion_open`.
- Dock close/maximize merged onto the same row as Results/Messages tabs.

## Theme / smoothness pass (goal)

- Flush editor: no rounded card, hairline only when focused; denser line rhythm (`1.48`).
- Dedicated editor tokens: gutter / current-line / selection / line-number colors.
- Dark buffer plane deeper (`#181818`); light buffer flush white with app.
- SQL syntax palette restrained (Zed-like) instead of raw UI accent clones.
- Completion: panel fill contrasts editor; selected row uses `accent_soft`; kind badges in soft pills.
- Context chip: outline, no heavy hover fill.
- Evidence: `screenshots/query-workspace-zed-shell/theme-dark-1280x800.png`, `theme-light-1280x800.png`.

## Perf / lag (P0)

Idle Query tab felt continuous-refresh laggy. Root causes + fixes:

1. **Caret** — `request_repaint()` every frame while focused → `request_repaint_after` only for the next blink toggle (`renderer.rs`).
2. **Lint** — `refresh_diagnostics()` ran sqlparser every `draw_query` frame → cache on `(doc_index, buffer.version(), driver)` and only re-merge `execution_diagnostic` on hit.
3. **Status strip** — `discover_sql_parameters` every frame → cache on buffer version.
4. **Scroll width** — `max_line_len_chars` scanned every line/`chars().count()` every frame → cache on `TextBuffer` rebuild.
5. **Completion** — full `items.clone()` each popup frame → index-based draw; clone only the clicked item.

## IDE-master pass

Further snappiness + chrome density:

1. **Debounced lint** — 180ms quiet window before sqlparser while typing; rematch skipped when execution diagnostic fingerprint unchanged.
2. **Single tokenize** — `reanalyze` reuses `CachedSqlTokens` via `SqlDocumentAnalysis::from_tokens` (was double-tokenize per keystroke).
3. **Prediction default `Subtle`** — quieter ghost text / less idle AI pressure; egress note retained.
4. **Chrome** — drop UTF-8 + shell Ln/Col duplicate; context chip uses warning + chevron icons.
5. **Tab indicator** — `request_repaint_after(16ms)` while animating instead of unbounded `request_repaint()`.
