---
schema_version: 1
id: "MEM-PLAN-explorer-first-tab-redesign-2b442da6e5f6"
title: "Plan archive: Explorer First Tab Redesign"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, explorer-first-tab-redesign]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "2b442da6e5f6e24f3862a3d7a190dc73e5643e76510a22765e7fcf270c1bb454"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/explorer-first-tab-redesign/PLAN.md"
  - "docs/plans/active/explorer-first-tab-redesign/FINDINGS.md"
  - "docs/plans/active/explorer-first-tab-redesign/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/explorer-first-tab-redesign/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Explorer First-Tab Visual Redesign

**State:** RUNTIME_VERIFY  
**Branch:** `feature/explorer-first-tab-redesign`  
**Design baseline:** `docs/design/sidebar-explorer/sidebar-explorer-baseline.md` and `docs/design/sidebar-explorer/sidebar-explorer-feature-research.md`  
**Source baseline:** `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`

## Goal

Make the first sidebar tab (Explorer / Database Navigator) feel like a deliberate, premium native database IDE surface: clear hierarchy, compact but readable tree, purposeful spacing, and consistent interaction states. Preserve existing behavior and use `DbProTheme` semantic tokens.

## Scope

- Explorer toolbar/search/refresh composition.
- Connection, database, schema, folder, and object-row visual hierarchy.
- Empty, loading, error, connected, connecting, disconnected, selected, hovered, and focused states where already represented.
- Native runtime review at 1280×800, 1440×900, and 1920×1080.

## Non-goals

- No new Explorer features from F1–F11 in the research report.
- No provider/runtime/database behavior changes.
- No activity rail, Files, Query, or other tab redesign in this slice.
- No new color token unless the current semantic system cannot express a demonstrated need.

## Design decisions

- Treat the research report as a capability inventory and risk map, not a visual specification or implementation commitment.
- Follow the native migration brief's calm density, layered hierarchy, compact typography, progressive disclosure, keyboard discoverability, and data-first principles.
- Prefer existing native components and spacing conventions; never hard-code colors where theme tokens exist.
- Improve grouping and information hierarchy without adding decorative cards or redundant borders.

## Acceptance criteria

