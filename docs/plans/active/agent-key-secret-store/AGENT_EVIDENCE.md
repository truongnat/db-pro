# Agent evidence — Agent API key secret-store lifecycle

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · focused coding lane |
| Issue(s) | #243 remainder: Agent key lifecycle (K-3/K-5) |
| Task state | In Progress |
| Baseline SHA | `941c397065a59f8e976bd212e8ee00287ca6f5ac` |
| Branch / PR | `fix/agent-key-secret-store` · PR not opened |
| Scope interpretation | Move the Agent API key lifecycle into the runtime-owned `SecretStore` and add a real forget path. |
| Out of scope | packaged platform smoke, debugger, SQL Server provider, UI screenshot verification |

## 2. Progress checkpoint

- Current HEAD: `941c397065a59f8e976bd212e8ee00287ca6f5ac`
- Completed acceptance rows: source trace and scope established.
- Remaining acceptance rows: implementation, focused tests, automated gates, self-review.
- Findings / risks: P1 raw keyring bypass; P2 failed request can remain busy (see `FINDINGS.md`).
- Tests already run: none on this branch yet.
- Dependency / blocker changes: native UI runtime verification explicitly skipped by user direction.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | pending implementation |
| Commit list | pending |
| File / surface inventory | pending |
| Acceptance mapping | pending |
| Commands and counts | pending; never claim unrun commands |
| CI run IDs / status | not run |
| Known limitations | native UI runtime evidence skipped |
| Migrations / config implications | preserve the existing `agent/groq_api_key` key name |
| Out-of-scope changes | provider protocol and packaged platform support |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | pending |
| Verdict | pending |
| P0 / P1 / P2 counts | pending |
| Findings | pending |
| CI disposition | not run |
| Next task(s) unblocked | pending |

## 5. Research / audit handoff

- Source date: 2026-09-16
- Source URLs / references: repository source at baseline SHA; issue #243
- Factual findings: recorded in `FINDINGS.md`.
- Inference: the shared runtime vault is the smallest safe owner because it already serves connection and backup secrets.
- Decision / recommendation: implement the focused runtime-owned lifecycle.
- Unresolved questions: packaged keyring behavior remains outside this slice.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã xác định lỗi Agent API key đang bypass `SecretStore`; đang chuyển load/save/delete
về runtime vault, giữ nguyên key name để tương thích. UI runtime smoke được bỏ qua
theo yêu cầu; chỉ ghi nhận automated evidence sau khi code xong.
