# Fixture contracts — release qualification

Plan only. Đây là specification để fixture owner chuẩn bị **sau khi được phép chạy**; các F-* không phải fixture đã provision. Source baseline: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`.

## 1. Safety và isolation bắt buộc

1. Mỗi run có `run_id`, mỗi instance có namespace riêng. Ưu tiên standalone test containers/VM và SQLite file mới; không suy ra endpoint an toàn từ localhost/port/name.
2. Run manifest phải ghi server identity/version, DB/schema/file, test role, run marker, exact connection IDs, quyền, schema/data oracle và tài nguyên được phép mutate/delete. Không ghi password/URI có password; chỉ secret reference được bảo vệ ngoài evidence public.
3. Default deny: không dùng saved connection/user app, không dùng database nghiệp vụ `fullstack_starter` trên 5433 hoặc server bất kỳ chỉ vì biết credential. Existing dbpro fixture chỉ reuse sau owner registration + baseline identity + namespace ownership checks; không reset toàn server.
4. **`DB_PRO_DATA_DIR` chỉ cô lập runtime metadata/secrets, chưa chứng minh cô lập UI state.** `main.rs:585–607` truyền `creation_context.storage`; `app_lifecycle.rs:44–138` ghi UI/settings/documents qua eframe Storage. `cargo +1.95.0 tree --locked -p db-pro-native -e features -i eframe` trong lượt planning không hiển thị feature `persistence`. Không coi missing feature là đã xác minh runtime failure, cũng không mặc định UI persistence tồn tại. LIF-04/SET-01/FIL-01/SET-03 phải kiểm tra actual storage/save/restart. Không thay bằng headless save/restore test.
5. Để chạy native UI: dùng disposable OS test profile/VM là đường chuẩn; profile phải có đủ Orca/desktop permissions riêng. Cách khác chỉ hợp lệ khi fixture owner chứng minh effective GUI-storage path, keyring namespace và mọi file write đều thuộc sandbox; **không đoán env override mà app không đọc**. Không có isolation proof => BLOCKED trước hành động.
6. Keychain/denial/TLS trust/roles/upgrade cần OS/test-server namespace thực. Không toggle/reset global Accessibility, Keychain, known_hosts, trust store, Docker volumes hay user layout để cứu automation.
7. Baseline/reset là transaction/restore/copy từ fixture manifest, không chạy seed cũ lên DB đã sửa. Mỗi action conflict resolution, transfer mode và restore failure dùng reset riêng. Suite/case teardown chỉ tài nguyên có manifest ownership; cleanup failure giữ lease và ghi BLOCKED, không trao desktop bị bẩn.
8. CLI được dùng để **setup/second writer/oracle/cleanup** trên fixture đã cho phép. Không dùng CLI hoàn thành thay thao tác app rồi ghi native UI PASS.

## 2. Asset đã có — source contracts, chưa phải live baseline

| Asset | Source baseline/count đáng dùng | Isolation/reset |
|---|---|---|
| `fixtures/sqlite/fixture.sql` | employees3, categories4, products4, orders4, order_items6, audit_logs3, documents3, weird-name2, Unicode-users4, empty_table0; composite PK và no-PK | File mới; không seed overlay. Dynamic date defaults không là byte-oracle. |
| `fixtures/postgres/{001_schema,002_seed,003_verify}.sql` | Verifier: categories4/products4/orders4/order_items6/decoder_matrix2 | DB mới thuộc run; test catalog/schema separately. |
| `fixtures/mysql/{001_schema,002_seed,003_verify}.sql` | decoder_matrix2, categories2, order_items3, order_item_audit3; exact UTF-8 assertions | Reset cả trigger/audit side effects; hiện chưa có no-PK fixture. |
| `fixtures/mssql/{001_schema,002_seed,003_verify}.sql` | decoder_matrix2; exact i64 max/BLOB | Chưa có composite/no-PK/scale fixture; không overlay script DROP/DELETE lên user DB. |
| `fixtures/smoke/sqlite/runtime_fixture.sql` | users60, user_roles120, orders120, order_items240; order IDs1..120 | File riêng; script DROP/recreate generic objects. |
| `fixtures/smoke/{postgres,sqlite}/` + `verify-smoke.sh` | 10 tables/2 views/1 trigger per matching smoke fixture; verify exact prefixed counts | Đừng trộn smoke_fixture với runtime_fixture; README row-total có drift, dùng verifier/data oracle. |
| `fixtures/smoke/large-er/large_er_fixture.sql` | README contract250 tables/249 chain FKs/5 views, phải independently verify khi provision | DB disposable riêng; DROP first table CASCADE không phải full cleanup250 tables. |

`scripts/provision-test-fixtures.sh` là **reference**, không phải worker reset command: persistent volumes được reuse, PG endpoint có thể khác server thực, script chạm user `~/.ssh/known_hosts` và để services running. Không chạy nguyên script từ GUI worker. Fixture owner tạo test topology riêng và private known_hosts; không dùng heredoc PASS cũ làm actual output.

## 3. Required fixture definitions

Mọi fixture sinh ra `fixture-manifest.json` dưới `runs/<run_id>/fixtures/<id>/`: source assets + hash, generated setup hash, server identity, ownership marker, resources, test role/permissions, safe endpoints, secret references, exact sorted rows/catalog expected, reset/cleanup actions, before/after sentinel hashes. Generated SQL/assets phải được giữ lại; chưa có generator committed không được giả là đã có.

### F-BASE — artifact, desktop và storage sandbox

- Actual extracted release archive (no capture feature); executable/archive SHA256; exact source SHA/compiler/features; platform/version and intended channel.
- Run-owned artifact install, runtime data, workspace/output/log dirs và disposable OS profile/verified GUI-storage namespace. Freeze app language English cho semantic labels; locale cases đổi riêng.
- One app PID/window bound to case manifest; desktop lease owner/epoch. One default sample connection có thể auto-seed: record rồi chỉ xóa qua UI trong empty-state case.
- Default OS-secret-store lane **không** set DB_PRO_DISABLE_KEYRING/ALLOW_FILE_FALLBACK. Fallback lane được set riêng và không qualify default credentials.
- **F-TRUST** alias: F-BASE plus unmodified downloaded artifact, available platform trust verifier and channel trust policy. Không strip quarantine để biến FAIL thành PASS.
- **F-UPGRADE** extends: approved previous-version binary/state snapshot trong sandbox, migrated clone, dedicated keyring identities và complete restore snapshot. Không đổi user HOME/global data. Binary-only downgrade không đủ state rollback.

### F-GRID — canonical cross-provider writable dataset

Provisioner tạo schema `qa_<run_suffix>` hoặc DB/file riêng theo provider, với names ≤ engine identifier limits; mọi ID được ghi trong manifest. SQL Server dùng dedicated DB/schema, MySQL schema=database; SQLite main chỉ trong file run-owned.

- `qa_rows(id INTEGER PRIMARY KEY, name VARCHAR(100) NOT NULL, age INTEGER NOT NULL CHECK(age>=0), note TEXT NULL, version INTEGER NOT NULL)`: **205 rows**, IDs1..205, name `row-0001`..`row-0205`, age=`id%80`, note NULL cho IDs chia5 còn lại `note-<id>`, version1.
- `qa_other(id INTEGER PRIMARY KEY, code VARCHAR(50))`: IDs1,2 và code `B-1`,`B-2`; cố tình không có `name`/`age` cho switch-filter/sort negative.
- `qa_empty(id INTEGER PRIMARY KEY, name VARCHAR(100))`: zero rows.
- `qa_delete`: clone baseline3 rows, chỉ dùng destructive-confirm case.
- `qa_parent(id PRIMARY KEY,label)` và `qa_child(id PRIMARY KEY,parent_id FK,label)`: parent1/2, child10→1,11→1,12→2; canonical join returns3 rows and named parent labels.
- `qa_view` selects id/name from qa_rows where id≤3; metadata view và native UI read-only boundary.
- Setup uses provider-safe generated DDL/quoted identifiers and bound seed values; NOT NULL/CHECK/type differences reviewed and recorded. Chính xác column/value/cardinality được query lại trước dispatch.
- Oracle client khác session của app; read sorted by full keys, preserve typed NULL/empty/value fields, no implicit float conversion.

### F-SQLITE / F-NETWORK / F-READONLY / F-TRANSACTION / F-MAINT

- **F-SQLITE:** F-GRID fresh SQLite copies at names with spaces/Unicode; separate missing pathname and text-not-database file. Existing target hash recorded; invalid-file case must not destroy it.
- **F-NETWORK:** F-GRID reachable server plus disposable wrong-password input; owned unreachable endpoint/port only. Empty/invalid-port form tests require no network side effect; không thử all ports or external hosts.
- **F-READONLY:** same seeded dataset through separate **application readonly profile** and DB-restricted role for remote engines; test both app policy and provider permission. Oracle write privilege separate, denied account cannot be used as writable positive fixture.
- **F-TRANSACTION:** fresh F-GRID with scalar/sentinel row; only DML for shared atomicity contract. MySQL DDL not transactional; unsupported transactional statement must be explicit instead of expected full rollback.
- **F-MAINT:** standalone disposable SQLite/PG with integrity oracle and case table; safe ANALYZE/integrity checks scoped. No VACUUM/REINDEX/maintenance of user/shared DB.

### F-BOUNDARY / F-IDENT / F-TARGETS

- **F-BOUNDARY:** `qa_n99`, `qa_n100`, `qa_n101` IDs1..N, one text value per row. SQLite may reuse `runtime_fixture.sql` orders WHERE id≤N in fresh file; source SQL has no explicit LIMIT. Toolbar cap exactly100. Oracles99/100/101 independently confirmed; SQLServer uses product-generated TOP/limits as applicable.
- **F-IDENT:** quoted table `weird"name`, Unicode `Ünïcödé Üsers`, long table name safely below engine limit and enough to truncate in UI; each unique sentinel row. Add same table name in two schemas where supported. Dangerous-looking identifier is data, never raw SQL concatenation.
- **F-TARGETS:** two saved connection IDs A/B, distinct DB/schema identity markers `A_ONLY`/`B_ONLY` and same table name with different values. Both genuinely usable; transfer requires actual open handles. Special QRY-10 A=SQLite/main, B=Postgres/qa_schema; distinct source fixture oracles, not preexisting saved user connections. Record first-party capabilities and compare target after every operation.

