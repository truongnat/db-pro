---
schema_version: 1
id: "MEM-PLAN-component-layer-structure-bca83379ca4d"
title: "Plan archive: Component Layer Structure"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, component-layer-structure]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "bca83379ca4d50cfe61607cd98091fdf2403a8ddfd5c748a417e5c871b4bb6ba"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/component-layer-structure/PLAN.md"
  - "docs/plans/active/component-layer-structure/FINDINGS.md"
  - "docs/plans/active/component-layer-structure/VERIFICATION.md"
  - "docs/plans/active/component-layer-structure/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/component-layer-structure/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Component UI Layer Structure

## State
- **State:** RUNTIME_VERIFY
- **Branch:** `main`
- **Baseline SHA:** `a2ded17c839724dc95fe8f0ed8e9a8f07a4e50dd`
- **Surface:** `crates/ui/src/components/` native egui component library.

## Goal
Organize every public UI component into a predictable folder with a single `mod.rs` entry/exporter, UI and handler layers, component-local `config.rs`, and separate design, usage, and API documentation. Shared design tokens and common UI primitives stay centralized and are reused by components.

## Required component shape
```text
<component>/
  mod.rs       # public entry point; declares modules and re-exports public API
  ui.rs        # egui drawing/layout; use ui/ for distinct UI parts
  handler.rs   # behavior and calculations; use handlers/ for distinct decision groups
  config.rs    # component-local defaults only; no core imports or mirrored core constants
  DESIGN.md    # UI design rationale and logic/event/performance design
  README.md    # short human-facing introduction and example
  API.md       # detailed developer/AI usage contract
```

`mod.rs`, `config.rs`, `DESIGN.md`, `README.md`, and `API.md` are required. The UI and handler layers may each be a single file or a directory with private submodules. Keep handler code meaningful: move behavior/decision code out of UI; do not create placeholder boilerplate. `config.rs` holds only component-owned values; consumers reference `tokens.rs`/`DbProTheme` directly where needed. Existing components migrate incrementally, starting with Button.

## Scope
- Migrate the complete public component surface exported by `crates/ui/src/components/mod.rs`, including already-multi-file components (`dialog`, `form`, `input`, `select`, `tabs`).
- Keep shared infrastructure (`animation`, `interact`, shared tokens/helpers, legacy compatibility facade) centralized rather than pretending they are standalone UI components.
- Preserve current public exports and runtime behavior during each migration.
- Add `README.md`, `DESIGN.md`, and `API.md` for every public component folder as each component is revisited. Keep the README introductory, the design rationale in DESIGN, and the detailed usage contract in API.
- Migrate in coherent batches on `main` under the owner workflow override, with compile/test checks when validation is requested. Select was the first completed family in the original migration; Button is the first family under this revised documentation contract.
- At this baseline, 38 components are exported. Button already has `README.md`, `DESIGN.md`, and `API.md`; the other 37 lack the two new contract documents, and `tabs`/`transaction` also lack `README.md`.

## Non-goals
- UI redesign, changes to behavior/API beyond necessary module path updates, and changes to database/runtime layers.
- A generic component framework, empty handler/config placeholders, or duplicated design tokens.

## Implementation guide

Use the durable [component authoring guide](../../../crates/ui/src/components/README.md) and migrated `Select` component as references when creating or migrating a component:

1. Inspect the current public API, callers, tests and shared UI patterns before moving files. Record the observable interaction contract (including focus, keyboard/mouse behavior, dismissal and accessibility identity where applicable).
2. Choose a stable caller-facing API, then make the component's `mod.rs` its public entry point. Preserve the expected `components/mod.rs` re-exports and existing behavior; update callers only when module paths require it.
3. Keep egui widget construction, layout, geometry acquisition and painting in `ui.rs` or `ui/`. Move meaningful decisions, state transitions, validation and UI-independent calculations into typed `handler.rs` or `handlers/` functions. Pass measurements/signals explicitly and have UI apply handler outcomes; do not move painting into handlers.
4. Put only component-owned defaults/constants in `config.rs`; it has no core imports. UI and handlers use shared `DbProTheme` and `tokens.rs` values directly. Give non-obvious factors/numbers semantic names. If a layer has no meaningful responsibility, record the exception rather than adding placeholder code.
5. Add focused tests for handler boundaries/state transitions. Write `README.md` for basic use, `DESIGN.md` for UI and logic rationale, and `API.md` for the full developer/AI usage contract.
   Comment non-obvious Rust flow in English beside the code, tracing caller inputs and egui events through handler decisions/state to UI rendering and returned actions. Explain precedence and intent, not individual statements.
6. Migrate one coherent component/family at a time. Review exports, callers, behavior and diff for drift; run formatting and the smallest relevant crate test/check, then applicable broader gates. Record only commands actually completed and distinguish automated evidence from runtime evidence in `VERIFICATION.md`.

Example layering: `select/ui/option.rs` obtains the text bounds and galley size from egui, then calls `option_galley_pos(left, vertical_center, galley_height)`; `select/handler.rs` performs the pure position calculation using `VERTICAL_CENTER_FACTOR` from `config.rs`.

