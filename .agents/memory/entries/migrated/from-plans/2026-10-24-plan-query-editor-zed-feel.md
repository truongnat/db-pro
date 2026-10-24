---
schema_version: 1
id: "MEM-PLAN-query-editor-zed-feel-e759d3b8a59c"
title: "Plan archive: Query Editor Zed Feel"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, query-editor-zed-feel]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "e759d3b8a59ca36e0da133b455902d9df38d52f6b181c038fdd0c7e5128b8681"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/query-editor-zed-feel/PLAN.md"
  - "docs/plans/active/query-editor-zed-feel/FINDINGS.md"
  - "docs/plans/active/query-editor-zed-feel/VERIFICATION.md"
  - "docs/plans/active/query-editor-zed-feel/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/query-editor-zed-feel/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Query editor — reliable Zed-like interaction

## Goal

Make the native SQL editor predictable and responsive enough for daily database work: Zed-like typing and navigation quality, with at least the core query-tab interaction reliability expected from DBeaver.

Correctness comes before suggestion volume. Ordinary typing must never trigger AI traffic, apply stale completion ranges, insert unrelated text, or move the caret unexpectedly.

## Scope

### Editor interaction engine

- `crates/ui/src/editor/renderer.rs` — keyboard/IME intent capture, scrolling, caret and text rendering
- `crates/ui/src/editor/interaction.rs` — centralized policy for completion and AI-prediction intents
- `crates/ui/src/editor/completion.rs` — version-bound completion sessions and navigation state
- `crates/ui/src/query_editor_panel.rs` — application orchestration, guarded completion application and popup presentation

### User experience

- Stable multilingual typing and undo grouping
- Completion opens explicitly with `Ctrl/Cmd+Space` or contextually after `.`
- AI prediction is opt-in and explicitly requested; never scheduled by ordinary typing/caret movement
- Completion rows expose kind, detail and explanatory documentation on selection/hover
- Wheel/trackpad scrolling, blinking caret and keep-caret-visible behavior
- `Ctrl/Cmd+S` saves without inserting text

## Architecture

```text
egui Event
  -> SqlEditor (renders and emits SqlEditorResponse intents)
  -> EditorInteractionPolicy (decides allowed state transition)
  -> CompletionState / explicit prediction schedule
  -> version-checked buffer edit
```

The renderer does not call providers or decide application behavior. Completion sessions are tied to the document version that produced their replacement ranges. The query panel rejects stale or invalid ranges instead of attempting a best-effort edit.

## Non-goals for this slice

- Full SQL LSP protocol/client
- Multi-cursor, minimap or sticky-scroll headers
- Pixel-perfect Zed clone
- Automatic AI requests while typing
- Replacing the existing query runtime/provider command path

## Acceptance

- [ ] Normal typing and IME input insert exactly the committed text
- [ ] Alphanumeric typing does not automatically open completion
- [ ] `.` and `Ctrl/Cmd+Space` open contextual/manual completion respectively
- [ ] Completion cannot apply after its source document version changes
- [ ] Hovering/selecting a suggestion explains its kind/detail/documentation
- [ ] AI prediction requires explicit user intent and an enabled mode
- [ ] Enter/Tab/navigation ownership is deterministic while completion is open
- [ ] Wheel/trackpad scrolling and keep-caret-visible remain functional
- [ ] `Ctrl/Cmd+S` saves without modifying the SQL buffer
- [ ] UI tests, clippy, formatter and native release build pass
- [ ] Runtime visual/interaction verification is recorded separately

## Provider matrix

The editor interaction engine is provider-neutral. Schema suggestions consume the already loaded `UiSchemaSummary`; SQL dialect selection remains independently capability-driven for PostgreSQL and SQLite. No query execution/provider behavior changes in this slice.


## Source: `FINDINGS.md`

# Findings

Baseline for this implementation pass: `f1a484059dbaf4915c96089715a03d9291507f28`.

## Resolved earlier — P1 editor content size unused

Long queries were clipped because computed content dimensions were not driving editor scroll state. Manual two-axis scrolling and keep-caret-visible behavior are now present; runtime evidence remains pending.