### F-COMPOSITE / F-CONSTRAINT / F-CONFLICT

- **F-COMPOSITE:** `qa_composite(tenant_id,id,name,PRIMARY KEY(tenant_id,id))`: (1,1,A),(1,2,B),(2,1,C). `qa_no_pk(name,note)` has two identical duplicate rows plus one distinct row. Native UI currently needs complete PK; negative no-PK/view edit cases require unchanged full digest. `no_pk_tables=true` capability alone does not imply native editability (`table_editor_context.rs:10–69`).
- **F-CONSTRAINT:** reset F-GRID IDs1/2. Valid row edit plus invalid CHECK/NOT NULL row; independent preimage exact. Invalid local value may be rejected before backend: use DB-only CHECK or controlled target failure so transaction-failure case actually reaches backend; record which boundary executed. UI validation-only PASS does not prove rollback.
- **F-CONFLICT:** baseline rowA=(id1,name=Original,version1), rowB=(id2,name=PendingB,version1). GUI stage A=Local and B=LocalB; separate oracle writer commits A=Server before Save. Each Keep Mine/Use Database/Retry scenario starts fresh. Oracle has handshake timing, both final rows and pending targets. For zero-affected case delete only case-owned A before Save. No timing guess or simulated dialog capture.

### F-VALUES / F-EXPORT