## Acceptance criteria
- Every public component gains the required folder/layers and three documents as it is revisited; Button is the first component under the revised contract.
- `mod.rs` is the only public component entry/exporter and public API remains compatible through `components` re-exports.
- UI files focus on egui rendering/layout and call typed handler functions for behavior/calculation.
- Component-specific settings are kept in config; shared spacing/colors use canonical tokens/theme.
- Light and dark appearances, component states, typography, spacing, and semantic colors remain consistent with the core theme.
- Interactive controls expose a meaningful name, role, state, keyboard activation, and visible focus; check contrast, target size, reduced motion, and accessibility-tree output.
- User-facing actions and loading/disabled/error/empty/narrow states remain understandable without color or motion alone.
- Shared visual changes come from theme/core tokens, while supported component variants remain easy to configure without duplicated values.
- Rust component refactors include detailed English comments for non-obvious logic and UI flow, explaining behavior step by step (inputs/events → handlers/state/outcomes → rendering). Comments focus on intent and rationale, not merely restating code.
- Rust formatting, workspace checks, clippy, tests and native release build are executed and recorded.

## Decisions / risks
- Do not move pure UI rendering to handlers; egui paint/layout remains in `ui.rs`. Handler owns decisions and state transitions, even if UI passes egui input signals and applies returned actions.
- Large folders may require additional private submodules below the named layer files, but only `mod.rs` is the public entry.
- Keep migrations staged so a broad filesystem refactor does not hide behavior regressions.

## Provider impact
N/A — UI architecture only; no PostgreSQL/SQLite behavior changes.

## Tổng kết bằng tiếng Việt
Phạm vi là toàn bộ component công khai, không dừng ở Select. Mỗi component có `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`; Select là nhóm đầu tiên. Phần hướng dẫn triển khai component mới dùng Select làm ví dụ, nhấn mạnh giữ API, tách phép tính thuần khỏi egui, tái sử dụng token/theme và kiểm chứng đúng mức.


## Source: `FINDINGS.md`

# Component Layer Structure — Findings

## Button quality self-review — 2026-10-01

Scope: Button source and core token inputs at baseline SHA `7423aea985fd878f447e229a968eac30cf3868f5`, plus the current uncommitted changes on `main`. This is an implementer self-review with focused tests; native UI, accessibility tree, and rendered-pixel contrast remain unchecked. The defects below were inherited from the baseline Button behavior.

| Requirement | Static result | Evidence and remaining check |
|---|---|---|
| Consistent appearance | Partial | Variants consume `DbProTheme` in `button/handlers/palette.rs`; sizes consume the core contract in `button/handlers/size.rs`. Light/dark and all states still need native visual review. |
| Accessibility | Partial | Filled-button contrast passes the 4.5:1 token check through rest/hover in both themes. Reduce motion reaches Button. Icon-only names are enforced in all builds. Focus appearance, Enter, and the native accessibility tree still need runtime review. |
| Approachable UI/UX | Partial | Loading and disabled block click. `ButtonGroup` now uses a 4-point shared gap, leaving room for adjacent 2-point focus rings. Narrow labels and loading comprehension require runtime review. |
| Dynamic customization | Partial | Theme is supplied to each Button and core tokens own shared dimensions. Variants and size presets are configurable; arbitrary per-instance styling is not exposed. Whether that is needed remains a product decision. |
| Clean code and formatting | Partial | `mod.rs` keeps the public API, UI/handlers are separated, and `config.rs` has no core imports. Focused tests, formatting, UI crate check/Clippy, and clean-code scan pass; the scan reports two warning categories, recorded in `VERIFICATION.md`. |

### Fixed inherited P1 — Filled-button text contrast

At the baseline SHA, `button/handler.rs:95-129` uses white foreground for Default and Destructive text. The declared tokens give white on light accent `#0285ff` **3.62:1**, white on dark accent `#4f8cff` **3.22:1**, and white on dark danger `#ef6b73` **2.99:1**. `DbProTheme::text_on_solid` now chooses black or white from the resolved fill. A focused test checks 101 hover steps for both filled variants in both themes; every measured pair reaches 4.5:1. Rendered-pixel evidence is still pending.

### Fixed inherited P1 — Reduce motion did not reach Button

`settings_appearance_view.rs` already offers `Reduce motion`. The preference now travels through the frame's `DbProTheme`; Button skips hover/press interpolation and holds the loading arc still without requesting spinner repaints. Runtime confirmation remains pending.

### Fixed inherited P2 — Icon-only name depended on debug-only enforcement

The icon-only `access_label` requirement now uses a normal assertion, including a nonblank check, so invalid controls cannot silently render in release. A source scan found no product call site without text or `access_label`; the only match is the intentional panic test. The native accessibility tree still needs review.

Review verdict: **ACCEPT WITH P2** for source changes (P0=0, P1=0; P2=1 for native visual and accessibility-tree evidence). This is implementer self-review, not independent approval. PostgreSQL and SQLite: n/a.

## Input batch (implementation)
- P2 — Input presentation was spread across sibling modules without a component-level presentation/decision boundary. Added `ui.rs` as the presentation entry, `handler.rs` for pure label/counter decisions, and retained existing widget APIs.
- P2 — Textarea's byte-length counter disagreed with user-visible character semantics for Unicode. The counter now uses `chars().count()` through a tested handler function. Runtime visual/accessibility evidence remains pending.

