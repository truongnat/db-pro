# #480 Findings

## Evidence
- `components/select/handler.rs:112-125`: `navigate_selection` applied
  `selected.saturating_add(1) < option_count` to an already-stale index —
  with `selected=5, option_count=2` both guards failed and 5 was returned
  forever ("ArrowDown stuck at 5").
- `ui/select.rs:332-339` writes the return value back to `*self.selected`
  each frame the menu is open → self-healing once clamped here.
- Popup render is safe: options iterated by `enumerate`, icons/disabled via
  `.get(idx)` (`ui/select.rs:365-377`); trigger label via
  `selected_label` → `options.get` (`handler.rs:59-64`).

## Severity
P2 — stuck highlight, wrong "current" option perception; no data corruption.
