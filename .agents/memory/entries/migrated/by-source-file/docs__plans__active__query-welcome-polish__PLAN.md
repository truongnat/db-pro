---
schema_version: 1
id: "MEM-SRC-99b50a8bb3a32c3c"
title: "Source mirror: docs/plans/active/query-welcome-polish/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/query-welcome-polish/PLAN.md"
source_sha256: "99b50a8bb3a32c3c9975c44a6ab7ab04550eab30920165f3cc143303ecd978f8"
mirror_body_sha256: "99b50a8bb3a32c3c9975c44a6ab7ab04550eab30920165f3cc143303ecd978f8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Query Select and Welcome polish

State: RUNTIME_VERIFY. Owner requested UI changes on 2026-10-06. Main checkout per owner override; baseline SHA 3dd988e2ff06ac71942c7ec331acf66389d3c5cb, previous uncommitted Data Grid/footer work preserved.

Scope: replace Query row-cap legacy ComboBox with shared Select; preserve limits and next-run policy. Reduce Welcome to a centered introduction with product name, description and supported providers. Remove its duplicate SQL composer, starter examples, connection cards, action list, draft row and only their private state/handlers. Shell actions remain the existing entry points. No SQL/service/provider change, no new dependencies.

Architecture: native egui; use DbProTheme and existing typography/spacing tokens. Shared Select owns popup styling, selected state, keyboard and dismissal. Public size and variant customize trigger context: Default/Sm and Outline/Ghost; Query uses Sm/Ghost and existing forms retain default configuration. Query adapter maps option index to Option<u64>; unknown custom caps remain unchanged until a user chooses another option. Welcome is static presentation without mutation or connection state.

Acceptance: light/dark popup uses semantic selected text/background with checkmark; four labels preserve values. Welcome remains simple and centered at three requested viewport widths. No inaccessible hidden actions, no obsolete Welcome draft state. Native captures needed; feature cannot complete with pending exact-height or interactive runtime evidence.
