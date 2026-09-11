# Verification

## Automated

Verified on 2026-09-11 from the current worktree:

```text
cargo fmt --all -- --check                                      PASS
cargo check --locked -p db-pro-runtime -p db-pro-ui -p db-pro-native PASS
cargo test --locked -p db-pro-runtime -p db-pro-ui                 PASS (26 UI tests; runtime has 4 tests)
cargo clippy --locked -p db-pro-runtime -p db-pro-ui -p db-pro-native --all-targets -- -D warnings PASS
cargo check --locked --workspace                                  PASS
cargo test --locked --workspace                                   PASS (305 passed; 10 PostgreSQL tests ignored)
cargo clippy --locked --workspace --all-targets -- -D warnings    PASS
cargo fmt --all -- --check                                      PASS (after Lucide icon integration)
cargo check --locked -p db-pro-ui -p db-pro-native                  PASS
cargo test --locked -p db-pro-ui -p db-pro-native                   PASS (26 UI tests)
cargo clippy --locked -p db-pro-ui -p db-pro-native --all-targets -- -D warnings PASS
native composite-PK identity unit coverage                             PASS (26 db-pro-ui tests; each key column and typed UiCell preserved)
native exact-decimal input unit coverage                                PASS (NUMERIC/DECIMAL exponent, scale and precision checks preserve the original decimal text)
common UI audit                                                     PASS (shared action controls, ghost state, wrapped toolbars)
workspace layout pass                                               PASS (tab strip, welcome card, topbar hierarchy)
settings surface pass                                               PASS (backup/restore moved out of global header)
responsive native UI pass                                           PASS (1024×640, 1280×800, 1440×900, 1920×1080)
native SQLite result-grid pass                                      PASS (2-row fixture, visible headers/filter/copy controls)
Agent close restoration regression                                     PASS (11 db-pro-ui tests; narrow viewport state restored)
Connection status indicator regression                                  PASS (12 db-pro-ui tests; selected/disconnected differs from connected)
native result-grid benchmark                                            PASS (1,000,000-row projection ~3.15 ms; 100-row visible window ~38 ns; Criterion --quick)
query-tab close state regression                                         PASS (15 db-pro-ui tests; active SQL document restored after close)
Codex-light base component/runtime review                                PASS (1280×800 welcome, Settings inputs, Query/editor/mutation controls; focus ring and sidebar states visually checked)
Agent-open mutation toolbar review                                        PASS (row mutation controls remain grouped in a framed header/action surface when the 380px Agent panel is open)
SQLite native end-to-end smoke                                            PASS (connected Native Test fixture, selected customers table, ran SELECT, rendered 2 result rows, opened Agent and generated a schema draft)
Explorer table selection persistence review                                PASS (selected `customers` keeps the active sidebar treatment after query navigation)
Native table workspace review                                               PASS (SQLite `main.customers` opened into Structure, Data and DDL views; 3 columns and generated CREATE script rendered)
Native table data runtime smoke                                             PASS (Data tab fetched 2 rows through `TableDataApi`; shared filter/copy grid rendered)
Provider-aware Explorer review                                             PASS (SQLite renders `main`; selected `customers` expands Columns/Indexes/Foreign keys in the sidebar)
Native table pagination runtime smoke                                       PASS (150-row SQLite fixture loaded `1–100 of 150`, then `101–150 of 150` through the next-page command)
Server-side table filter runtime smoke                                      PASS (SQLite `id contains 101` returned `1–1 of 1` through `TableDataApi`)
Server-side table sort runtime smoke                                        PASS (SQLite `ORDER BY id DESC` returned the first page beginning at id `150`; UI restored to `ASC`)
Native view definition runtime smoke                                        PASS (SQLite `active_customers` appeared under Views and opened its `CREATE VIEW` definition)
Native trigger definition runtime smoke                                     PASS (SQLite `customers_after_insert` appeared under Triggers with `customers · AFTER · INSERT` metadata and opened its definition)
Native view data runtime smoke                                               PASS (SQLite `active_customers` loaded `1–75 of 75` through `TableDataApi`; shared filter/sort/pagination grid rendered active rows)
Native function DTO/UI review                                                PASS (PostgreSQL catalog mapping preserves schema, routine type, return type and definition; restarted SQLite app renders `Functions (0)` for the unsupported provider)
Shared-runtime wiring review                                              PASS (Tauri setup constructs `DbProRuntime`; Arc service/connector/registry states are managed for command adapters; workspace compiles and tests pass)
Native ER diagram runtime smoke                                            PASS (restarted SQLite fixture rendered `main.customers` and `main.orders`, showed `2 tables · 1 relationships`, drew the `customer_id → id` orthogonal connector, and clicking a node opened its Table/Structure workspace at 1280×832)
Native table Data inline-edit UI smoke                                      PASS (SQLite `main.customers` loaded `1–100 of 150`; selecting a cell and pressing Edit cell opened an in-grid editor with the violet focus ring, and Escape cancelled without submitting a mutation)
Native table Data New row UI smoke                                          PASS (SQLite `main.customers` opened a Codex-light metadata-driven Insert row dialog with `id · INTEGER`, `name · TEXT` and `status · TEXT` fields; the dialog was cancelled without changing the fixture)
Native DBeaver-style table Data UI smoke                                   PASS (restarted SQLite app rendered Data Editor toolbar, fixed `#` row-number gutter, full-row selection, Refresh, Edit cell, Delete row confirmation action and `Page 1 of 2`)
Native Agent provider-boundary smoke                                       PASS (restarted SQLite preview shows `Offline draft`, `Pure Rust responder · writes stay unexecuted`, and generated SQL remains an inspectable draft)
Native Codex provider fallback smoke                                      PASS (no `OPENAI_API_KEY` on host; runtime worker emits provider-unavailable event and UI falls back to Offline Draft without blocking the native window)
Native Quick Open/Command Palette smoke                                    PASS (Command Palette opens from toolbar, `diagram` filters to `Open ER diagram`, Enter navigates to the ER workspace; palette renders with the Codex-light card/input/list primitives)
Native force-refresh schema smoke                                          PASS (external SQLite index drop remained stale in the open Structure view, Command Palette `Refresh schema` bypassed the cache and rendered `0 indexes`)
Native Agent read-only execution smoke                                     PASS (connected SQLite fixture → Agent starter prompt → explicit `Run read-only` action → Query workspace returned one aggregate result row; no mutation was submitted)
Native DDL editor/apply runtime smoke                                    PASS (connected SQLite fixture → editable single-statement DDL → explicit Apply/Execute confirmation → index appeared in refreshed Structure metadata → DROP cleanup restored 0 indexes)
clean-code diff scan                                                      PASS (33 heuristic checks; macOS xargs compatibility warning emitted by the scanner, no reported smell failures)
perf-audit                                                           FAIL (10 frontend budget tests PASS; raw JS bundle 2.35 MB > 2 MB critical threshold; largest chunk 594 KB warning; CSS 122 KB warning; Rust check/clippy PASS)
```

The first check exposed and fixed a missing struct-field comma, missing `uuid`
runtime dependency, missing saved-query facade methods, and a duplicate native
event-mapping arm. Clippy then exposed and fixed native/runtime quality-gate
issues; the final commands above are clean.

Commands to run when the toolchain is available:

```bash
cargo fmt --all -- --check
cargo check -p db-pro-runtime -p db-pro-ui -p db-pro-native
cargo test -p db-pro-runtime -p db-pro-ui
cargo clippy --workspace --all-targets -- -D warnings
```

## Run native preview

```bash
DB_PRO_DATA_DIR=/tmp/db-pro-native-lucide-data cargo run --locked -p db-pro-native
```

Launch evidence on 2026-09-11: the binary compiled with Lucide font assets and
remained alive as `target/debug/db-pro-native` with no process output or crash
after the workspace layout pass. Manual screenshots were reviewed with the
native window at 1280×800, 1440×900 and 1920×1080. The 1440×900 and 1920×1080
checks included the Agent side panel; the query toolbar remained usable and did
not clip after the wrapped-layout pass.

The History surface was also checked against persisted native metadata: saved
queries render under an expandable folder tree, child rows expose rename/delete
actions, and folder deletion opens a confirmation dialog explaining that saved
queries become unfiled.

The app reads saved connections from the existing metadata schema. Select a connection in Explorer to connect, then open Query and run a statement. PostgreSQL/SQLite credentials remain in the configured keyring; no password is passed to the UI bridge.

## Manual

Native runtime review completed for this UI slice:

- Launch native binary.
- Check light theme, panel hierarchy, typography and focus/hover states.
- Check sidebar resize/collapse and agent panel open/close.
- Check `Cmd/Ctrl+P`, `Cmd/Ctrl+B`, and Escape; Escape restores Explorer after a narrow-window Agent session.
- Stress-test the minimum supported window at 1024×640 with Agent open; Explorer hides temporarily and query actions wrap without overlap.
- Review at 1280×800, 1440×900 and 1920×1080.
