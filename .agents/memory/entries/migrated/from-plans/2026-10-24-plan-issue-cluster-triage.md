---
schema_version: 1
id: "MEM-PLAN-issue-cluster-triage-d4de2e53d03c"
title: "Plan archive: Issue Cluster Triage"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-cluster-triage]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d4de2e53d03c2102f5724af9f14c7221208363d29962153774a31ce26d194a47"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-cluster-triage/PLAN.md"
  - "docs/plans/active/issue-cluster-triage/FINDINGS.md"
  - "docs/plans/active/issue-cluster-triage/VERIFICATION.md"
  - "docs/plans/active/issue-cluster-triage/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-cluster-triage/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Continuous issue clusters — 2026-10-16

Lifecycle: REVIEW (selected verified wave published; inherited global gates and provider-blocked work remain explicit).
Baseline: `76b120724a7542de8b587a002811ddacad28156a`.
Owner authorization: commit/push directly on main, group related issues, continue without per-ticket approval, close each fully verified issue. Preserve user changes and existing PR ownership. Never close from title/source inspection alone.

## Requirements and acceptance

- [x] Publish #448 and close with exact commit/runtime evidence.
- [x] Classify all 156 issues in the starting snapshot by shared implementation/verification boundaries.
- [x] Implement selected uncovered coherent clusters, not one large mixed fix; published receipts in PUBLISHING.md.
- [x] Execute regression/runtime evidence for every claimed fix, account for PostgreSQL/SQLite independently and other providers when relevant.
- [x] Commit/push each verified slice; close only its satisfied acceptance, keeping #441/#400 partial/provider-blocked items open.

## Cluster map

This is an inventory, not a claim that every old issue still reproduces or that all will be implemented together. Some have open draft PRs or already-present fixes. The snapshot includes #448, closed after its verified fix was published.

| Cluster | Issues | Current handling |
|---|---|---|
| Data diff identity | #457 #485 | Typed-value/key fix published; both verified and closed |
| Provider-generated SQL | #441 #442 #499 #508 #365 #373 #443 | #441/#442 share dialect correctness verification; remaining items require their own evidence |
| SSH / connection transport | #435 #451 #475 #400 #452 #478 #479 #483 | #435/#451/#475 published/closed; #400 partial remains open; #479/#483 draft PR ownership preserved |
| Scheduled tasks | #504 #462 #375 | #504/#462 published/closed; #375 tracking scope separate |
| SQLite integrity / maintenance | #461 #498 #491 #403 #493 #494 | #491/#498 published/closed; #461 closed not planned after actual bundled-default proof; #403 source guard is not closure proof; #493/#494 owned by PR #527 |
| Query editor text / analysis | #511 #510 #509 #506 #488 #474 #472 #471 #470 #445 #446 | Search/grapheme overlaps PR #524/#523; analyze remaining state bugs separately |
| Query execution / safety | #477 #456 #438 #410 #374 #356 #351 #383 #382 #387 #507 | Respect PR #519/#518; confirmation/session ownership and execution cancellation are separate subclusters |
| Grid selection / clipboard / editing | #505 #497 #496 #487 #480 #463 #447 #444 #413 #412 #408 #389 #388 #369 #352 | Respect PR #526/#525; verify stale issues against main before new fixes |
| Schema workbench / DDL safety | #411 #402 #398 #390 #385 #381 #380 #371 #370 #368 #364 #361 #357 #354 #353 #350 #348 #347 #346 | Several baseline fixes exist; source alone does not justify closure |
| Connection-bound state / monitoring | #501 #500 #490 #489 #482 #481 #473 #467 #454 #440 #439 #407 #406 #399 #386 | Respect PR #520/#521; group by session identity/request ownership |
| Files / Git workspace | #513 #469 #468 #405 #393 #391 #377 | Filesystem/Git process lifecycle and stale-file state |
| Export / import / masking | #503 #502 #486 #484 #404 #396 #395 #384 | Respect PR #515; distinguish format fidelity from export snapshot/file safety |
| Agent orchestration / UI | #448 #409 #397 #394 #360 #358 #337 | #448 published/closed; remaining controller/UI acceptance separate |
| ER canvas | #512 #455 #450 #449 | Respect PR #522 for duplicate self-loop reports; spatial validation separate |
| General UI interaction / layout | #495 #464 #453 #401 #392 #379 #378 #376 #372 #367 #366 #363 #362 #355 #349 #345 #344 #343 #342 #341 #340 #339 #338 #336 #335 #334 | Require actual native visual/input evidence; no source-only closure |
| Provider execution / roadmap | #460 #459 #458 #359 #260 #252 #182 | Provider-specific runtime prerequisites and broad roadmap acceptance; not safe to close as a narrow bug batch |

## Execution / ownership

Parent owns integration, shared changelog/status, publish/closure, SQLite maintenance/error mapping and verification infrastructure. Independent implementers own data diff, generated SQL, scheduler, and SSH transport files. No mid-flight gates in child tasks; parent runs checks after integration.

Each coding cluster owns exactly one separate active plan with requirements/context/DoD/tasks and evidence. Do not overwrite historical plans. New fixes must not inherit unsupported closure claims from old plans.

