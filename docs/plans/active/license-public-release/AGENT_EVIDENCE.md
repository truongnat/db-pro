# Agent evidence — license and public-release policy

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · release-governance lane |
| Issue(s) | #119 |
| Task state | Review |
| Baseline SHA | `a81757aadcb97062c74259b83b304ede051ca5ed` |
| Branch / PR | `fix/auren-license-policy` · PR not created |
| Scope interpretation | Implement the owner-approved MIT policy and reconcile current release documentation. |
| Out of scope | Product rename, runtime qualification, signing, legal trademark clearance, final binary notice packaging. |

## 2. Progress checkpoint

- Implementation SHA: `11ff3534ab059748e449a0ab8791f4b60e95ee72`; evidence-only follow-up commits are `573494d8a2a578432e659a492ffa110df66c938f` and `ab7187a1e26adc9ee75e94377ccaa0cb6fc420dd`.
- Completed acceptance rows: MIT policy selected; source files added; metadata/doc reconciliation complete; required gates passed.
- Remaining acceptance rows: final binary notice bundle before public binary distribution.
- Findings / risks: P2 notice bundle remains before public binary distribution.
- Tests already run: `cargo metadata --format-version 1 --locked` → exit 0.
- Dependency / blocker changes: `R-LICENSE` is resolved by this change; runtime/platform blockers remain.

## 3. Implementation handoff / review request

| Exact SHA | `11ff3534ab059748e449a0ab8791f4b60e95ee72` |
| Commit list | `11ff353` docs(release): adopt MIT license policy |
| File / surface inventory | `LICENSE`; `THIRD_PARTY_NOTICES.md`; six package manifests plus workspace `Cargo.toml`; README/changelog and current release governance/readiness documents; active plan/evidence files |
| Acceptance mapping | #119 license decision → MIT `LICENSE`; package metadata → workspace/package `license`; third-party scope → `THIRD_PARTY_NOTICES.md`; release truth → current readiness, handoff, checklist, risk register and notes |
| Commands and counts | `cargo fmt --all -- --check` PASS; `cargo check --workspace` PASS; `cargo clippy --workspace --all-targets -- -D warnings` PASS; `cargo test --workspace` PASS — 1133 passed / 0 failed / 38 ignored; `cargo build --release --locked -p db-pro-native` PASS; `cargo metadata --format-version 1 --locked` PASS — six local packages MIT; `git diff --check` PASS |
| CI run IDs / status | not run; no push/PR created |
| Known limitations | Final platform-specific dependency notice bundle not generated; existing candidate archives predate this policy and were not rebuilt |
| Migrations / config implications | No runtime or persisted-state migration; future release archives must include the license and applicable third-party notices |
| Out-of-scope changes | Product rename, runtime qualification, signing, legal trademark clearance |

## 4. Review outcome

Pending independent review; self-verification is not independent approval.

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