- **F-VALUES:** exact integer **9007199254740993**, signed i64 max where supported; remote NUMERIC/DECIMAL **12345678901234567890.1234**, MySQL unsigned max where type contract supports it; SQLite exact decimal stored as TEXT, **not REAL**.
- Separate rows distinguish NULL, empty string, `O'Reilly`, quote/comma/tab/newline, `Tiếng Việt`, binary bytes, UTC/non-UTC temporal values and microseconds. Use native decoder_matrix assets for remote type-specific values; record provider type and canonical export format expectations, not generic string coercion.
- Include text starting `=1+1` for spreadsheet formula boundary, and large text whose actual provider/DTO truncation marker is observed. If a specific truncation marker cannot be produced, that boundary BLOCKED, not passed by short text.
- **F-EXPORT:** F-VALUES frozen query snapshot, run-owned output dir, existing sentinel file and controlled permission/quota-failure subdir inside sandbox. Client parse CSV/TSV/JSON/Markdown/Excel using format-aware parser; generated SQL replay goes only to empty fixture target. Save/restore clipboard only when same owned test session; never leak synthetic secret parameters to screenshots.

### F-SCHEMA / F-COMPARE / F-ROUTINE / F-LARGE

- **F-SCHEMA:** F-GRID plus nullable/default/generated columns as provider supports; explicit index, composite FK, view and at least parent/child/grandchild+isolated graph. Manifest exact tables/edges/FK column pairs/capabilities, one zero-table schema for ER empty. Writable owner for DDL; read-only principal for negative action.
- **F-COMPARE:** snapshot/target schema drift one new column/index, both catalogs before/after; different-target sentinel. Schema Compare UI is **Snapshot → Diff now → Generate migration plan**, not guessed two-DB synchronization. Record preview direction before applying. Data Compare separate target-ID/key-aware sample ≤1000 rows; **Sync SQL preview-only** at `schema_compare_view.rs:452–461`.
- **F-ROUTINE:** Postgres test echo function/procedure with declared typed input incl NULL and quote payload; one deliberate error with DETAIL/CONTEXT; no installed debugger extension by default. Extension-enabled stepping requires separate provision/owner consent and not assumed.
- **F-LARGE:** generated200/500/1000-table dedicated DB/schema with declared FK pattern. Existing250-table asset is reference, not interchangeable with requested sizes. Preserve generator SQL/hash/catalog counts; independent provisioning ownership and cleanup whole isolated DB. Record hardware/sample method and compare frozen perf budgets; screenshots do not measure frames.

