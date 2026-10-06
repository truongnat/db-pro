# Research: Automated UI checks for DB Pro's native egui app

**Date:** 2026-10-04
**Scope:** Determine how far DB Pro can automatically detect layout, interaction, accessibility, and visual regressions in its native Rust/egui UI.

**Repository evidence snapshot:** `HEAD 710ba002612c6d71ef2605f99f2a49743b51c3a4` plus the working-tree diff for `crates/native-app/src/capture.rs`, `crates/ui/src/workspace_actions.rs`, and `crates/ui/src/app_tests.rs` (`sha256 4e1d4c0da34d71455390d8ca0f03171b321536340195b2c4e1145535b393796f`). Local implementation observations below refer to that combined snapshot; these files were modified and uncommitted during the review.

## Answer

Yes, DB Pro can automatically catch many *specific, asserted* UI defects: content painting outside its clip, controls that no longer fit a viewport, missing or incorrectly labelled widgets, broken keyboard/click flows, and visual changes against reviewed reference captures. It cannot reliably decide on its own whether a screen looks attractive, has the right hierarchy, communicates its data clearly, or feels appropriate for a database IDE. Those still need a person to review captures and interaction flows.

The useful target is a **surface-and-state regression suite**, not an unspecified scanner that knows every UI is wrong. For each important screen, provide deterministic fixtures and encode the geometry, content, accessibility, and interaction invariants that matter.

## What the repository already has

- `crates/native-app/src/capture.rs` is a native framebuffer capture driver. It waits for egui to settle, pins the requested window size, selects deterministic screens through environment variables, writes a PNG, and exits. It includes routes for Query, Table, Profile, Indexes, Quick Open, loading, and connection error captures.
- `crates/ui/src/workspace_actions.rs` provides the corresponding seeded capture fixtures. These make screenshot evidence reproducible without a live database for the covered screens.
- The UI crate already uses synthetic egui frames in ordinary Rust tests (`Context::run` with `RawInput`). In `crates/ui/src/app_tests.rs`, `the_sidebar_paints_nothing_past_its_clip` examines rendered `ClippedShape`s at several sidebar widths and fails if painted geometry escapes its clip. This is a concrete geometry regression check that can be reused for other surfaces.
- `docs/10-egui-native-migration-plan.md` already requires native screenshots at 1280×800, 1440×900, and 1920×1080 and calls for normal plus loading/error/empty state review.
- `crates/ui/Cargo.toml` and `crates/native-app/Cargo.toml` pin `egui`/`eframe` 0.29; `Cargo.lock` resolves 0.29.1. AccessKit is enabled, and `crates/native-app/src/main.rs` calls `enable_accesskit()`.

## Check types and their limits

| Check | Good at detecting | Does not establish |
|---|---|---|
| Geometry/layout assertions on egui output | A column or widget exceeding its allotted width/clip; a required control not rendered; viewport regressions; content no longer visible in bounds | Whether spacing, hierarchy, or typography looks good |
| Widget/accessibility-tree assertions | Missing roles/names/states; controls unreachable by the test; wrong state after click or keyboard input | Complete screen-reader usability on every OS or correct visual representation |
| Interaction tests | Broken click, typing, focus, keyboard navigation, dialog, sorting/filtering, or scroll behavior in the exercised path | Unexercised workflows or platform-specific native integration |
| Screenshot capture and image snapshots | Unexpected pixel/layout changes in a known viewport and state; reproducible evidence for review | Whether the new image is better; pixel diffs can be noisy across renderer, font, OS, or dependency changes |
| Human visual review | Hierarchy, density, readability, visual quality, misleading emphasis, awkward empty states, overall product fit | Exhaustive coverage or regressions in every future run |

egui's normal frame API returns shapes and supports deterministic frame execution from `RawInput`, which is the basis for the existing geometry assertions and more surface-level checks ([egui `Context::run`](https://docs.rs/egui/0.29.1/egui/struct.Context.html#method.run), [egui `RawInput`](https://docs.rs/egui/0.29.1/egui/struct.RawInput.html)). Shapes carry clip rectangles in frame output; checking rendered bounds against the relevant clip is more reliable than trying to infer fit from column declarations alone.

