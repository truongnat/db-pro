# Agent evidence — #507 PostgreSQL RLS expression validation

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | OpenClaw parent + OpenCode implementation, parent review |
| Issue(s) | #507 |
| Task state | Review |
| Baseline SHA | `beaa1882a6253d902748c27fba53b99d9969b635` |
| Branch / PR | `fix/validate-rls-policy-expressions` / PR [#518](https://github.com/truongnat/db-pro/pull/518) (`Draft`) |
| Scope interpretation | Prevent RLS predicate text from escaping CREATE/ALTER policy DDL. |
| Out of scope | Other issues, providers, and general DDL executor changes. |

## 2. Progress checkpoint

- Current implementation SHA: `7eacf0e19a3d237b704028c7d2b1bfd2da6eb1a8`.
- Completed acceptance: validation is wired in CREATE/ALTER `USING` and `WITH CHECK`; complete PostgreSQL expression parser and EOF check; regressions for injected statements, valid forms, and trailing line-comment delimiter.
- Remaining: final focused verification, commit/rebase/push/PR, independent review and CI.
- Findings: P1 injection identified at baseline; fix awaits independent review.
- Tests: focused results and workspace check are recorded in `VERIFICATION.md`; full suite/clippy not run.
- Dependencies: direct `sqlparser = 0.52` dependency added to `db-pro-core`.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `7eacf0e19a3d237b704028c7d2b1bfd2da6eb1a8` |
| Commit list | `7eacf0e1 fix(security): validate RLS policy expressions` |
| File/surface inventory | core SQL validation; CREATE/ALTER RLS builder; core and app regression tests; Cargo dependency; plan documentation |
| Acceptance mapping | See PLAN.md and CHECKLIST.md |
| Commands and counts | See VERIFICATION.md |
| CI run IDs / status | Rust checks in progress on PR #518; Kilo reviewer skipped while draft |
| Known limitations | Parser can fail closed on PostgreSQL syntax not supported by sqlparser 0.52 |
| Migrations / config implications | None |
| Out-of-scope changes | None |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | pending |
| Verdict | PENDING independent review |
| P0 / P1 / P2 counts | PENDING / 1 fixed pending review / 1 compatibility limitation |
| Findings | See FINDINGS.md |
| CI disposition | Rust checks in progress; Kilo reviewer skipped on draft; independent review pending |
| Next task(s) unblocked | Re-scan open related RLS/security issues after PR creation |

## 5. Research / audit handoff

- Source date: 2026-10-16
- References: issue #507; `crates/core/src/application/object_mutation_builders.rs`; `crates/core/src/application/object_mutation_service.rs`; schema DDL execution path.
- Factual finding: expression fields were concatenated into policy DDL; RLS is PostgreSQL-only and SQLite is gated unsupported.
- Decision: use PostgreSQL parser and require EOF instead of a hand-written SQL quote scanner.
- Unresolved: independent review and PR CI. PR #518 is open as draft.

## 6. Tổng kết (Vietnamese summary)

Đã xác thực đường inject và chuyển sang parser PostgreSQL để bắt buộc mỗi trường là đúng một expression; regression tests bao phủ CREATE/ALTER, quote escape và delimiter. Chưa chạy full workspace, live DB, independent review hoặc CI. Parent đã xác minh kiểm tra tập trung, rebase và mở draft PR #518 gắn `Fixes #507`; CI Rust đang chạy, review độc lập còn chờ.
