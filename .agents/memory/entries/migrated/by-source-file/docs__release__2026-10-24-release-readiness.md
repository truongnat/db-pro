---
schema_version: 1
id: "MEM-SRC-cc77fd5dbac8b0f4"
title: "Source mirror: docs/release/2026-10-24-release-readiness.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/2026-10-24-release-readiness.md"
source_sha256: "cc77fd5dbac8b0f4486732c8d299abd56edc69e5f4c2723f38115172da4c569b"
mirror_body_sha256: "cc77fd5dbac8b0f4486732c8d299abd56edc69e5f4c2723f38115172da4c569b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Đánh giá khả năng release — 2026-10-24

## 1. Claim — phạm vi và kết luận

| Mục | Giá trị |
|---|---|
| Agent / lane | omp; release-readiness assessment, hai scout read-only về packaging và safety evidence |
| Task state | Done — assessment/report, không phải release sign-off |
| Baseline / reviewed SHA | `dd2bf5e4090e6d509d34c0c8c58ca95015d91640` |
| Branch / PR | `main`; không tạo PR/tag/release hoặc dispatch CI |
| Artifact | `db-pro-native` 0.1.0, native Rust/egui; portable archive |
| Host | macOS 25.6.0 ARM64 |
| Phạm vi | Fresh local pinned gates; live fixture suites; đóng gói, khởi động/query/theme/đóng/mở lại app macOS; release/security/evidence policy |
| Ngoài phạm vi thực thi | Production DB; Windows/Linux runtime; actual upgrade/uninstall; signed delivery; real AI provider; exhaustive penetration/accessibility/performance testing |

**Kết luận: chưa nên phát hành public hoặc beta ngoài nhóm kỹ thuật.** Có thể tiếp tục thử nghiệm nội bộ trên database disposable với việc công khai lỗi shutdown; đây không phải chứng nhận an toàn cho production.

| Kênh | Quyết định | Lý do |
|---|---|---|
| Engineering preview nội bộ | GO có điều kiện | Build/tests/provider suites tốt; bản macOS chạy được. Chỉ tester biết lỗi, fixture disposable, không coi là RC đã qualified. |
| External beta / release candidate thông thường | NO-GO | Đóng cửa sổ bản release thực tế gây SIGABRT; clean lifecycle, default credential persistence và candidate CI chưa được qualified. |
| Public/stable, ba platform đang cấu hình | NO-GO | Thêm thiếu license/dependency notices trong archive, chưa có signing/notarization, chưa có current-candidate cross-platform artifacts/runtime/upgrade proof. |

Không đưa phần trăm readiness tùy ý: số test xanh không đo được phần rủi ro còn thiếu. **Chưa xác nhận P0/P1 mới trong phạm vi audit; điều này không chứng minh không còn P0/P1.** Năm nhóm finding P2 dưới đây gồm một crash runtime, các vấn đề packaging/docs và độ tin cậy evidence. P2 vẫn có thể chặn release nếu làm hỏng acceptance gate.

**Quy ước source:** mọi `path:line` dưới đây, trừ khi ghi revision lịch sử khác, thuộc đúng SHA đầy đủ ở bảng trên. Snapshot source không thay đổi trong assessment; các file mới chỉ là báo cáo và evidence.

## 2. Progress checkpoint — bằng chứng đã thu

### 2.1 Fresh quality gates

Môi trường mặc định đặt `RUSTUP_TOOLCHAIN=stable`, trỏ tới Rust 1.99.0. Chuỗi đó đã bị hủy, **không dùng để qualify release**. Đã chạy lại tường minh `+1.95.0`, đúng `rust-toolchain.toml:8`; formatting dùng nightly theo quy định repository.

| Gate thực thi | Kết quả |
|---|---|
| `cargo +nightly fmt --all -- --check` | PASS, exit 0 |
| `cargo +1.95.0 check --workspace --locked` | PASS, exit 0 |
| `cargo +1.95.0 clippy --workspace --all-targets --locked -- -D warnings` | PASS, exit 0 |
| `cargo +1.95.0 test --workspace --locked` | Harness: **2.315 passed / 0 failed / 65 ignored**, 27 suites |
| `cargo +1.95.0 build --release --locked -p db-pro-native` | PASS, exit 0; không bật feature `capture` |
| `bash scripts/check-ui-architecture.sh` | PASS, exit 0 |

