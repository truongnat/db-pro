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