## Resolved earlier — P2 static caret and heavy chrome

The editor now has an idle blink cycle, recent-input solid period, softer gutter/current-line treatment and editor-focused spacing.

## P1 — stale completion ranges can mutate the wrong buffer segment

A completion item stores a byte replacement range. Before this pass, that range had no source document version and the popup could survive a subsequent edit. Accepting the item could therefore replace a valid but unrelated range, presenting as text jumping or being inserted unpredictably.

Decision: bind every completion session to `TextBuffer::version()` and fail closed if the current version differs or the range is invalid/not on UTF-8 boundaries.

## P1 — persisted prediction mode can keep automatic AI scheduling active

Changing only the default prediction mode to `Off` does not affect users with an older persisted `Subtle` value. The query panel still scheduled prediction after ordinary edits/caret movement.

Decision: automatic scheduling is removed. Prediction requires an explicit editor action and a non-`Off` mode.

## P2 — completion UI exposes labels but not useful explanation

Completion items already carry `detail` and `documentation`, but the popup only used truncated label/detail tooltips. Users could not understand why a suggestion was offered.

Decision: selected suggestions get a persistent explanation area; hovering a row shows the same structured explanation.

## Remaining P2

- Continuous completion filtering currently closes stale sessions rather than recomputing them. This is intentionally conservative for typing correctness and should be added as a version-safe session update.
- General SQL symbol hover and function signature help are not yet implemented.


## Source: `VERIFICATION.md`

# Verification

Source revision: `f1a484059dbaf4915c96089715a03d9291507f28+dirty` (implementation is not committed).

## Automated — 2026-09-19

```text
cargo fmt --all -- --check
cargo check -p db-pro-ui -p db-pro-native
cargo clippy -p db-pro-ui -p db-pro-native --all-targets -- -D warnings
cargo test -p db-pro-ui
git diff --check
```

Result: PASS, including `536 passed / 0 failed / 0 ignored` UI unit tests and zero clippy warnings.

Focused interaction tests also passed:

```text
cargo test -p db-pro-ui editor::interaction
# 4 passed / 0 failed

cargo test -p db-pro-ui query_editor_panel
# 2 passed / 0 failed
```

## Clean-code scan

```text
bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci
```

Result: exit 0; 12 pass classes, 4 warning classes, 0 failures. Remaining warnings are ratcheted size/parameter/numeric-cast heuristics across the full dirty working tree; no swallowed errors, `unwrap`/`expect`, debug output, undocumented unsafe or boundary violation was found.

## Performance/build

```text
bash .skills/perf-audit/scripts/perf-scan.sh
```

Result: PASS (partial), 4 executed checks passed, release binary built successfully at 31.2 MB. Native UI benchmarks, backend benchmarks, ER window runtime and live DB query performance were not executed.

## Runtime

