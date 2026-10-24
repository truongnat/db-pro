---
schema_version: 1
id: "MEM-SRC-ea6460e3c7c839d7"
title: "Source mirror: docs/plans/archived/core-ui-modernization/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/core-ui-modernization/FINDINGS.md"
source_sha256: "ea6460e3c7c839d739f98d8b52cc0e4bfd6d7ce1b84e78275d6540a6293119c3"
mirror_body_sha256: "ea6460e3c7c839d739f98d8b52cc0e4bfd6d7ce1b84e78275d6540a6293119c3"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Core UI Modernization — Findings

Recorded as the work progresses. Each finding links to a concrete file/line or test result.

## F1 — Existing button has no shadow or lift on hover

**Evidence**: `frontend/src/components/ui/button.tsx:9` — only `transition-colors` in the
base classes; no `shadow-*`, no `active:scale-[0.98]`, no icon-shift pattern.

**Failure scenario**: Compared to Linear / Vercel primary CTAs, the current button feels
flat and lacks the "expensive" lift that users associate with premium desktop apps.

**Severity**: P2 (visual polish, not a correctness defect).

**Smallest coherent fix**: Add `shadow-xs` default, `hover:shadow-sm` lift, subtle
`active:translate-y-px` press, and a refined `default` variant that uses the canonical
`--accent` with a subtle gradient overlay (CSS-only via `bg-clip-padding`).

## F2 — Input has no surface tint

**Evidence**: `frontend/src/components/ui/input.tsx:19` — `bg-transparent` baseline; only
focus ring and border change on focus.

**Failure scenario**: Inputs read as "empty rectangles" against the editor surface; no
visual signal that they are interactive until the user clicks them.

**Severity**: P2.

**Smallest coherent fix**: Add `bg-muted/40` baseline that resolves to `--surface-hover`
via the shadcn alias; on focus, swap to a slightly elevated `bg-background` with a
refined focus ring. Same change for `textarea.tsx`.

## F3 — Several common shadcn primitives are missing

**Evidence**: `ls frontend/src/components/ui/` shows 18 files; no `skeleton`, `spinner`,
`kbd`, `separator`, `toggle`, `slider`, `progress`, `sheet`, `command`, `card`.

**Failure scenario**: Every module reinvents loading placeholders and divider styling
inconsistently. This is a known source of visual drift.

**Severity**: P2.

**Smallest coherent fix**: Add the missing primitives using the same `data-slot`,
`cn()`, CVA, and canonical-token patterns as the existing files.

## F4 — Token layer can support more nuanced dark mode without breaking P3.1

**Evidence**: `frontend/scripts/check-token-drift.mjs` pins the LIGHT/DARK snapshots.
Adding new tokens (e.g. `--shadow-xs`, `--motion-spring`) is permitted as long as both
themes define them.

**Severity**: P2 (enabler for F1, F2, F3).

**Smallest coherent fix**: Add the new tokens in BOTH `:root` and `[data-theme="dark"]`,
update the SNAPSHOT maps in `check-token-drift.mjs` in the same commit.

## F5 — Dark mode surfaces are flat

**Evidence**: `frontend/src/styles/globals.css:304-343` — dark `--surface-*` tokens are
solid colors; no subtle elevation gradient.

**Failure scenario**: Modern apps use very subtle radial gradients or noise textures on
the app frame surface to add depth without changing color. The current flat dark mode
looks like stock shadcn.

**Severity**: P2.

**Smallest coherent fix**: Add a `--surface-gradient` token (a CSS gradient string) that
composes existing `--surface-app` / `--surface-panel` / `--surface-editor` values. Apply
it via a single utility class, optional per-screen.
