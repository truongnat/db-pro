---
schema_version: 1
id: "MEM-PLAN-release-qualification-matrix-14eed387c20e"
title: "Plan archive: Release Qualification Matrix"
type: workflow
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, workflow, release-qualification-matrix]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "14eed387c20e49c016be067af7ab65ec58691c8e110afb2d73f044b2cafc4115"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/release-qualification-matrix/PLAN.md"
  - "docs/plans/active/release-qualification-matrix/FINDINGS.md"
  - "docs/plans/active/release-qualification-matrix/VERIFICATION.md"
  - "docs/plans/active/release-qualification-matrix/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/release-qualification-matrix/`. Loại suy luận: `workflow`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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
- Không phục hồi archived frontend/Tauri, không add permanent test framework/runner/dependency, không tạo OS accounts/certificates/real API

… [truncated for memory entry; see source file]


## Source: `FINDINGS.md`

# Findings — release qualification matrix planning

State: **PLANNING**. Source baseline for all code anchors: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`. Planning decisions, historical failures and unexecuted hypotheses are distinct. Không QA runtime test hoặc fix app trong lượt này.

## 1. Historical release blockers to retain, not re-prove by prose

- Prior [release assessment](../../../release/2026-10-24-release-readiness.md) records actual non-capture macOS release close → SIGABRT/NSRangeException; source/artifact/host and raw log in that evidence folder. LIF-02/03/04 address new-candidate regression/lifecycle. Không seed case PASS, không gọi capture-only hoặc count as new execution.
- Notice/license wording, tag/version mismatch, old candidate dossier and test guard-skip evidence are R-02..05 in assessment. REL-02/04, SEC-03, CON-07/09 and aggregation guard checks cover them.
- Prior current-SHA workspace/provider suite PASS hỗ trợ test-basis, **không qualify native mutation/auth/restore/platform cases trong matrix mới**. 12 MySQL harness passed included3 early-return guard cases; preserve that distinction.
- Assessment P0/P1 count means none identified within its scope, not exhaustive zero-defect guarantee or owner release approval.

## 2. Planning source discoveries and decisions

### P-01 — UI-storage/persistence identity needs explicit qualification

- Runtime data dir in `native-app/main.rs:171–184` is not proof of GUI/settings storage isolation; `main.rs:585–607` calls eframe with app identity `DB Pro` and creation_context.storage, while `app_lifecycle.rs:44–138` writes settings/docs/session layout through eframe Storage.
- Planning command `cargo +1.95.0 tree --locked -p db-pro-native -e features -i eframe` shows accesskit/default_fonts/glow/wayland/x11, **no persistence feature**. This is source/dependency evidence only. It raises a concrete persistence risk, not a fresh runtime diagnosis of all stored-state behavior.
- Prior audit's temp DB_PRO_DATA_DIR qualification proves runtime metadata isolation/SQLite connection identity only. It must **not be generalized to user GUI preferences or full workspace isolation/persistence**. New matrix requires OS sandbox/verified GUI-storage namespace and actual normal save/Quit/restart.
- No code toggle/fix added; LIF-04/06, FIL-01, SET-01/03, ER-02 measure actual behavior. If missing storage causes observable requirement failure in valid sandbox, report FAIL, not an infrastructure exemption.

### P-02 — Actual affordances differ from generic QA assumptions

- Query parameters are inline `Parameters · N` panel, not a guessed modal (`query_parameters_view.rs:13–94`). Values intentionally memory-only; GUI restart must clear parameter values rather than require their persistence.
- Native data edits need connected/writable table with complete PK; no-PK capability alone is not editable-grid support (`table_editor_context.rs:10–69`). Composite positive and no-PK/view negative oracles are separate.
- Table toolbar action is localized **Save**; dirty navigation offers **Apply Changes / Discard Changes / Cancel**. Conflict/retry must preserve unrelated pending edits (`table_mutation_actions.rs:409–567`).
- Schema Compare is snapshot/diff/migration preview+apply; Data Compare sync is preview-only (`schema_compare_view.rs:382–461`). Do not demand a nonexistent sync Apply control or silently replay SQL via CLI as UI PASS.
- Transfer real **Copy tables** uses actual connection-ID fields and open source/target handles. Nearby synthetic/export/import/cancel harness controls do not prove real database transfer (`transfer_activity_surface_view.rs:390–526`).
- Current copy string-binds values with first-column conflict key (`core/application/table_copy.rs:50–266`); baseline copy fixture uses first-column PK/text, typed copy boundary not silently equated with typed query fidelity.
- Backup/Restore real UI in Settings; SQLite restore requires disconnected writable target; PostgreSQL restore explicitly not atomic (`settings_backup_view.rs:22–170`, `core/application/backup_service.rs:39–151`, `infrastructure/backup/pg_dump.rs:125–143,316–341`).
- Saved Tasks scheduling is in-app and UI Schedule60s; no arbitrary interval/daemon or New export/maintenance task inferred from payload enum (`saved_tasks_surface_view.rs:57–302`). Builder Apply SQL → editor does not execute (`visual_query_builder_surface_view.rs:276–329`).

### P-03 — Fixture assets are not ready environments

Seed/count contracts in FIXTURES source exist; conflict second-writer, remote99/100/101, positive TLS/CA/mTLS, MySQL reserved-field special server, standalone admin, fault-controller and GUI storage/OS profile still need preparation. Auto-seeded sample is readonly, not suitable for writable positive cases. Missing fixture blocks case, not proof product defect.

Provision script reuse is not pristine reset: persistent DB volumes, host5432 topology assumptions, user known_hosts writes and prewritten PASS snippets. No worker may execute it blindly. New run manifests include server identity, owned namespace/sentinels and independently verified baseline/reset.

### P-04 — GUI concurrency and automation limitations

One desktop has shared focus/pointer/keyboard/AX state. Default schedule serializes all GUI lanes even with independent apps/DBs; parallel only independent desktops or noninterfering read-only tasks. AX success without observable state is not PASS; keyboard/pointer tests require actual modality.

Native close crash may make AX report permission/window errors afterward; independent process exit/stderr disambiguates. Do not global reset Orca/permissions or send unchecked keystrokes into another app. Host clamp is a geometry blocker, not passing a renamed target image.

## 3. Requirements applicability and matrix size

118 definitions contain105 functional and13 visual scenarios; full raw Cartesian candidate list is **3,663 instances** (963 functional,2,700 visual;1,221 per platform). These are planning cells, **not executed tests or promises every static sub-surface has each async state**. Lead preflight approves state/capability applicability with source proof and retains N_A records in denominator accounting. No host/permission/fixture absence removed from candidate set.

High cell count is auditable scope, not instruction to run every combination simultaneously. Dispatch can split ordered waves/independent hosts without losing original IDs or assertions. Do not hide remaining cells behind lane PASS. Actual execution may remain BLOCKED when platform/trust/OS/API prerequisites are unavailable; final report still lists them.

## 4. Open decisions / readiness blockers for future execution

| Item | Owner / required input | Execution effect |
|---|---|---|
| Execution authorization and candidate/channel | Repository owner | No QA run started by this planning request |
| Shipped providers/platforms/preview exceptions | Owner + QA Lead | Default matrix retained; no worker scope reduction |
| GUI/default-keyring/upgrade OS sandbox | Fixture owner + authorized OS admin | No global user state changes; relevant native cases BLOCKED until safe |
| Windows/Linux hosts + Orca support | QA Lead | Cross compile does not close runtime variants |
| Real TLS/SSH/fault/admin fixtures | Fixture owner | Explicit readiness vs BLOCKED, not fake generic localhost |
| AI key/endpoint/synthetic payload/spend | Owner of API account | No external call/billing without separate permission |
| Performance budgets proposed in RUNBOOK | Owner approves plan; lead freezes before sample | No retrospective threshold selection |
| Platform signing/unsigned channel policy | Release owner | Cannot manufacture certs/notarization/trust from local build |

These are prerequisites, not questions about repo-provided facts. Planning deliverable is complete when its integrity checks pass; product release/readiness is not changed by documentation.


## Source: `VERIFICATION.md`

# Verification — planning deliverable only

Plan state: **PLANNING**. Product baseline `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`. Không build/lint/app test/Orca GUI/database mutation trong planning task này; prior assessment measurements không được tính là matrix execution.

## 1. Source/intake checks actually performed

- `git rev-parse HEAD && git status --short`: baseline đúng full SHA; existing untracked release assessment/evidence được giữ nguyên, không stage/commit/push. Đây là intake identity check, không claim new artifact.
- Đọc lifecycle/STATUS, release assessment, actual capability presets, fixture tree, app feature paths và canonical agent template.
- Hai scout read-only maps: data/tools actual affordances và fixture/query/connection/storage requirements; không delegate top-level plan, không subagent GUI execution.
- `cargo +1.95.0 tree --locked -p db-pro-native -e features -i eframe`: exit0; output accesskit/default_fonts/glow/wayland/x11, không persistence. Source/dependency introspection, **không phải build/test/runtime persistence proof**.

## 2. Catalog and schema validation actually performed

Python Eval cell **“Validating catalog traceability and result schema”** read actual saved CASES.json/RESULT.schema.json/REQUIREMENTS/FIXTURES/DISPATCH, using installed `jsonschema.Draft202012Validator`.

| Integrity check | Observed result |
|---|---|
| Case definitions unique/complete |118, no duplicate IDs, all required design fields nonempty |
| Requirement traceability |24 groups; every case+requirement referenced in REQUIREMENTS |
| Fixture contract references |37 IDs; all referenced in FIXTURES |
| Lane ownership |12 lanes L0–L11; each case exactly once in assignment map |
| Baseline / execution status |Every definition full baseline SHA and initial NOT_TESTED |
| Visual state recipes |Every declared visual state has a concrete recipe |
| Cartesian candidate IDs |3,663 unique;963 functional +2,700 visual;1,221/platform before lead applicability |
| JSON Schema meta-validation |PASS, Draft2020-12 valid |
| In-memory structural probes |**12/12** expectations satisfied, no failed probe |
| Catalog/traceability errors |**0** |

Probes were explicitly synthetic **in-memory schema checks**, not real result files: valid NOT_TESTED shape; valid PASS shape; reject PASS with absent checks/evidence, unperformed/failing check, zero evidence or blocked cleanup; reject BLOCKED without dependency, N_A without approval, FAIL without evidence/defect, short source SHA and unknown fields. No synthetic PASS saved under runs or reported as product test success.

Schema only enforces structure. Cross-artifact, check coverage, real screenshot/oracle, lease, semantic N_A and exact geometry validation remain future lead responsibilities described in AGGREGATION.md. No aggregator implementation claimed.

## 3. Final document/lifecycle integrity

Python Eval **“Validating full plan links and lifecycle registration”** completed after all handoff files existed: 12 expected files nonempty; all 6 local Markdown links resolved; no malformed checkboxes; canonical handoff sections 1–6 present; exactly one STATUS entry matching PLANNING; no `runs/` output or fake execution records created. **0 errors.** No app runtime gate is implied by document integrity.

## 4. Explicitly NOT EXECUTED

- No QA worker assignment/run manifest/fixture provisioning/expected-instances run output created.
- No GUI lease or Orca action, native capture, DB operation, Keychain/TLS/SSH/AI call.
- No product implementation, permanent tests/runner/dependency, artifact build/signing or CI dispatch.
- No current matrix PASS/FAIL/BLOCKED/N_A observations; definitions remain NOT_TESTED. Historical open risks remain historical findings, not new execution results.
- No commit/push; owner receives plan before authorizing later execution.


## Source: `AGENT_EVIDENCE.md`

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