### F-COPY / F-BACKUP / F-BACKUP-WAL

- **F-COPY:** separate source/target registered connections, matching target table preexists; first column PK and remaining text-only columns for supported baseline. Target `(1,Existing),(9,TargetSentinel)`; source `(1,Replacement),(2,New)` for normal mode; for failure target CHECK rejects `Replacement` after DELETE. Before/after entire source+target snapshot; separate fresh datasets for abort/skip/replace. Current copy path string-binds values and takes first column as conflict key (`core/application/table_copy.rs:50–266`); do not infer arbitrary typed copy fidelity.
- **F-BACKUP:** SQLite/PG source with schema/views/index/FK and exact sentinel rows, separate empty restore target, dedicated artifact path. PG requires saved synthetic secret plus version-compatible pg_dump and psql/pg_restore binaries. In app use Settings Backup / Restore; SQLite target must be disconnected and writable before restore, then reconnect. Verify backup independently before changing target; source never restore target.
- **F-BACKUP-WAL:** fresh SQLite source/backup with committed sentinel in WAL held open by separate connection; no forced checkpoint before operation. Independent target query + integrity_check proves WAL inclusion. Preserve pre-operation hashes and never copy file-only then call it valid backup.
- PG partial-failure script is generated in run-owned files with explicit successful prefix and deliberate later error; destination disposable. Expected truthful partial-restore warning, **not atomic rollback**. Missing tools case changes child PATH only.

### F-LONG / F-FAULT / F-VIS

- **F-LONG:** provider-specific bounded read: PG pg_sleep(10), MySQL SLEEP(10), SQLServer WAITFOR DELAY00:00:10 then SELECT, SQLite bounded expensive recursive/cross-join SELECT with safety max/time cap; no permanent mutation. Separate owned client verifies only this app's request/session cancels; own unique application/run marker; next scalar42 proves recovery. A fast empty stub is not a long-query fixture.
- **F-FAULT:** run-owned protocol-stall socket/proxy, unreachable target, permission-denied test relation and controlled delay endpoint. Controller supports explicit begin-hold/release markers and records sockets; no sleeping/arbitrary timing as sole oracle. Never kill Docker/user network services. No committed fault controller exists yet: fixture owner must supply and retain working setup or report BLOCKED.
- **F-VIS:** composes appropriate F-* per surface and `CASES.json.state_recipes`. Pending/error/empty/hover/active states must be real native state on candidate; state unavailable by actual contract is N_A only after lead approval/source proof; too fast/uncontrollable/no fixture is BLOCKED. Each named subvariant is separate (e.g. Hub vs Monitor vs Roles), do not use one screenshot for all. No fabricated loading/error card or debug capture fixture can qualify release behavior.

