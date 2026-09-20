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
