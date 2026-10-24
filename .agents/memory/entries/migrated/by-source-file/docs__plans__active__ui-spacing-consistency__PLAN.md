---
schema_version: 1
id: "MEM-SRC-d2b564ec54cd0d82"
title: "Source mirror: docs/plans/active/ui-spacing-consistency/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-spacing-consistency/PLAN.md"
source_sha256: "d2b564ec54cd0d82467a218efb586a96f7e97c95610bbbabdad8728e9869c08f"
mirror_body_sha256: "d2b564ec54cd0d82467a218efb586a96f7e97c95610bbbabdad8728e9869c08f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# UI spacing consistency

State: IMPLEMENTING. Baseline: `6c41cc4fc93eb3e12eb9cada2069cd3514e9b3f2`.

Fix the owner-approved spacing audit SP01–SP07 and workspace tab padding. Shared layout/component geometry owns its gaps; no renderer, font or database changes. Keep egui, selection colors, button animation and removal of the workspace tab accent line.

Acceptance: 300pt fields retain width with clear/toggle/chevron controls; standard form fields use the existing 38pt token; grid gutters do not affect cell content; required markers and helpers use explicit semantic gaps; Gallery groups adapt to local width; tabs gain inset and separation without overlapping titles/close targets.

Provider impact: PostgreSQL/SQLite n/a, presentation only. Required quality gates: fmt, check, clippy, workspace tests, native release build. Capture affected normal/error states at three widths; report any display height clamp honestly.