## Verification and stop conditions

Existing adjacent regression tests plus actual public-service/provider smoke were executed. Disposable PostgreSQL18 and MySQL8.4.11 loopback fixtures and independent SQLite databases were used; existing application databases were untouched. SQLServer and actual bastion-auth evidence remain absent and are not inferred from other providers or a relay. Owned temporary fixtures/projects were removed after proof. Full gate/closure receipts: PUBLISHING.md and VERIFICATION.md.


## Source: `FINDINGS.md`

# Agent evidence — continuous clustered issue work

## 1. Claim
Parent omp coding assistant; owner authorized grouped fixes, commit/push on main, and verified issue closure. Baseline `76b120724a7542de8b587a002811ddacad28156a`; task In Progress. Out of scope: replacing existing draft PR work, unverified bulk issue closure, touching unrelated user databases.

## 2. Progress checkpoint
Starting snapshot: 156 issues, 13 open draft PRs. All issue numbers grouped in PLAN.md. #448 published and closed. Active uncovered slices: #457/#485, #441/#442, #504/#462, #435/#451/#475, #491/#498. #461 requires runtime verification of bundled defaults before deciding it is a defect; #400 is partial pending stage-level diagnostics evidence.

## 3. Implementation handoff / review request
Per-cluster plans carry acceptance, file inventory and verification; parent alone integrates and publishes. Not a handoff of all 156 issues as completed.

## 4. Review outcome
n/a — no independent blanket review. Existing source inspection shows many stale issue reports; no closure based solely on that evidence. Known inherited workspace Clippy failure at baseline is recorded in agent-replay-step-budget/VERIFICATION.md.

## 5. Research / audit handoff
Fetched GitHub issues/PRs on 2026-10-16 via gh CLI. Sources: issue URLs https://github.com/truongnat/db-pro/issues/<number> in PLAN.md, open PR list, exact-baseline source sections read for each selected slice. Inference: grouping by shared root cause avoids duplicate changes/testing; final closure still requires individual acceptance evidence. Repository behavior-search service rejects its judge requests; source-literal fallback used. LSP exits on references requests; no unsafe automated code action applied.

## 6. Tổng kết
Đã push/đóng #448, gom toàn bộ snapshot 156 issue thành 16 cụm và triển khai các cụm độc lập chưa có PR. Chỉ đóng ticket khi đã kiểm chứng; ticket partial hoặc thiếu provider/runtime vẫn để mở.


## Source: `VERIFICATION.md`

# Verification — continuous issue clusters

Baseline: `76b120724a7542de8b587a002811ddacad28156a`.

- `git commit` created 76b120724a7542de8b587a002811ddacad28156a; `git push origin main` succeeded. Remote reported owner-authorized bypass of PR/status-check rules; this is not a green CI claim.
- `gh issue close 448 --reason completed --comment ...` succeeded with exact commit, regression and runtime evidence plus inherited Clippy warning.
- Open issue/PR and all-PR overlap searches for #457/#485/#441/#442 returned no linked existing implementation.
- Backlog mapping script counted 156 starting issues; 153 initially grouped, then remaining #443/#446/#507 assigned to their matching clusters in PLAN.md.
- Default local PostgreSQL socket :5432 does not respond. No existing application container/database modified.
- Started isolated disposable `postgres:18` container `db-pro-issue-smoke-pg-20261016` on 127.0.0.1:25432. Created separate dbpro_smoke_target database; pg_isready reports accepting connections. Existing dbpro_smoke supplied by container initialization.
- Actual MySQL8.4.11 default-mode fixture later established independently; real PK/DDL/cross-database FK checks PASS. SQLServer runtime NOT VERIFIED; #441 remains open.
- Per-fix test and smoke results are recorded in the individual plan directories. No blanket pass or completed-backlog claim.
- Final #533 candidate 6f94372dd5caba22e525fbc1aff918b936e0491d pushed and issue closed with five actual PG tests + unchanged original public INTEGER-PK smoke PASS. Full workspace tests 2,003/0/50; focused mapper 16/0/0; fmt/owned Clippy PASS. Evidence artifacts159/160.
- GitHub final inventory: 144 open; #400/#441 OPEN, #533 CLOSED. Starting156 issues, twelve original issues closed plus newly filed-and-fixed #533 = thirteen closures total, not a completed-backlog claim.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — grouped GitHub issue integration

## 1. Claim

