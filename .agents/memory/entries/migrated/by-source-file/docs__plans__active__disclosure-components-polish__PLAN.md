---
schema_version: 1
id: "MEM-SRC-663bbec7e31f9a84"
title: "Source mirror: docs/plans/active/disclosure-components-polish/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/disclosure-components-polish/PLAN.md"
source_sha256: "663bbec7e31f9a84a7d51c5d28f54827b07dbffd05011245364285e794cbaf01"
mirror_body_sha256: "663bbec7e31f9a84a7d51c5d28f54827b07dbffd05011245364285e794cbaf01"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Collapsible & Accordion UI Polish

## State

`RUNTIME_VERIFY`

## Goal

Make Collapsible and Accordion read as one DB Pro disclosure family: quiet at
rest, surfaced on hover/open, aligned body content, and one predictable
expand/collapse animation.

## Scope

- Native egui Collapsible and Accordion rendering.
- Shared disclosure surface, color, spacing, focus, chevron, and body animation.
- Component-gallery capture for the affected state.

## Non-goals

- No public builder/API removal.
- No database/provider behavior.
- No React/Tauri frontend changes.
- No unrelated component-gallery redesign.

## Acceptance

1. Closed disclosure headers have no persistent fill; hover and open states use
   semantic theme surfaces.
2. Collapsible and Accordion share header geometry, body inset, focus treatment,
   and chevron transition.
3. Body expansion clips height and fades content without reserving the full
   expanded height during the transition.
4. Existing single/multi selection, disabled behavior, keyboard activation, and
   public APIs remain intact.
5. Native capture and Rust gates are recorded before completion.

## Provider impact

Presentation-only. PostgreSQL and SQLite behavior are unchanged.
