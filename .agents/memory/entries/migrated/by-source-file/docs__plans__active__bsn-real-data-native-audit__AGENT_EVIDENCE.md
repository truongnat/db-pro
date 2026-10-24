---
schema_version: 1
id: "MEM-SRC-1d58664179940169"
title: "Source mirror: docs/plans/active/bsn-real-data-native-audit/AGENT_EVIDENCE.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/bsn-real-data-native-audit/AGENT_EVIDENCE.md"
source_sha256: "1d58664179940169df621f9701484ee8391333f4eb2c012825f1c9ece3ef4f0f"
mirror_body_sha256: "1d58664179940169df621f9701484ee8391333f4eb2c012825f1c9ece3ef4f0f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Agent evidence — DB Pro read-only QA audit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · read-only QA audit |
| Issue(s) | n/a |
| Task state | Review |
| Baseline SHA | `cb6e00d633772de538498384a8b14d32d25f7915` — repository also had pre-existing uncommitted changes |
| Branch / PR | `main` / no PR |
| Scope interpretation | Prioritized read-only QA across native UI/UX, code quality, architecture, functional behavior, security, performance, tests and real BSN-facing surfaces. Findings distinguish live reproduction, source/test/configuration evidence, synthetic fixture evidence and NOT VERIFIED. |
| Out of scope | Product-source fixes, tests, database writes/DDL, admin or destructive actions, exports/backups/transfers, settings writes, full dark/wide capture matrix, complete focus/accessibility traversal, and unsafe BSN operations. |

## 2. Progress checkpoint

- Current HEAD: `cb6e00d633772de538498384a8b14d32d25f7915`.
- Completed acceptance rows: [x] prioritized report with file/line, evidence, impact and recommended fix; [x] 25-feature checklist with explicit LIVE/SYNTH/SRC/NOT VERIFIED status; [x] read-only BSN visual walkthrough across available safe surfaces; [x] screenshot provenance/privacy and finding evidence; [x] source/runtime/test/coverage findings separated; [x] safety exclusions recorded.
- Remaining acceptance rows: [ ] full feature-state matrix; [ ] Light/Dark and 1440×900/1920×1080 live capture; [ ] full keyboard/focus and accessibility-tree/screen-reader verification; [ ] remaining provider-specific functional/error cases; [ ] independent PostgreSQL rollback test on a disposable fixture.
- Findings / risks: P0=0; P1=2 open (`QA_AUDIT_REPORT.md`, targeted retry clears unrelated staged edits on success and live stale table filter on table switch); P2=22 (16 verified source/live/configuration/coverage findings and 6 suspected source-traced runtime/performance risks). Six live BSN UI/safety-affordance findings include the conditional synthetic INSERT gate risk; report states limits and no mutation.
- Tests already run: no tests/build/lint run in this visual continuation. Prior results are recorded in `QA_AUDIT_REPORT.md` and `VERIFICATION.md` and are explicitly qualified; the 610-app-test run used a temporary patch and is not a final-tree verification. Screenshot hashes for 33 retained PNGs are recorded in `evidence/runtime/MANIFEST.md` and must match the final `sha256sum` check below.
- Dependency / blocker changes: added live evidence for additional BSN surfaces and two P2 UI/safety findings; the full matrix, dark/wide rendering, and independent disposable PostgreSQL runtime remain unverified/blocked. BSN remains writable, so no mutation was attempted.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `cb6e00d633772de538498384a8b14d32d25f7915` |
| Commit list | No commits created |
| File / surface inventory | `QA_AUDIT_REPORT.md` — prioritized findings and evidence; `docs/plans/active/bsn-real-data-native-audit/PLAN.md` — read-only continuation status; `CHECKLIST.md` — per-feature dispositions; `FINDINGS.md` — current source/runtime findings; `VERIFICATION.md` — exact runtime and test limitations; `evidence/runtime/MANIFEST.md` — capture provenance, dimensions and SHA-256; this handoff file — evidence contract. No product source/test file modified in this continuation. |
| Acceptance mapping | Audit dimensions and severity → `QA_AUDIT_REPORT.md`; feature scope/status → `CHECKLIST.md` F01–F25; live screenshot evidence → `evidence/runtime/MANIFEST.md`; safe-runtime constraints and unavailable gates → `VERIFICATION.md`; status/remaining acceptance → `PLAN.md`. |
| Commands and counts | `sha256sum docs/plans/active/bsn-real-data-native-audit/evidence/runtime/*.png` — 33 hashes verified, exit 0. No tests, build, lint or CI run in this continuation. Historical command results and their limitations are reported in the QA report. |
| CI run IDs / status | Not run in this continuation |
| Known limitations | Only Light mode at 1280×800 was captured live; remaining activity/error/loading/large-data states, dark and wide layouts, full accessibility/focus traversal, visual i18n, provider-specific cases and PostgreSQL rollback remain NOT VERIFIED. Source findings are tied to the recorded HEAD while the worktree includes earlier uncommitted changes; report explicitly identifies that state. |
| Migrations / config implications | None. No preferences, profiles, credentials or database state changed. |
| Out-of-scope changes | Product source, production data, external framework/dependency changes, and remediation. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `cb6e00d633772de538498384a8b14d32d25f7915` |
| Verdict | n/a — research/audit handoff; no independent code-review approval claimed |
| P0 / P1 / P2 counts | Current findings: P0=0, P1=2 open, P2=22 (16 verified source/live/configuration/coverage; 6 suspected). No product-code regression introduced by this documentation/evidence-only continuation is known. |
| Findings | Two P1 defects remain open, including one live BSN reproduction. Six live BSN UI/safety-affordance findings include clipped Structure types, ER search with blank canvas, stale Security status, predefined-role Drop, clipped Workbench actions and the conditional synthetic INSERT gate risk; one separate statusbar collision is synthetic-only. Other P2 source, test and suspected-risk findings are separated in `QA_AUDIT_REPORT.md`. |
| CI disposition | Not run in this continuation; prior CI/test states are separately qualified in the report. |
| Next task(s) unblocked | Separate remediation task may address the confirmed defects after this read-only audit is reviewed. Keep the audit plan active while screenshot/focus/provider gates remain open. |

