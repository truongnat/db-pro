---
schema_version: 1
id: "MEM-SRC-0e30dfc0843e30df"
title: "Source mirror: docs/plans/active/release-qualification-matrix/AGENT_EVIDENCE.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/AGENT_EVIDENCE.md"
source_sha256: "0e30dfc0843e30df37ae961117b153965124f8031c0cba7e550f18f7b75ebe0a"
mirror_body_sha256: "0e30dfc0843e30df37ae961117b153965124f8031c0cba7e550f18f7b75ebe0a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Agent evidence — release qualification requirements matrix planning

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp planning lane; two bounded read-only scouts for source/fixture mapping |
| Issue(s) | Owner request: detailed requirements matrix and Orca/subagent execution plan before authorization |
| Task state | Run `rqm-20261024T104824Z` **INVALIDATED** (rustc pin breach + retracted PASS); successor preflight `runs/rqm-next-pin195/` only |
| Baseline SHA | `dd2bf5e4090e6d509d34c0c8c58ca95015d91640` — product source used to design cases |
| Branch / PR | `main` / no PR; planning docs uncommitted working-tree artifacts, not claimed to exist in baseline commit |
| Scope interpretation | Detailed traceable native requirement/case catalog, safe fixtures, worker execution and machine-result/aggregation contracts |
| Out of scope | Product fixes; running GUI tests/provisioning; source dependencies; publication/commit/push; external AI calls |

## 2. Progress checkpoint

- [x] Release assessment, current native features/capabilities, fixture source and lifecycle reviewed.
- [x] 24 requirements → 118 definitions (105 functional + 13 visual); all initial NOT_TESTED.
- [x] 37 fixture contracts and 12 lane assignments, one GUI lease per desktop, current artifact/secret/storage safety rules.
- [x] Result schema, append-only evidence/attempt/cleanup contract and lead release decision rules.
- [x] Catalog/schema validation: 0 errors; 3,663 unique pre-applicability candidate cells; 12/12 in-memory schema probes meet expected accept/reject behavior.
- [ ] Owner reviews and authorizes a future run; actual artifact/fixture manifests and all case execution still pending.
- Risk: prior release crash/evidence/packaging gaps retained; no product PASS imported from old audit. New planning storage concern is source/dependency evidence, not freshly tested runtime failure.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Product source `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`; new plan docs not committed |
| Commit list | none in planning task |
| File / surface inventory | New `PLAN.md`, `REQUIREMENTS.md`, `CASES.json`, `FIXTURES.md`, `RUNBOOK.md`, `DISPATCH.md`, `RESULT.schema.json`, `AGGREGATION.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md`, this `AGENT_EVIDENCE.md`; update `docs/plans/STATUS.md` with PLANNING row |
| Acceptance mapping | User detailed list→CASES/REQUIREMENTS; future Orca worker→RUNBOOK/DISPATCH/FIXTURES; pass/fail report→RESULT.schema/AGGREGATION; prerequisite/history→FINDINGS |
| Commands and counts | Intake `git rev-parse HEAD && git status --short`; `cargo +1.95.0 tree --locked -p db-pro-native -e features -i eframe` exit0; Python Eval catalog/schema validation0 errors/12 probes successful; full detail in VERIFICATION |
| CI run IDs / status | not run for planning task; historical assessment CI facts remain historical |
| Known limitations | No runner/fixtures provisioned; platform/OS/AI/trust prerequisites need separate permission/readiness; future case execution initially NOT_TESTED |
| Migrations / config implications | none; no source/config/schema/keyring/system changes |
| Out-of-scope changes | Existing untracked prior release assessment/evidence preserved; no application edit |

## 4. Review outcome

- Reviewed product SHA: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640` only as planning basis, not fresh product review.
- Verdict: n/a — author self-check of documentation, not independent release approval.
- P0/P1/P2 counts: n/a new product audit; historical assessment findings not relabeled new regressions.
- CI disposition: not run; plan-only integrity checks in VERIFICATION do not substitute runtime.
- Next task(s) unblocked: owner review; afterward candidate/fixture/environment readiness and bounded execution per PLAN. No downstream execution auto-started.

## 5. Research / audit handoff

- Source date:2026-10-24.
- Source references @ baseline full SHA: `crates/core/src/domain/capabilities/capability_presets.rs:15–345`; `crates/native-app/src/main.rs:585–607`; `crates/app/src/app/app_lifecycle.rs:44–138`; native table/query/schema/tools/settings files listed in REQUIREMENTS/FIXTURES; seed/verify assets under `fixtures/`; canonical lifecycle/template and prior assessment.
- Factual findings: real query parameters are inline; table Save/PK requirement; Data Compare preview-only; transfer harness distinct from Copy tables; scheduler in-app60s; default runtime-data isolation is not GUI-storage proof. Actual dependency feature graph lacks eframe persistence feature; no runtime conclusion asserted from that alone.
- Inference: storage/persistence needs explicit safe qualification and may explain incomplete prior GUI-state continuity; root cause/product defect status remains unverified until actual normal lifecycle test.
- Decision/recommendation: approve/revise plan, then issue worker packets with frozen actual candidate/fixture manifests and exclusive desktop leases; lead aggregates evidence, not boolean claims.
- Unresolved prerequisites: actual execution authorization/channel/scope; sandbox OS/keyring/storage; Windows/Linux hosts; real fault/TLS/SSH/admin/MySQL-specific fixtures; explicitly consented AI test key/spend; performance policy approval.
- Downstream tasks activated: none automatically; execution checklist remains unchecked.

## 6. Tổng kết bằng tiếng Việt

Đã tạo bộ plan chi tiết có24 nhóm requirements,118 case definitions,37 fixture contracts,12 lane ownership và schema trả kết quả để subagent dùng Orca tự thao tác/kiểm thử rồi lead tổng hợp. Đã kiểm tra catalog/traceability/schema; không chạy app test, không sửa code và không commit/push. Plan giữ PLANNING để chủ sở hữu xem trước; mọi case mới NOT_TESTED, không lấy PASS cũ để lấp chỗ trống. Điểm bắt buộc là cô lập runtime lẫn UI/keyring, chỉ một GUI worker mỗi desktop, oracle độc lập, giữ nguyên blocked/unsupported/history và không dùng mock/harness/ảnh sai kích thước thay runtime thật.