The official `egui_kittest` project combines AccessKit-backed queries and interaction with optional rendered snapshot tests. Its examples query widgets by role/label, click and run the app, and compare rendered images; its own guidance warns that image tests are relatively slow and brittle and recommends ordinary Rust assertions when those suffice ([egui_kittest README](https://github.com/emilk/egui/blob/main/crates/egui_kittest/README.md)). However, `egui_kittest`'s initial release supports egui 0.30, so it must **not** be added directly to this repository's egui 0.29.1 dependency graph ([egui_kittest changelog](https://github.com/emilk/egui/blob/main/crates/egui_kittest/CHANGELOG.md)).

AccessKit exposes semantic nodes such as role and label for assistive technologies; egui documents custom-widget metadata and querying those semantics through `egui_kittest` ([egui accessibility guide](https://github.com/emilk/egui/blob/main/docs/accessibility.md), [AccessKit overview](https://github.com/AccessKit/accesskit)). DB Pro already enables AccessKit output, but currently has no `egui_kittest` dependency. These semantics are useful assertions, not a substitute for VoiceOver/NVDA/platform accessibility checks.

## Recommended approach

### 1. Extend the existing geometry-test pattern first

Stay on egui 0.29.1 initially. Build small deterministic render helpers for high-risk surfaces, then assert explicit invariants on each frame's output. Start with the reported Table Profile and Indexes cases:

- render at the required viewport widths (and relevant sidebar sizes);
- identify the surface/content bounds and assert required headers/cells/controls fit inside them;
- assert painted extents do not escape their assigned clip;
- assert required text is present and is not silently absent; where truncation is intended, assert tooltip/full-value availability;
- include long identifiers, wide values, empty data, and conditional statistic columns in fixtures.

Prefer stable semantic/layout assertions over hard-coded absolute coordinates. A useful failure message should name the viewport, surface, element, measured bounds, and allowed bounds. The existing sidebar clip test demonstrates this style in `crates/ui/src/app_tests.rs`.

### 2. Use the native capture harness as a visual evidence matrix

Keep using `DB_PRO_CAPTURE_TO`, `DB_PRO_WINDOW_SIZE`, and the per-surface environment switches to capture the actual app with deterministic fixtures. For each changed surface, capture light and dark where applicable, required viewport sizes, and the meaningful success/loading/error/empty states. Store reviewed reference images with the relevant plan or UI-audit evidence.

An image-diff gate can report unexpected changes against those references, but every reference update should still be reviewed. The existing migration plan explicitly requires human review at its three viewports and states. The native framebuffer also checks renderer/runtime output that a source-only component test cannot see.

### 3. Add semantic and interaction coverage

Add tests for workflows that have stable state and accessible elements: open the tab, focus controls, search/filter, open a detail dialog, dismiss with Escape, scroll, and navigate via keyboard. Assert accessible roles/names/states for custom-painted controls when appropriate. Once the repo upgrades egui and eframe to at least the compatible egui_kittest release, evaluate it for AccessKit queries and automated widget interactions rather than immediately replacing all existing tests.

### 4. Keep human review as an explicit acceptance step

Have a reviewer inspect the captured screen for hierarchy, alignment, density, typography, color balance, empty-state usefulness, and whether data is easy to scan. Automation should point to a likely regression and preserve reproducible evidence; it should not mark subjective visual quality as passed merely because a screenshot matches a baseline.

## Practical rollout

1. Add targeted bounds/content regression tests for Profile and Indexes using the current test idiom.
2. Add any missing deterministic fixture states to the existing capture driver, then record 1280×800, 1440×900, and 1920×1080 evidence for each changed surface.
3. Make capture review required in the UI plan checklist; attach evidence and record which viewport/state combinations were actually collected.
4. Build a broader risk-weighted matrix for data grid, dialogs/forms, workspace tabs, Explorer, Query, and Settings. Test both themes, narrow/wide viewports, long content, empty/loading/error, keyboard, and scroll where relevant.
5. After a deliberate egui upgrade, prototype `egui_kittest` on one self-contained component. Keep it only if its interaction/snapshot value outweighs the dependency and maintenance cost.

## Boundaries and limitations

- There is no one-call API that semantically scans every egui screen for all UI defects. A useful suite needs explicit screen fixtures, states, and assertions.
- A clipped shape is not always a defect: intentional clipping exists in scroll areas. Assertions must target a widget's expected container/visible area, not ban all clipping globally.
- A correct widget rectangle does not prove text is legible or that controls are visually discoverable; use screenshots and human review for those.
- Screenshot comparisons can vary due to OS, font rasterization, scale factor, GPU/renderer, and egui version. Keep comparison tolerances tight and review baseline changes rather than accepting mass snapshot regeneration.
- The capture driver is actual native rendering and therefore requires a working display/renderer environment. A deterministic fixture proves only the seeded UI state; it does not prove database-backed runtime behavior.
- The repository's current egui 0.29.1 predates egui_kittest's initial egui 0.30 support. Do not cite latest egui_kittest APIs as directly usable in this checkout without an upgrade or compatibility proof.

## Sources

### Primary project sources

- [egui 0.29.1 `Context::run`](https://docs.rs/egui/0.29.1/egui/struct.Context.html#method.run)
- [egui 0.29.1 `RawInput`](https://docs.rs/egui/0.29.1/egui/struct.RawInput.html)
- [egui_kittest README and snapshot-testing guidance](https://github.com/emilk/egui/blob/main/crates/egui_kittest/README.md)
- [egui_kittest changelog, initial 0.30 release](https://github.com/emilk/egui/blob/main/crates/egui_kittest/CHANGELOG.md)
- [egui accessibility guide](https://github.com/emilk/egui/blob/main/docs/accessibility.md)
- [AccessKit project overview](https://github.com/AccessKit/accesskit)

### Local implementation evidence

- `crates/native-app/src/capture.rs` — deterministic native screenshot capture and surface/state routing.
- `crates/ui/src/workspace_actions.rs` — deterministic capture fixtures.
- `crates/ui/src/app_tests.rs` — synthetic egui frame tests and sidebar painted-extent/clip regression.
- `crates/ui/Cargo.toml`, `crates/native-app/Cargo.toml`, `Cargo.lock` — egui/eframe 0.29.1 and AccessKit feature configuration.
- `crates/native-app/src/main.rs` — AccessKit generation is enabled for the native app.
- `docs/10-egui-native-migration-plan.md` — existing screenshot viewport and state acceptance gate.

No code, tests, builds, screenshots, or runtime states were changed or executed for this research.

## Agent evidence handoff

### 1. Claim

| Field | Value |
|---|---|
| Agent identity | `/root` · research lane, with background research agent |
| Issue(s) | n/a |
| Task state | Done |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus the relevant working-tree diff identified above |
| Branch / PR | `main` / docs-only / no PR |
| Scope interpretation | Assess practical automated UI checks for the native egui application and recommend a low-risk route for Profile and Indexes. |
| Out of scope | Implementing checks, running UI tests/builds, collecting new screenshots, and subjective design sign-off. |

### 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`
- Completed acceptance rows: [x] answer feasibility; [x] identify repository baseline and limits; [x] recommend a staged approach with primary references.
- Remaining acceptance rows: none for this research request.
- Findings / risks: P2 — screenshot diffs may be noisy and cannot assess whether a visual change is better; see `Boundaries and limitations`.
- Tests already run: none (research only). `git diff --check -- docs/ui-audit/automated-native-ui-checks-research.md` → passed, exit 0.
- Dependency / blocker changes: none.

### 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus working-tree diff digest above |
| Commit list | none |
| File / surface inventory | `docs/ui-audit/automated-native-ui-checks-research.md` — research and recommendation only |
| Acceptance mapping | Feasibility and check layers → `Answer` and `Check types and their limits`; DB Pro proof → `What the repository already has`; next steps → `Recommended approach` and `Practical rollout` |
| Commands and counts | `git diff --check -- docs/ui-audit/automated-native-ui-checks-research.md` → exit 0; no tests/builds run |
| CI run IDs / status | not run |
| Known limitations | No automated scan was executed in this task; recommendations rely on source inspection and official documentation. |
| Migrations / config implications | none |
| Out-of-scope changes | No implementation, fixtures, screenshots, or runtime configuration changed. |

### 4. Review outcome

n/a — research note, no code review requested.

### 5. Research / audit handoff

- Source date: 2026-10-04.
- Source URLs / references: primary source links in `Sources`; local paths in `What the repository already has` and `Repository evidence snapshot`.
- Factual findings: local implementation statements refer to the exact HEAD plus working-tree diff digest in the header; external API statements link to official egui/AccessKit documentation and source repositories.
- Inference: the staged rollout and recommendation to start with geometry assertions are proposals based on current repo capabilities, not existing project policy.
- Decision / recommendation: extend current egui geometry-test idioms for Profile and Indexes first; then add interaction/semantic assertions and reviewed native screenshot comparisons.
- Unresolved questions: which Profile/Indexes invariants should become permanent regression requirements; whether future egui upgrade timing justifies `egui_kittest`.
- Downstream tasks activated: none.

### 6. Tổng kết (Vietnamese summary)

Đã khảo cứu cách tự động tìm lỗi UI cho DB Pro native egui. Repo đã có capture fixture và một kiểm tra geometry; nên bắt đầu bằng test bounds/content cho Profile và Indexes, sau đó bổ sung tương tác và so sánh ảnh có người duyệt. Chưa chạy scanner hoặc sửa implementation; ảnh chụp vẫn cần người đánh giá hierarchy và độ dễ đọc.
