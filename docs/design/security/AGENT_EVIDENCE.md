# Agent evidence — Security feature research

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Main · product research lane |
| Issue(s) | n/a — user requested Security after Monitoring |
| Task state | Done — native source baseline, feature assessment, and handoff recorded |
| Baseline SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Branch / PR | `docs/files-tab-design` / no PR |
| Scope interpretation | Source-grounded assessment of native Security navigation, role/password/membership/grant/RLS paths, service/provider boundaries, safety, references, and Phase E/product-goal drift. |
| Out of scope | Product implementation, security fix, goal/status transition, automated/provider tests, live database testing, native UI runtime evidence, and independent approval. |

## 2. Progress checkpoint

- Current source baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`.
- Acceptance rows: [x] traced activity and role/RLS flow; [x] checked PostgreSQL-only service boundary and SQLite unsupported path; [x] compared PostgreSQL role/grant semantics and DBeaver object-context navigation; [x] assessed secret handling, grants, confirmation, audit, scope and documentation drift; [x] wrote three artifacts.
- Remaining rows: none for this research task. Implementation, plan/status transitions, and provider/native-UI verification remain separate work.
- Findings / risks (source claims refer to the baseline SHA):
  - **P1 password confidentiality risk, source-derived, not runtime-reproduced:** `PostgresUserManager::update_password` embeds the password in a dollar-quoted SQL string and executes it without bind parameters. PostgreSQL's official `CREATE ROLE` documentation warns unencrypted passwords are transmitted in cleartext and may be logged in server logs. Phase E's “never logged” requirement cannot guarantee behavior of PostgreSQL server logging (`crates/infrastructure/src/postgres/user_manager.rs:73-89,184-190`; `crates/infrastructure/src/postgres/connector.rs:175-196`; `crates/ui/src/security_role_details_view.rs:89-100`; `docs/goals/goal-phase-e-security.md:105-107`; PostgreSQL CREATE ROLE docs).
  - **P2 access-change safeguards:** role/grant/membership/password UI commands dispatch directly; Drop only uses a simple yes/no warning and there is no role/grant change preview or environment-bound confirmation. The service enforces read-only mode and PostgreSQL routing, but `UserService::drop_role` has no active-connection-role guard (`crates/ui/src/security_activity_view.rs:71-169`; `security_confirmation_view.rs:20-34`; `core/src/application/user_service.rs:29-39,74-78,178-185`).
  - **P2 incomplete role/membership view:** UI exposes only selected role attributes; membership view is member-to-role only, and models omit PostgreSQL membership options/effective access details (`crates/ui/src/security_role_details_view.rs:46-244`; `crates/core/src/domain/user.rs:5-62`; `crates/infrastructure/src/postgres/user_manager.rs:115-219`).
  - **P2 partial privilege results:** the adapter silently omits schema/sequence/database results when secondary queries error, with no partial-result marker (`crates/infrastructure/src/postgres/user_manager.rs:239-320`).
  - **P2 missing audit and feedback contract:** no durable audit write was found in the inspected UserService/UserApi mutation path; an Audit reader elsewhere is not evidence of mutation records (`crates/core/src/application/user_service.rs:10-20,62-176`; `crates/runtime/src/api.rs:872-979,1073-1087`).
  - **P2 current-state/IA drift:** Phase E and capability-matrix claims of no native UI conflict with a source-wired Security activity; the goal excludes RLS but current surface includes it; target goal placement is Explorer rather than top-level rail (`crates/ui/src/activity_bar_view.rs:85-96`; `crates/ui/src/security_surface_view.rs:23-59`; `docs/goals/goal-phase-e-security.md:1-19,87-96,105-158,691-693`; `docs/goals/goal-full-product.md:239-258`; capability matrix §11).
- Tests already run: none. No build, native app, PostgreSQL/SQLite provider, role mutation, password change, or destructive action was exercised. Source tests were not run.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Commit list | none — research artifacts are uncommitted |
| File / surface inventory | `security-baseline.md` — current implementation/provider/safety and source drift; `security-feature-research.md` — product model, IA, provider matrix, references, prioritized follow-up; this file — evidence handoff. |
| Acceptance mapping | Security topic → three artifacts; PG/SQLite provider split → baseline and assessment; password/action safety → P1/P2 findings; product placement/scope → recommendation and docs drift; external workflow/provider contract → PostgreSQL and DBeaver references. |
| Commands and counts | Source reads/search and official documentation reads only. No build/test/runtime command executed. |
| CI run IDs / status | not run |
| Known limitations | Findings are source-only; no provider/runtime validation, no independent review, no implementation, no evidence of actual password logging. |
| Migrations / config implications | none — no source or persistent data changed |
| Out-of-scope changes | No code, tests, plans/status, capability matrix, Phase E goal, UI, database, or runtime behavior changed. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (source baseline; research artifacts uncommitted) |
| Verdict | n/a — research handoff, not independent code review or implementation approval |
| P0 / P1 / P2 counts | introduced by this documentation-only change: 0 / 0 / 0; present at source baseline: 0 / 1 / 5. Findings are source-level gaps, not reproduced incidents. |
| Findings | See §2 and `security-feature-research.md` “Prioritized open work”; no code verdict is asserted. |
| CI disposition | not run — documentation-only research |
| Next task(s) unblocked | Remove plaintext password statement exposure; define safe apply, current-role deletion, and audit contracts; make privilege-query partial failure visible; align Security under Explorer and RLS under table scope; reconcile Phase E/capability documentation. Verify PostgreSQL and SQLite separately and collect native UI evidence before any lifecycle completion. |

## 5. Research / audit handoff

- Source date: 2026-09-24.
- Repository references, all at exact SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`:
  - UI/navigation: `crates/ui/src/activity_bar_view.rs:85-96`; `sidebar_view.rs:64-121`; `security_surface_view.rs:23-59`; `security_roles_view.rs:9-75`; `security_role_details_view.rs:46-244`; `security_confirmation_view.rs:15-40`; `security_rls_view.rs`; `security_activity_view.rs:71-169,184-303`; `security_state.rs`; `security_rls.rs:1-98`.
  - Protocol/runtime/service: `crates/ui/src/runtime_protocol.rs:5-6,320-391,532-549`; `crates/ui/src/event_router.rs:59-66`; `crates/ui/src/operation_events.rs:267-287`; `crates/ui/src/management_events.rs:101-137`; `crates/runtime/src/worker.rs:2344-2365`; `crates/runtime/src/api.rs:872-979,1073-1087`; `crates/runtime/src/lib.rs:253-266`; `crates/core/src/application/user_service.rs:29-185`; `core/src/application/rls_service.rs:20-61`; `core/src/application/schema_service.rs:217-235`.
  - Domain/provider: `crates/core/src/domain/user.rs:5-62`; `crates/core/src/ports/user_manager.rs`; `crates/infrastructure/src/postgres/user_manager.rs:11-89,114-405`; `crates/infrastructure/src/postgres/connector.rs:175-196`; `crates/core/src/application/object_mutation_service.rs:71-98`.
  - Product documents: `docs/goals/goal-phase-e-security.md:1-19,21-62,87-96,105-158,691-693`; `docs/goals/goal-full-product.md:239-258`; `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §11; `docs/architecture/security-boundaries.md`.
- External references read 2026-09-24:
  - https://www.postgresql.org/docs/current/sql-createrole.html
  - https://www.postgresql.org/docs/current/sql-grant.html
  - https://dbeaver.com/docs/dbeaver/Database-driver-PostgreSQL/
- Factual findings: Security role administration and RLS are source-wired; role mutations are PostgreSQL-only and read-only-gated; the adapter quotes identifiers and allowlists privilege strings; password value is embedded in executed SQL text; secondary privilege-query errors can be ignored; SQLite has no role/RLS support in this path. No runtime result is claimed.
- Inference: PostgreSQL statement logging/configuration may capture password text; this is a documented risk, not observed exposure. An absent service-level active-role check and simple confirmation are safety gaps; database acceptance/effects were not tested. Product IA recommendation derives from the stated full-product goal and DBeaver's connection-object organization.
- Decision / recommendation: use Explorer-scoped connection/database Security for roles and table-scoped RLS entry; fix password confidentiality first; add review/apply/audit semantics and explicit partial-result states before claiming the admin workflows complete.
- Unresolved questions: exact product support for high-risk PostgreSQL attributes (SUPERUSER, REPLICATION, BYPASSRLS); intended scope of app audit versus server audit; which privileges/role membership options are in the initial user workflow; password-setting protocol that meets supported PostgreSQL versions and connection configurations.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã khảo sát Security tại SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và ghi ba tài liệu baseline, đánh giá tính năng, và evidence handoff. Mã nguồn hiện có giao diện native cho quản lý role, membership, privilege và RLS; service chỉ cho PostgreSQL và chặn mutation trên kết nối read-only. Phát hiện P1 source-level: mật khẩu được ghép vào SQL statement text dù đã dollar-quote, trong khi PostgreSQL cảnh báo dạng mật khẩu này có thể được truyền rõ và ghi vào server log; chưa quan sát rò rỉ runtime. Các khoảng trống P2 gồm xác nhận/audit thay đổi quyền, hiển thị lỗi truy vấn privilege bị bỏ qua, role/membership model chưa đầy đủ và tài liệu Phase E lệch với UI hiện tại. Không sửa code, không chạy build/test, không khởi chạy UI hay kết nối PostgreSQL/SQLite.
