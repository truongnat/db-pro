# UAT Fixes Wave — Checklist

Severity scheme follows AGENTS.md: **P0** catastrophic/security/unusable ·
**P1** corruption/unsafe mutation/broken major flow · **P2** UX/maintainability/
edge coverage. (No separate P3 tier in the repo scale — findings labeled P3 in
FINDINGS.md map to the low end of P2 and are listed as minor.)

- [ ] **UAT-4 (P2)** Row-limit cap not disclosed. Trace established that providers
      stop at `max_rows` and set `row_count` to fetched rows without probing an
      extra row; exact-N cannot prove truncation. Disclose the applied query cap
      in completion status and results header, without claiming truncation.
      Files: query dispatch/result reducer, `query_document`, result header,
      and inline-query pagination status.
- [x] **UAT-1 (P2)** Explorer filter-options outside-click dismissal now uses an
      invisible click sink below Tooltip popovers, so the dismissing press does
      not activate rows beneath it. Component-level egui UI test
      `dismiss_scrim_consumes_the_outside_click`
      passes; native dark/light captures inspected at requested 1280×800,
      1440×900, and 1920×1080 (larger heights host-clamped to 838 logical pt).
      Commit: `76ead2690a874eef8f55b5cb710dad5ed56d4aa4`.
- [ ] **UAT-2 (P2)** egui "Second use of widget ID" on table surfaces. Find the
      duplicated `Id::new`/`push_id` in `crates/app/src/table*` or
      `crates/app/src/result_grid*`; make the salt unique per view instance.
      Debug-only overlay but noisy on every capture.
- [x] **UAT-5 (P2)** Workspace tab strip scrolls the newly active tab into view
      only when selection changes, preserving manual scroll on later frames;
      selected tab close × remains reachable. Regression tests cover target
      transitions and actual horizontal overflow. Dark/light native captures
      inspected at requested 1280×800, 1440×900, and 1920×1080 (larger heights
      host-clamped to 838 logical pt). Commit: `a7362d39`.
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