Log đầy đủ: [pinned-quality-gates.log](evidence/readiness-2026-10-24/pinned-quality-gates.log). Test có guard-return có thể được harness tính passed; không dùng tổng này thay live provider qualification. Không chạy Criterion/frame-time benchmark mới, không tuyên bố performance đã đạt mọi budget. Binary mới **34.239.776 bytes**, dưới mục tiêu 50 MiB trong `.skills/perf-audit/SKILL.md:34–38`; đây chỉ là size measurement.

### 2.2 Provider runtime — từng engine độc lập

Các suite dùng `cargo +1.95.0 test --locked -p db-pro-infrastructure --test <suite> -- --ignored --nocapture --test-threads=1`; cấu hình fixture từ `scripts/provision-test-fixtures.sh`, không dùng database nghiệp vụ. PostgreSQL fixture được xác minh identity/count trước chạy. MySQL dùng cả `dbpro_reference`. Không provisioning mới.

| Provider | Fixture | Kết quả thực tế | Giới hạn |
|---|---|---|---|
| PostgreSQL | Homebrew PostgreSQL 18.6, `127.0.0.1:5432/dbpro_fixture`, role fixture `dbpro` | **40 passed / 0 failed / 0 ignored**, không guard-skip trong log | Gồm copy-Replace rollback failure injection, search_path, decoding, cancellation; không phải toàn bộ native journeys |
| SQLite | Workspace in-process/file suites và packaged app sample riêng | Workspace xanh; native query trả **3 rows**, khớp `sqlite3 SELECT count(*) FROM employees` | Không thao tác mutation qua native UI trong lượt này |
| MySQL | Repo-owned fixture `127.0.0.1:33306`, `dbpro_fixture` + `dbpro_reference` | Harness **12 passed**, nhưng **9 ca thực thi + 3 guard-skip** | Ba ca #452 reserved/Unicode fields, wrong password, strict TLS thiếu `DB_PRO_MYSQL_452_*`; không được tính đã qualified |
| SQL Server | Repo-owned SQL Server 2022 fixture `127.0.0.1:1433/dbpro_fixture` | **9 passed / 0 failed / 0 ignored**, không guard-skip trong log | Bao gồm transaction rollback, cancel/timeout/recovery, SHOWPLAN; không đủ proof cho mọi TLS/type/native scenario |

Log: [live-provider-suites.log](evidence/readiness-2026-10-24/live-provider-suites.log), đặc biệt dòng 161–166 ghi rõ MySQL skips. Tổng 65 ignored của workspace vẫn giữ nguyên; kết quả chạy riêng không được cộng để giả thành một full-suite run khác. SSH, pg_object_identity và provider-conformance theo từng remote engine không chạy riêng trong lượt này.

### 2.3 Package và native lifecycle

Đã chạy:

```text
bash scripts/release/package-macos.sh 0.1.0 target/release/db-pro-native /tmp/dbpro-release-dd2bf5e4
```

| Artifact identity | Giá trị đo được |
|---|---|
| Archive | `/tmp/dbpro-release-dd2bf5e4/db-pro-v0.1.0-macos-arm64.tar.gz` |
| Archive bytes | 14.800.921 |
| Archive SHA-256 | `1446de6dd00765a9b0b36ff7849d526a58bdb7a9cdce971d39fd6c7a25d9ba9a` |
| Executable SHA-256 | `e27bff07a2dcba3ff1908cda6fdecc5e9477f4d767a6f53ffe93a7c4263fa4f4` |
| Measured minimum macOS | 11.0 từ `LC_BUILD_VERSION`; không có runtime proof trên macOS 11 |
| Layout/plist | Package script PASS, đúng `.app` + `README-INSTALL.txt` |
| Signature | `codesign -dv --verbose=4`: ad-hoc/linker-signed; không TeamIdentifier, Info.plist không bound, không sealed resources |
| Gatekeeper | `spctl -a -vvv -t execute <bundle>` exit **1**: `code has no resources but signature indicates they must be present` |
| Stapled notarization | `xcrun stapler validate <bundle>` exit **65**: không có ticket stapled |

