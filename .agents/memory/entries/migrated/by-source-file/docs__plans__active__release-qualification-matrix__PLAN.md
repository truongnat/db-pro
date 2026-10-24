---
schema_version: 1
id: "MEM-SRC-69a83bf8fdfb8875"
title: "Source mirror: docs/plans/active/release-qualification-matrix/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/PLAN.md"
source_sha256: "69a83bf8fdfb8875c70cf0c1574837c23df7b248086edd424ff6ac94bb44f557"
mirror_body_sha256: "69a83bf8fdfb8875c70cf0c1574837c23df7b248086edd424ff6ac94bb44f557"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Release Qualification Matrix — Implementation and Test Execution Plan

> **For agentic workers:** Khi owner cho phép thực thi, đọc installed `subagent-driven-development` hoặc `executing-plans`, cùng `qa-orchestrator`, `requirements-test-designer`, `release-readiness-auditor`. Đây là plan cho việc chuẩn bị/chạy kiểm thử, **không cho phép implementation fixes**. Chủ sở hữu sẽ giao subagent execution sau khi xem plan; dừng ở lập kế hoạch trong lượt hiện tại.

**Goal:** Một danh sách requirements/cases traceable để subagent tự điều khiển DB Pro bằng Orca, xác minh bằng oracle độc lập, trả PASS/FAIL/BLOCKED/NOT_TESTED/N_A và được QA Lead tổng hợp thành release recommendation có bằng chứng.

**Architecture:** Một canonical CASES.json chứa test design; frozen run manifest mở rộng thành các instance theo platform/provider/theme/size/state/surface. Native workers dùng real release artifact và một GUI lease duy nhất trên mỗi desktop; DB clients chỉ làm fixture/second writer/oracle/cleanup. Typed append-only results + evidence hashes được lead/reviewer đối chiếu với expected instances trước quyết định release.

**Tech Stack:** Rust workspace, native eframe/egui, SQLite/PostgreSQL/MySQL/SQL Server, Orca computer-use, platform trust/tools và DB-native clients. Không React/WebView/Node build hoặc browser automation thay native app.

**Spec:** [REQUIREMENTS.md](REQUIREMENTS.md), [CASES.json](CASES.json), [FIXTURES.md](FIXTURES.md), assessment [2026-10-24 release readiness](../../../release/2026-10-24-release-readiness.md).

**State:** `PLANNING` — plan đã được author/validate, **chưa authorise hoặc execute matrix**. Product source baseline `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`; future candidate freeze lại full SHA, không mặc định luôn HEAD. Làm trực tiếp `main` theo owner override; không tạo worktree/feature branch.

## Global Constraints

- Actual packaged release/no capture là evidence chính. Debug/capture/harness/source/unit suite chỉ supplemental và luôn gắn artifact role riêng.
- Release compiler pin hiện tại **1.95.0**; formatting nightly theo repo; actual version phải được lưu. `RUSTUP_TOOLCHAIN=stable` không thay qualifying pin.
- Database/provider/OS test độc lập; selectable engine không đồng nghĩa đã qualified. Default target matrix: macOS ARM64, Windows x86-64, Linux x86-64; bốn providers có case applicability rõ.
- Six-cell visual profile: Light/Dark × content1280×800,1440×900,1920×1080; actual geometry/state được chứng minh, clamp không PASS requested height. Không nhân six-cell vào mọi functional scenario.
- Mỗi case có fixture/preconditions/steps/independent oracle/cleanup/priority/evidence; không biến thiếu fixture/host/input permission thành PASS hoặc N_A tiện lợi.
- Single GUI worker trên mỗi OS desktop; independent app windows không cho phép parallel pointer/keyboard. CLI chỉ parallel khi không thay artifact hoặc dữ liệu của active case.
- Không source edit, bypass/suppress warning, shim/mock/retry implementation trong QA. Defect → riêng remediation/new candidate, không tự sửa rồi tiếp tục như cùng artifact.
- Không mutation user/production data, không global keychain/SSH/trust/permissions cleanup, không publish/tag/push; AI/live external call/billing cần authorization riêng.
- `DB_PRO_DATA_DIR` không đủ proof GUI/settings isolation; default keyring test không thay bằng file-fallback. Missing OS/GUI-storage isolation => BLOCKED before action.
- Repository severity P0/P1/P2; priority critical/high/normal là test scheduling, không phải finding severity. Known P2 shutdown có thể chặn release.