## Baseline
- Historical migration baseline: `2fb54db0` (`main` before the earlier component-layer migration).
- Current continuation baseline: `a2ded17c839724dc95fe8f0ed8e9a8f07a4e50dd` on clean local `main`, matching `origin/main` when work began.
- Current inventory: 38 public components; Button has all three contract documents. The remaining 37 need `DESIGN.md` and `API.md`; `tabs` and `transaction` also need `README.md`. Existing checklist marks cover the earlier structure and do not count as acceptance of these documents.
- Select previously mixed egui rendering with popup keyboard/state decisions, accessible-label construction, and width calculations.
- Most exported components are standalone `.rs` files; complex families already use subdirectories but inconsistent file boundaries.

## Revised contract — Batch 1
- At baseline `a2ded17c839724dc95fe8f0ed8e9a8f07a4e50dd`, AspectRatio, Badge, Card, Code, DevTools, and Separator already had the intended UI/handler/config module shape and README; this batch added the missing design/API contracts without manufacturing more layers.
- Removed Badge's private `BADGE_RADIUS` mirror and Separator's private shared-spacing mirrors; rendering now reads `RADIUS_BADGE` and `SPACE_SM` directly from shared tokens.
- Added Gallery coverage for AspectRatio clipping/ratio and both separator orientations. Static scoped review found no P0/P1; required native viewport/accessibility evidence is still P2 and remains open.

## Decisions
- `mod.rs` is the public component entry/exporter.
- Every public component is in scope and must have the named layers plus a usage README.
- UI owns egui rendering/layout and applies egui side effects; handler owns decision logic and calculations called by UI.
- Config owns component-specific defaults; shared design tokens/theme stay centralized.
- Migrate in batches to keep behavior and API stable; empty handler/config files are not accepted, so implementation must move meaningful responsibilities into them.

## Findings
- P2 — Select mixed UI and logic. Its keyboard navigation, trigger/accessibility labels, widths, option selection, dismissal, load-more decisions and dropdown geometry now live in typed handler functions.
- P2 — AspectRatio mixed ratio validation and size calculation into its egui allocation method. The pure decisions now live in typed handler functions with regression tests; UI still owns allocation, clipping and child rendering.
- AgentComposer UI review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): P1 blank keyboard submission bypassed prompt validation; fixed by shared handler decision. Prompt label and wrapped toolbar added. P2 native narrow-width/accessibility evidence remains pending.
- AgentPrimitives UI review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): initial P1 removable-chip hit target, destructive approval hierarchy and long-title overlap were fixed; disclosure headers gained keyboard/focus/accessibility semantics and unconditional repaint removed. Follow-up reports P0=0/P1=0, with P2 runtime interaction/layout evidence pending.
- Alert UI review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): initial P1 destructive-by-default/backdrop dismissal, narrow-width sizing and modal identity/focus risks were addressed by opt-in destructive mode, backdrop policy, Escape, Cancel focus request, id salt and width-aware layout. Follow-up reports P0=0/P1=0; P2 focus trap/restoration and runtime verification remain open.
- Calendar review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): P1 month/day normalization, lost view navigation state, and disabled popup mutation were fixed; current-local-date default, selection close and Escape close were added. Follow-up review reports P0=0/P1=0, with P2 egui interaction/accessibility and popup placement evidence pending.
- Database source review at implementation SHA `5ff99485b3205dc58369714e2c76ea31c9e51bb5`: initial P2 long-text overflow and repeated Connect while Connecting were fixed with shrink-aware labels/tooltips and disabled pending action; Error now labels recovery Retry while preserving the Connect action. Follow-up ACCEPT WITH P2 (P0=0/P1=0): extremely narrow cards may still be narrower than fixed status/SSL affordances; runtime viewports remain unverified.
- No provider/database backend impact; this is native UI architecture work.

## Toggle and ScrollArea batch
- P2 — Toggle previously combined size presets, width/appearance decisions, grouped corner geometry, and selection transitions with egui allocation and painting. The refactor moves those typed decisions and caller-owned click mutations into `toggle/handler.rs`, keeps egui galley measurement and painting in `toggle/ui.rs`, and preserves the caller-facing builders and re-exports.
- P2 — ScrollArea previously encoded the egui axis tuple directly in its presentation layer. Axis ordering is now a tested handler decision; scrollbar visual mutation remains a handler-owned save/restore lifecycle and the clip-margin value is named in local config.
- Source-only review at baseline SHA `6cedac0a0ff0492133d101f8272c0b801fd2e28f` plus the uncommitted batch: no P0/P1 found. Automated crate checks are blocked by pre-existing missing `form/{config,handler,ui}.rs` and `input/{handler,ui}.rs` modules outside this scope. Native runtime evidence remains pending.