Archive được extract rồi chạy trực tiếp executable bên trong `.app`; **không phải LaunchServices/quarantine/download install acceptance**. Dùng `DB_PRO_DATA_DIR` temp mới và `DB_PRO_DISABLE_KEYRING=1`, `DB_PRO_WINDOW_SIZE=1280x800`; không chạm app/data đang mở của user. Vì tắt Keychain, smoke này không chứng minh hành vi credential mặc định của bản phân phối.

- UI kết nối SQLite và introspect **11 tables / 2 views / 1 trigger**.
- Bấm query tab rồi Run thực sự chạy SQL seeded `select id, first_name, last_name, hire_date from employees limit 100;`; UI trả 3 rows, khớp independent SQLite oracle.
- AXSetValue/keyboard paste không thay được seeded SQL; **không tuyên bố đã chạy scalar `SELECT 41 + 1`**. Đây là giới hạn automation đã quan sát, không tự kết luận lỗi nhập SQL của người dùng.
- Light/Dark query screenshots đã mở và kiểm tra. Ảnh **2560×1664**, scale 2, cửa sổ **1280×832 logical gồm titlebar**, nội dung yêu cầu 1280×800; không coi là full three-size/state visual matrix.
- Đóng bằng native close control khiến process **-6 / SIGABRT**, log `NSRangeException` khi `_NSTouchBarFinderObservation` remove observer `nextResponder` của `WinitView`. Orca trả lỗi AX sau khi process đóng; exit process và exception log là bằng chứng độc lập, không suy từ status automation.
- Mở lại cùng artifact/data dir thành công, kết nối và introspect lại; **một saved connection giữ nguyên ID**. Không quan sát mất dữ liệu SQLite. Theme quan sát sau restart là Light thay vì Dark vừa chọn; chưa đủ để certify full workspace/settings persistence.
- Restart process được cleanup bằng SIGTERM (-15), **không tính là clean GUI quit**. Clipboard được khôi phục. Temp artifact/state giữ lại phục vụ điều tra, không xóa bằng chứng.

Evidence: [runtime log](evidence/readiness-2026-10-24/packaged-runtime.log), [Light](evidence/readiness-2026-10-24/query-light.png), [Dark](evidence/readiness-2026-10-24/query-dark.png), [Restart](evidence/readiness-2026-10-24/restart-light.png), [observations + hashes](evidence/readiness-2026-10-24/observations.json).

## 3. Implementation handoff / review request

**n/a — không sửa implementation.** Báo cáo và thư mục `evidence/readiness-2026-10-24/` là deliverable mới. Không commit/push trong lượt assessment này; không thay version, policy, lifecycle state hay release manifests.

### Ma trận test design / kết quả