## Review Focus

1. **Wrong target:** document/schema/global connection differ; copy/diff/restore/monitor signals must never target a convenience connection. Owned sentinel/oracle before and after.
2. **Silent loss:** staged edit/targeted retry/conflict/restart/upgrade; preimage and independent full-key final rows required, not just success toast.
3. **False automation PASS:** AXSetValue failed SQL entry, AXPress used as keyboard proof, synthetic transfer harness, queued task, guard-skip suite total, screenshots wrong surface.
4. **Host/artifact mismatch:** debug process already running, capture build replacing release, ambient compiler override, pixel/titlebar/clamped-height confusion, shared eframe/UI storage.
5. **Unqualified safe-looking operations:** secret save/restart, partial PG restore, MySQL nontransactional DDL, no-PK edit restriction, preview-only data sync and explicit no-key AI preview. Source contract determines expected result, not assumed feature parity.

Each risk maps to concrete cases in REQUIREMENTS.md; these checks are not free-form optional exploratory notes.

## Deliverable map — one source of detail per concern

| File | Responsibility |
|---|---|
| PLAN.md | Scope, sequencing and execution acceptance |
| REQUIREMENTS.md | 24 requirement groups, case/risk traceability and coverage interpretation |
| CASES.json | **118 detailed definitions:105 functional +13 visual**; all initial NOT_TESTED, concrete actions/oracles |
| FIXTURES.md | 37 fixture contracts, real reusable assets vs required extensions, safe isolation/reset |
| RUNBOOK.md | Native Orca loop, GUI lease, status/visual/input/measurement/safety rules |
| DISPATCH.md | L0–L11 ownership, skill routes, real labels and copyable worker packet |
| RESULT.schema.json | Machine-readable per-attempt record contract, no fake results |
| AGGREGATION.md | Expected-instance expansion, evidence validation, counters, dedup and channel gates |
| CHECKLIST.md | Plan readiness separated from future execution/sign-off |
| FINDINGS.md | Historical release risks, planning discoveries and unresolved prerequisites |
| VERIFICATION.md | Checks actually performed on documents, not QA execution results |
| AGENT_EVIDENCE.md | Canonical planning handoff with exact SHA and Vietnamese summary |

Future run artifacts live under this directory's `runs/<run_id>/`; **no empty mock run/results scaffold is created now**. Parent `docs/plans/STATUS.md` registers this single plan as PLANNING.

## Task 1 — owner review and candidate/scope freeze (L0)

**Inputs:** this plan, release assessment and actual chosen artifact/source. **Outputs:** run manifest, authorization/exception decisions, candidate planned instance set.

- [ ] Owner reviews plan; record execution authorization/channel/provider/platform decisions. No execution implied by authoring plan.
- [ ] Record actual candidate/source/binary/compiler/features and existing dirty changes. Existing readonly audit results linked as history, not copied PASS.
- [ ] Freeze expected instance IDs from CASES.json; record state/capability N_A approvals and conditionally advertised features before results. Missing host/fixture stays visible BLOCKED.
- [ ] Assign unique lane/worker/desktop/fixture owner, output paths, deadlines and GUI lease policy. Prove no shared mutable user state.

**Acceptance:** no ambiguous artifact/ownership or unapproved mutation scope; task states Ready only when manifest inputs exist. No execution outputs invented.

## Task 2 — artifact and fixture readiness (L1 + fixture owner)

**Inputs:** frozen source/scope; FIXTURES contracts. **Outputs:** actual qualifying artifact/package+hash, verified fixture manifests/independent baselines/reset recipes.

- [ ] Single owner runs pinned release gates/build/package, records logs and skip guards; source not edited. No GUI worker build/lint/formatter mid-flight.
- [ ] Register sandbox OS/UI-storage/keyring and backend topology; only manifest-owned resources mutable. Prepare missing scale/conflict/CA/SSH/admin/test-key assets explicitly or mark their cases BLOCKED.
- [ ] Query provider identities/expected cardinalities/types/catalog and outside sentinels; verify reset on a clone. Do not reuse persistent fixture state as pristine.
- [ ] L0 completes ENV-01..03, fixture trust and final instance freeze; downloaded/trust package check distinct from direct launch diagnostic.

**Acceptance:** fixture ready evidence genuine; native app PID/hash/window/profile known; no user saved connection or global policy touched.

## Task 3 — functional/native execution (L2–L10, serial desktop leases)

