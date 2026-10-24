---
schema_version: 1
id: "MEM-PLAN-component-gallery-redesign-f179ab9e59f7"
title: "Plan archive: Component Gallery Redesign"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, component-gallery-redesign]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "f179ab9e59f78bfd4b46f75bf5603d60e84d060499250822cc60609af5d20518"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/component-gallery-redesign/PLAN.md"
  - "docs/plans/active/component-gallery-redesign/FINDINGS.md"
  - "docs/plans/active/component-gallery-redesign/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/component-gallery-redesign/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Component Gallery Redesign

- **State:** RUNTIME_VERIFY
- **Branch:** `feature/component-gallery-redesign`
- **Baseline SHA:** `7bbfb08e289fa2b77867d40feb0d886dbc097d31`
- **Surface:** native `egui` Component Gallery only

## Problem

The current gallery uses a single unscrollable 14-item segmented tab row, renders every complex sample in the default `All` view, and presents the component system as rounded shadow cards. This clips navigation at normal workspace widths, creates an unnecessarily heavy immediate-mode page, and contradicts the DB Pro workstation direction in `docs/DESIGN.md`.

## Decision

Reshape the gallery as a restrained workstation browser:

- persistent vertical category rail;
- one selected category rendered at a time;
- compact 36px utility header with canonical naming and actions;
- flat, token-driven planes and separators;
- shared typography/spacing/radius tokens;
- deterministic native capture entry point.

Keep all existing interactive component demos. Do not create a second theme or component implementation.

## Scope

1. Replace horizontal category tabs and `All` mode with reachable vertical navigation.
2. Standardize gallery title, category naming, section hierarchy and spacing.
3. Bring the shared Card primitive into the documented 4px, shadow-free workstation contract.
4. Add a Component Gallery capture hook.
5. Verify through the existing UI suite and native captures; do not add tautological wiring tests.
6. Capture the affected surface at 1280×800, 1440×900 and 1920×1080.

## Non-goals

- Rewriting individual component behavior.
- Adding new database/provider operations.
- Replacing the result grid or Explorer implementations with gallery-only samples.
- Redesigning archived frontend assets.
- Claiming PostgreSQL or SQLite runtime behavior; this surface is provider-independent.

## Architecture

`WorkspaceTab::ComponentGallery` remains the routing contract. `ComponentGalleryState::category` owns selection. A fixed navigation rail emits category changes; the detail pane owns vertical scrolling and renders only the selected section. `DbProTheme` and `tokens.rs` remain the only visual-token sources.

## Acceptance criteria

- Every gallery category is reachable at 1280px viewport width without horizontal clipping.
- Only the selected category's component section is rendered.
- Header and navigation use canonical `Component Gallery` naming and Lucide icons; no decorative emoji.
- Gallery layout uses theme surfaces, 1px separators, spacing tokens and 4px dense-surface radius.
- Shared Card has no decorative shadow and uses semantic card radius/padding.
- Capture harness can open the gallery deterministically.
- Focused tests, workspace check/clippy and native release build outcomes are recorded honestly.
- Native screenshots exist for 1280×800, 1440×900 and 1920×1080, or the exact runtime blocker is recorded.


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# Component Gallery Redesign Verification

- **State:** RUNTIME_VERIFY
- **Baseline SHA:** `7bbfb08e289fa2b77867d40feb0d886dbc097d31`

## Automated gates

Executed from the repository root:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo test -p db-pro-ui --lib` | PASS — 679 passed / 0 failed / 0 ignored, exit 0 |
| `cargo check --workspace` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo build --release --locked -p db-pro-native` | PASS, exit 0 |
| `cargo build -p db-pro-native --features capture` | PASS, exit 0 |
| `cargo test -p db-pro-ui badge --lib` | PASS — 5 passed / 0 failed / 0 ignored, exit 0 |
| `cargo check -p db-pro-ui` | PASS, exit 0 |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` | PASS, exit 0 |

## Native runtime evidence

The capture build opened `WorkspaceTab::ComponentGallery` through `DB_PRO_CAPTURE_COMPONENT_GALLERY=1`. The general captures show the complete vertical category rail and flat token-driven cards. The focused badge capture opens the corrected Badges & Status Pills category directly.

| Requested viewport | Captured framebuffer | Artifact | Result |
|---|---:|---|---|
| 1280×800 | 2560×1600 Retina | `screenshots/component-gallery-1280x800.png` | PASS — every category is visible; no horizontal category clipping |
| 1440×900 | 2880×1676 Retina | `screenshots/component-gallery-1440x900.png` | PARTIAL — artifact is valid, but macOS capped logical height at 838px and the capture process timed out after writing it |
| 1920×1080 | 3840×1676 Retina | `screenshots/component-gallery-1920x1080.png` | PARTIAL — artifact is valid, but macOS capped logical height at 838px |
| 1280×800 badge focus | 2560×1600 Retina | `screenshots/component-gallery-badges-1280x800.png` | PASS — canonical Ready / Running / Succeeded / Failed sequence, 4px shape and semantic boundaries are visible |

The Component Gallery is provider-independent and has no database loading/error/empty lifecycle. The selected component samples expose disabled and loading states. PostgreSQL and SQLite runtime verification are not applicable.

## Review outcome

- **Verdict:** ACCEPT WITH P2 for the implementation working tree based on baseline `7bbfb08e289fa2b77867d40feb0d886dbc097d31`.
- **Introduced findings:** P0 0 / P1 0 / P2 0.
- **Inherited/environment findings:** P0 0 / P1 0 / P2 1 — macOS limits the 1440×900 and 1920×1080 requests to 838 logical pixels of height; the 1440 capture command wrote a valid PNG but did not self-close before timeout.
- **Independent review:** not run.

## Known limitations

- The 1440×900 and 1920×1080 artifacts prove the rendered width but not the requested logical height because of the host window-height cap.
- Runtime evidence covers the default Buttons category and the corrected Badges category. Other categories were not separately captured.