| ID / priority | Preconditions, fixture | Action | Independent expected result | Status / evidence | Cleanup |
|---|---|---|---|---|---|
| REL-01 / high | Clean candidate, Rust 1.95.0 | Fmt/check/Clippy/test/build | Exit 0; failed=0; ignored riêng | PASS local; gate log | Chỉ build artifacts |
| REL-02 / high | Executable vừa build | Package/extract, hash, plist/layout checks | Archive contract, same executable bytes | PASS macOS local; observations | Temp artifact giữ lại |
| REL-03 / high | Fresh temp data, no user credentials | Launch actual non-capture package | Visible UI + async connection/introspection completes | PASS; runtime log/screenshots | Riêng process/data temp |
| REL-04 / high | Connected sample SQLite | Run visible seeded read-only query | 3 rows, SQLite independent count=3 | PASS; Light screenshot | Không DB mutation |
| REL-05 / medium | REL-04 | Switch theme | Actual pixels change, query rows retained | PASS Light/Dark | Chỉ temp app preferences |
| REL-06 / high | REL-04/05 | Close native window | Normal exit, no uncaught exception | **FAIL**, SIGABRT/NSRangeException | Process đã abort |
| REL-07 / high | Saved temp metadata sau abort | Relaunch same binary/data | Same connection ID, reconnect/introspect | PARTIAL PASS: connection continuity only | Restart SIGTERM, không gọi clean exit |
| REL-08 / high | Disposable PG fixture | Run 40 integration cases | Independent assertions: values, rollback, target/schema | PASS; provider log | Suite cleanup test-owned objects |
| REL-09 / high | Disposable MySQL fixture + reference DB | Run 12 cases | Mutations/rollback/FK/type assertions | PARTIAL: 9 exercised, 3 guards | Suite cleanup test-owned objects |
| REL-10 / high | Disposable SQL Server fixture | Run 9 cases | Rollback, cancellation/session recovery/plan assertions | PASS; provider log | Suite cleanup test-owned objects |
| REL-11 / high | Extracted package | codesign/spctl/stapler | Publisher identity and distribution trust accepted | **FAIL / missing qualification**; observations | Read-only |
| REL-12 / high | Exact candidate SHA | Query GitHub workflow/check status | Rust/release jobs traceable to this SHA | NOT RUN: 0 workflow runs | Read-only, no dispatch |
| REL-13 / high | Exact Cargo.lock | Dependency advisory/license disposition | Advisory findings triaged, notices included | NOT VERIFIED: cargo-audit absent; notices missing | No tool installed/policy changed |
| REL-14 / high | Default packaged Keychain; prior-version state | Save/restart/reconnect, upgrade/rollback | Credentials/state preserved; denied prompt safe | NOT TESTED | Không chạm user credentials |
| REL-15 / high | Windows/Linux/native engines | Critical interactive journeys + restart | Platform-specific user outcomes | NOT TESTED | Không có host trong lượt này |

Read-only/native desktop app: web/mobile/ARIA tests là n/a. X11/Wayland, Windows desktop, macOS VoiceOver, keyboard-only traversal, high-DPI/large-state matrices vẫn áp dụng nhưng chưa thực thi đầy đủ. Crash, interruption và restart được thử ở scope trên; không coi suite PASS là proof cho backup restore, native concurrency, AI egress hay accessibility.

## 4. Review outcome — finding và release consequence

**Verdict: BLOCK public/external release sign-off.** Baseline/introduced split: assessment snapshot, không diff-review; không quy những finding dưới đây là regression do riêng commit `dd2bf5e4` gây ra.

### Confirmed findings — 0 P0 / 0 P1 identified / 5 nhóm P2

1. **R-01 — P2, native release shutdown crash; release blocker.** Reproduction: extract newly built macOS archive → fresh temp data, Keychain disabled → sample query → toggle theme → close native window. Expected normal exit; actual SIGABRT with AppKit observer exception. Evidence `packaged-runtime.log:11–51`, artifact identity §2.3. Confidence high cho sequence/host này; không chứng minh mọi máy/mọi quit path đều crash, không xác định upstream root cause từ stack alone. Candidate regression: native package launch/query/window-close → exit 0/no exception plus restart/state oracle. **Không bỏ qua như “harmless capture teardown” vì binary này không có capture feature.**
2. **R-02 — P2, public distribution notice contract chưa thỏa.** `scripts/release/package-macos.sh:101–143` và strict archive allowlists chỉ ship binary/note, không LICENSE/dependency notices; `THIRD_PARTY_NOTICES.md:25–31` yêu cầu exact-lockfile notices trước public binary release. `scripts/release/README-INSTALL.txt:47–51` còn nói project không có license/không được redistribution, trái MIT ở `Cargo.toml:27–30` và LICENSE. Actual archive xác nhận chỉ `.app` + note. Candidate regression: kiểm tra distribution archive có applicable notices và copy không mâu thuẫn. Không đưa ý kiến pháp lý ngoài policy/metadata đã quan sát.
3. **R-03 — P2, hướng dẫn tag RC không khớp workflow.** `docs/release/0.1.0-handoff.md:285–289` gợi ý `v0.1.0-rc.1`; `.github/workflows/release.yml:136–147` đòi tag suffix bằng native manifest version `0.1.0`, nên cách đó sẽ bị từ chối. Static deterministic finding, không tạo tag để thử. Candidate regression: accepted/rejected tag–manifest matrix trong release tooling.
4. **R-04 — P2, release evidence/claims drift.** `docs/release/0.1.0-readiness.md:6–18`, `evidence-manifest.json` và checklist cũ qualify SHA `85a7fa3cc0a84c56ac2a5049ce08130db06e0a20`, không candidate này. `known-limitations.md:55–75` còn mô tả MySQL disabled/no binder; current `crates/app/src/connection/config.rs:3–35` cho chọn cả bốn drivers. Tài liệu PostgreSQL cancellation/SSH và một số BSN P1-open claims cũng đã stale. Candidate regression: release evidence identity/claim reconciliation dựa artifact SHA; không chỉ test JSON schema. **Báo cáo mới này không âm thầm biến old release dossier thành current qualification.**
5. **R-05 — P2, provider-test guards tạo false confidence nếu chỉ đọc harness totals.** MySQL guard-return ở `mysql_integration.rs:643–675`; ba skip đã quan sát trực tiếp. SQL Server `sqlserver_integration.rs:18–56` có thể đổi setup failure thành None rồi test return thành công; không gặp guard-skip ở run 9 ca lần này. Candidate regression/release gate: configured provider không kết nối được phải fail, thiếu fixture phải được tính unexecuted rõ ràng; log phải phân biệt attempted/exercised/skip. Không nâng thành bug data-corruption.

