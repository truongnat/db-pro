# Findings — #489

- Poll cadence was hard-coded `Duration::from_secs(5)` in
  `request_monitoring_poll_if_due`; now `MONITORING_POLL_INTERVAL`.
- Without a wake-up the whole draw path (surface → sessions → confirm dialogs)
  stops: poll, toasts of monitoring results, everything.
- `prepare_frame` already coalesces repaint requests for runtime work; the
  monitoring schedule composes — `request_repaint_after` takes the earliest.
- Clamp floor 50ms avoids a zero-delay busy frame when a poll is due this frame.
- Clock skew safety: `Instant::elapsed()` can panic when `last_poll > now`;
  `monitoring_poll_remaining` uses `saturating_duration_since` and returns ZERO
  for overdue, interval for in-the-future.
