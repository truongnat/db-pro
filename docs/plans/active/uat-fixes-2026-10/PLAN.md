# Plan — UAT Fixes Wave (2026-10)

Branch policy: owner requires direct edits on `main` (AGENTS.md override).
Plan state: PLANNING (implementation starts after scope sign-off; fixes land as small
commits, each with a unit test or runtime evidence).

## Scope (this wave)

| Order | ID | Fix approach |
|---|---|---|
| 1 | UAT-4 truncation indicator | **Premise unverified** — `UiQueryResult` has no `has_more` field; first trace the fetch path (`query_service` → infrastructure `max_rows`) to learn whether truncation is even knowable. If not derivable, surface the applied limit in the status text ("N rows · limit M") or add a backend flag — choose whichever the trace supports. |
| 2 | UAT-1 popover click-through | In `explorer_toolbar_view.rs` popover: capture/dismiss on outside click *consuming* the click (check `egui::Popup` options or manual backdrop area that swallows the press before it reaches rows). |
| 3 | UAT-2 widget ID collision | Find the duplicated id (likely a fixed `ui.push_id`/`Id::new` in table editor per-row/per-view render); make it unique per view/row. |
| 4 | UAT-5 tab strip | On tab activation, `scroll_to_rect` the active tab; ensure × reachable — verify `draw_static_tab` × visibility logic when selected. |
| 5 | UAT-6 status bar binding | `draw_statusbar`: prefer active document's bound connection label; fall back to global. Keep ping/global info if harmless. |
| 6 | UAT-8 `main` label on PG | Context picker: render the connection's actual default schema (`public` for PG) instead of hardcoded `main`. |
| 7 | UAT-10 port copy | Empty port → "Port is required."; keep range message for non-empty invalid. |
| 8 | UAT-9 autocomplete popup | Dismiss completion popup on query execute (check run action clears completion state). Cmd+A may follow. |
| 9 | UAT-12 saved query names | `SAVED QUERIES` rows: display saved name/title if stored; else suffix counter. Investigate why all show "Query 1". |
| 10 | UAT-13 settings exit | Add a "Back"/Close affordance in settings surface header (rail-only exit today). |
## Deferred (recorded, not this wave)
- UAT-15 alertdialog tooltip — **removed from scope**: the owner's follow-up clarified the real complaint was missing header/body/footer structure, already fixed in `64a96f57` (dialog separators).
- UAT-3 context-menu a11y exposure — needs egui accesskit plumbing for floating Areas; larger design question.
- UAT-7 disconnect-on-switch — product decision (single-session vs multi-live).
- UAT-11 toast AX duplication — needs accesskit node audit.
- UAT-14 nav virtualization AX — egui virtualization limitation; workaround = Quick Open (works).
- UAT-16 Copy Cell Value — retest manually without PiP overlay.

## Gates per commit
`cargo +nightly fmt --all -- --check`, `cargo check --workspace`,
`cargo clippy -p db-pro-ui -p db-pro-app --all-targets -- -D warnings`,
focused `cargo test`, runtime screenshot where the fix is visual.
