# Verification — #489

- `monitoring_poll_remaining` unit test: None→0, fresh→5s, 2s elapsed→3s,
  stale→0, future timestamp→saturates (no panic).
- `include_str!("../../app/app_lifecycle.rs")` test pins that prepare_frame
  calls `monitoring_poll_remaining` + `request_repaint_after` — the exact code
  path that wakes egui while idle.
- `request_monitoring_poll_if_due` now shares the same helper, so due-ness and
  the scheduled wake can never disagree.
- Runtime visual proof intentionally not required (per issue); logic + code
  path are covered.