## 5. Research / audit handoff

- Source date: 2026-10-20.
- Source URLs / references: `AGENTS.md`, `RULES.md`, `docs/plans/FEATURE_LIFECYCLE.md`, active audit plan/checklist/findings/verification, `docs/plans/_template/AGENT_EVIDENCE.md`, inspected source/test/configuration paths cited by exact line in `QA_AUDIT_REPORT.md`, and `evidence/runtime/MANIFEST.md`.
- Factual findings: live BSN screenshots reproduce stale table-filter failure and stale activity-status feedback; live Structure types and Workbench view actions clip; ER search reports matches with a blank canvas; PostgreSQL role UI exposes Drop; Transfer shows a synthetic INSERT action without Production confirmation on the connected profile (conditional risk; not exercised). Safe query History shows `SELECT 1`. A synthetic fixture reproduces statusbar label collision. Source-confirmed targeted retry clears the entire staged set. Full evidence and limitations are in the report and manifest.
- Inference: suspected concurrency and performance impacts are explicitly marked suspected; no customer data mutation, SQL write, or destructive operation was used to substantiate a finding.
- Decision / recommendation: keep the audit plan active and hand off the prioritized defects for remediation; do not treat any NOT VERIFIED matrix item as a pass.
- Unresolved questions: exact cause of the user-reported read-only screen; full activity/feature-state coverage; focus restoration/order and accessibility semantics; dark/wide layout; provider-specific permission and timeout behavior; whether the BSN profile’s environment mapping reflects the intended production policy; independent disposable PostgreSQL transaction-failure verification.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

- Đã tiếp tục kiểm tra UI thực tế BSN ở các màn hình an toàn, bổ sung checklist 25 feature, hai finding P2 về clipping ở Schema Workbench và affordance INSERT có điều kiện, cùng ảnh đã kiểm tra/redact/hash trong manifest. Không sửa source/test và không ghi dữ liệu BSN. Còn 2 lỗi P1 cần remediation; focus/accessibility, dark/wide, nhiều feature và PostgreSQL rollback vẫn chưa xác minh. Giữ plan ở trạng thái active.
