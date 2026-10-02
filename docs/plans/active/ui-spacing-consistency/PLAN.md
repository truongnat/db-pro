# UI spacing consistency

State: IMPLEMENTING. Baseline: `6c41cc4fc93eb3e12eb9cada2069cd3514e9b3f2`.

Fix the owner-approved spacing audit SP01–SP07 and workspace tab padding. Shared layout/component geometry owns its gaps; no renderer, font or database changes. Keep egui, selection colors, button animation and removal of the workspace tab accent line.

Acceptance: 300pt fields retain width with clear/toggle/chevron controls; standard form fields use the existing 38pt token; grid gutters do not affect cell content; required markers and helpers use explicit semantic gaps; Gallery groups adapt to local width; tabs gain inset and separation without overlapping titles/close targets.

Provider impact: PostgreSQL/SQLite n/a, presentation only. Required quality gates: fmt, check, clippy, workspace tests, native release build. Capture affected normal/error states at three widths; report any display height clamp honestly.
