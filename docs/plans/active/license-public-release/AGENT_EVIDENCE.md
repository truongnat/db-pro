# Agent evidence — license and public-release policy

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · release-governance lane |
| Issue(s) | #119 |
| Task state | In Progress |
| Baseline SHA | `a81757aadcb97062c74259b83b304ede051ca5ed` |
| Branch / PR | `fix/auren-license-policy` · PR not created |
| Scope interpretation | Implement the owner-approved MIT policy and reconcile current release documentation. |
| Out of scope | Product rename, runtime qualification, signing, legal trademark clearance, final binary notice packaging. |

## 2. Progress checkpoint

- Current HEAD: pending implementation commit.
- Completed acceptance rows: policy selected; source files added; metadata/doc reconciliation in progress.
- Remaining acceptance rows: exact gates; final binary notice bundle.
- Findings / risks: P2 notice bundle remains before public binary distribution.
- Tests already run: `cargo metadata --format-version 1 --locked` → exit 0.
- Dependency / blocker changes: `R-LICENSE` is resolved by this change; runtime/platform blockers remain.

## 3. Implementation handoff / review request

Pending final commit and gate execution.

## 4. Review outcome

Pending independent review.

## 5. Research / audit handoff

- Source date: 2026-09-16
- Source references: `Cargo.toml`, `crates/*/Cargo.toml`, `README.md`, `docs/release/0.1.0-readiness.md`, `docs/release/risk-register.md`, `crates/ui/assets/fonts/OFL.txt`.
- Factual finding: baseline had no project `LICENSE` or manifest license metadata.
- Inference: MIT is the lowest-friction permissive policy for the intended public desktop tool; legal counsel remains authoritative for entity-specific terms.
- Decision: MIT selected by the project owner; Auren trademarks remain separate.
- Unresolved questions: exact binary notice bundle generation before public distribution.
- Downstream tasks activated: none.

## 6. Tổng kết bằng tiếng Việt

Đã bắt đầu triển khai policy MIT cho Auren trên branch riêng. Scope chỉ gồm license,
manifest metadata, third-party notice index và release docs; không đổi runtime hay rename
source. Còn phải chạy gate và tạo notice bundle hoàn chỉnh trước khi public binary release.