**Inputs:** assigned case instances + artifact/fixture manifests. **Outputs:** append-only result.json and evidence per attempted instance, worker AGENT_EVIDENCE.

- [ ] Execute lifecycle/connection before dependent data paths; known crash gets inherited-risk linkage, new-candidate verification uses actual release. Stop mutation for wrong target/P0/P1 concern.
- [ ] Navigate real native actions via fresh Orca snapshots; verify actual SQL buffer/keyboard/pointer modality and expected DB outcomes.
- [ ] Run assigned scenario all oracles/boundaries, including errors/cancel/repeated/concurrent/restart where listed; fixture reset each independent variant.
- [ ] Preserve screenshots/action/DB/process logs with hashes and actual sizes; emit RESULT.schema-compliant statuses; cleanup and release GUI lease.
- [ ] For unsupported controls verify CAP-01 negative boundaries; no CLI/mock substitute for missing UI route. Unavailable AI/OS/trust fixtures blocked, not guessed.

**Acceptance:** every assigned instance explicit result or missing/unattempted accounting; no generic “section PASS”; pending/queued and harness/source evidence classified separately.

## Task 4 — visual/input/accessibility/performance execution (L11)

**Inputs:**13 visual definitions with real state recipes, native fixture/controller, declared budgets and approved platform state applicability. **Outputs:** inspected state/theme/geometry evidence and measured input/response records.

- [ ] Acquire GUI lease, reproduce named sub-surface/state on actual release; size/content/DPI measured before capture.
- [ ] Cover requested six-cell profile per eligible state; keep clamped/unavailable variants BLOCKED with actual dimensions.
- [ ] Run keyboard-only, focus, pointer hover/cursor and approved screen-reader flow separately from AX tree actions.
- [ ] Collect timed responsiveness/ER/idle samples per RUNBOOK budgets; no smoothness claim from images. Missing instrumentation is BLOCKED.

**Acceptance:** actual states/surface/input matched, all images worker-inspected; no fabricated loading/empty or duplicated screenshot filling different instance IDs.

## Task 5 — reconcile evidence and issue release recommendation (L0 + reviewer)

**Inputs:** immutable expected instances + worker results/evidence. **Outputs:** SUMMARY.md, coverage.json, defects.json and canonical aggregate AGENT_EVIDENCE (future run outputs).

- [ ] Validate structural schema and semantic identity/oracle/hash/lease/cleanup/geometry rules in AGGREGATION; reject unsupported PASS and report absent outputs.
- [ ] Deduplicate confirmed defects, preserve all impacted instance IDs; separate inherited/introduced/unknown, hypothetical risk and actual failure.
- [ ] Review all P0/P1/high-risk failures independently via evidence; unsafe repeated reproduction not required. No cherry-pick green retry.
- [ ] Produce counts by requirement/provider/platform/state, reasons for all BLOCKED/NOT_TESTED/N_A, proposed next action/owner and distinct internal/external/public verdicts.
- [ ] Owner makes release decision. Update canonical plan status only to actual stage, never COMPLETED with required evidence missing; product fixes get separate candidate and mapped regression run.

**Acceptance:** honest denominator and independently reviewable result, no fabricated readiness percentage. Report can be complete even if app NO-GO; matrix execution itself is not complete while required instances unproven without explicit scope disposition.

## Non-goals / plan boundaries

- Không fix shutdown/persistence/packaging hay lập feature implementation trong lượt này; chúng là findings cần QA/implementation owners xử lý riêng.
- Không phục hồi archived frontend/Tauri, không add permanent test framework/runner/dependency, không tạo OS accounts/certificates/real API subscriptions tự động.
- Không yêu cầu agent làm mọi platform trên một Mac hoặc giả Windows/Linux bằng compile-only. Không yêu cầu PostgreSQL restore atomic trái documented contract hay four-provider feature parity.
- Không xóa/rewrite historical smoke checklist; dùng nó làm coverage input, bỏ assumptions lỗi thời (Monaco, Coming soon, old cancellation flags), không copy165 unchecked items thành PASS.

## Ready for execution when

Owner đã xem plan, chooses/fixes candidate/scope, required manifests và sandbox permissions có thật, fixtures Ready hoặc được đánh dấu BLOCKED đầy đủ. Chưa đáp ứng thì plan vẫn PLANNING; không phải lý do tự giảm matrix hay bắt đầu thao tác GUI.