### Qualification gaps / explicit deferrals — không phải P1 được chứng minh

- **Signing/notarization:** chưa Developer ID/Authenticode; ad-hoc package không được Gatekeeper accept trong check này, không có stapled ticket. Source pipeline cố ý deferred. Cần implement/qualify hoặc owner chấp nhận unsigned preview rõ ràng, không tự suy ra waiver public.
- **Exact-head CI:** [API workflow runs](https://api.github.com/repos/truongnat/db-pro/actions/runs?head_sha=dd2bf5e4090e6d509d34c0c8c58ca95015d91640&per_page=20) trả 0. Check-runs chỉ Socket report success; combined status `pending` có 0 contexts, **không phải Rust CI đang chạy**. CI `.github/workflows/ci.yml:3–7` chỉ PR/manual. [39407358630](https://github.com/truongnat/db-pro/actions/runs/39407358630) fail fmt ở SHA `531937fcadb4cfadbdec8751e85e929f7671178e`; [39351355878](https://github.com/truongnat/db-pro/actions/runs/39351355878) fail Clippy ở SHA `cb6e00d633772de538498384a8b14d32d25f7915`; không quy hai fail này cho candidate.
- **Current multi-platform packages:** local macOS proof không thay release workflow cho Windows x86-64/Linux x86-64. Old successful release [34860902181](https://github.com/truongnat/db-pro/actions/runs/34860902181) là SHA tháng 9; API hiện chỉ liệt kê provenance artifact, không còn archive/checksum set để dùng như candidate mới.
- **Dependency security:** `cargo audit --version` trả no such command; không có fresh advisory scan/disposition, không khẳng định CVE cụ thể hay dependency-safe. Repo không thấy RustSec/cargo-deny gate trong workflows được kiểm tra. Cargo.lock không thay advisory scan.
- **Native safety/auth:** chưa verify save Keychain → quit/restart → reconnect; reject/delete secret; native staged-edit conflict/targeted retry/failure recovery; exported SQL thực thi lại; backup/restore qua app; full PostgreSQL/MySQL/SQL Server UI roundtrip. Postgres restore có accepted partial-restore semantics, không được quảng cáo atomic (`crates/infrastructure/src/backup/pg_dump.rs:125–157,316–341`).
- **Coverage/UI:** UAT-4 loading/error/empty evidence còn thiếu; UAT checklist/lifecycle chưa đồng bộ. Thiếu full requested size/state matrix, keyboard/accessibility proof và current end-to-end performance/idle resource budgets. Không có căn cứ cho accessibility/performance compliance score.
- **Upgrade/recovery:** metadata schema có migrations/newer-schema guard nhưng chưa có fresh old-version upgrade, automatic pre-migration backup/down-migration hay state-rollback rehearsal. Auto-updater/installers không nằm trong portable v0.1 contract; không tính chúng thành tính năng bắt buộc bị thiếu.
- **Product scope/identity:** phải chốt shipped provider/platform claims và các quyết định branding đang ghi pending trong registry, không coi source selectable tương đương public support certification.

## 5. Research / audit handoff — hành động ưu tiên

### Release checklist đề xuất

| Thứ tự | Việc cần làm | Acceptance / owner lane |
|---|---|---|
| 1 | Sửa native shutdown crash | Native app/runtime owner: đúng packaged release sequence exit 0; không exception; settings/query/connection continuity sau restart. Không suppress exception làm “fix”. |
| 2 | Hoàn thiện artifact legal/trust | Release owner: exact-lockfile applicable notices + LICENSE, sửa install note; Developer ID/notarization/Windows signing hoặc unsigned-preview decision được ghi rõ. |
| 3 | Freeze candidate + CI/artifact identity | Release engineering: thống nhất prerelease version/tag, chạy pinned exact-SHA CI/release trên 3 target; giữ artifacts/hash/provenance và test actual downloaded package. |
| 4 | Đóng critical native/provider/auth gaps | QA: disposable fixture; save/restart/reconnect default Keychain; mutate/rollback/conflict/retry/cancel; export/restore safety; PG/MySQL/SQL Server UI independently; 3 MySQL #452 cases đúng fixture riêng. |
| 5 | Security và compatibility qualification | Security/release: dependency advisory triage, license disposition; upgrade/state-backup/rollback rehearsal; Windows/Linux runtime; accessibility và platform prerequisites. |
| 6 | Đồng bộ release dossier | Docs/QA: refresh exact-SHA readiness/manifest/known limitations/provider claims, UAT accepted/deferred/verified rows, visual-state gaps; ghi exceptions/sign-off và support/rollback runbook. |

Chỉ chuyển sang external RC khi #1 và các gate RC áp dụng đã đạt hoặc có exception được owner ký rõ; public cần thêm đầy đủ notice/trust/platform/support gates. Không đánh đồng “tất cả test xanh” với “app đã release-ready”.

**Historical defects:** hai BSN P1 về filter/sort khi switch table và xóa staged edits khi targeted retry đã có remediation tại `workspace_actions.rs:120–139` và `table_mutation_actions.rs:519–570`; không giữ nguyên count “open P1” từ audit cũ nếu chưa tái hiện trên candidate. Fresh suite PASS hỗ trợ implementation, không thay native replay.

**Standards/test basis:** repo AGENTS/lifecycle/native UI gates; portable distribution contract; Rust compiler/lockfile reproducibility; native code-signing/Gatekeeper behavior; repository third-party notice policy. Không tuyên bố chứng nhận ISO, WCAG/VoiceOver hay security compliance.

**Nguồn bổ sung:** `docs/plans/FEATURE_LIFECYCLE.md`, `docs/plans/STATUS.md`, `docs/plans/active/uat-fixes-2026-10/`, `docs/plans/active/issue-537-539-edit-export-safety/VERIFICATION.md`, `docs/plans/active/viewport-select-fit/VERIFICATION.md`, `docs/plans/active/sqlserver-provider/VERIFICATION.md`, `.github/workflows/release.yml`, `scripts/release/*`, `THIRD_PARTY_NOTICES.md`. Các historical test totals chỉ dùng để định hướng, không được report là fresh measurements.

CI không dispatch; source không sửa; không tạo issue hoặc yêu cầu credentials từ user trong lượt assessment. Đề xuất downstream work, không tự kích hoạt remediation.

## 6. Tổng kết bằng tiếng Việt

Code hiện tại có nền tảng kiểm thử tốt: đúng Rust 1.95.0, 2.315 harness tests xanh, live PG 40 và SQL Server 9 ca xanh; MySQL chỉ 9 ca thực thi dù harness báo 12. Package macOS mới đã được mở và chạy query SQLite, Light/Dark đã nhìn trực tiếp.

**Chưa release public/external beta:** normal close của chính binary release gây SIGABRT, artifact thiếu notice bundle và chưa có distribution trust; current-candidate CI/cross-platform/default-Keychain/upgrade/native safety proof còn thiếu. SQLite connection sống sót sau restart không chứng minh mọi state hay secrets được bảo toàn. Ưu tiên sửa shutdown, hoàn thiện package/qualification và đồng bộ release evidence; chỉ engineering preview nội bộ trên disposable data ở thời điểm này.