## Unresolved
- Inventory each public component's meaningful handler/config responsibilities during its batch; do not duplicate shared settings or fabricate placeholder logic.
- Batch 10 source/docs implementation is recorded at `6cbc35d3001c99a027b8033a0a048354eeffc8c7`; the 38-folder contract inventory now has no missing README/DESIGN/API files. No source P0/P1 was found in this batch.
- P2 — Gallery runtime coverage is partial: 85 exact-size PNGs out of 90 requested captures, two 1920×1080 captures have smaller framebuffers, and three 1920×1080 theme/category captures are missing. Native keyboard/focus/accessibility-tree evidence is also open; screenshots and source metadata do not prove runtime traversal.
- P2 — The performance scan reports a 51.0 MB native release binary against the 50 MB target (100 MB critical threshold). It skipped optional benchmarks/runtime sections; no performance claim is made.
- Explain review at source SHA `fc1c4f815887c25696bbad13fd72bc9d6b1ae17d`: ACCEPT, introduced/inherited P0/P1/P2 all 0. Previous per-loop correctness and deep/wide traversal findings were fixed in commits `64096e67`, `49791a98`, and `fc1c4f81`; truncation findings persist across repeated heuristic application and render once.
- Explain provider scope: current query-output parser is PostgreSQL EXPLAIN JSON; SQLite EXPLAIN normalization remains unsupported/pending and was not changed. Required runtime screenshots/accessibility evidence and workspace-wide gates remain open plan-level items.
- Feedback was already organized into the five required layers before this batch; the inventory omission was stale. Source follow-up at `7a119f968ca2c4c14f21ded5fb2e9059de525583` sanitizes progress inputs, adds ProgressIndicator accessibility values/labels, and guards beam-edge overflow. Initial review P2s were fixed; final source review ACCEPT with introduced/inherited P0/P1/P2 all 0. Runtime screenshots/accessibility tree evidence remains pending.

## Common layer continuation on `main`
- P2 — `common_utils.rs` was a mixed shared-support bucket containing layout geometry and display formatting. The implementation is now split into named `common/layout.rs` and `common/format.rs` modules, with explicit root exports and a compatibility facade so existing callers do not break.
- Scope is UI-only; no PostgreSQL/SQLite/provider behavior changed. Automated source checks pass; native runtime screenshots/accessibility traversal remain pending at the initiative level.

## Revised contract — Batch 2
- `chrome`, `feedback`, `logs`, `nav`, `responsive_layout`, and `scroll_area` already had meaningful UI/handler boundaries; this batch completed the design/API contract without adding placeholder modules.
- P1 — Skeleton pulse/shimmer, i

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# Component UI Layer Structure — Verification

## Revised contract — Batch 1
- Source baseline SHA: `a2ded17c839724dc95fe8f0ed8e9a8f07a4e50dd`; implementation SHA: `a3a069bc8e249a83923997cb3dd5426dde581bf3` (`refactor(ui): document core component batch`).
- Added `DESIGN.md` and `API.md` for AspectRatio, Badge, Card, Code, DevTools, and Separator. Their existing module boundaries and README files were already sufficient. Added Gallery samples for aspect-ratio clipping and horizontal/vertical separators.
- Removed Badge's private `BADGE_RADIUS` mirror and Separator's `DEFAULT_HORIZONTAL_MARGIN`/`LABEL_PADDING` mirrors; shared values now come from `RADIUS_BADGE` and `SPACE_SM` in `tokens.rs`. No public re-export or caller used these config-only constants.
- `cargo fmt --all -- --check`: PASS (exit 0; first check showed only rustfmt import wrapping and was corrected with `cargo fmt --all`).
- `cargo test -p db-pro-ui --lib`: PASS (931 passed, 0 failed, 0 ignored; exit 0).
- `cargo check -p db-pro-ui`: PASS (exit 0).
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS (exit 0).
- `cargo build --release --locked -p db-pro-native`: PASS (exit 0).
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: PASS (15 pass, 1 warning, 0 fail; exit 0).
- `git diff --check`: PASS (exit 0).
- Review outcome: self-review only; no P0/P1 identified. P2 remains for native screenshots and keyboard/accessibility inspection at required viewports/states. No PostgreSQL/SQLite or backend behavior changed.
- Runtime screenshot/accessibility evidence: NOT RUN. Initiative remains `IMPLEMENTING`; this batch does not satisfy final runtime gates.

## Revised contract — Batch 2
- Source baseline SHA: `aa8c51b7d579d95350b28b9e06f654edfe37ccdc`; implementation SHA: `5b45be613bb29f0abc92ec6e10cd6de5c486bf36` (`fix(ui): honor reduced motion in feedback`).
- Added `DESIGN.md` and `API.md` for Chrome, Feedback, Logs, Nav, ResponsiveLayout, and ScrollArea. No placeholder layers were needed.
- Routed Skeleton, indeterminate Progress, and Spinner animation through `animation::should_animate`; reduced motion now freezes pulse/shimmer, beam motion, and spinner rotation. Added tests for the shared motion gate and static beam geometry.
- Removed Chrome radius mirrors, Feedback radius/stroke mirrors, and Logs' renderer token mirrors. No in-repository callers used the removed aliases. Feedback and Logs API documents identify the removed public config aliases and canonical token replacements.
- `cargo fmt --all -- --check`: PASS (exit 0).
- `cargo test -p db-pro-ui --lib`: PASS (933 passed, 0 failed, 0 ignored; exit 0).
- `cargo check -p db-pro-ui`: PASS (exit 0).
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS (exit 0).
- `cargo build --release --locked -p db-pro-native`: PASS (exit 0).
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: PASS (13 pass, 3 warning categories, 0 fail; exit 0). Warnings are heuristic size/parameter/cast findings at existing functions; none identifies new lines added by this batch.
- `git diff --check`: PASS (exit 0).
- Review outcome: source self-review only; no P0/P1 remains in scoped source. Runtime screenshots and keyboard/accessibility-tree inspection remain P2 and were NOT RUN. No PostgreSQL/SQLite or backend behavior changed.