- Agent: parent integration owner; task state Done for the selected published wave, not the entire backlog.
- Authorization: owner requested continuous grouped fixes, commit/push and per-issue closure directly on main; no PR requested.
- Baseline: 76b120724a7542de8b587a002811ddacad28156a (the already-published #448 fix).
- Scope: independently reproduced bounded fixes in schedule/data-diff/SQLite/SSH/generated-SQL/PostgreSQL preparation; open-PR ownership preserved.
- Out of scope: wholesale roadmap completion, unavailable SQLServer runtime, full staged SSH diagnostics, unrelated inherited lint debt and UI redesign.

## 2. Progress checkpoint

- Final product source SHA: 6f94372dd5caba22e525fbc1aff918b936e0491d. Final receipt/test-only follow-through removes clone-on-Copy from existing provider test setup and records evidence; no new runtime behavior.
- Thirteen issues closed total: #448 #504 #462 #457 #485 #491 #498 #461 #435 #451 #475 #442 #533. #461 closed not planned after actual bundled-default non-reproduction, not misrepresented as a new actor fix.
- 144 issues remain open; #400/#441 explicitly confirmed OPEN. Individual issue comments link published code and evidence. Inventory is not proof that every old report reproduces.
- Five explicitly executed PostgreSQL preparation tests pass, final mapper16 pass, workspace2003pass/0fail/50ignored. Source reviews do not count as runtime evidence.

## 3. Implementation handoff / review request

- Exact product SHA and all preceding cluster SHAs: PUBLISHING.md. File inventory/acceptance mapping: each linked plan and source commit; cluster map: PLAN.md.
- Exact commands: `cargo +nightly fmt --all -- --check` PASS; `cargo check --workspace` PASS; `cargo clippy -p db-pro-core -p db-pro-infrastructure -p db-pro-runtime --all-targets -- -D warnings` PASS; `cargo test --workspace` 2003/0/50 PASS; `cargo build --release --locked -p db-pro-native` PASS, all exit0.
- Explicit real-PG command: `DATABASE_URL=postgres://...@127.0.0.1:25432/dbpro_smoke cargo test --locked -p db-pro-infrastructure --test pg_integration pg_typed_query_ -- --ignored --test-threads=1` 5/0/0 PASS. Mapper command `cargo test -p db-pro-infrastructure --lib postgres::query_mapper::tests` 16/0/0 PASS. Actual public-service/provider probes and prior focused suites: individual VERIFICATION.md files; final artifacts159/160.
- Workspace all-target Clippy FAIL: five unchanged app diagnostics. Full performance scan FAIL: expected target/release artifact absent, while actual Cargo target is /data/cargo-target; two executed checks pass, one warning, one fail, four sections unexecuted. Locked native release build independently passes. No performance improvement/benchmark claim.
- Clean-code ratchet12pass/4warn/0fail. Full scanner was not globally green; introduced oversized query/catalog functions were extracted, inherited debt remains. No suppressions/baseline resets used to hide a new defect.
- CI: not verified; remote accepted owner-authorized PR/status-check bypass. Successful push is not CI approval.
- Migrations/config: no dependency/schema migration. Saved-task backlog policy is now interval-aware; SSH intent/credentials/lifetime preserved; SQL capability/provider distinctions retained. Disposable local fixtures require opt-in credentials for reruns; all owned temporary projects/fixtures removed after proof.
- Known limitations: #441 live SQLServer pending; #400 full stage diagnostics pending; SSH relay proves forwarding/process/query lifetime, not real bastion authentication. Existing SQLx same-SQL/different-binary-type cache limitation remains. Archived frontend untouched.

## 4. Review outcome

- Separate source reviewers inspected candidate source later published at the exact SHAs in PUBLISHING.md, without post-review runtime behavior edits. Final bounded PostgreSQL candidate: 6f94372dd5caba22e525fbc1aff918b936e0491d.
- Verdict: ACCEPT within each implemented source scope after findings were corrected; introduced unresolved P0/P1/P2:0/0/0 in those final source reviews. No claim that self-verification replaces external repository approval.
- Inherited global lint failures and unimplemented issue acceptance are reported separately, not reclassified as passes. Candidate review initially exposed PostgreSQL tail draining and NULL inference; both were fixed and verified by actual provider cases.
- CI disposition: unverified. Plans remain REVIEW or RUNTIME_VERIFY where repository-wide/runtime gates remain unmet; no false COMPLETED move.

## 5. Research / audit handoff

- Source date: 2026-10-16. Sources: GitHub issue/PR inventory for truongnat/db-pro, exact cluster commits, installed SQLx0.8.6 and libsqlite3-sys0.30.1 source; references and decisions in individual FINDINGS.md files.
- Findings: 156 starting issues mapped into16 clusters. Baseline/candidate standalone probes established actual failures before fixes; SQLite FK report did not reproduce in the shipped bundled configuration. Source-only old-safety audit closed nothing.
- Inference: none used as closure proof. MySQL/Pg/SQLite evidence is independent; no SQLServer/bastion behavior inferred.
- Downstream unresolved: #400 full diagnostic staging and #441 authorized live SQLServer fixture; other backlog items retain existing ownership and acceptance.

## 6. Tổng kết (Vietnamese summary)

Đã commit/push các cụm độc lập và đóng13 issue có bằng chứng riêng; #461 đóng dạng không tái hiện với SQLite bundled, không nhận là sửa lỗi mới. Test workspace2003/0/50, build release khóa dependency và Clippy các crate đã sửa PASS. Clippy toàn workspace/perf scan vẫn FAIL như mô tả; chưa xác nhận CI. Giữ #400/#441 mở đúng phần còn thiếu, không đóng theo suy đoán; đã dọn fixture/probe do phiên này tạo.