Pending owner verification: open Query, type multilingual SQL continuously, open completion with `.` and `Ctrl/Cmd+Space`, filter with typing/backspace, inspect hover explanation, accept with Enter/Tab, save with `Ctrl/Cmd+S`, and verify scroll/caret behavior at required viewport sizes.

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/02-query-editor-workspace.md` verdict: **EVIDENCE_GAP**
— all code claims verified at HEAD (`EditorInteractionPolicy`, version-bound
completion `document_version`/`can_apply_to_version`, `scroll_caret_into_view`,
caret blink, `Cmd+S` → `wants_save`, signature help), but runtime
visual/interaction evidence is still unchecked. Static captures of the query
editor landed this run (`docs/plans/_audit-2026-10-13/captures/query{,-light,-row-limit}_*.png`,
`table-completion_*.png`); they render the shipped chrome but do not exercise
typing, dot-completion, `Cmd+Space`, `Cmd+S`, or caret/scroll behavior — the owner
verification list above remains the open gate. State stays IMPLEMENTING.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Query editor reliable Zed-like interaction

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Zed coding agent · native editor lane |
| Issue(s) | n/a — owner-directed Query Editor hardening |
| Task state | In Progress |
| Baseline SHA | `f1a484059dbaf4915c96089715a03d9291507f28` |
| Branch / PR | `main` · no PR |
| Scope interpretation | Make ordinary SQL typing deterministic, isolate completion/prediction policy, reject stale suggestion edits, and expose suggestion explanations. |
| Out of scope | Full SQL LSP, multi-cursor, minimap, provider/runtime execution changes, owner-supplied UI runtime verification. |

## 2. Progress checkpoint

- Current HEAD: `f1a484059dbaf4915c96089715a03d9291507f28` (implementation is currently uncommitted; no source-behavior claim is attached to a new SHA yet).
- Completed acceptance rows: interaction policy boundary; quiet ordinary typing; dot/manual completion intents; version-safe continuous filtering; explicit-only AI prediction scheduling; document-version completion guard; completion explanation UI; save shortcut; automated gates and release build.
- Remaining acceptance rows: general SQL symbol hover; function signature help; owner runtime evidence.
- Findings / risks: P1 stale completion replacement range; P1 persisted prediction setting retaining automatic requests; P2 missing suggestion explanation. See `FINDINGS.md`.
- Tests already run: `cargo test -p db-pro-ui` — 536 passed / 0 failed / 0 ignored; check/clippy/fmt/diff checks passed; clean-code scanner 12 pass / 4 warning classes / 0 failures; performance audit 4 executed checks passed and release binary built at 31.2 MB.
- Dependency / blocker changes: no dependency added; runtime interaction evidence remains owner-provided.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | pending commit; implementation is in the working tree |
| Commit list | n/a — not committed |
| File / surface inventory | `crates/ui/src/editor/interaction.rs` policy boundary; `completion.rs` versioned sessions; `renderer.rs` quiet triggers/shortcuts; `renderer_tests.rs` input regression; `query_editor_panel.rs` orchestration, guarded apply and explanation UI; plan documents updated. |
| Acceptance mapping | See `PLAN.md` acceptance list and `CHECKLIST.md`. |
| Commands and counts | fmt/check/clippy/diff checks exit 0; UI tests 536/0/0; clean-code scanner 12 pass classes / 4 warning classes / 0 failures; performance audit PASS (partial), 4 checks executed. See `VERIFICATION.md`. |
| CI run IDs / status | not run |
| Known limitations | General SQL symbol hover and signature help remain open; no owner runtime evidence yet. Window/live-provider benchmark sections were not executed. |
| Migrations / config implications | Prediction default changes to off for new/default settings; existing persisted values remain readable but no longer cause automatic typing-time requests. |
| Out-of-scope changes | Existing unrelated New Connection/accessibility working-tree changes are preserved and not claimed by this handoff. |

## 4. Review outcome

n/a — implementation and final gates are still in progress.

## 5. Research / audit handoff

- Source date: 2026-09-19.
- Source URLs / references: repository sources under `crates/ui/src/editor/`, `crates/ui/src/query/schema_completion.rs`, `crates/ui/src/query_editor_panel.rs`, and active query-editor plans.
- Factual findings: recorded in `FINDINGS.md` against baseline SHA `f1a484059dbaf4915c96089715a03d9291507f28`.
- Inference: versionless completion sessions are the most likely direct cause of reported text jumping when accepting a stale suggestion; this is a reasoned failure scenario, not owner runtime evidence.
- Decision / recommendation: keep completion conservative and synchronous now; add continuous filtering only through version-safe session refresh, not by retaining stale ranges.
- Unresolved questions: desired LSP/provider strategy for advanced semantic hover and signature help.
- Downstream tasks activated: continuous completion filtering, SQL symbol hover, function signature help.

## 6. Tổng kết (Vietnamese summary)

Đã tái cấu trúc Query Editor theo hướng renderer chỉ phát intent, policy điều phối completion/prediction, và mọi completion phải khớp đúng phiên bản buffer. Suggestion hỗ trợ filtering liên tục an toàn và giải thích khi chọn/hover; 536 test UI cùng fmt/check/clippy/build đều xanh. Còn symbol hover, signature help và runtime visual do chủ dự án xác nhận.