## Revised contract — Batch 3
- Source baseline SHA: `924f0857d382864cedfe3a80d7c8f44153ab2eef`; implementation SHA: `0c233293158b807daa8282193d25d5bcf5cde117` (`refactor(ui): remove input token aliases`).
- Added `DESIGN.md`/`API.md` for Input and Form. Existing Input `ui.rs` already owns private Text/Search/Password/Textarea presentation modules; Form's `field`, `rules`, and `state` files already preserve compatibility module paths. Kept `input::layout` unchanged.
- Removed the four public shared-token aliases from `input::config` and replaced every production use with `RADIUS_XS`, `SPACE_SM`, or `SPACE_XS`. Updated the shared input token contract comment and removed the test that asserted alias values equal their canonical tokens.
- `cargo fmt --all -- --check`: PASS (exit 0).
- `cargo test -p db-pro-ui components::input:: --lib`: PASS (12 passed, 0 failed, 0 ignored; 920 filtered; exit 0).
- `cargo test -p db-pro-ui components::form:: --lib`: PASS (9 passed, 0 failed, 0 ignored; 923 filtered; exit 0).
- `cargo check -p db-pro-ui`: PASS (exit 0).
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS (exit 0).
- `cargo build --release --locked -p db-pro-native`: PASS (exit 0).
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: PASS (13 pass, 3 warning categories, 0 fail; exit 0). Warning categories match the previously observed inherited cast/parameter/function-size heuristics.
- `git diff --check`: PASS (exit 0).
- Review outcome: source self-review only; no P0/P1 identified. External consumers of the four removed config aliases must update imports. Runtime viewport/focus/accessibility inspection remains NOT RUN and open at initiative level.

## Revised contract — Batch 4
- Source baseline SHA: `fbdd21bc7cc39da116d46df2484e61e38a1af35f`; implementation SHA: `2e648f38dfa76d24c6bfe6fb3e6cdb32eefc3c80` (`refactor(ui): document and layer selection components`).
- Added `DESIGN.md` and `API.md` for Select, Selection, RadioGroup, and Toggle; added README/DESIGN/API for Tabs. Existing README and module boundaries for Select, Selection, RadioGroup, and Toggle were retained.
- Moved tab drawing modules under `tabs/ui/` and retained `SegmentedTabs`/`UnderlineTabs` re-exports. `tabs/handler.rs` now resolves egui click/focus/key signals into the selected index; focused arrow movement wraps and keeps the existing keyboard-before-click precedence. Reduced motion returns the indicator's target geometry without animation or repaint scheduling. Added handler and reduced-motion regression tests.
- Added a RadioGroup Gallery example with a group label and option descriptions. Select/Tabs config inspection found no shared-token alias to remove; component-owned sizes and shared canonical `RADIUS_*`/`SPACE_*` token references remain.
- `cargo fmt --all -- --check`: PASS (exit 0).
- `cargo test -p db-pro-ui components::tabs:: --lib`: PASS (7 passed, 0 failed; 928 filtered; exit 0).
- `cargo test -p db-pro-ui components::radio_group:: --lib`: PASS (3 passed, 0 failed; 932 filtered; exit 0).
- `cargo test -p db-pro-ui --lib`: PASS (935 passed, 0 failed, 0 ignored; exit 0).
- `cargo check -p db-pro-ui`: PASS (exit 0).
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS (exit 0).
- `cargo build --release --locked -p db-pro-native`: PASS (exit 0; 42.05 seconds incremental after the initial cold build).
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: PASS (13 pass, 3 warning categories, 0 fail). Warnings remain heuristic findings in shared animation conversion, argument counts, and inherited function lengths; no scan failures.
- `git diff --check`: PASS (exit 0).
- Review outcome: implementer source review only; P0=0, P1=0, P2=1 (native viewport, keyboard/focus and accessibility evidence not collected). No PostgreSQL/SQLite behavior changed. Runtime screenshots/accessibility evidence: NOT RUN.

