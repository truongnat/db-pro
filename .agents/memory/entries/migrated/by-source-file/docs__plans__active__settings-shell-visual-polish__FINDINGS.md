---
schema_version: 1
id: "MEM-SRC-7b7565a03ee2e292"
title: "Source mirror: docs/plans/active/settings-shell-visual-polish/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/settings-shell-visual-polish/FINDINGS.md"
source_sha256: "7b7565a03ee2e292c7428cffa7de4c906274faf107d5cbd3218b846dbb21d48b"
mirror_body_sha256: "7b7565a03ee2e292c7428cffa7de4c906274faf107d5cbd3218b846dbb21d48b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings

## F-1 — Settings was rendered as a sidebar activity, not a settings workspace

**Severity:** P2 (visual hierarchy)

**Evidence:** Before the change, `Activity::Settings` only routed through
`draw_sidebar_activity_body`, while the central panel continued rendering the
active workspace tab. The baseline capture showed Settings controls in the
sidebar beside a Welcome workspace.

**Decision:** Route Settings through the central panel and suppress unrelated
workspace chrome for this mode. Existing settings content and action dispatch
remain the source of truth.

## F-2 — Navigation had no grouped active treatment

**Severity:** P2 (navigation clarity)

**Evidence:** The old navigation used plain `selectable_label` rows with a
fixed 128px width. It did not express the grouped rail + active pill shown in
the supplied references.

**Decision:** Group sections into Workspace, Connections, and System; paint
semantic active/hover fills and Lucide icons using `DbProTheme` tokens.

## Self-review

- P0: 0
- P1: 0
- P2: 2 (F-1, F-2), addressed in this slice.
- No database/provider/runtime behavior changed.

## Provider matrix

| Provider | Impact | Evidence |
|---|---|---|
| PostgreSQL | None; presentation-only | N/A |
| SQLite | None; presentation-only | N/A |
