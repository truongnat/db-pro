---
schema_version: 1
id: "MEM-SRC-4441871d57c76f07"
title: "Source mirror: docs/plans/active/component-gallery-redesign/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/component-gallery-redesign/FINDINGS.md"
source_sha256: "4441871d57c76f07f0d0449326008b2fce49b9a045320a87d681b1b22158256d"
mirror_body_sha256: "4441871d57c76f07f0d0449326008b2fce49b9a045320a87d681b1b22158256d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Component Gallery Redesign Findings

- **Baseline SHA:** `7bbfb08e289fa2b77867d40feb0d886dbc097d31`
- **Evidence level:** source inspection only until `VERIFICATION.md` records runtime captures.

## P1

### P1-1 — Category navigation clips at normal widths

`crates/ui/src/component_gallery_view.rs:285-309` places fourteen labels in one non-wrapping `SegmentedTabs` row. The component has no horizontal scroll or wrap behavior. Categories on the right become unreachable when the workspace is narrower than the track.

**Decision:** use a vertical category rail with one selected detail surface.

### P1-2 — Default `All` mode renders every complex demo

`crates/ui/src/component_gallery_view.rs:315-392` renders all thirteen sections when `GalleryCategory::All` is selected. This creates a long, expensive immediate-mode surface and weakens information hierarchy.

**Decision:** remove `All`; select one category by default.

## P2

### P2-1 — Gallery surface contradicts workstation design direction

Gallery sections rely heavily on `Card::new`. At the baseline, `crates/ui/src/components/card.rs:15-28` uses 16px padding, 8px radius and a drop shadow. `docs/DESIGN.md` specifies planes/separators, 12px panel padding and 4px radius for dense controls/cards.

**Decision:** make Card itself conform to the shared semantic contract: `CARD_INNER_PAD`, `RADIUS_CARD`, no decorative shadow.

### P2-2 — Header and naming are inconsistent

The same surface is called `Components`, `Component Gallery`, `Common UI Design System`, and `Component Gallery (UI Design System)` across workspace, status, palette and gallery header.

**Decision:** canonical user-facing name is `Component Gallery`; supporting copy is `DB Pro workstation primitives`.

### P2-3 — Typography and spacing bypass tokens

The gallery header and section headings use hard-coded font sizes and several off-grid spacing values.

**Decision:** route primary hierarchy through token font helpers and spacing constants. Individual component samples may retain sizes that intentionally demonstrate component variants.

### P2-4 — Badge primitive diverges from the Stitch operational-status specimen

At the baseline, `Badge` uses fully rounded pills and most semantic variants have no border. The canonical component screen (`docs/design/stitch_screens/19_db_pro_component_library.html:878-897`) specifies compact 4px status badges with a visible boundary and the operational sequence Ready / Running / Succeeded / Failed.

**Decision:** use `RADIUS_BADGE`, a 1px semantic boundary, 6px leading dot, 12px icon, and the canonical execution-status composition. Remove the unrelated avatar demonstration from this section.

## Provider impact

- PostgreSQL: n/a — provider-independent design-system surface.
- SQLite: n/a — provider-independent design-system surface.

## Existing coverage

No dedicated Component Gallery capture route or focused state/layout test was found at the baseline SHA.