## Revised contract — Batch 5
- Source baseline SHA: `e9a2df373a278c28f3a1abf4b1437612b37ed9f9`; implementation SHA: `c34ba416b0bad6ef4bad27ac5e4db737c3d28475` (`fix(ui): improve calendar and disclosure access`).
- Added `DESIGN.md`/`API.md` for Accordion, Calendar, Collapsible, and HoverCard. Updated Calendar README with the real constructor usage and its keyboard/accessibility behavior; Accordion/Collapsible README now describes reduced-motion behavior.
- Calendar's custom-painted trigger/month/day controls now expose accessible widget metadata and focus rings, activate on focused Enter/Space, and restore focus to the trigger after selection/Escape. DatePicker marks its response changed for selection/open state transitions and reports the final accessible state after popup handling. Arrow-key day-grid traversal remains unsupported and is documented.
- Accordion/Collapsible now bypass egui disclosure animation and body clipping when reduced motion is enabled; shared header hover becomes immediate. Calendar hover states also become immediate. Existing popup bounds, Escape/outside close, and HoverCard timer/placement logic were reviewed without API changes.
- `cargo test -p db-pro-ui components::calendar:: --lib`: PASS (11 passed, 0 failed; 926 filtered; exit 0).
- `cargo test -p db-pro-ui components::accordion:: --lib`: PASS (5 passed, 0 failed; 932 filtered; exit 0).
- `cargo test -p db-pro-ui components::collapsible:: --lib`: PASS (4 passed, 0 failed; 933 filtered; exit 0).
- `cargo test -p db-pro-ui components::hover_card:: --lib`: PASS (14 passed, 0 failed; 923 filtered; exit 0).
- `cargo test -p db-pro-ui --lib`: PASS (937 passed, 0 failed, 0 ignored; exit 0).
- `cargo fmt --all -- --check`: PASS (exit 0); `cargo check -p db-pro-ui`: PASS (exit 0); `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS (exit 0).
- `cargo build --release --locked -p db-pro-native`: PASS (exit 0; 29.53 seconds).
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: PASS (13 pass, 3 warning categories, 0 fail). Warnings are numeric casts and heuristic parameter/function-length counts; no scan failures.
- `git diff --check`: PASS (exit 0).
- Review outcome: implementer source review only; P0=0, P1=0, P2=1 (native viewport/focus/accessibility evidence still not collected; Calendar arrow-key day traversal is a documented limitation). No PostgreSQL/SQLite behavior changed. Runtime screenshots/accessibility evidence: NOT RUN.

## Button architecture update (2026-10-01)

- Baseline source SHA: `7423aea985fd878f447e229a968eac30cf3868f5`; changes are uncommitted on `main`.
- Scope: Button UI/handler directory split, core contrast and motion flow, release accessible-name contract, Button `DESIGN.md`/`API.md`, and the component authoring contract.
- `cargo test -p db-pro-ui components::button --locked`: PASS (9 passed; initial run failed at Default hover step 45, then fixed in core and rerun).
- `cargo fmt --all -- --check`: PASS.
- `cargo check -p db-pro-ui --locked`: PASS.
- `cargo clippy -p db-pro-ui --all-targets --locked -- -D warnings`: PASS.
- `git diff --check`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: PASS (14 pass, 2 warning categories, 0 fail). Warnings: four-argument animation helper and three function-length flags, including inherited long functions in `theme.rs`. These are nonblocking scanner heuristics; the animation helper's arguments are short frame-state inputs.
- Workspace-wide check/clippy/tests and `cargo build --release --locked -p db-pro-native`: NOT RUN; the owner asked for a coding review without a full runtime pass.
- Native UI captures and state traversal: NOT RUN; the owner asked for a coding review without a full runtime pass.
- PostgreSQL/SQLite impact: n/a; source changes are limited to native UI organization and documentation.
- Implementer self-review: ACCEPT WITH P2 for source and focused tests; native rendered pixels, accessibility tree, keyboard traversal, and viewports remain unverified. Token contrast checks are calculations from declared opaque colors, not native pixel measurements.

## Historical Input batch verification (isolated snapshot; superseded below)
- Baseline SHA: `41d731b53cab93514ee8fe992f888d7db673664f` (worktree was clean at inspection).
- Source changes are limited to `crates/ui/src/components/input/**` and this plan directory. Public `Input`, `PasswordInput`, `SearchInput`, and `Textarea` exports remain unchanged.
- `cargo fmt --all`: FAILED before formatting because pre-existing `crates/ui/src/components/form/mod.rs` declares missing `config`, `handler`, and `ui` modules; that unrelated file was not touch

… [truncated for memory entry; see source file]


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Input component layer refactor

## 1. Claim
- Agent identity: implementation subagent.
- Task state: In Progress (source refactor complete; automated/runtime verification blocked or pending).
- Baseline SHA: `41d731b53cab93514ee8fe992f888d7db673664f`.
- Scope: `crates/ui/src/components/input/**` plus component-layer plan/evidence files only; existing unrelated work preserved.

## 2. Evidence
- Added `ui.rs`, `handler.rs`, and `README.md`; `mod.rs` remains the public entry point.
- Moved pure accessible-label selection and Unicode textarea counter decisions into tested handler functions.
- Public widget names and constructors remain unchanged.

## 3. Verification
- `cargo fmt --all`: FAILED due to pre-existing missing `crates/ui/src/components/form/{config,handler,ui}.rs` declarations.
- `cargo test -p db-pro-ui components::input --lib`: FAILED at crate compilation for the same unrelated missing form modules.
- Native UI runtime screenshots/states: PENDING; not claimed.

## 4. Findings
- P0=0, P1=0, P2=1 (runtime visual/accessibility gate pending).

## 5. Tổng kết bằng tiếng Việt
Đã tách lớp Input thành entry point, presentation, handler thuần, cấu hình và README; giữ nguyên API công khai. Kiểm thử bị chặn bởi module Form thiếu sẵn trong worktree và bằng chứng UI runtime vẫn chờ thu thập.

---

# Agent evidence — HoverCard UI Product Review batch

## 1. Claim
- Agent identity: lead implementation + independent reviewer/tester handoffs.
- Task state: In Progress (source implementation and automated checks done; runtime verification pending).
- Baseline SHA: `5b38eb6543d5fa66783fe7f772e52b97665183a6`.
- Branch: `feature/component-layer-structure`; no commit/PR.
- Scope: fix HoverCard placement after component sizing while preserving public API; no database/provider changes.
- Out of scope: Form layer files and other component migrations; preserved existing dirty changes.

## 2. Progress checkpoint
- Current HEAD: `5b38eb6543d5fa66783fe7f772e52b97665183a6` (implementation is uncommitted).
- Completed: HoverCard layer migration existed; `.constrain(false)` prevents egui Area from constraining with stale prior-frame geometry; UI regression test asserts calculated trigger-bottom-plus-gap placement.
- Remaining: native visual/runtime verification at required viewport sizes; oversized first-frame content placement remains unverified.
- Findings: P2 — runtime visuals, interaction/accessibility, and oversized first-frame geometry are not covered; see VERIFICATION.md.
- Tests: `cargo test -p db-pro-ui hover_card --lib` → 14 passed / 0 failed / 0 ignored, exit 0.
- Other gates: fmt, package check, package clippy, native release build and diff check passed; exact commands in VERIFICATION.md.

## 3. Implementation handoff
- Exact source SHA: dirty worktree based on `5b38eb6543d5fa66783fe7f772e52b97665183a6`; source diff is uncommitted.
- Files: HoverCard `ui.rs`/handler/config/README changes already present in batch; this turn's code changes were in `ui.rs`. Plan evidence/checklist/verification updated.
- Review: final reviewer verdict ACCEPT WITH P2; initial test issue fixed; no P0/P1. Independent tester reran gates.
- Database/provider impact: N/A.
- Limitation: screenshots/runtime evidence not collected. No commit made.

## 4. Review outcome
- Reviewed source SHA: `5b38eb6543d5fa66783fe7f772e52b97665183a6` plus dirty working-tree diff.
- Verdict: ACCEPT WITH P2.
- P0/P1/P2: introduced P0=0, P1=0; P2=2 (runtime state/evidence and oversized first-frame content placement unverified).
- CI disposition: not run; local fmt/test/check/clippy/release build passed.
- Next tasks: runtime UI validation and continue remaining public component migrations.

## 5. Research / audit handoff
- References: UI Product Reviewer evaluation model/design-system audit; component plan; `crates/ui/src/components/hover_card/{ui.rs,handler.rs,config.rs}`.
- Observed failure before fix: egui Area constraint placed a short card at y=200 although trigger bottom was y=332 and calculated placement was y=338. Disabling the second, stale-size constraint restores handler-selected placement.
- Independent tester flagged a weak first replacement assertion; final assertion now pins expected y to trigger bottom plus `TRIGGER_GAP`.
- Inference/unknown: first-frame placement of oversized content and live keyboard/focus/viewport behavior need runtime evidence.
- Provider impact: PostgreSQL/SQLite N/A.

## 6. Tổng kết bằng tiếng Việt
Đã sửa lỗi HoverCard bị egui Area dịch vị trí theo kích thước frame trước; test UI hiện kiểm tra tọa độ đúng theo trigger và gap, handler test riêng kiểm tra quyết định lật popup theo chiều cao đo được. 14 test, fmt, check, clippy và native release build đều đạt. Reviewer chấp nhận kèm P2; cần xác minh runtime/screenshot và trường hợp nội dung lớn ở frame đầu. Refactor các component còn lại vẫn tiếp tục.

---

# Agent evidence — accordion moved-value compile fix

## 1. Claim
| Field | Value |
|---|---|
| Agent identity | Senior Software Engineering Subagent · implementation lane |
| Issue(s) | User task: fix `show_multi` moved-value compile error |
| Task state | Done |
| Baseline SHA | Not captured before implementation; repository source was inspected before editing. |
| Branch / PR | `feature/component-layer-structure` / no PR info |
| Scope interpretation | Preserve disabled-item behavior while ensuring `show_multi` compiles after passing `item` by value to `show_single`. |
| Out of scope | Other accordion behavior and plan lifecycle/status transitions. |

## 2. Progress checkpoint
- Current HEAD: not queried.
- Completed acceptance rows: [x] cache `disabled` before moving item; [x] inspect handler tests/status.
- Remaining acceptance rows: none.
- Findings / risks: none identified.
- Tests already run: not run in this session; handler test exists for disabled state preservation.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request
| Field | Value |
|---|---|
| Exact SHA | not captured (uncommitted working tree) |
| Commit list | none; no commit or staging performed |
| File / surface inventory | `crates/ui/src/components/accordion/ui.rs` — cache `item.disabled` before ownership moves into `show_single`; use cached value for state synchronization. |
| Acceptance mapping | Compile error → cache before move; disabled-state preservation → existing handler `multi_toggle_preserves_disabled_item_state` test remains applicable. |
| Commands and counts | Not run. |
| CI run IDs / status | not run |
| Known limitations | Compile/test verification not executed. |
| Migrations / config implications | none |
| Out-of-scope changes | no handler test/status/docs edits; no stage/commit. |

## 4. Review outcome
| Field | Value |
|---|---|
| Reviewed SHA | n/a |
| Verdict | n/a |
| P0 / P1 / P2 counts | n/a |
| Findings | n/a |
| CI disposition | not run |
| Next task(s) unblocked | Run focused UI crate check and accordion handler tests. |

## 5. Research / audit handoff
- Source date: 2026-09-16.
- Source URLs / references: `crates/ui/src/components/accordion/ui.rs`; `crates/ui/src/components/accordion/handler.rs`; `docs/plans/active/component-layer-structure/VERIFICATION.md`.
- Factual findings: `show_multi` passed `item` by value to `show_single` and then referenced `item.disabled`; the handler test `multi_toggle_preserves_disabled_item_state` checks disabled items retain their prior open-set state. The verification record says accordion tests/checks had not yet been executed.
- Inference: caching the Copy boolean before the move is the smallest fix and retains the intended disabled-state synchronization behavior.
- Decision / recommendation: run `cargo test -p db-pro-ui components::accordion::handler::` and `cargo check -p db-pro-ui`.
- Unresolved questions: whether the focused commands pass.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)
Đã sửa lỗi dùng `item.disabled` sau khi `item` được chuyển quyền sở hữu: lưu `disabled` trước lời gọi `show_single`, giữ nguyên logic đồng bộ trạng thái disabled. Handler test liên quan đã được kiểm tra nhưng chưa chạy; cần chạy test và `cargo check` để xác minh. Chưa stage hoặc commit.

## 7. AgentComposer migration handoff

### 1. Claim
| Field | Value |
|---|---|
| Agent identity | Senior Software Engineering Subagent · implementation lane |
| Issue(s) | Migrate `AgentComposer` into the component layer structure without changing its caller-facing API or action behavior. |
| Task state | Done for source migration and targeted automated verification; runtime UI evidence remains pending. |
| Baseline SHA | `81238b4df2b173e6faa15180eddecddbbc36bdf2` |
| Branch / PR | `feature/component-layer-structure` / no PR info |
| Scope interpretation | Only `AgentComposer`; preserve all existing dirty progress and do not perform UI review or commit. |
| Out of scope | AgentPrimitives, other component migrations, runtime/database changes, UI review, and unrelated dirty files. |

### 2. Progress checkpoint
- Current HEAD: `81238b4df2b173e6faa15180eddecddbbc36bdf2` (uncommitted working tree).
- Completed acceptance rows: [x] folder layers; [x] public module/re-exports preserved; [x] handler decision/geometry extraction; [x] focused handler tests; [x] README; [x] detailed English flow comments.
- Remaining acceptance rows: native UI screenshot/accessibility evidence at required sizes and states; feature-wide quality gates remain pending.
- Findings / risks: UI Product Review found keyboard submission bypassed blank-prompt validation (fixed), and narrow toolbar layout needed wrapping (changed). Runtime appearance/accessibility remains unverified.
- Tests already run: `cargo fmt --all -- --check` PASS; `cargo test -p db-pro-ui components::agent_composer::` PASS (6 passed, 0 failed, 805 filtered); `cargo check -p db-pro-ui` PASS with warnings in transaction component; `git diff --check` PASS.
- Dependency / blocker changes: fixed a dangling module doc comment and extra closing brace in the dirty transaction component that had prevented the crate from parsing; preserved other unrelated dirty progress.

### 3. Implementation handoff / review request
| Field | Value |
|---|---|
| Exact SHA | `81238b4df2b173e6faa15180eddecddbbc36bdf2` (working-tree source, no commit) |
| Commit list | none; no commit or staging performed |
| File / surface inventory | Added `crates/ui/src/components/agent_composer/{mod.rs,ui.rs,handler.rs,config.rs,README.md}`; removed `crates/ui/src/components/agent_composer.rs`; no consumer changes were needed. |
| Acceptance mapping | `ui.rs` owns egui layout/painting; `handler.rs` owns submit/blank/generating-button decisions and badge geometry; `config.rs` owns local badge dimensions; `mod.rs` preserves `AgentMode`, `AgentComposerAction`, and `AgentComposer` exports. |
| Commands and counts | `cargo fmt --all -- --check` PASS; `cargo test -p db-pro-ui components::agent_composer::` PASS (6 passed / 0 failed / 805 filtered); `cargo check -p db-pro-ui` PASS (transaction unused-import/dead-code warnings); `git diff --check` PASS. |
| CI run IDs / status | not run |
| Known limitations | Runtime UI visuals and accessibility remain unverified; broader feature quality gates have not run. |
| Migrations / config implications | UI-only migration; no provider or database impact. |
| Out-of-scope changes | no consumer edits, no UI review, no commit, no unrelated dirty-file changes. |

### 4. Review outcome
| Field | Value |
|---|---|
| Reviewed SHA | `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus uncommitted AgentComposer changes |
| Verdict | ACCEPT WITH P2 (source-only review; runtime acceptance pending) |
| P0 / P1 / P2 counts | Introduced review: P0=0, P1=0 after fix, P2=1 runtime narrow-width/accessibility confirmation pending; inherited findings n/a |
| Findings | Keyboard blank-prompt bypass fixed by validating in the handler; narrow toolbar wraps and Prompt label added. Runtime sizing, accessibilit

… [truncated for memory entry; see source file]