1. Connection and schema hierarchy is scannable within two seconds; connection status, active selection, and expandable structure are distinguishable without relying on color alone.
2. Search and refresh form one compact, aligned toolbar; narrow sidebar widths do not overlap, clip important controls, or create horizontal scrolling.
3. Rows align icon, label, badges, and disclosure affordance; long names remain safely clipped with discoverable full text.
4. Empty/loading/error and connected/disconnected states remain understandable and provide recovery/action where existing behavior allows.
5. Existing Explorer actions, reducers, guards, keyboard shortcuts, and provider capability behavior remain unchanged.
6. Screenshots are captured and visually inspected at all three required resolutions; runtime evidence is recorded honestly.
7. `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the release native build are run and results recorded.

## Provider scope

Pure presentation change; PostgreSQL and SQLite behavior is intentionally unchanged. Any provider-specific state displayed must continue to come from existing capability/state inputs.


## Source: `FINDINGS.md`

# Findings — Explorer First-Tab Visual Redesign

## Baseline review

- `docs/design/sidebar-explorer/sidebar-explorer-baseline.md` is a source inventory, explicitly not runtime evidence.
- `docs/design/sidebar-explorer/sidebar-explorer-feature-research.md` lists product capability gaps F1–F11 and a capability roadmap. It is not a visual redesign spec; implementing those recommendations would expand the user's visual-polish scope.
- Both Explorer documents pin claims to `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`. The inspected current Explorer source files have no changes after that baseline in `HEAD`, so the inventory remains current for this slice.
- The appropriate visual source is `docs/10-egui-native-migration-plan.md` §1.1, together with `DbProTheme`; use the actual theme tokens rather than proposed colors in the historical migration brief.

## Findings

### F-1 — Research roadmap and visual redesign are different scopes

**Severity:** P2 (scope/UX)  
**Evidence:** the feature report's Phase 1–3 proposes object filters, favorites, quick docs, compare, DDL workflows, provider folders, and dependency navigation. These are capabilities, not visual treatment.  
**Decision:** excluded from this feature. Use research only to preserve existing behavior and identify important row states.

### F-2 — Visual design references and current token implementation differ

**Severity:** P2 (design consistency)  
**Evidence:** migration-plan §1.1 describes dark-first graphite with lavender/indigo accent; `crates/ui/src/theme.rs` currently defines light as default, warm-minimalist surfaces, and blue accents.  
**Decision:** do not introduce a competing palette in Explorer. Treat the current `DbProTheme` as canonical for implementation; capture current visual behavior before proposing token-level changes.


### F-3 — Connecting connection is collapsed by default

**Severity:** P2 (state discoverability)  
**Evidence:** the pre-change 1280×800 Explorer capture shows a connecting root row collapsed with only a small status dot. Its `Connecting…` hint is rendered only when the row is open (`explorer_connection_node_view.rs`), while `ConnectionRowContext::draw` defaulted open only for connected/failed states (`explorer_connection_row_view.rs`).  
**Decision:** include `is_connecting` in the row's default-open condition. This exposes the existing progress hint; it adds no action, changes no reducer/provider path, and preserves a user-collapsed row through egui's persistent state.

### F-4 — Active connection had no selected-row treatment

**Severity:** P2 (visual hierarchy)  
**Evidence:** the connection row passed `is_selected: false` for every lifecycle state, even though the connected row represents the active connection.  
**Decision:** render the connected connection as selected, using the existing selected-row token treatment. The current runtime fixture could not reach connected state, so this visual branch remains source-verified only.

### F-5 — Explorer tree ignored mouse-wheel input

**Severity:** P2 (navigation usability)

**Evidence:** a headless test rendered 40 connections through `DbProApp::draw_sidebar`, placed the pointer inside the tree, and sent a vertical `MouseWheel` event. egui received a negative `smooth_scroll_delta`, but the persisted tree offset stayed at zero.

**Root cause:** `sidebar_ui` was constructed with a new `LayerId::Middle` that did not match the SidePanel layer under the pointer. egui's `ScrollArea` only consumes wheel input when its UI layer is the top layer at that point.

**Decision:** reuse the layer id returned by the actual SidePanel when constructing its content UI, then rely on egui's built-in scroll handling. The manual wheel-to-offset workaround was removed.

## Self-review

- **P0:** 0. **P1:** 0. **P2:** 5 (F-1–F-5; F-1/F-2 are scoped dispositions).
- No action variants, reducer mappings, database commands, provider capabilities, or persistence formats changed.
- The `Connecting…` row opens only by default when no persisted disclosure state exists; users can still collapse it.
- The connected-row selected treatment derives from the existing `is_connected` state.
- Runtime review remains incomplete: no connected PostgreSQL schema tree or Explorer error-state capture; the host cannot reach the required 900/1080 logical heights.
- Independent review has not run; the feature remains `RUNTIME_VERIFY`, not lifecycle-completed.

## Provider impact

| Provider | Impact | Runtime requirement |
|---|---|---|
| PostgreSQL | None; presentation-only change | N/A |
| SQLite | None; presentation-only change | N/A |


## Source: `VERIFICATION.md`

# Verification — Explorer First-Tab Visual Redesign

## Source baseline

- Design inventory source SHA: `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`.
- Implementation commit: `a9efb730` (merged to `origin/main`).

## Runtime evidence

The native `db-pro-native` capture harness was exercised with the seeded PostgreSQL connection:

| Requested viewport | Captured framebuffer | State | Artifact |
|---|---|---|---|
| 1280×800 | 2560×1600 (Retina 2×) | Connecting | `screenshots/explorer-before-1280x800.png` |
| 1280×800 | 2560×1600 (Retina 2×) | Connecting, expanded | `screenshots/explorer-after-1280x800.png` |
| 1440×900 | 2880×1676 (window height capped by display) | Connecting, expanded | `screenshots/explorer-after-1440x900.png` |
| 1920×1080 | 3840×1676 (window height capped by display) | Connecting, expanded | `screenshots/explorer-after-1920x1080.png` |
| 1280×800 | 2560×1600 (Retina 2×) | Empty, isolated release data directory | `screenshots/explorer-empty-1280x800.png` |

The pending row is now visibly expanded and renders `Connecting…` without a click; this was
confirmed in all three connecting-state captures. An empty Explorer capture was also collected
from an isolated release data directory. The PostgreSQL fixture timed out in the connection
worker, so these captures do not verify a connected schema tree or database behavior. The host
display caps the native window at 838 logical pixels high; the 1440×900 and 1920×1080 targets were
requested but not achieved. The error state has not been captured.

## Gates

| Gate | Result | Evidence |
|---|---|---|
| Native runtime baseline | PASS — connecting and empty states observed | Before/after + empty screenshots above |
| Explorer captures at requested resolutions | PARTIAL | Host caps the native window at 838 logical px; 1440×900 and 1920×1080 heights not achieved |
| `cargo fmt --all -- --check` | PASS | Executed; exit 0 |
| `cargo test -p db-pro-ui` | PASS — 678 passed | Focused UI crate suite |
| `cargo check --workspace` | PASS | Executed; exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS | Executed; exit 0, no warnings |
| `cargo test --workspace` | PASS — 1327 passed / 0 failed / 41 ignored | Executed; ignored provider/fixture tests are not counted as passing |
| `cargo build --release --locked -p db-pro-native` | PASS | Executed; release profile completed |
| Clean-code scan (`rust --diff --ratchet --ci`) | PASS — 16 checks | 2 changed production Rust files; 0 warnings |

## Mouse-wheel regression

| Command | Before fix | After fix |
|---|---|---|
| `cargo test -p db-pro-ui navigator_tree_scrolls_with_the_mouse_wheel --lib -- --nocapture` | FAIL: `mouse wheel did not advance tree offset: 0` | PASS: 1 passed, 0 failed |

The test exercises the real sidebar render path with 40 connection rows, a pointer over the tree,
and an egui `MouseWheel` event. It verifies the stored ScrollArea offset advances. The native app
was not launched; owner verification on a real build remains pending.

## Post-fix source gates

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test -p db-pro-ui --lib` | PASS — 936 passed, 0 failed, 0 ignored |
| `cargo check -p db-pro-ui` | PASS |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` | PASS |
| `cargo build --release --locked -p db-pro-native` | PASS — built only; not launched |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 16 checks, 0 warnings/failures |
| `git diff --check` | PASS |

## Provider matrix

| Provider | Supported behavior changed | Automated evidence | Live runtime evidence | Capability gate |
|---|---|---|---|---|
| PostgreSQL | No | N/A | N/A — visual-only | Existing capability/state unchanged |
| SQLite | No | N/A | N/A — visual-only | Existing capability/state unchanged |