### F-KEYCHAIN / F-KEYCHAIN-DENIED / F-TLS / F-SSH / F-MYSQL-RESERVED

- **F-KEYCHAIN:** disposable OS profile; same genuine packaged artifact/default runtime policy as distribution; synthetic remote DB password and unique app connection IDs. Save/read reconnect/delete through app only; no dump of user keychain or secrets.json. Verify only owned identities and sanitized failure messages.
- **F-KEYCHAIN-DENIED:** separate test profile/keychain where access can be denied without affecting user; permission prompts/return verified in pixels. Missing authorized test account/OS permission => BLOCKED, not create/reset user account automatically.
- **F-TLS:** standalone endpoints: valid trusted testCA+hostname; wrongCA; mismatched hostname; plaintext-only; optional mTLS client cert/key when advertised. Test trust confined to OS profile/app config. Cert generation/setup not in repo currently; provider requirements differ, record exact negotiated protocol/CA/hostname and no plaintext fallback. No weakening global verify settings.
- **F-SSH:** own bastion and DB target, own test keys/private known_hosts under sandbox, controlled changed/unknown host-key, wrong-key and denied-forwarding scenarios. Verify destination version/database marker; localhost5432 proxy must not accidentally reach Homebrew/user instance. Do not reuse global ~/.ssh/known_hosts mutations from provision script.
- **F-MYSQL-RESERVED:** fixture owner generates test-only Unicode/reserved-character raw username/password/database on dedicated server; DB name excludes slash/backslash/dot, stable identity queried independently. Server/account setup uses safe driver binding/quoting and never logs secrets; no preexisting external password needed. Separate plaintext-only server for strict-TLS-negative case, not positive trust proof. `DB_PRO_MYSQL_452_*` source contract exists but actual fixture assets/values not committed; missing values are unexecuted cases, never harness PASS.

### F-ADMIN / F-TASK / F-WORKSPACE / F-AI-NOKEY / F-AI-LIVE

- **F-ADMIN:** dedicated standalone PG cluster (not merely shared DB) because roles/session signals are cluster-wide. Unique child role+parent SELECT-only group, RLS tenant1/tenant2 table and restricted nonowner/non-BYPASSRLS login. Second client long read has known PID/backend_start/application_name. Only signalled own session; no predefined/system roles or arbitrary terminate. Manifest records grants and safe teardown order.
- **F-TASK:** saved task SQL has no bind parameters; target audit table logs run token + DB timestamp; expected initial Run once then Schedule60s while app open. Observe two due windows, Disable then another quiet window. UI schedule currently60s only; zero interval/missed-window boundaries are supplementary existing automated/API tests if callable, **not invented GUI controls**. SQL/backup task creation real; Export/Maintenance enum alone does not imply New Task UI.
- **F-WORKSPACE:** copy tree outside repo with two SQL files distinct names/scalars11/22, nested folder, external-change file, run-owned outside sentinel and symlink pointing only to that sentinel. Disk hashes before/after; no user repo folder selected. Saved-query and named-session fixtures separate; sessions store identities/layout, not SQL text/secrets/results as a substitute for document persistence.
- **F-AI-NOKEY:** isolated profile with no AI keys and synthetic DB only. Instrument only app-owned egress, never user browser traffic. No fake provider response accepted as real API evidence.
- **F-AI-LIVE:** requires **separate explicit authorization** for external provider endpoint, dedicated test key, allowed synthetic payload, spend ceiling and cancellation/revoke ownership. User request to plan is not consent to spend/send. Keys supplied through secure references, not docs or worker prompt. No real-key fixture => relevant instances BLOCKED; no mocks or fallback-generated answer labeled PASS.

## 4. Readiness proof trước giao worker

Fixture owner phải giao: manifest/hash; exact endpoint/connection IDs; approved mutable resources; independent baseline oracle; reset/cleanup recipe đã smoke-prove trên clone; controller/second-writer handshakes; sandbox GUI storage/keyring proof. Lead kiểm tra ngoài-namespace sentinel trước/sau lane. Chỉ khi đủ mới chuyển Ready. Thiếu fixture không phải product FAIL, nhưng vẫn chặn gate tương ứng; worker không tự chọn nguồn dữ liệu tiện nhất.
