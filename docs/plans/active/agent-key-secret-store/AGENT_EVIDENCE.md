# Agent evidence — Agent API key secret-store lifecycle

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · focused coding lane |
| Issue(s) | #243 remainder: Agent key lifecycle (K-3/K-5) |
| Task state | Review |
| Baseline SHA | `941c397065a59f8e976bd212e8ee00287ca6f5ac` |
| Branch / PR | `fix/agent-key-secret-store` · PR not opened |
| Scope interpretation | Move the Agent API key lifecycle into the runtime-owned `SecretStore` and add a real forget path. |
| Out of scope | packaged platform smoke, debugger, SQL Server provider, UI screenshot verification |

## 2. Progress checkpoint

- Current HEAD: `6c78ebe` (full SHA below)
- Completed acceptance rows: [x] runtime-owned load/save/delete; [x] real forget action; [x] focused provider and translation tests; [x] automated gates; [x] failure clears the busy request.
- Remaining acceptance rows: native UI runtime traversal is intentionally skipped.
- Findings / risks: no introduced P0/P1/P2 remains in the fixed path; packaged keyring behavior is not runtime-qualified here.
- Tests already run: `cargo test --workspace` → 1137 passed / 0 failed / 38 ignored, exit 0.
- Dependency / blocker changes: native UI runtime verification explicitly skipped by user direction.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `6c78ebe3c087e4b14e2ac3081212b600a4d74f0f` |
| Commit list | `6c78ebe fix(agent): route API key lifecycle through secret store` |
| File / surface inventory | runtime `DbProRuntime`/worker secret lifecycle; native command/event mapping; UI forget action and error reset; native-app keyring dependency removal; focused plan/evidence |
| Acceptance mapping | raw native keyring bypass → `crates/native-app/src/main.rs`; shared lifecycle → `crates/runtime/src/lib.rs` + `crates/runtime/src/worker.rs`; provider selection → `crates/runtime/src/agent.rs`; command/event routing → `crates/native-app/src/translate_*`; forget UI/error state → `crates/ui/src/agent_view.rs`, `events.rs`, `events_query.rs`; regression tests → `translate_tests.rs` + runtime tests |
| Commands and counts | fmt/check/clippy/release pass; workspace 1137/0/38; infrastructure secret 11/0/0; native 19/0/0; runtime focused 2/0/0; UI focused 1/0/0 |
| CI run IDs / status | not run |
| Known limitations | native UI runtime evidence skipped; clean-code scanner retains inherited long-file/function failures |
| Migrations / config implications | preserve the existing `agent/groq_api_key` key name |
| Out-of-scope changes | provider protocol and packaged platform support |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `6c78ebe3c087e4b14e2ac3081212b600a4d74f0f` |
| Verdict | `ACCEPT WITH P2` — automated gates pass; UI runtime evidence skipped |
| P0 / P1 / P2 counts | introduced-by-this-SHA: 0 / 0 / 0; inherited product findings: 0 / 0 / 0 |
| Findings | clean-code scanner reports inherited long translator/UI/worker functions and files; no new long function or file was introduced |
| CI disposition | not run |
| Next task(s) unblocked | provider/platform runtime qualification remains separate |

## 5. Research / audit handoff

- Source date: 2026-09-16
- Source URLs / references: repository source at baseline SHA; issue #243
- Factual findings: recorded in `FINDINGS.md`.
- Inference: the shared runtime vault is the smallest safe owner because it already serves connection and backup secrets.
- Decision / recommendation: implement the focused runtime-owned lifecycle.
- Unresolved questions: packaged keyring behavior remains outside this slice.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã chuyển toàn bộ vòng đời Agent API key về runtime `SecretStore`, thêm Forget key
xoá thật qua vault, và thêm regression tests. Commit `6c78ebe`; workspace 1137/0/38,
clippy và release build pass. UI runtime smoke được bỏ qua theo yêu cầu.
