# UAT Fixes Wave — Checklist

Severity scheme follows AGENTS.md: **P0** catastrophic/security/unusable ·
**P1** corruption/unsafe mutation/broken major flow · **P2** UX/maintainability/
edge coverage. (No separate P3 tier in the repo scale — findings labeled P3 in
FINDINGS.md map to the low end of P2 and are listed as minor.)

- [ ] **UAT-4 (P2)** Row-limit truncation indicator. `UiQueryResult` has no
      `has_more` flag → check runtime `query_service` whether the backend knows
      it clipped (LIMIT n or n+1 strategy). If not derivable, surface the
      applied limit in the status text: "100 rows · limit 100". Files:
      `crates/app/src/query_workbench/query_result_events.rs`,
      `query_status_bar_surface_view.rs`.
- [ ] **UAT-1 (P2)** Explorer filter-options popover outside-click passes
      through to rows. File: `crates/app/src/explorer/explorer_toolbar_view.rs`
      (`egui::Popup` config). Consume the dismiss click or use a backdrop area.
- [ ] **UAT-2 (P2)** egui "Second use of widget ID" on table surfaces. Find the
      duplicated `Id::new`/`push_id` in `crates/app/src/table*` or
      `crates/app/src/result_grid*`; make the salt unique per view instance.
      Debug-only overlay but noisy on every capture.
- [ ] **UAT-5 (P2)** Workspace tab strip: scroll active tab into view on
      activation; keep × reachable for the selected tab. File:
      `crates/app/src/workspace/workspace_tabs_surface_view.rs`.
- [ ] **UAT-6 (minor)** Status bar shows global connection, not active
      document binding. File: `crates/app/src/shell/shell_chrome_view.rs`
      `draw_statusbar` → prefer doc binding.
- [ ] **UAT-8 (minor)** PG context picker shows `main` instead of the real
      default schema (`public`). Find the picker schema label path.
- [ ] **UAT-10 (minor)** Empty port → "Port is required." instead of range
      error. Validation code in connection draft checks.
- [ ] **UAT-9 (minor)** Dismiss autocomplete popup on execute; investigate
      Cmd+A under open popup.
- [ ] **UAT-12 (minor)** Saved queries all render "Query 1" — find the display
      label path (files view) and show stored name or suffix.
- [ ] **UAT-13 (minor)** Settings surface exit affordance (Back/close button).
- [ ] **UAT-15 (minor)** Verify AlertDialog close-× tooltip vs title overlap;
      fix only if trivially reproducible.

## Deferred with reason
- UAT-3 (context-menu AX exposure) — needs egui/accesskit plumbing for
  floating Areas; design-level change.
- UAT-7 (disconnect-on-switch) — product decision, not a code defect yet.
- UAT-11 (toast AX dup) — accesskit node audit, out of scope.
- UAT-14 (nav virtualization AX) — egui limitation; Quick Open is the
  workaround.
- UAT-16 (Copy Cell Value) — retest manually w/o PiP overlay.

## Done when
Each checked item has a commit on `main`, focused tests pass
(`cargo test -p db-pro-ui -p db-pro-app`), clippy/fmt clean, and VERIFICATION.md
lists runtime evidence per visual fix (capture or live drive).
