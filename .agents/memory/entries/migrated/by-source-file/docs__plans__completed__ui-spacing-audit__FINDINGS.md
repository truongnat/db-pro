---
schema_version: 1
id: "MEM-SRC-ec824f601d8224dc"
title: "Source mirror: docs/plans/completed/ui-spacing-audit/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/ui-spacing-audit/FINDINGS.md"
source_sha256: "ec824f601d8224dc53987df4449200d0fb82fcd91a387f6a26a8697ee5470407"
mirror_body_sha256: "ec824f601d8224dc53987df4449200d0fb82fcd91a387f6a26a8697ee5470407"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings / learning pass

Authoritative findings: REPORT.md, SP01–SP07. All are P2; no P0/P1 identified within the audited spacing scope.

Reusable lessons recorded here and in report:

1. egui `add_space` advances a cursor already advanced by `item_spacing`; measure actual bounds rather than reading token names as final gaps.
2. Grid gutter belongs to row/cell placement, not internal component density. Restore child spacing after parent layout overrides.
3. Trailing control reservation includes native sibling spacing. PasswordInput is the existing pattern to reuse for Input/Select.
4. Zero-gap galleries can hide width overflow present in normal parents. Validate shared components in more than one parent-spacing context.
5. Logical points and physical screenshots differ on Retina. Width-only captures with height clamp cannot claim full viewport QA.

No global memory writes. No colors/fonts/renderer changes. No provider mutations.
